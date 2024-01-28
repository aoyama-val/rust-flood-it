use core::panic;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const WINDOW_TITLE: &str = "rust-flood-it";
pub const SCREEN_WIDTH: i32 = FIELD_W as i32 * CELL_SIZE + INFO_WIDTH;
pub const SCREEN_HEIGHT: i32 = FIELD_H as i32 * CELL_SIZE;
pub const INFO_WIDTH: i32 = BUTTON_WIDTH * 2 + MARGIN_X * 3;
pub const INFO_X: i32 = SCREEN_WIDTH - INFO_WIDTH;
pub const BUTTON_WIDTH: i32 = 60;
pub const BUTTON_HEIGHT: i32 = 60;
pub const MARGIN_X: i32 = 25;
pub const BUTTONS: [[i32; 2]; 6] = [
    [INFO_X + MARGIN_X, 300],
    [INFO_X + MARGIN_X + BUTTON_WIDTH + MARGIN_X, 300],
    [INFO_X + MARGIN_X, 380],
    [INFO_X + MARGIN_X + BUTTON_WIDTH + MARGIN_X, 380],
    [INFO_X + MARGIN_X, 460],
    [INFO_X + MARGIN_X + BUTTON_WIDTH + MARGIN_X, 460],
];

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(WINDOW_TITLE, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    init_mixer();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    'running: loop {
        let started = SystemTime::now();
        let mut command = Command::None;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    match code {
                        Keycode::Escape => {
                            break 'running;
                        }
                        _ => {}
                    };
                }
                Event::MouseMotion { x, y, .. } => {
                    game.hover_color = get_selected_color(x, y);
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        if game.is_clear || game.is_over {
                            game = Game::new();
                        } else {
                            let color = get_selected_color(x, y);
                            if color >= 0 {
                                command = Command::Paint(color);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        game.update(command);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / model::FPS as u32);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
        fonts: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    let entries = fs::read_dir("./resources/sound").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".wav") {
            let chunk = mixer::Chunk::from_file(path_str)
                .expect(&format!("cannot load sound: {}", path_str));
            let basename = path.file_name().unwrap().to_str().unwrap();
            resources.chunks.insert(basename.to_string(), chunk);
        }
    }

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        28,
        "boxfont",
    );

    resources
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let font = resources.fonts.get_mut("boxfont").unwrap();

    // render field
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let color = get_block_color(game.field[y][x]);
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                x as i32 * CELL_SIZE,
                y as i32 * CELL_SIZE,
                CELL_SIZE as u32,
                CELL_SIZE as u32,
            ))?;
        }
    }

    // render info
    let font_color = Color::RGB(224, 224, 224);
    render_font(
        canvas,
        font,
        format!("MOVES {:2}", ALLOWED_STEP_COUNT - game.painted_count).to_string(),
        INFO_X + MARGIN_X,
        230,
        font_color,
        false,
    );

    // render buttons
    for color_num in 0..COLOR_COUNT {
        let x = BUTTONS[color_num as usize][0];
        let y = BUTTONS[color_num as usize][1];

        if color_num == game.hover_color {
            let border_width = 4;
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.fill_rect(Rect::new(
                x - border_width as i32,
                y - border_width as i32,
                BUTTON_WIDTH as u32 + border_width * 2,
                BUTTON_HEIGHT as u32 + border_width * 2,
            ))?;
        }

        let color = get_block_color(color_num);
        canvas.set_draw_color(color);
        canvas.fill_rect(Rect::new(x, y, BUTTON_WIDTH as u32, BUTTON_HEIGHT as u32))?;
    }

    if game.is_clear {
        render_font(
            canvas,
            font,
            "EXCELLENT!".to_string(),
            INFO_X + MARGIN_X,
            160,
            Color::RGB(255, 255, 128),
            false,
        );
    }

    if game.is_over {
        render_font(
            canvas,
            font,
            "GAME OVER".to_string(),
            INFO_X + MARGIN_X,
            160,
            Color::RGB(255, 128, 128),
            false,
        );
    }

    canvas.present();

    Ok(())
}

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    center: bool,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = if center {
        x - texture.query().width as i32 / 2
    } else {
        x
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}

fn get_block_color(color_num: i32) -> Color {
    match color_num {
        0 => Color::RGB(255, 128, 128),
        1 => Color::RGB(255, 255, 128),
        2 => Color::RGB(128, 255, 128),
        3 => Color::RGB(128, 255, 255),
        4 => Color::RGB(128, 128, 255),
        5 => Color::RGB(255, 128, 255),
        _ => {
            println!("invalid color: {}", color_num);
            panic!();
        }
    }
}

fn get_selected_color(x: i32, y: i32) -> i32 {
    for color_num in 0..COLOR_COUNT {
        if BUTTONS[color_num as usize][0] <= x
            && x < BUTTONS[color_num as usize][0] + BUTTON_WIDTH
            && BUTTONS[color_num as usize][1] <= y
            && y < BUTTONS[color_num as usize][1] + BUTTON_HEIGHT
        {
            return color_num;
        }
    }
    return -1;
}
