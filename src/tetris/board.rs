use std::fmt;

use super::Block;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Filled(u8),
}

pub struct Board {
    cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
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

    pub fn is_valid_spawn_position(&self, block: &Block) -> bool {
        block.blocks().iter().all(|&(x, y)| {
            let x = x as usize;

            // Only check horizontal bounds and collisions for parts inside the board
            if x >= BOARD_WIDTH {
                return false;
            }

            // If the block part is inside the board, check for collision
            if y >= 0 {
                let y = y as usize;
                if y < BOARD_HEIGHT {
                    return matches!(self.cells[y][x], Cell::Empty);
                }
            }

            // Allow parts to be above the board
            true
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
            self.cells[y as usize][x as usize] = Cell::Filled(block.kind.color());
        }
        true
    }

    pub fn is_row_full(&self, row: usize) -> bool {
        if row >= BOARD_HEIGHT {
            return false;
        }
        self.cells[row]
            .iter()
            .all(|&cell| matches!(cell, Cell::Filled(_)))
    }

    pub fn clear_row(&mut self, row: usize) {
        if row >= BOARD_HEIGHT {
            return;
        }

        // Move all rows above the cleared row down
        for y in (1..=row).rev() {
            self.cells[y] = self.cells[y - 1];
        }

        // Fill the top row with empty cells
        self.cells[0] = [Cell::Empty; BOARD_WIDTH];
    }

    pub fn clear_full_rows(&mut self) -> u32 {
        let mut cleared_rows = 0;
        let mut row = BOARD_HEIGHT - 1;

        while row > 0 {
            if self.is_row_full(row) {
                self.clear_row(row);
                cleared_rows += 1;
            } else {
                row -= 1;
            }
        }

        cleared_rows
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
