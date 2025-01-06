# Rust Tetris

A modern implementation of the classic Tetris game written in Rust using the Raylib graphics library.

## Features

- Classic Tetris gameplay mechanics
- Smooth animations and screen shake effects
- Sound effects and background music
- Hold piece functionality
- Next piece preview
- Level progression system with increasing speed
- Score tracking
- Ghost piece preview
- Pause functionality

## Controls

- **Left Arrow**: Move piece left
- **Right Arrow**: Move piece right
- **Down Arrow**: Soft drop
- **Up Arrow**: Rotate piece
- **Space**: Hard drop
- **Left Shift/C**: Hold piece
- **P**: Pause/Resume game
- **R**: Restart game (when game over)

## Scoring System

- Single line clear: 100 × level
- Double line clear: 300 × level
- Triple line clear: 500 × level
- Tetris (4 lines): 800 × level

## Level System

- Level increases every 10 lines cleared
- Each level increases the falling speed by 20%
- Starting speed: 800ms per tile
- Level formula: `level = (lines_cleared / 10) + 1`

## Building from Source

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Cargo package manager
- Raylib development libraries

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/tetris.git
cd tetris
```

2. Build and run the game:
```bash
cargo run --release
```

## Dependencies

- `raylib`: Graphics and input handling
- `rodio`: Audio playback

## License

This project is open source and available under the MIT License.

## Credits

Sound effects and background music should be placed in:
- `assets/sounds/` - For sound effects
- `assets/` - For background music (background.mp3)

Required sound files:
- move.wav
- rotate.wav
- hard_drop.wav
- line_clear.wav
- game_over.wav
- hold.wav
- pause.wav 
