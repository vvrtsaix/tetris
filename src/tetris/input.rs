use std::time::{Duration, Instant};

// Key repeat timing constants
pub const KEY_REPEAT_DELAY: Duration = Duration::from_millis(150);
pub const KEY_REPEAT_RATE: Duration = Duration::from_millis(30);
pub const ROTATION_REPEAT_DELAY: Duration = Duration::from_millis(200);
pub const ROTATION_REPEAT_RATE: Duration = Duration::from_millis(150);

pub struct KeyState {
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
    pub fn new(is_rotation: bool) -> Self {
        Self {
            last_press: Instant::now(),
            is_pressed: false,
            is_rotation,
        }
    }

    pub fn update(&mut self, is_down: bool) -> bool {
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
