use std::collections::{HashMap, HashSet, VecDeque};
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
enum PartialSquare {
    Unknown,
    Known(Square),
}

impl PartialSquare {
    fn collapse(self) -> Square {
        match self {
            PartialSquare::Unknown => panic!("Cannot collapse Partial::Unknown"),
            PartialSquare::Known(inner) => inner,
        }
    }

    fn is_known(&self) -> bool {
        match self {
            PartialSquare::Unknown => false,
            PartialSquare::Known(_) => true,
        }
    }

    fn reveal(&mut self, x: Square) -> bool {
        let res = match *self {
            PartialSquare::Known(old) if old == x => false,
            _ => true,
        };

        *self = PartialSquare::Known(x);

        res
    }
}

impl fmt::Display for PartialSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PartialSquare::Unknown  => write!(f, " "),
            PartialSquare::Known(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
struct PartialRun {
    lo: usize,
    hi: usize,
}

impl PartialRun {
    pub fn new() -> Self {
        PartialRun {
            lo: 0,
            hi: usize::max_value(),
        }
    }

    fn update(&mut self, lo: usize, hi: usize) {
        self.lo = max(self.lo, lo);
        self.hi = min(self.hi, hi);
    }
}

struct PartialLine<'a> {
    hints: &'a [usize],
    runs: &'a mut [PartialRun],
    line: ArrayViewMut1<'a, PartialSquare>,
    dirty: HashSet<usize>,
}

impl<'a> PartialLine<'a> {
    fn reveal(&mut self, i: usize, x: Square) {
        if self.line[i].reveal(x) {
            self.dirty.insert(i);
        }
    }

    fn reveal_all<I>(&mut self, bag: I, x: Square)
        where I: IntoIterator<Item = usize> {
        for i in bag.into_iter() {
            self.reveal(i, x);
        }
    }

    fn reveal_run(&mut self, run_index: usize, lo: usize, hi: usize) {
        self.reveal_all(lo..hi, Square::Full);

        if hi > self.hints[run_index] {
            self.runs[run_index].update(hi - self.hints[run_index],
                                        lo + self.hints[run_index]);
        } else {
            self.runs[run_index].update(0,
                                        lo + self.hints[run_index]);
        }
    }
}

struct SolverWorker<'a> {
    solver: &'a Solver,
    puzzle: &'a Puzzle,
    grid: Grid<PartialSquare>,
    queue: VecDeque<LineIndex>,
    runs: HashMap<LineIndex, Vec<PartialRun>>,

    // Keep track of other solved features (ex., some tile is part of some run)
    // Deductions should be able to accept extra features
    // I don't know yet what features will be helpful for certain deductions
}

impl<'a> SolverWorker<'a> {
    fn new(solver: &'a Solver, puzzle: &'a Puzzle) -> SolverWorker<'a> {
        let mut queue = VecDeque::with_capacity(puzzle.w() + puzzle.h());
        let mut runs = HashMap::with_capacity(puzzle.w() + puzzle.h());

        for li in puzzle.index_iter() {
            queue.push_back(li);
            runs.insert(li, vec![PartialRun::new(); puzzle.hints(li).len()]);
        }

        SolverWorker {
            solver,
            puzzle,
            grid: Grid(Array2::from_elem((puzzle.h(), puzzle.w()), PartialSquare::Unknown)),
            queue,
            runs,
        }
    }

    fn line(puzzle: &'a Puzzle,
            runs: &'a mut HashMap<LineIndex, Vec<PartialRun>>,
            grid: &'a mut Grid<PartialSquare>,
            li: LineIndex) -> PartialLine<'a> {
        PartialLine {
            hints: puzzle.hints(li),
            runs: runs.get_mut(&li).unwrap(),
            line: grid.line(li),
            dirty: HashSet::new(),
        }
    }

    fn verify(&self) -> SolverError {
        // TODO: Possibly return SolverError::Invalid (that is, perform validation!)

        if self.grid.0.iter().any(|x| *x == PartialSquare::Unknown) {
            SolverError::Stuck
        } else {
            SolverError::Solved
        }
    }

    fn step(&mut self) -> Result<(), SolverError> {
        while let Some(li) = self.queue.pop_front() {
            let mut line = SolverWorker::line(self.puzzle, &mut self.runs, &mut self.grid, li);

            loop {
                let num_reveals = line.dirty.len();

                for deduction in &self.solver.deductions {
                    deduction(&mut line);
                }

                if line.dirty.len() == num_reveals {
                    break;
                }
            }

            if !line.dirty.is_empty() {
                self.queue.extend(line.dirty
                    .into_iter()
                    .map(move |i| li.line_through(i)));

                return Ok(())
            }
        }

        Err(self.verify())
    }

    fn solve(mut self) -> Result<(), SolverError> {
        loop {
            println!("{}", self.grid);

            match self.step() {
                Err(SolverError::Solved) => return Ok(()),
                Err(e) => return Err(e),
                _ => (),
            }
        }
    }
}

pub struct Solver {
    deductions: Vec<Box<Fn(&mut PartialLine)>>,
}

impl Solver {
    fn new() -> Self {
        Solver {
            deductions: vec![Box::new(deduce_overlap),
                             Box::new(deduce_run_gaps)],
        }
    }

    fn delegate<'a>(&'a self, puzzle: &'a Puzzle) -> SolverWorker<'a> {
        SolverWorker::new(self, puzzle)
    }
}


fn deduce_overlap(partial: &mut PartialLine) {
    let gap_span = if partial.hints.is_empty() { 0 } else { partial.hints.len() - 1 };
    let span = partial.hints.iter().sum::<usize>() + gap_span;
    let flexibility = partial.line.len() - span;

    let mut left = 0;
    for (i, hint) in partial.hints.iter().enumerate() {
        let lo = left + flexibility;
        let hi = left + hint;

        if lo < hi {
            partial.reveal_run(i, lo, hi);
        }

        left = hi + 1;
    }
}

fn deduce_run_gaps(partial: &mut PartialLine) {
    let end = partial.line.len();

    if partial.runs.is_empty() {
        partial.reveal_all(0..end, Square::Empty);
    } else {
        let lo = partial.runs.first().unwrap().lo;
        let hi = partial.runs.last().unwrap().hi;

        partial.reveal_all(0..lo, Square::Empty);
        partial.reveal_all(hi..end, Square::Empty);

        for i in 0..partial.runs.len() - 1 {
            let lo = partial.runs[i].hi;
            let hi = partial.runs[i + 1].lo;

            partial.reveal_all(lo..hi, Square::Empty);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy_puzzle_snake() {
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
        let worker = solver.delegate(&puzzle);

        assert!(worker.solve().is_ok());
    }

    #[test]
    fn easy_puzzle_checkerboard() {
        let puzzle = Puzzle::new()
            .push_row(vec!(1, 1, 1))
            .push_row(vec!(1, 1))
            .push_row(vec!(1, 1, 1))
            .push_row(vec!(1, 1))
            .push_row(vec!(1, 1, 1))
            .push_col(vec!(1, 1, 1))
            .push_col(vec!(1, 1))
            .push_col(vec!(1, 1, 1))
            .push_col(vec!(1, 1))
            .push_col(vec!(1, 1, 1));

        let solver = Solver::new();
        let worker = solver.delegate(&puzzle);

        assert!(worker.solve().is_ok());
    }

    #[test]
    fn easy_puzzle_stairs() {
        let puzzle = Puzzle::new()
            .push_row(vec!(2))
            .push_row(vec!(3))
            .push_row(vec!(2, 1))
            .push_row(vec!(2, 1))
            .push_row(vec!(5))
            .push_col(vec!(2))
            .push_col(vec!(3))
            .push_col(vec!(2, 1))
            .push_col(vec!(2, 1))
            .push_col(vec!(5));

        let solver = Solver::new();
        let worker = solver.delegate(&puzzle);

        assert!(worker.solve().is_ok());
    }

    #[test]
    fn nonlinear_puzzle_smiley() {
        let puzzle = Puzzle::new()
            .push_row(vec!(2, 2))
            .push_row(vec!(2, 2))
            .push_row(vec!())
            .push_row(vec!(1, 1))
            .push_row(vec!(3))
            .push_col(vec!(2, 1))
            .push_col(vec!(2, 1))
            .push_col(vec!(1))
            .push_col(vec!(2, 1))
            .push_col(vec!(2, 1));

        let solver = Solver::new();
        let worker = solver.delegate(&puzzle);

        assert_eq!(worker.solve(), Err(SolverError::Stuck));
    }
}

