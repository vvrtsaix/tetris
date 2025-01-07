use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::Block;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Filled(i32),
}

impl Cell {
    pub fn to_option(&self) -> Option<i32> {
        match self {
            Cell::Empty => None,
            Cell::Filled(value) => Some(*value),
        }
    }

    pub fn from_option(opt: Option<i32>) -> Self {
        match opt {
            None => Cell::Empty,
            Some(value) => Cell::Filled(value),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Board {
    cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    pub fn get_cells_for_network(&self) -> Vec<Vec<Option<i32>>> {
        let mut result = vec![vec![None; BOARD_WIDTH]; BOARD_HEIGHT];
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                result[y][x] = self.cells[y][x].to_option();
            }
        }
        result
    }

    pub fn update_from_network(&mut self, cells: Vec<Vec<Option<i32>>>) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if let Some(cell) = cells.get(y).and_then(|row| row.get(x)) {
                    self.cells[y][x] = Cell::from_option(*cell);
                }
            }
        }
    }

    pub fn add_garbage_lines(&mut self, count: i32) {
        for _ in 0..count {
            // Shift all rows up
            for y in (1..BOARD_HEIGHT).rev() {
                for x in 0..BOARD_WIDTH {
                    self.cells[y][x] = self.cells[y - 1][x];
                }
            }

            // Add garbage line at bottom with one random hole
            let hole = rand::thread_rng().gen_range(0..BOARD_WIDTH);
            for x in 0..BOARD_WIDTH {
                self.cells[0][x] = if x == hole {
                    Cell::Empty
                } else {
                    Cell::Filled(8)
                }; // 8 represents garbage block
            }
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<Cell> {
        if row < BOARD_HEIGHT && col < BOARD_WIDTH {
            Some(self.cells[row][col])
        } else {
            None
        }
    }

    pub fn is_valid_position(&self, block: &Block) -> bool {
        block.blocks().iter().all(|&(x, y)| {
            let x = x as usize;

            // Check horizontal bounds
            if x >= BOARD_WIDTH {
                return false;
            }

            // Allow blocks to be above the board
            if y < 0 {
                return true;
            }

            let y = y as usize;
            // Check vertical bounds and collision
            if y >= BOARD_HEIGHT {
                return false;
            }

            // Check collision with existing blocks
            matches!(self.cells[y][x], Cell::Empty)
        })
    }

    pub fn place_block(&mut self, block: &Block) -> bool {
        if !self.is_valid_position(block) {
            return false;
        }

        // Place the block
        for (x, y) in block.blocks() {
            if y < 0 {
                return false;
            }
            self.cells[y as usize][x as usize] = Cell::Filled(block.kind.color() as i32);
        }
        true
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut lines_cleared = 0;
        let mut y = 0;
        while y < BOARD_HEIGHT {
            if self.is_line_complete(y) {
                self.remove_line(y);
                lines_cleared += 1;
            } else {
                y += 1;
            }
        }
        lines_cleared
    }

    fn is_line_complete(&self, y: usize) -> bool {
        (0..BOARD_WIDTH).all(|x| matches!(self.cells[y][x], Cell::Filled(_)))
    }

    fn remove_line(&mut self, y: usize) {
        // Move all lines above down
        for row in (1..=y).rev() {
            for x in 0..BOARD_WIDTH {
                self.cells[row][x] = self.cells[row - 1][x];
            }
        }
        // Clear top line
        for x in 0..BOARD_WIDTH {
            self.cells[0][x] = Cell::Empty;
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                match self.cells[row][col] {
                    Cell::Empty => write!(f, " ")?,
                    Cell::Filled(_) => write!(f, "#")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
