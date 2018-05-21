use std::collections::HashMap;

use ndarray::{ArrayViewMut1, Array2};

use model::{
    Square,
    Puzzle,
    LineIndex,
};


enum SolverError {
    Solved,
    Invalid,
    Stuck,
}

type PartialSquare = Option<Square>;
type PartialGrid = Array2<PartialSquare>;

// The corresponding run of full squares must:
// - Lie within [lo, hi)
// - Contain [start, end)
#[derive(Clone)]
struct PartialRun {
    lo: usize,
    hi: usize,
    found: Option<(usize, usize)>,
}

impl PartialRun {
    pub fn new(hi: usize) -> Self {
        PartialRun {
            lo: 0,
            hi,
            found: None,
        }
    }
}

struct PuzzleLine<'a> {
    hints: &'a [usize],
    runs: &'a mut [PartialRun],
    line: ArrayViewMut1<'a, PartialSquare>,
    // index: LineIndex
}

struct DeductionStep {
    line: LineIndex,
    modified: Vec<usize>,
}

struct Solver {
    deductions: Vec<Box<Fn(PuzzleLine) -> Option<Vec<usize>>>>,

    // Keep track of other solved features (ex., some tile is part of some run)
    // Deductions should be able to accept extra features
    // I don't know yet what features will be helpful for certain deductions
}

type Solution = Vec<DeductionStep>;

struct SolverWorker<'a> {
    solver: &'a Solver,
    puzzle: &'a Puzzle,
    grid: PartialGrid,
    dirty: HashMap<LineIndex, bool>,
    runs: HashMap<LineIndex, Vec<PartialRun>>,
    steps: Solution,
}

impl<'a> SolverWorker<'a> {
    fn new(solver: &'a Solver, puzzle: &'a Puzzle) -> SolverWorker<'a> {
        let mut dirty = HashMap::with_capacity(puzzle.w() + puzzle.h());
        let mut runs = HashMap::with_capacity(puzzle.w() + puzzle.h());

        for li in puzzle.index_iter() {
            dirty[&li] = true;
            runs[&li] = vec![PartialRun::new(puzzle.w()); puzzle.line(li).len()];
        }

        SolverWorker {
            solver,
            puzzle,
            grid: Array2::from_elem((puzzle.h(), puzzle.w()), None),
            dirty,
            runs,
            steps: Vec::new(),
        }
    }

    fn step(&mut self) -> Result<DeductionStep, SolverError> {
        for li in self.puzzle.index_iter() {
            let line = self.puzzle.line(li);

            for deduction in self.solver.deductions.iter() {
                if let Some(changes) = deduction(line) {
                    for change in changes {
                        self.dirty[&li.line_through(change)] = true;
                    }
                }
            }
        }

        Err(SolverError::Stuck)
    }

    fn solve(&mut self) -> Result<Solution, SolverError> {
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
        SolverWorker::new(self, &puzzle)
    }
}

fn deduce_overlap(line: PuzzleLine) -> Option<Vec<usize>> {
    let mut changes = Vec::new();

    let hints = line.hints;
    let mut line = line.line;

    let span = hints.iter().sum::<usize>() + hints.len() - 1;
    let flexibility = line.len() - span;

    let mut left = 0;
    for (i, hint) in hints.iter().enumerate() {
        let start = left + flexibility;
        let end = left + hint;

        for j in start..end {
            if line[j] != Some(Square::Full) {
                line[j] = Some(Square::Full);
                changes.push(j);
            }
        }

        left = end + 1;
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
    fn overlap_solves_easy_puzzles() {
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

        let mut grid = puzzle.gen();

        assert_eq!(deduce_overlap(grid.row(0)), Some(vec![0, 1, 2, 3, 4]));
        assert_eq!(deduce_overlap(grid.row(1)), None);
        assert_eq!(deduce_overlap(grid.row(2)), Some(vec![0, 1, 2, 3, 4]));
        assert_eq!(deduce_overlap(grid.row(3)), None);

        assert_eq!(deduce_overlap(grid.col(0)), Some(vec![1, 4]));
        assert_eq!(deduce_overlap(grid.col(1)), Some(vec![4]));
        assert_eq!(deduce_overlap(grid.col(2)), Some(vec![4]));
        assert_eq!(deduce_overlap(grid.col(3)), Some(vec![4]));
        assert_eq!(deduce_overlap(grid.col(4)), Some(vec![3, 4]));
    }
}

