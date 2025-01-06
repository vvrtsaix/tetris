use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum BlockKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl BlockKind {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => BlockKind::I,
            1 => BlockKind::J,
            2 => BlockKind::L,
            3 => BlockKind::O,
            4 => BlockKind::S,
            5 => BlockKind::T,
            _ => BlockKind::Z,
        }
    }

    pub fn color(&self) -> u8 {
        match self {
            BlockKind::I => 0,
            BlockKind::J => 1,
            BlockKind::L => 2,
            BlockKind::O => 3,
            BlockKind::S => 4,
            BlockKind::T => 5,
            BlockKind::Z => 6,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub kind: BlockKind,
    pub x: i32,
    pub y: i32,
    pub rotation: u8,
}

impl Block {
    pub fn new(kind: BlockKind) -> Self {
        Self {
            kind,
            x: 4,
            y: -2,
            rotation: 0,
        }
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    pub fn blocks(&self) -> [(i32, i32); 4] {
        let base_positions = match self.kind {
            BlockKind::I => [(0, 0), (-1, 0), (1, 0), (2, 0)],
            BlockKind::J => [(0, 0), (-1, 0), (1, 0), (-1, -1)],
            BlockKind::L => [(0, 0), (-1, 0), (1, 0), (1, -1)],
            BlockKind::O => [(0, -1), (1, -1), (0, 0), (1, 0)],
            BlockKind::S => [(0, -1), (-1, -1), (0, 0), (1, 0)],
            BlockKind::T => [(0, -1), (-1, 0), (1, 0), (0, 0)],
            BlockKind::Z => [(0, -1), (1, -1), (0, 0), (-1, 0)],
        };

        let mut rotated = [(0, 0); 4];
        for (i, &(x, y)) in base_positions.iter().enumerate() {
            let (rx, ry) = match self.rotation {
                0 => (x, y),
                1 => (-y, x),
                2 => (-x, -y),
                3 => (y, -x),
                _ => unreachable!(),
            };
            rotated[i] = (rx + self.x, ry + self.y);
        }
        rotated
    }

    pub fn reset(&mut self) {
        self.x = 4;
        self.y = -2;
        self.rotation = 0;
    }

    pub fn move_left(&mut self) {
        self.x -= 1;
    }

    pub fn move_right(&mut self) {
        self.x += 1;
    }

    pub fn move_down(&mut self) {
        self.y += 1;
    }
}
