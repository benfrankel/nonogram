use std::fmt;


pub enum TileState {
    Unknown,
    Empty,
    Occupied,
}

type LineHints = Vec<usize>;
type GridState = Vec<Vec<TileState>>;

pub struct PuzzleGrid<'a> {
    puzzle: &'a Puzzle,
    grid: GridState,
}

impl<'a> PuzzleGrid<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        let mut grid = Vec::with_capacity(puzzle.h());

        for i in 0..puzzle.h() {
            grid.push(Vec::with_capacity(puzzle.w()));
            for _ in 0..puzzle.w() {
                grid[i].push(TileState::Unknown);
            }
        }

        PuzzleGrid {
            puzzle,
            grid,
        }
    }

    pub fn w(&self) -> usize {
        self.puzzle.w()
    }

    pub fn h(&self) -> usize {
        self.puzzle.h()
    }
}

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TileState::Unknown  => write!(f, " "),
            &TileState::Empty    => write!(f, "X"),
            &TileState::Occupied => write!(f, "O"),
        }
    }
}

impl<'a> fmt::Display for PuzzleGrid<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_owned();

        for i in 0..self.h() {
            for j in 0..self.w() {
                s.push_str(&format!("{}", self.grid[i][j]));
            }
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

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
    fn new_puzzle_grid_all_tiles_are_unknown() {
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

        for row in grid.grid {
            for tile in row {
                assert!((match tile {
                    TileState::Unknown => true,
                    _ => false,
                }));
            }
        }
    }
}
