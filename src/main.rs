use raylib::prelude::*;
use std::time::{Duration, Instant};

mod tetris;
use tetris::*;

struct SoundEffects<'a> {
    move_sound: Sound<'a>,
    rotate_sound: Sound<'a>,
    hard_drop_sound: Sound<'a>,
    line_clear_sound: Sound<'a>,
    game_over_sound: Sound<'a>,
    last_line_clear: Instant,
}

impl<'a> SoundEffects<'a> {
    fn new(rl: &'a RaylibAudio) -> Self {
        Self {
            move_sound: rl
                .new_sound("assets/sounds/move.wav")
                .expect("Failed to load move sound"),
            rotate_sound: rl
                .new_sound("assets/sounds/rotate.wav")
                .expect("Failed to load rotate sound"),
            hard_drop_sound: rl
                .new_sound("assets/sounds/hard_drop.wav")
                .expect("Failed to load hard drop sound"),
            line_clear_sound: rl
                .new_sound("assets/sounds/line_clear.wav")
                .expect("Failed to load line clear sound"),
            game_over_sound: rl
                .new_sound("assets/sounds/game_over.wav")
                .expect("Failed to load game over sound"),
            last_line_clear: Instant::now(),
        }
    }

    fn play_move(&mut self) {
        self.move_sound.set_volume(0.5);
        self.move_sound.play();
    }

    fn play_rotate(&mut self) {
        self.rotate_sound.set_volume(0.2);
        self.rotate_sound.play();
    }

    fn play_hard_drop(&mut self) {
        self.hard_drop_sound.set_volume(0.5);
        self.hard_drop_sound.play();
    }

    fn try_play_line_clear(&mut self) {
        if self.last_line_clear.elapsed() >= Duration::from_millis(200) {
            self.line_clear_sound.set_volume(1.0);
            self.line_clear_sound.play();
            self.last_line_clear = Instant::now();
        }
    }

    fn play_game_over(&mut self) {
        self.game_over_sound.set_volume(0.3);
        self.game_over_sound.play();
    }
}

#[tokio::main]
async fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tetris")
        .vsync()
        .build();

    rl.set_target_fps(FPS);

    // Initialize audio device
    let audio_device = RaylibAudio::init_audio_device().expect("Failed to initialize audio device");

    // Load sound effects
    let mut sound_effects = SoundEffects::new(&audio_device);

    // Load and play background music
    let mut music = audio_device
        .new_music("assets/background.mp3")
        .expect("Failed to load background music");
    music.set_volume(0.2);
    music.play_stream();

    let mut game = Game::default();

    // Connect to multiplayer server
    if let Err(e) = game.connect_multiplayer("ws://localhost:8080").await {
        eprintln!("Failed to connect to multiplayer server: {}", e);
    }

    game.start_game();

    let mut left_key = KeyState::new(false);
    let mut right_key = KeyState::new(false);
    let mut down_key = KeyState::new(false);
    let mut rotate_key = KeyState::new(true);

    while !rl.window_should_close() {
        // Update music stream
        music.update_stream();

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
                music.pause_stream();
            } else {
                music.resume_stream();
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) && game.state == GameState::GameOver {
            game.start_game();
            music.resume_stream();
        }

        let prev_state = game.state;

        // Check if lines were cleared and play sound
        if game.lines_just_cleared {
            sound_effects.try_play_line_clear();
            game.lines_just_cleared = false;
        }

        game.update();

        // Play game over sound if state changed to GameOver
        if prev_state != GameState::GameOver && game.state == GameState::GameOver {
            sound_effects.play_game_over();
            music.pause_stream();
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

        // Draw scoreboard
        draw_scoreboard(
            &mut d,
            game.score.points,
            game.score.lines,
            game.score.level,
            &game.other_players,
            game.player_id.as_deref(),
        );

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
