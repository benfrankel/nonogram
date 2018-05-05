pub enum TileState<T> {
    Unknown,
    Empty,
    Full(T),
}

type LineHints<T> = Vec<(usize, T)>;

pub struct Puzzle<'a, T: 'a> {
    row_hints: &'a Vec<LineHints<T>>,
    col_hints: &'a Vec<LineHints<T>>,
    grid: Vec<Vec<TileState<T>>>,
}

pub struct PuzzleTemplate<T> {
    row_hints: Vec<LineHints<T>>,
    col_hints: Vec<LineHints<T>>,
}

impl<T> PuzzleTemplate<T> {
    pub fn new() -> Self {
        PuzzleTemplate {
            row_hints: Vec::new(),
            col_hints: Vec::new(),
        }
    }

    pub fn row(&mut self, row: LineHints<T>) -> &mut Self {
        self.row_hints.push(row);
        self
    }

    pub fn col(&mut self, col: LineHints<T>) -> &mut Self {
        self.col_hints.push(col);
        self
    }

    pub fn gen(&self) -> Puzzle<T> {
        let w = self.col_hints.len();
        let h = self.row_hints.len();

        let mut grid = Vec::with_capacity(h);
        for i in 0..h {
            grid.push(Vec::with_capacity(w));
            for j in 0..w {
                grid[i].push(TileState::Unknown);
            }
        }

        Puzzle {
            row_hints: &self.row_hints,
            col_hints: &self.col_hints,
            grid,
        }
    }
}

impl<'a, T> Puzzle<'a, T> {
    pub fn w(&self) -> usize {
        self.col_hints.len()
    }

    pub fn h(&self) -> usize {
        self.row_hints.len()
    }
}

struct Full;

impl PuzzleTemplate<Full> {
    pub fn push_row(&mut self, row: Vec<usize>) -> Self {
        self.row_hints.push(row);
        self
    }

    pub fn push_col(&mut self, col: Vec<usize>) -> Self {
        self.col_hints.push(col);
        self
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
