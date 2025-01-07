use raylib::prelude::*;
use super::{Block, BlockKind, Board, Cell, BOARD_HEIGHT, BOARD_WIDTH};
use std::collections::HashMap;

pub const WINDOW_WIDTH: i32 = 750;
pub const WINDOW_HEIGHT: i32 = 800;
pub const FPS: u32 = 60;

// Constants for rendering
pub const CELL_SIZE: i32 = 30;
pub const BOARD_OFFSET_X: i32 = 250;
pub const BOARD_OFFSET_Y: i32 = 50;
pub const PREVIEW_CELL_SIZE: i32 = 25;
pub const BLOCK_ROUNDNESS: f32 = 0.3;
pub const GHOST_ALPHA: u8 = 50;
pub const CELL_PADDING: i32 = 3;

// Scoreboard constants
pub const SCOREBOARD_X: i32 = BOARD_OFFSET_X + (BOARD_WIDTH as i32 * CELL_SIZE) + 30;
pub const SCOREBOARD_Y: i32 = BOARD_OFFSET_Y + 150;
pub const SCOREBOARD_SPACING: i32 = 25;

// Background color
pub const BACKGROUND_COLOR: Color = Color::new(46, 52, 64, 255); // Nord0 - Polar Night
pub const GRID_COLOR: Color = Color::new(59, 66, 82, 255); // Nord1 - Slightly lighter

pub const COLORS: [Color; 7] = [
    Color::new(136, 192, 208, 255), // I - Nord8 - Frost
    Color::new(129, 161, 193, 255), // J - Nord9 - Frost
    Color::new(191, 97, 106, 255),  // L - Nord11 - Aurora
    Color::new(235, 203, 139, 255), // O - Nord13 - Aurora
    Color::new(163, 190, 140, 255), // S - Nord14 - Aurora
    Color::new(180, 142, 173, 255), // T - Nord15 - Aurora
    Color::new(208, 135, 112, 255), // Z - Nord12 - Aurora
];

pub fn draw_rounded_block(d: &mut RaylibDrawHandle, x: i32, y: i32, size: i32, color: Color) {
    d.draw_rectangle_rounded(
        Rectangle::new(
            (x + CELL_PADDING) as f32,
            (y + CELL_PADDING) as f32,
            (size - CELL_PADDING * 2) as f32,
            (size - CELL_PADDING * 2) as f32
        ),
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
        Rectangle::new(
            (x + CELL_PADDING) as f32,
            (y + CELL_PADDING) as f32,
            (size - CELL_PADDING * 2) as f32,
            (size - CELL_PADDING * 2) as f32
        ),
        BLOCK_ROUNDNESS,
        8,
        2.0,
        highlight_color,
    );
}

pub fn draw_block(d: &mut RaylibDrawHandle, block: &Block, offset_x: i32, offset_y: i32) {
    let color = COLORS[block.kind.color() as usize];
    for (x, y) in block.blocks() {
        let screen_x = offset_x + x * CELL_SIZE;
        let screen_y = offset_y + y * CELL_SIZE;
        draw_rounded_block(d, screen_x, screen_y, CELL_SIZE, color);
    }
}

pub fn draw_ghost_block(
    d: &mut RaylibDrawHandle,
    block: &Block,
    board: &Board,
    offset_x: i32,
    offset_y: i32,
) {
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

pub fn draw_preview_block(
    d: &mut RaylibDrawHandle,
    block_kind: BlockKind,
    offset_x: i32,
    offset_y: i32,
) {
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

pub fn draw_board(d: &mut RaylibDrawHandle, board: &Board, offset_x: i32, offset_y: i32) {
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
                        Rectangle::new(
                            (screen_x + CELL_PADDING) as f32,
                            (screen_y + CELL_PADDING) as f32,
                            (CELL_SIZE - CELL_PADDING * 2) as f32,
                            (CELL_SIZE - CELL_PADDING * 2) as f32,
                        ),
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

pub fn draw_scoreboard(
    d: &mut RaylibDrawHandle,
    player_score: u32,
    player_lines: u32,
    player_level: u32,
    other_players: &HashMap<String, i32>,
    current_player_id: Option<&str>,
) {
    // Draw scoreboard title
    d.draw_text(
        "SCOREBOARD",
        SCOREBOARD_X,
        SCOREBOARD_Y,
        25,
        Color::WHITE,
    );

    // Sort all players by score (including current player)
    let mut all_players = Vec::new();
    for (id, score) in other_players {
        all_players.push((id.as_str(), *score));
    }
    if let Some(player_id) = current_player_id {
        all_players.push((player_id, player_score as i32));
    }
    all_players.sort_by(|a, b| b.1.cmp(&a.1));

    // Display top 10 players
    for (i, (player_id, score)) in all_players.iter().take(10).enumerate() {
        let y_offset = SCOREBOARD_Y + SCOREBOARD_SPACING * (2 + i as i32);
        let id_short = &player_id[..6.min(player_id.len())]; // Show first 6 characters of UUID
        
        // Highlight current player
        let (text, color) = if Some(*player_id) == current_player_id {
            (format!("YOU: {}", score), Color::YELLOW)
        } else {
            (format!("{}... : {}", id_short, score), Color::WHITE)
        };

        d.draw_text(
            &text,
            SCOREBOARD_X,
            y_offset,
            20,
            color,
        );
    }

    // Show total player count if there are more players
    let total_players = all_players.len();
    if total_players > 10 {
        let total_y = SCOREBOARD_Y + SCOREBOARD_SPACING * 13;
        d.draw_text(
            &format!("+ {} more players", total_players - 10),
            SCOREBOARD_X,
            total_y,
            20,
            Color::WHITE,
        );
    }

    // Draw player stats
    let stats_y = SCOREBOARD_Y + SCOREBOARD_SPACING * 15;
    d.draw_text(
        "YOUR STATS",
        SCOREBOARD_X,
        stats_y,
        20,
        Color::YELLOW,
    );
    d.draw_text(
        &format!("Lines: {}", player_lines),
        SCOREBOARD_X,
        stats_y + SCOREBOARD_SPACING,
        20,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Level: {}", player_level),
        SCOREBOARD_X,
        stats_y + SCOREBOARD_SPACING * 2,
        20,
        Color::WHITE,
    );
}
