/// https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
///
/// Stepping takes about 24 nanoseconds per cell on my
/// somewhat slow laptop.

use std::fmt::{Display, Formatter, Error};
use std::ops::{Index, IndexMut};
use std::time::Instant;

struct Board {
    cells: Vec<bool>,
    size: usize
}

impl Board {
    fn new(size: usize) -> Board {
        let mut cells = Vec::new();
        for _ in 0..(size * size) {
            cells.push(false);
        }
        Board{cells: cells, size: size}
    }

    fn count_neighbors(&self, row: usize, col: usize) -> usize {
        let mut count = 0usize;
        if row > 0 && row + 1 < self.size && col > 0 && col + 1 < self.size {
            for i in &[row - 1, row, row + 1] {
                for j in &[col - 1, col, col + 1] {
                    if *i == row && *j == col {
                        continue
                    }
                    if self[(*i, *j)] {
                        count += 1;
                    }
                }
            }
        } else {
            let additions = [0, 1, self.size - 1];
            for i in &additions {
                for j in &additions {
                    if *i == 0 && *j == 0 {
                        continue
                    }
                    if self[((row + *i) % self.size, (col + *j) % self.size)] {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn step(&self) -> Board {
        let mut res = Board::new(self.size);
        for i in 0..self.size {
            for j in 0..self.size {
                let count = self.count_neighbors(i, j);
                if self[(i, j)] {
                    if count == 2 || count == 3 {
                        res[(i, j)] = true;
                    }
                } else if count == 3 {
                    res[(i, j)] = true;
                }
            }
        }
        res
    }
}

impl Index<(usize, usize)> for Board {
    type Output = bool;

    fn index(&self, index: (usize, usize)) -> &bool {
        assert!(index.0 < self.size);
        assert!(index.1 < self.size);
        &self.cells[index.0 * self.size + index.1]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut bool {
        assert!(index.0 < self.size);
        assert!(index.1 < self.size);
        &mut self.cells[index.0 * self.size + index.1]
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for i in 0..self.size {
            if i != 0 {
                write!(f, "\n")?;
            }
            for j in 0..self.size {
                if self[(i, j)] {
                    write!(f, "##")?;
                } else {
                    write!(f, "  ")?;
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut board = Board::new(18);
    create_icolumn(&mut board, 0, 3);
    for _ in 0..16 {
        println!("{}", board);
        println!("-------");
        board = board.step();
    }

    board = Board::new(1000);
    for i in 0..(board.size / 18) {
        for j in 0..(board.size / 18) {
            create_icolumn(&mut board, i*18, j*18);
        }
    }
    let start = Instant::now();
    board.step();
    let elapsed = start.elapsed() / 1000000;
    println!("time: {}ns/cell", elapsed.subsec_nanos());
}

fn create_icolumn(board: &mut Board, row: usize, col: usize) {
    for i in 5..13 {
        for j in 4..7 {
            if j != 5 || (i != 6 && i != 11) {
                board[(i + row, j + col)] = true;
            }
        }
    }
}
