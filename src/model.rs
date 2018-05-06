pub enum TileState {
    Unknown,
    Empty,
    Occupied,
}

type LineHints = Vec<usize>;
type PuzzleState = Vec<Vec<TileState>>;

pub struct Puzzle {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
    grid: PuzzleState,
}

impl Puzzle {
    pub fn new(row_hints: Vec<LineHints>, col_hints: Vec<LineHints>) -> Self {
        let w = col_hints.len();
        let h = row_hints.len();

        let mut grid = Vec::with_capacity(h);
        for i in 0..h {
            grid.push(Vec::with_capacity(w));
            for j in 0..w {
                grid[i].push(TileState::Unknown);
            }
        }

        Puzzle {
            row_hints,
            col_hints,
            grid,
        }
    }

    pub fn w(&self) -> usize {
        self.col_hints.len()
    }

    pub fn h(&self) -> usize {
        self.row_hints.len()
    }
}

pub struct PuzzleBuilder {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
}

impl PuzzleBuilder {
    pub fn new() -> Self {
        PuzzleBuilder {
            row_hints: Vec::new(),
            col_hints: Vec::new(),
        }
    }

    pub fn row(mut self, row: LineHints) -> Self {
        self.row_hints.push(row);
        self
    }

    pub fn col(mut self, col: LineHints) -> Self {
        self.col_hints.push(col);
        self
    }

    pub fn gen(&self) -> Puzzle {
        Puzzle::new(self.row_hints.clone(), self.col_hints.clone())
    }

    pub fn freeze(self) -> Puzzle {
        Puzzle::new(self.row_hints, self.col_hints)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
