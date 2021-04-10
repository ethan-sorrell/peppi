# Peppi

Peppi is a Rust parser for [.slp](https://github.com/project-slippi/slippi-wiki/blob/master/SPEC.md) game replay files for [Super Smash Brothers Melee](https://en.wikipedia.org/wiki/Super_Smash_Bros._Melee) for the Nintendo GameCube. These replays are generated by Jas Laferriere's [Slippi](https://github.com/JLaferri/project-slippi) recording code, which runs on a Wii or the [Dolphin](https://dolphin-emu.org/) emulator.

Peppi is fairly full-featured already, but still under active development. **APIs are still subject to breaking change** until version 1.0 is released.

## Installation

In your `Cargo.toml`:

```toml
[dependencies]
peppi = { git = "https://github.com/hohav/peppi" }
```

## Usage

### Object-based parsing:

```rust
use std::path::Path;

fn main() {
    let mut buf = io::BufReader::new(
        fs::File::open("game.slp").unwrap());
    let game = peppi::game(&mut buf, None).unwrap();
    println!("{:#?}", game);
}
```

### Event-driven parsing:

```rust
use std::{fs, io};

use peppi::parse::{Handlers, FrameEvent, PortId};
use peppi::frame;

struct FramePrinter {}

impl Handlers for FramePrinter {
    fn frame_post(&mut self, post: FrameEvent<PortId, frame::Post>) -> io::Result<()> {
        println!("{}:{}", post.id.port, post.id.index);
        Ok(())
    }
}

fn main() {
    let f = fs::File::open("game.slp").unwrap();
    let mut r = io::BufReader::new(f);
    let mut handlers = FramePrinter {};

    peppi::parse(&mut r, &mut handlers, None).unwrap();
}
```

## Inspector

⚠ The `slp` tool has moved to the [peppi-slp](https://github.com/hohav/peppi-slp) crate.
