use ndarray::ArrayViewMut1;

use model::{Puzzle, PuzzleLine, TileState};


fn overlap(line: PuzzleLine) -> Vec<usize> {
    let mut changes = vec![];

    let hints = line.hints;
    let mut line = line.line;


    let span = hints.iter().sum::<usize>() + hints.len() - 1;
    let flexibility = line.len() - span;

    let mut left = 0;
    for hint in hints.iter() {
        let start = left + flexibility;
        let end = left + hint;

        for i in start..end {
            if line[i] != TileState::Occupied {
                line[i] = TileState::Occupied;
                changes.push(i);
            }
        }

        left = end + 1;
    }

    changes
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_solves_easy_puzzles() {
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

        let mut grid = puzzle.gen();

        assert_eq!(overlap(grid.row(0)), vec![0, 1, 2, 3, 4]);
        assert_eq!(overlap(grid.row(1)), vec![]);
        assert_eq!(overlap(grid.row(2)), vec![0, 1, 2, 3, 4]);
        assert_eq!(overlap(grid.row(3)), vec![]);

        assert_eq!(overlap(grid.col(0)), vec![1, 4]);
        assert_eq!(overlap(grid.col(1)), vec![4]);
        assert_eq!(overlap(grid.col(2)), vec![4]);
        assert_eq!(overlap(grid.col(3)), vec![4]);
        assert_eq!(overlap(grid.col(4)), vec![3, 4]);
    }
}

