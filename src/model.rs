use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const FIELD_W: usize = 14;
pub const FIELD_H: usize = 14;
pub const CELL_SIZE: i32 = 40;
pub const COLOR_COUNT: i32 = 6;
pub const ALLOWED_STEP_COUNT: i32 = 25;
pub const PAINT_WAIT: i32 = 1;

// $varの値が
//   > 0 : ウェイト中
//  == 0 : ブロック実行
//   < 0 : ブロック実行せず、ウェイトも減らさない
macro_rules! wait {
    ($var:expr, $block:block) => {
        if $var > 0 {
            $var -= 1;
        }
        if $var == 0 {
            $block
        }
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Paint(i32),
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum State {
    #[default]
    Controllable,
    Painting,
}

#[derive(Debug)]
pub struct Game {
    pub rng: StdRng,
    pub is_clear: bool,
    pub is_over: bool,
    pub frame: i32,
    pub requested_sounds: Vec<&'static str>,
    pub state: State,
    pub field: [[i32; FIELD_W]; FIELD_H],
    pub effect: [[bool; FIELD_W]; FIELD_H],
    pub painted_count: i32,
    pub hover_color: i32,
    pub sum: usize,
    pub paint_wait: i32,
    pub last_color: i32,
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
            state: State::Controllable,
            field: [[0; FIELD_W]; FIELD_H],
            effect: [[false; FIELD_W]; FIELD_H],
            painted_count: 0,
            hover_color: -1,
            sum: 0,
            paint_wait: 0,
            last_color: -1,
        };

        game.set_state(State::Controllable);
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

        match self.state {
            State::Controllable => match command {
                Command::None => {}
                Command::Paint(color) => {
                    self.paint(color);
                }
            },
            State::Painting => {
                wait!(self.paint_wait, {
                    let mut effect_done = false;
                    for y in 0..FIELD_H {
                        for x in 0..FIELD_W {
                            if x + y == self.sum && self.field[y][x] == self.last_color {
                                self.effect[y][x] = true;
                                effect_done = true;
                            } else {
                                self.effect[y][x] = false;
                            }
                        }
                    }
                    self.sum += 1;
                    if !effect_done {
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
                        self.set_state(State::Controllable);
                    }
                    self.paint_wait = PAINT_WAIT;
                });
            }
        }
    }

    pub fn set_state(&mut self, new_state: State) {
        match new_state {
            State::Controllable => {
                assert!(self.state == State::Controllable || self.state == State::Painting);
            }
            State::Painting => {
                assert!(self.state == State::Controllable);
                self.sum = 0;
                self.paint_wait = 0;
            }
        }
        self.state = new_state;
    }

    pub fn paint(&mut self, color: i32) {
        if color == self.field[0][0] {
            self.requested_sounds.push("ng.wav");
            return;
        }
        self.set_color(0, 0, self.field[0][0], color);
        self.painted_count += 1;
        self.last_color = color;
        self.set_state(State::Painting);
    }

    pub fn set_color(&mut self, x: usize, y: usize, from_color: i32, to_color: i32) {
        if self.field[y][x] != from_color {
            return;
        }
        self.field[y][x] = to_color;
        if x + 1 < FIELD_W {
            self.set_color(x + 1, y, from_color, to_color);
        }
        if y + 1 < FIELD_H {
            self.set_color(x, y + 1, from_color, to_color);
        }
    }
}
