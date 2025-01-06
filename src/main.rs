use raylib::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

mod tetris;
use tetris::*;

struct SoundEffects {
    move_sink: Sink,
    rotate_sink: Sink,
    hard_drop_sink: Sink,
    line_clear_sink: Sink,
    game_over_sink: Sink,
    last_line_clear: Instant,
}

impl SoundEffects {
    fn new(stream_handle: &rodio::OutputStreamHandle) -> Self {
        Self {
            move_sink: Sink::try_new(stream_handle).unwrap(),
            rotate_sink: Sink::try_new(stream_handle).unwrap(),
            hard_drop_sink: Sink::try_new(stream_handle).unwrap(),
            line_clear_sink: Sink::try_new(stream_handle).unwrap(),
            game_over_sink: Sink::try_new(stream_handle).unwrap(),
            last_line_clear: Instant::now(),
        }
    }

    fn play_sound(&self, sink: &Sink, file_path: &str, volume: f32) {
        sink.stop(); // Stop any previous sound
        let file = BufReader::new(File::open(file_path).unwrap());
        let source = rodio::Decoder::new(file).unwrap();
        let source = source.amplify(volume);
        sink.append(source);
        sink.play(); // Ensure it starts playing immediately
    }

    fn play_move(&self) {
        self.play_sound(&self.move_sink, "assets/sounds/move.wav", 0.5);
    }

    fn play_rotate(&self) {
        self.play_sound(&self.rotate_sink, "assets/sounds/rotate.wav", 0.2);
    }

    fn play_hard_drop(&self) {
        self.play_sound(&self.hard_drop_sink, "assets/sounds/hard_drop.wav", 0.5);
    }

    fn try_play_line_clear(&mut self) {
        // Only play if enough time has passed since last play (200ms cooldown)
        if self.last_line_clear.elapsed() >= Duration::from_millis(200) {
            self.play_sound(&self.line_clear_sink, "assets/sounds/line_clear.wav", 1.0);
            self.last_line_clear = Instant::now();
        }
    }

    fn play_game_over(&self) {
        self.play_sound(&self.game_over_sink, "assets/sounds/game_over.wav", 0.3);
    }
}

fn play_background_music(sink: &Sink, file_path: &str) {
    let file = BufReader::new(File::open(file_path).unwrap());
    let source = Decoder::new(file).unwrap().repeat_infinite();
    sink.append(source);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tetris")
        .vsync()
        .build();

    rl.set_target_fps(FPS);

    // Initialize audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sound_effects = SoundEffects::new(&stream_handle);
    let music_sink = Sink::try_new(&stream_handle).unwrap();

    // Start background music
    play_background_music(&music_sink, "assets/background.mp3");
    music_sink.set_volume(0.2);

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
                if moved {
                    sound_effects.play_move();
                }
            }
            if right_key.update(rl.is_key_down(KeyboardKey::KEY_RIGHT)) && !moved {
                moved = game.move_current_block(1, 0);
                if moved {
                    sound_effects.play_move();
                }
            }
            if rotate_key.update(rl.is_key_down(KeyboardKey::KEY_UP)) {
                if game.rotate_current_block() {
                    sound_effects.play_rotate();
                }
            }

            game.timer.soft_drop = down_key.update(rl.is_key_down(KeyboardKey::KEY_DOWN));

            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                if game.hard_drop() {
                    sound_effects.play_hard_drop();
                } else {
                    sound_effects.play_hard_drop();
                }
            }
            if (rl.is_key_pressed(KeyboardKey::KEY_LEFT_SHIFT)
                || rl.is_key_pressed(KeyboardKey::KEY_C))
                && !game.has_held
            {
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
                game.has_held = true;
                sound_effects.play_move();
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            game.toggle_pause();
            if game.state == GameState::Paused {
                music_sink.pause();
            } else {
                music_sink.play();
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) && game.state == GameState::GameOver {
            game.start_game();
            music_sink.play();
        }

        let prev_state = game.state;
        let prev_level = game.score.level;

        // Check if lines were cleared and play sound
        if game.lines_just_cleared {
            sound_effects.try_play_line_clear();
            game.lines_just_cleared = false; // Reset the flag after playing sound
        }

        game.update();

        // Play game over sound if state changed to GameOver
        if prev_state != GameState::GameOver && game.state == GameState::GameOver {
            sound_effects.play_game_over();
            music_sink.pause();
        }

        // Check if level changed and play sound
        if game.score.level > prev_level {
            // Don't play line clear sound for level up, it's too much
        }

        // Render
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND_COLOR);

        // Get screen shake offset
        let (shake_x, shake_y) = game.screen_shake.get_offset();

        // Apply shake offset to board and all game elements
        draw_board(
            &mut d,
            &game.board,
            BOARD_OFFSET_X + shake_x,
            BOARD_OFFSET_Y + shake_y,
        );

        if game.state == GameState::Playing {
            draw_ghost_block(
                &mut d,
                &game.current_block,
                &game.board,
                BOARD_OFFSET_X + shake_x,
                BOARD_OFFSET_Y + shake_y,
            );
            draw_block(
                &mut d,
                &game.current_block,
                BOARD_OFFSET_X + shake_x,
                BOARD_OFFSET_Y + shake_y,
            );
        }

        d.draw_text(
            "Next:",
            BOARD_OFFSET_X + (BOARD_WIDTH as i32 * CELL_SIZE) + 30 + shake_x,
            BOARD_OFFSET_Y + shake_y,
            20,
            Color::WHITE,
        );
        draw_preview_block(
            &mut d,
            game.next_block.kind,
            BOARD_OFFSET_X + (BOARD_WIDTH as i32 * CELL_SIZE) + 30 + shake_x,
            BOARD_OFFSET_Y + 30 + shake_y,
        );

        d.draw_text(
            "Hold:",
            20 + shake_x,
            BOARD_OFFSET_Y + 100 + shake_y,
            20,
            Color::WHITE,
        );
        if let Some(held_block) = &game.hold_block {
            draw_preview_block(
                &mut d,
                held_block.kind,
                20 + shake_x,
                BOARD_OFFSET_Y + 130 + shake_y,
            );
        }

        d.draw_text(
            &format!("Score: {}", game.score.points),
            20 + shake_x,
            BOARD_OFFSET_Y + shake_y,
            20,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Level: {}", game.score.level),
            20 + shake_x,
            BOARD_OFFSET_Y + 30 + shake_y,
            20,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Lines: {}", game.score.lines),
            20 + shake_x,
            BOARD_OFFSET_Y + 60 + shake_y,
            20,
            Color::WHITE,
        );

        match game.state {
            GameState::Paused | GameState::GameOver => {
                // Draw semi-transparent black overlay
                d.draw_rectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color::new(0, 0, 0, 128));

                if game.state == GameState::Paused {
                    d.draw_text(
                        "PAUSED",
                        WINDOW_WIDTH / 2 - 50,
                        WINDOW_HEIGHT / 2,
                        30,
                        Color::WHITE,
                    );
                    d.draw_text(
                        "Press P to resume",
                        WINDOW_WIDTH / 2 - 80,
                        WINDOW_HEIGHT / 2 + 40,
                        20,
                        Color::WHITE,
                    );
                } else {
                    d.draw_text(
                        "GAME OVER",
                        WINDOW_WIDTH / 2 - 70,
                        WINDOW_HEIGHT / 2,
                        30,
                        Color::WHITE,
                    );
                    d.draw_text(
                        "Press R to restart",
                        WINDOW_WIDTH / 2 - 80,
                        WINDOW_HEIGHT / 2 + 40,
                        20,
                        Color::WHITE,
                    );
                }
            }
            _ => {}
        }
    }
}
