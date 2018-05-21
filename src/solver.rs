use std::collections::HashMap;
use std::fmt;
use std::cmp::{min, max};

use ndarray::{ArrayViewMut1, Array2};

use model::{
    Puzzle,
    Square,
    Grid,
    LineIndex,
};


#[derive(PartialEq, Eq, Debug)]
enum SolverError {
    Solved,
    Invalid,
    Stuck,
}

#[derive(Clone, PartialEq, Eq)]
enum Partial<T> {
    Unknown,
    Known(T),
}

impl<T> Partial<T> {
    fn collapse(self) -> T {
        match self {
            Partial::Unknown => panic!("Cannot collapse Partial::Unknown"),
            Partial::Known(inner) => inner,
        }
    }
}

impl fmt::Display for Partial<Square> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Partial::Unknown  => write!(f, " "),
            Partial::Known(s) => write!(f, "{}", s),
        }
    }
}

// The final run of full squares must be:
// - A subset   of [lo, hi) if bound is Some((lo, hi))
// - A superset of [lo, hi) if found is Some((lo, hi))
#[derive(Clone)]
struct PartialRun {
    bound: Option<(usize, usize)>,
    found: Option<(usize, usize)>,
}

impl PartialRun {
    pub fn new() -> Self {
        PartialRun {
            bound: None,
            found: None,
        }
    }

    fn update_bound(&mut self, lo: usize, hi: usize) {
        if lo >= hi {
            return;
        }

        self.bound = match self.bound {
            None => Some((lo, hi)),
            Some((old_lo, old_hi)) => Some((max(old_lo, lo), min(old_hi, hi))),
        }
    }

    fn update_found(&mut self, lo: usize, hi: usize) {
        if lo >= hi {
            return;
        }

        self.found = match self.found {
            None => Some((lo, hi)),
            Some((old_lo, old_hi)) => Some((min(old_lo, lo), max(old_hi, hi))),
        }
    }
}

struct PartialLine<'a> {
    hints: &'a [usize],
    runs: &'a mut [PartialRun],
    line: ArrayViewMut1<'a, Partial<Square>>,
    // index: LineIndex
}

#[derive(PartialEq, Eq, Debug)]
struct DeductionStep {
    li: LineIndex,
    changes: Vec<usize>,
}

struct Solver {
    deductions: Vec<Box<Fn(PartialLine) -> Option<Vec<usize>>>>,
}

type Solution = Vec<DeductionStep>;

struct SolverWorker<'a> {
    solver: &'a Solver,
    puzzle: &'a Puzzle,
    grid: Grid<Partial<Square>>,
    dirty: HashMap<LineIndex, bool>,
    runs: HashMap<LineIndex, Vec<PartialRun>>,
    steps: Solution,

    // Keep track of other solved features (ex., some tile is part of some run)
    // Deductions should be able to accept extra features
    // I don't know yet what features will be helpful for certain deductions
}

impl<'a> SolverWorker<'a> {
    fn new(solver: &'a Solver, puzzle: &'a Puzzle) -> SolverWorker<'a> {
        let mut dirty = HashMap::with_capacity(puzzle.w() + puzzle.h());
        let mut runs = HashMap::with_capacity(puzzle.w() + puzzle.h());

        for li in puzzle.index_iter() {
            dirty.insert(li, true);
            runs.insert(li, vec![PartialRun::new(); puzzle.hints(li).len()]);
        }

        SolverWorker {
            solver,
            puzzle,
            grid: Grid(Array2::from_elem((puzzle.h(), puzzle.w()), Partial::Unknown)),
            dirty,
            runs,
            steps: Vec::new(),
        }
    }

    fn line(&mut self, li: LineIndex) -> PartialLine {
        PartialLine {
            hints: self.puzzle.hints(li),
            runs: self.runs.get_mut(&li).unwrap(),
            line: self.grid.line(li),
        }
    }

    fn step(&mut self) -> Result<DeductionStep, SolverError> {
        for li in self.puzzle.index_iter() {
            for deduction in self.solver.deductions.iter() {
                if let Some(changes) = deduction(self.line(li)) {
                    for &change in changes.iter() {
                        self.dirty.entry(li.line_through(change)).or_insert(true);
                    }

                    return Ok(DeductionStep { li, changes });
                }
            }
        }

        Err(SolverError::Stuck)
    }

    fn solve(mut self) -> Result<Solution, SolverError> {
        loop {
            match self.step() {
                Ok(step) => self.steps.push(step),
                Err(SolverError::Solved) => return Ok(self.steps),
                Err(e) => return Err(e),
            }
        }
    }
}

impl Solver {
    fn new() -> Self {
        Solver {
            deductions: vec![Box::new(deduce_overlap)],
        }
    }

    fn delegate<'a>(&'a self, puzzle: &'a Puzzle) -> SolverWorker<'a> {
        SolverWorker::new(self, puzzle)
    }
}

fn deduce_overlap(partial: PartialLine) -> Option<Vec<usize>> {
    let mut changes = Vec::new();

    let hints = partial.hints;
    let mut line = partial.line;

    let span = hints.iter().sum::<usize>() + hints.len() - 1;
    let flexibility = line.len() - span;

    let mut left = 0;
    for (i, hint) in hints.iter().enumerate() {
        let lo = left + flexibility;
        let hi = left + hint;

        partial.runs[i].update_found(lo, hi);

        for j in lo..hi {
            if line[j] != Partial::Known(Square::Full) {
                line[j] = Partial::Known(Square::Full);
                changes.push(j);
            }
        }

        left = hi + 1;
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_solves_easy_puzzle() {
        let puzzle = Puzzle::new()
            .push_row(vec!(5))
            .push_row(vec!(1))
            .push_row(vec!(5))
            .push_row(vec!(1))
            .push_row(vec!(5))
            .push_col(vec!(3, 1))
            .push_col(vec!(1, 1, 1))
            .push_col(vec!(1, 1, 1))
            .push_col(vec!(1, 1, 1))
            .push_col(vec!(1, 3));

        let solver = Solver::new();
        let mut worker = solver.delegate(&puzzle);

        assert_eq!(worker.step(), Ok(DeductionStep {
            li: LineIndex::Row(0),
            changes: vec![0, 1, 2, 3, 4]
        }));

        assert_eq!(worker.step(), Ok(DeductionStep {
            li: LineIndex::Row(2),
            changes: vec![0, 1, 2, 3, 4]
        }));

        assert_eq!(worker.step(), Ok(DeductionStep {
            li: LineIndex::Row(4),
            changes: vec![0, 1, 2, 3, 4]
        }));

        assert_eq!(worker.step(), Ok(DeductionStep {
            li: LineIndex::Col(0),
            changes: vec![1]
        }));

        assert_eq!(worker.step(), Ok(DeductionStep {
            li: LineIndex::Col(4),
            changes: vec![3]
        }));

        assert_eq!(worker.step(), Err(SolverError::Stuck));

        println!("{}", worker.grid);
    }
}

