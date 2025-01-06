use std::time::{Duration, Instant};

use super::{Block, BlockKind, Board};

pub const INITIAL_FALL_INTERVAL: Duration = Duration::from_millis(800);
pub const SOFT_DROP_FACTOR: f32 = 0.05;
pub const SHAKE_DURATION: Duration = Duration::from_millis(300);
pub const SHAKE_INTENSITY_PER_LINE: f32 = 3.0;

// Level speed factors (each level will be this much faster than the previous)
pub const LEVEL_SPEED_FACTOR: f32 = 0.8; // 20% faster each level

pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Duration,
    pub start_time: Option<Instant>,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: Duration::from_millis(0),
            start_time: None,
        }
    }
}

impl ScreenShake {
    pub fn start(&mut self, lines_cleared: u32) {
        self.intensity = lines_cleared as f32 * SHAKE_INTENSITY_PER_LINE;
        self.duration = SHAKE_DURATION;
        self.start_time = Some(Instant::now());
    }

    pub fn get_offset(&self) -> (i32, i32) {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            if elapsed >= self.duration {
                return (0, 0);
            }

            let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
            let decay = 1.0 - progress;
            let angle = progress * 20.0; // Increase for more rapid shaking

            let x_offset = (angle.sin() * self.intensity * decay) as i32;
            let y_offset = (angle.cos() * self.intensity * decay) as i32;
            
            (x_offset, y_offset)
        } else {
            (0, 0)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

pub struct Score {
    pub points: u32,
    pub lines: u32,
    pub level: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            points: 0,
            lines: 0,
            level: 1,
        }
    }
}

pub struct GameTimer {
    pub fall_interval: Duration,
    pub last_fall: Instant,
    pub soft_drop: bool,
}

impl GameTimer {
    pub fn get_fall_interval(&self, level: u32) -> Duration {
        // Calculate speed based on level
        let speed_factor = LEVEL_SPEED_FACTOR.powi(level as i32 - 1);
        let interval = INITIAL_FALL_INTERVAL.as_secs_f32() * speed_factor;
        Duration::from_secs_f32(interval)
    }
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            fall_interval: INITIAL_FALL_INTERVAL,
            last_fall: Instant::now(),
            soft_drop: false,
        }
    }
}

pub struct Game {
    pub board: Board,
    pub current_block: Block,
    pub next_block: Block,
    pub hold_block: Option<Block>,
    pub state: GameState,
    pub score: Score,
    pub timer: GameTimer,
    pub has_held: bool,
    pub screen_shake: ScreenShake,
    pub lines_just_cleared: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::new(),
            current_block: Block::new(BlockKind::random()),
            next_block: Block::new(BlockKind::random()),
            hold_block: None,
            state: GameState::Playing,
            score: Score::default(),
            timer: GameTimer::default(),
            has_held: false,
            screen_shake: ScreenShake::default(),
            lines_just_cleared: false,
        }
    }
}

impl Game {
    pub fn move_current_block(&mut self, dx: i32, dy: i32) -> bool {
        let mut new_block = self.current_block;
        new_block.x += dx;
        new_block.y += dy;

        if self.board.is_valid_position(&new_block) {
            self.current_block = new_block;
            true
        } else {
            false
        }
    }

    pub fn rotate_current_block(&mut self) -> bool {
        let mut new_block = self.current_block;
        new_block.rotate();

        if self.board.is_valid_position(&new_block) {
            self.current_block = new_block;
            return true;
        }

        new_block.x = self.current_block.x - 1;
        if self.board.is_valid_position(&new_block) {
            self.current_block = new_block;
            return true;
        }

        new_block.x = self.current_block.x + 1;
        if self.board.is_valid_position(&new_block) {
            self.current_block = new_block;
            return true;
        }

        false
    }

    pub fn hard_drop(&mut self) -> bool {
        while self.move_current_block(0, 1) {}
        self.lock_current_block()
    }

    pub fn lock_current_block(&mut self) -> bool {
        if !self.board.place_block(&self.current_block) {
            self.state = GameState::GameOver;
            return false;
        }

        let lines_cleared = self.board.clear_full_rows();
        if lines_cleared > 0 {
            self.screen_shake.start(lines_cleared);
            self.lines_just_cleared = true;
        }
        self.update_score(lines_cleared);
        self.current_block = self.next_block;
        self.next_block = Block::new(BlockKind::random());
        self.has_held = false;

        lines_cleared > 0
    }

    pub fn update_score(&mut self, lines_cleared: u32) {
        let points = match lines_cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        } * self.score.level;

        self.score.points += points;
        self.score.lines += lines_cleared;
        self.score.level = (self.score.lines / 10) + 1;
    }

    pub fn update(&mut self) {
        if self.state != GameState::Playing {
            self.lines_just_cleared = false;
            return;
        }

        // Update fall interval based on current level
        self.timer.fall_interval = self.timer.get_fall_interval(self.score.level);

        let fall_interval = if self.timer.soft_drop {
            self.timer.fall_interval.mul_f32(SOFT_DROP_FACTOR)
        } else {
            self.timer.fall_interval
        };

        if self.timer.last_fall.elapsed() >= fall_interval {
            self.timer.last_fall = Instant::now();

            if !self.move_current_block(0, 1) {
                self.lock_current_block();
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => self.state = GameState::Paused,
            GameState::Paused => self.state = GameState::Playing,
            _ => {}
        }
    }

    pub fn start_game(&mut self) {
        *self = Game::default();
    }
}
