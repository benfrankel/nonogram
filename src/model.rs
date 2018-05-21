use std::fmt;

use ndarray::{
    Array2,
    ArrayViewMut1,
    Axis,
};


pub struct Puzzle {
    row_hints: Vec<Vec<usize>>,
    col_hints: Vec<Vec<usize>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Square {
    Empty,
    Full,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum LineIndex {
    Row(usize),
    Col(usize),
}

pub struct Grid<T>(pub Array2<T>);

impl<T> Grid<T> {
    pub fn h(&self) -> usize {
        self.0.len_of(Axis(0))
    }
    pub fn w(&self) -> usize {
        self.0.len_of(Axis(1))
    }

    pub fn line<'a>(&'a mut self, li: LineIndex) -> ArrayViewMut1<'a, T> {
        match li {
            LineIndex::Row(i) => self.0.slice_mut(s![i, ..]),
            LineIndex::Col(j) => self.0.slice_mut(s![.., j]),
        }
    }
}

impl LineIndex {
    pub fn line_through(&self, k: usize) -> LineIndex {
        match self {
            LineIndex::Row(_) => LineIndex::Col(k),
            LineIndex::Col(_) => LineIndex::Row(k),
        }
    }
}

pub struct LineIndexIterator {
    w: usize,
    h: usize,
    li: Option<LineIndex>,
}

impl<'a> Iterator for LineIndexIterator {
    type Item = LineIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.li {
            None => None,

            Some(li) => {
                self.li = match li {
                    LineIndex::Row(ref i) if i + 1 < self.h =>
                        Some(LineIndex::Row(i + 1)),

                    LineIndex::Row(ref i) if i + 1 >= self.h && self.w >= 1 =>
                        Some(LineIndex::Col(0)),

                    LineIndex::Col(ref j) if j + 1 < self.w =>
                        Some(LineIndex::Col(j + 1)),

                    _ => None,
                };

                Some(li)
            }
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Square::Empty => write!(f, "x"),
            Square::Full  => write!(f, "█"),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Grid<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_owned();

        s.push('┌');
        for _ in 0..self.w() {
            s.push('─');
        }
        s.push('┐');

        s.push('\n');
        for row in self.0.outer_iter() {
            s.push('│');
            for square in row.iter() {
                s.push_str(&format!("{}", square));
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

//impl<'a> fmt::Display for PuzzleLineViewMut<'a> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        let mut s = "".to_owned();
//
//        s.push('|');
//        for tile in self.line.iter() {
//            s.push_str(&format!("{}", tile));
//        }
//        s.push('|');
//
//        for hint in self.hints.iter() {
//            s.push_str(&format!(" {}", hint));
//        }
//
//        write!(f, "{}", s)
//    }
//}

impl Puzzle {
    pub fn new() -> Self {
        Puzzle {
            row_hints: Vec::new(),
            col_hints: Vec::new(),
        }
    }

    pub fn with_capacity(w: usize, h: usize) -> Self {
        Puzzle {
            row_hints: Vec::with_capacity(h),
            col_hints: Vec::with_capacity(w),
        }
    }

    pub fn w(&self) -> usize {
        self.col_hints.len()
    }

    pub fn h(&self) -> usize {
        self.row_hints.len()
    }

    pub fn push_row(mut self, hints: Vec<usize>) -> Self {
        self.row_hints.push(hints);
        self
    }

    pub fn push_col(mut self, hints: Vec<usize>) -> Self {
        self.col_hints.push(hints);
        self
    }

    pub fn hints(&self, li: LineIndex) -> &[usize] {
        match li {
            LineIndex::Row(i) => &self.row_hints[i],
            LineIndex::Col(j) => &self.col_hints[j],
        }
    }

    pub fn index_iter(&self) -> LineIndexIterator {
        // FIXME: Require that Row(0) is valid (>= 1 rows)
        LineIndexIterator {
            w: self.w(),
            h: self.h(),
            li: Some(LineIndex::Row(0)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_puzzle_grid_has_correct_dimensions() {
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

        assert_eq!(puzzle.w(), 5);
        assert_eq!(puzzle.h(), 5);
    }
}
