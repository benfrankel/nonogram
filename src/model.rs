use std::fmt;

use ndarray::{Array2, ArrayViewMut1};


#[derive(Clone, PartialEq)]
pub enum TileState {
    Unknown,
    Empty,
    Occupied,
}

pub struct PuzzleGrid<'a> {
    puzzle: &'a Puzzle,
    grid: Array2<TileState>,
}

pub struct PuzzleLine<'a> {
    pub hints: &'a LineHints,
    pub line: ArrayViewMut1<'a, TileState>,
}

impl<'a> PuzzleGrid<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        PuzzleGrid {
            puzzle,
            grid: Array2::from_elem((puzzle.h(), puzzle.w()), TileState::Unknown),
        }
    }

    pub fn w(&self) -> usize {
        self.puzzle.w()
    }

    pub fn h(&self) -> usize {
        self.puzzle.h()
    }

    pub fn row(&mut self, i: usize) -> PuzzleLine {
        PuzzleLine {
            hints: &self.puzzle.row_hints[i],
            line: self.grid.slice_mut(s![i, ..])
        }
    }

    pub fn col(&mut self, j: usize) -> PuzzleLine {
        PuzzleLine {
            hints: &self.puzzle.col_hints[j],
            line: self.grid.slice_mut(s![.., j])
        }
    }
}

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TileState::Unknown  => write!(f, " "),
            TileState::Empty    => write!(f, "x"),
            TileState::Occupied => write!(f, "█"),
        }
    }
}

impl<'a> fmt::Display for PuzzleLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_owned();

        s.push('|');
        for tile in self.line.iter() {
            s.push_str(&format!("{}", tile));
        }
        s.push('|');

        for hint in self.hints.iter() {
            s.push_str(&format!(" {}", hint));
        }

        write!(f, "{}", s)
    }
}

impl<'a> fmt::Display for PuzzleGrid<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_owned();

        s.push('┌');
        for _ in 0..self.w() {
            s.push('─');
        }
        s.push('┐');

        s.push('\n');
        for row in self.grid.outer_iter() {
            s.push('│');
            for tile in row.iter() {
                s.push_str(&format!("{}", tile));
            }
            s.push('│');
            s.push('\n');
        }

        s.push('└');
        for _ in 0..self.w() {
            s.push('─');
        }
        s.push('┘');

        write!(f, "{}", s)
    }
}

pub type LineHints = Vec<usize>;

pub struct Puzzle {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
}

impl Puzzle {
    pub fn new() -> Self {
        Puzzle {
            row_hints: Vec::new(),
            col_hints: Vec::new(),
        }
    }

    pub fn w(&self) -> usize {
        self.col_hints.len()
    }

    pub fn h(&self) -> usize {
        self.row_hints.len()
    }

    pub fn row(mut self, row: LineHints) -> Self {
        self.row_hints.push(row);
        self
    }

    pub fn col(mut self, col: LineHints) -> Self {
        self.col_hints.push(col);
        self
    }

    pub fn gen(&self) -> PuzzleGrid {
        PuzzleGrid::new(&self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_puzzle_grid_has_correct_dimensions() {
        let puzzle = Puzzle::new()
            .row(vec!(5))
            .row(vec!(1))
            .row(vec!(5))
            .row(vec!(1))
            .row(vec!(5))
            .col(vec!(3, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 3));

        let grid = puzzle.gen();

        assert_eq!(puzzle.w(), 5);
        assert_eq!(puzzle.h(), 5);
    }

    #[test]
    fn new_puzzle_grid_has_all_unknown_tiles() {
        let puzzle = Puzzle::new()
            .row(vec!(5))
            .row(vec!(1))
            .row(vec!(5))
            .row(vec!(1))
            .row(vec!(5))
            .col(vec!(3, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 1, 1))
            .col(vec!(1, 3));

        let grid = puzzle.gen();

        for tile in grid.grid.iter() {
            assert!(match *tile {
                TileState::Unknown => true,
                _ => false,
            });
        }
    }
}
