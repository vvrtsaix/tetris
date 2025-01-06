use std::time::{Duration, Instant};

use super::{Block, BlockKind, Board};

pub const INITIAL_FALL_INTERVAL: Duration = Duration::from_millis(800);
pub const SOFT_DROP_FACTOR: f32 = 0.05;

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
        self.update_score(lines_cleared);
        self.current_block = self.next_block;
        self.next_block = Block::new(BlockKind::random());

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
            return;
        }

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
