use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const FIELD_W: usize = 14;
pub const FIELD_H: usize = 14;
pub const CELL_SIZE: i32 = 40;
pub const COLOR_COUNT: i32 = 6;
pub const ALLOWED_STEP_COUNT: i32 = 25;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Paint(i32),
}
#[derive(Debug)]
pub struct Game {
    pub rng: StdRng,
    pub is_clear: bool,
    pub is_over: bool,
    pub frame: i32,
    pub requested_sounds: Vec<&'static str>,
    pub field: [[i32; FIELD_W]; FIELD_H],
    pub painted_count: i32,
    pub hover_color: i32,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        //let rng = StdRng::seed_from_u64(1706226338);

        let mut game = Game {
            rng: rng,
            is_clear: false,
            is_over: false,
            frame: -1,
            requested_sounds: Vec::new(),
            field: [[0; FIELD_W]; FIELD_H],
            painted_count: 0,
            hover_color: -1,
        };

        game.set_field_random();
        game
    }

    pub fn set_field_random(&mut self) {
        for y in 0..FIELD_H {
            for x in 0..FIELD_W {
                self.field[y][x] = self.rng.gen_range(0..COLOR_COUNT);
            }
        }
    }

    pub fn update(&mut self, command: Command) {
        if self.is_clear || self.is_over {
            return;
        }

        self.frame += 1;

        match command {
            Command::None => {}
            Command::Paint(color) => {
                self.paint(color);
            }
        }
    }

    pub fn paint(&mut self, color: i32) {
        if color == self.field[0][0] {
            self.requested_sounds.push("ng.wav");
            return;
        }
        self.set_color(0, 0, self.field[0][0], color);
        self.painted_count += 1;
        let mut is_clear = true;
        for y in 0..FIELD_H {
            for x in 0..FIELD_W {
                if self.field[y][x] != self.field[0][0] {
                    is_clear = false;
                }
            }
        }
        if is_clear {
            self.is_clear = true;
            self.requested_sounds.push("bravo.wav");
        } else if ALLOWED_STEP_COUNT - self.painted_count == 0 {
            self.is_over = true;
            self.requested_sounds.push("crash.wav");
        }
    }

    pub fn set_color(&mut self, x: usize, y: usize, from_color: i32, to_color: i32) {
        if self.field[y][x] != from_color {
            return;
        }
        self.field[y][x] = to_color;
        if x + 1 < FIELD_W {
            self.set_color(x + 1, y, from_color, to_color);
        }
        if x > 0 {
            self.set_color(x - 1, y, from_color, to_color);
        }
        if y + 1 < FIELD_H {
            self.set_color(x, y + 1, from_color, to_color);
        }
        if y > 0 {
            self.set_color(x, y - 1, from_color, to_color);
        }
    }
}
