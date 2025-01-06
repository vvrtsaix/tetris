use raylib::prelude::*;

mod tetris;
use tetris::*;

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
        // Handle input
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

        // Render
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
