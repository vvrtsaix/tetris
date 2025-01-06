#![allow(unused)]

use raylib::prelude::*;
use std::time::{Duration, Instant};

mod tetris;
use tetris::*;

const WINDOW_WIDTH: i32 = 750;
const WINDOW_HEIGHT: i32 = 800;
const FPS: u32 = 60;

// Constants for rendering
const CELL_SIZE: i32 = 30;
const BOARD_OFFSET_X: i32 = 250;
const BOARD_OFFSET_Y: i32 = 50;
const PREVIEW_CELL_SIZE: i32 = 25;
const BLOCK_ROUNDNESS: f32 = 0.3;
const GHOST_ALPHA: u8 = 50;

// Background color
const BACKGROUND_COLOR: Color = Color::new(46, 52, 64, 255);    // Nord0 - Polar Night
const GRID_COLOR: Color = Color::new(59, 66, 82, 255);          // Nord1 - Slightly lighter

// Key repeat timing constants
const KEY_REPEAT_DELAY: Duration = Duration::from_millis(150);
const KEY_REPEAT_RATE: Duration = Duration::from_millis(30);
const ROTATION_REPEAT_DELAY: Duration = Duration::from_millis(200);
const ROTATION_REPEAT_RATE: Duration = Duration::from_millis(150);

const COLORS: [Color; 7] = [
    Color::new(136, 192, 208, 255),   // I - Nord8 - Frost
    Color::new(129, 161, 193, 255),   // J - Nord9 - Frost
    Color::new(191, 97, 106, 255),    // L - Nord11 - Aurora
    Color::new(235, 203, 139, 255),   // O - Nord13 - Aurora
    Color::new(163, 190, 140, 255),   // S - Nord14 - Aurora
    Color::new(180, 142, 173, 255),   // T - Nord15 - Aurora
    Color::new(208, 135, 112, 255),   // Z - Nord12 - Aurora
];

struct KeyState {
    last_press: Instant,
    is_pressed: bool,
    is_rotation: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            last_press: Instant::now(),
            is_pressed: false,
            is_rotation: false,
        }
    }
}

impl KeyState {
    fn new(is_rotation: bool) -> Self {
        Self {
            last_press: Instant::now(),
            is_pressed: false,
            is_rotation,
        }
    }

    fn update(&mut self, is_down: bool) -> bool {
        let now = Instant::now();
        let (repeat_delay, repeat_rate) = if self.is_rotation {
            (ROTATION_REPEAT_DELAY, ROTATION_REPEAT_RATE)
        } else {
            (KEY_REPEAT_DELAY, KEY_REPEAT_RATE)
        };

        let should_trigger = if is_down {
            if !self.is_pressed {
                self.last_press = now;
                true
            } else {
                let elapsed = now.duration_since(self.last_press);
                if elapsed >= repeat_delay {
                    let repeat_elapsed = elapsed - repeat_delay;
                    if repeat_elapsed >= repeat_rate {
                        self.last_press = now - repeat_delay;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        } else {
            if self.is_pressed {
                self.last_press = now;
            }
            false
        };

        self.is_pressed = is_down;
        should_trigger
    }
}

fn draw_rounded_block(d: &mut RaylibDrawHandle, x: i32, y: i32, size: i32, color: Color) {
    d.draw_rectangle_rounded(
        Rectangle::new(x as f32, y as f32, (size - 1) as f32, (size - 1) as f32),
        BLOCK_ROUNDNESS,
        8,
        color,
    );
    
    let highlight_color = Color::new(
        (color.r as u16 + 40).min(255) as u8,
        (color.g as u16 + 40).min(255) as u8,
        (color.b as u16 + 40).min(255) as u8,
        color.a,
    );
    d.draw_rectangle_rounded_lines(
        Rectangle::new(x as f32, y as f32, (size - 1) as f32, (size - 1) as f32),
        BLOCK_ROUNDNESS,
        8,
        2.0,
        highlight_color,
    );
}

fn draw_block(d: &mut RaylibDrawHandle, block: &Block, offset_x: i32, offset_y: i32) {
    let color = COLORS[block.kind.color() as usize];
    for (x, y) in block.blocks() {
        let screen_x = offset_x + x * CELL_SIZE;
        let screen_y = offset_y + y * CELL_SIZE;
        draw_rounded_block(d, screen_x, screen_y, CELL_SIZE, color);
    }
}

fn draw_ghost_block(d: &mut RaylibDrawHandle, block: &Block, board: &Board, offset_x: i32, offset_y: i32) {
    let mut ghost = *block;
    while board.is_valid_position(&ghost) {
        ghost.y += 1;
    }
    ghost.y -= 1;

    let color = COLORS[block.kind.color() as usize];
    let ghost_color = Color::new(color.r, color.g, color.b, GHOST_ALPHA);
    
    for (x, y) in ghost.blocks() {
        let screen_x = offset_x + x * CELL_SIZE;
        let screen_y = offset_y + y * CELL_SIZE;
        draw_rounded_block(d, screen_x, screen_y, CELL_SIZE, ghost_color);
    }
}

fn draw_preview_block(d: &mut RaylibDrawHandle, block_kind: BlockKind, offset_x: i32, offset_y: i32) {
    let color = COLORS[block_kind.color() as usize];
    let base_positions = match block_kind {
        BlockKind::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
        BlockKind::J => [(-1, -1), (-1, 0), (0, 0), (1, 0)],
        BlockKind::L => [(1, -1), (-1, 0), (0, 0), (1, 0)],
        BlockKind::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
        BlockKind::S => [(-1, 0), (0, 0), (0, -1), (1, -1)],
        BlockKind::T => [(0, -1), (-1, 0), (0, 0), (1, 0)],
        BlockKind::Z => [(-1, -1), (0, -1), (0, 0), (1, 0)],
    };

    for (x, y) in base_positions {
        let screen_x = offset_x + (x + 1) * PREVIEW_CELL_SIZE;
        let screen_y = offset_y + (y + 1) * PREVIEW_CELL_SIZE;
        draw_rounded_block(d, screen_x, screen_y, PREVIEW_CELL_SIZE, color);
    }
}

fn draw_board(d: &mut RaylibDrawHandle, board: &Board, offset_x: i32, offset_y: i32) {
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let screen_x = offset_x + (x as i32) * CELL_SIZE;
            let screen_y = offset_y + (y as i32) * CELL_SIZE;
            
            match board.get_cell(y, x) {
                Some(Cell::Filled(color)) => {
                    draw_rounded_block(d, screen_x, screen_y, CELL_SIZE, COLORS[color as usize]);
                }
                _ => {
                    d.draw_rectangle_rounded_lines(
                        Rectangle::new(screen_x as f32, screen_y as f32, CELL_SIZE as f32, CELL_SIZE as f32),
                        0.1,
                        4,
                        1.0,
                        GRID_COLOR,
                    );
                }
            }
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tetris")
        .vsync()
        .build();

    rl.set_target_fps(FPS);
    
    let mut game = Game::default();
    game.start_game();

    let mut left_key = KeyState::new(false);
    let mut right_key = KeyState::new(false);
    let mut down_key = KeyState::new(false);
    let mut rotate_key = KeyState::new(true);

    while !rl.window_should_close() {
        if game.state == GameState::Playing {
            let mut moved = false;
            
            if left_key.update(rl.is_key_down(KeyboardKey::KEY_LEFT)) {
                moved = game.move_current_block(-1, 0);
            }
            if right_key.update(rl.is_key_down(KeyboardKey::KEY_RIGHT)) && !moved {
                game.move_current_block(1, 0);
            }
            if rotate_key.update(rl.is_key_down(KeyboardKey::KEY_UP)) {
                game.rotate_current_block();
            }
            
            game.timer.soft_drop = down_key.update(rl.is_key_down(KeyboardKey::KEY_DOWN));

            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                game.hard_drop();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_pressed(KeyboardKey::KEY_C) {
                if let Some(held_block) = game.hold_block {
                    let mut temp = held_block;
                    temp.reset();
                    game.hold_block = Some(game.current_block);
                    game.current_block = temp;
                } else {
                    game.hold_block = Some(game.current_block);
                    game.current_block = game.next_block;
                    game.next_block = Block::new(BlockKind::random());
                }
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            game.toggle_pause();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) && game.state == GameState::GameOver {
            game.start_game();
        }

        game.update();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        draw_board(&mut d, &game.board, BOARD_OFFSET_X, BOARD_OFFSET_Y);
        
        if game.state == GameState::Playing {
            draw_ghost_block(&mut d, &game.current_block, &game.board, BOARD_OFFSET_X, BOARD_OFFSET_Y);
            draw_block(&mut d, &game.current_block, BOARD_OFFSET_X, BOARD_OFFSET_Y);
        }

        d.draw_text("Next:", BOARD_OFFSET_X + (BOARD_WIDTH as i32 * CELL_SIZE) + 30, 
            BOARD_OFFSET_Y, 20, Color::WHITE);
        draw_preview_block(&mut d, game.next_block.kind, 
            BOARD_OFFSET_X + (BOARD_WIDTH as i32 * CELL_SIZE) + 30, 
            BOARD_OFFSET_Y + 30);

        d.draw_text("Hold:", 20, BOARD_OFFSET_Y + 100, 20, Color::WHITE);
        if let Some(held_block) = &game.hold_block {
            draw_preview_block(&mut d, held_block.kind, 20, BOARD_OFFSET_Y + 130);
        }

        d.draw_text(&format!("Score: {}", game.score.points), 20, BOARD_OFFSET_Y, 20, Color::WHITE);
        d.draw_text(&format!("Level: {}", game.score.level), 20, BOARD_OFFSET_Y + 30, 20, Color::WHITE);
        d.draw_text(&format!("Lines: {}", game.score.lines), 20, BOARD_OFFSET_Y + 60, 20, Color::WHITE);

        match game.state {
            GameState::Paused => {
                d.draw_text("PAUSED", WINDOW_WIDTH/2 - 50, WINDOW_HEIGHT/2, 30, Color::WHITE);
                d.draw_text("Press P to resume", WINDOW_WIDTH/2 - 80, WINDOW_HEIGHT/2 + 40, 20, Color::WHITE);
            }
            GameState::GameOver => {
                d.draw_text("GAME OVER", WINDOW_WIDTH/2 - 70, WINDOW_HEIGHT/2, 30, Color::WHITE);
                d.draw_text("Press R to restart", WINDOW_WIDTH/2 - 80, WINDOW_HEIGHT/2 + 40, 20, Color::WHITE);
            }
            _ => {}
        }
    }
}
