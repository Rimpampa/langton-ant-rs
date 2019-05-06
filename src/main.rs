extern crate sdl2;
extern crate rand;

use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;

mod ant_map;
use ant_map::Direction as Go;
use ant_map::Rotation as Turn;
use ant_map::AntMap;

use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::render::Canvas;
use sdl2::video::Window;

const BG_COLOR: Color = Color { r: 255, g: 255, b: 255, a: 255 };
const TXT_COLOR: Color = Color { r: 0, g: 0, b: 0, a: 255 };
const ANT_COLOR: Color = Color { r: 0, g: 0, b: 0, a: 255 };
const DEL_COLOR: Color = Color { r: 255, g: 0, b: 0, a: 255 };

fn main() {
    let sdl = sdl2::init().unwrap(); // Initialize sdl2 crate
    let video_subsystem = sdl.video().unwrap(); // Get the video subsystem

    // Create a new window
    let window = video_subsystem
        .window("Langton's Ant", 700, 700)
        .resizable()
        .maximized()
        .build().unwrap();

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build().unwrap();

    canvas.set_draw_color(BG_COLOR);
    canvas.clear();

    canvas.present();

    let mut map = AntMap::new(3, 3, Go::Up, Turn::vec_from_string("RL"));
    
    let mut rng = rand::thread_rng();
    let mut colors = gen_random_unique_colors(&mut rng, map.stages().len());

    let mut map_rect = get_map_area(canvas.window());
    let mut sq_rect = get_sequence_area(canvas.window());
    let mut sq_hitboxes = get_sequence_hitboxes(sq_rect, map.stages());

    let mut speed = 1_usize;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::Window {
                    win_event: WindowEvent::Resized(..), ..
                } => {
                    canvas.set_draw_color(BG_COLOR);
                    canvas.clear();

                    map_rect = get_map_area(canvas.window());
                    draw_map(&mut canvas, &map_rect, &map, &colors);
                    
                    sq_rect = get_sequence_area(canvas.window());
                    sq_hitboxes = get_sequence_hitboxes(sq_rect, map.stages());
                    draw_sequence_ui(&mut canvas, &sq_hitboxes.0, map.stages(), &colors);
                    canvas.present();
                },
                Event::MouseButtonDown {x, y, ..} => {
                    if sq_rect.contains_point((x, y)) {
                        for (i, r) in sq_hitboxes.1.iter().enumerate() {
                            if r.contains_point((x, y)) {
                                colors.remove(i + 1);
                                map.remove_stage(i + 1);

                                sq_hitboxes = get_sequence_hitboxes(sq_rect, map.stages());
                                canvas.set_draw_color(BG_COLOR);
                                canvas.fill_rect(sq_rect).unwrap();
                                draw_sequence_ui(&mut canvas, &sq_hitboxes.0, map.stages(), &colors);
                                
                                map.reset();
                                map.shrink();
                                
                                continue 'main;
                            }
                        }
                        for (i, r) in sq_hitboxes.0.iter().enumerate().take(map.stages().len()) {
                            if r.contains_point((x, y)) {
                                canvas.set_draw_color(colors[i]);
                                canvas.fill_rect(*r).unwrap();
                                canvas.set_draw_color(TXT_COLOR);
                                map.invert_rotation(i);

                                map.reset();
                                map.shrink();

                                continue 'main;
                            }
                        }
                        if let Some(r) = sq_hitboxes.0.last() {
                            if r.contains_point((x, y)) {
                                map.add_stage(Turn::Right);
                                colors.push(gen_random_unique_color(&mut rng, &colors));
                                sq_hitboxes = get_sequence_hitboxes(sq_rect, map.stages());
                                canvas.set_draw_color(BG_COLOR);
                                canvas.fill_rect(sq_rect).unwrap();
                                draw_sequence_ui(&mut canvas, &sq_hitboxes.0, map.stages(), &colors);

                                map.reset();
                                map.shrink();

                                continue 'main;
                            }
                        }
                    }
                },
                Event::MouseWheel {y, ..} => {
                    if speed as i32 + y > 0 {
                        if y > 0 { speed += y as usize; } else { speed -= (-y) as usize; }
                    }
                    else { speed = 1; }
                }
                _ => {}
            }
        }   
        for _ in 0..speed {
            if !map.step_ahead() {
                map.shrink();
                map.scale(5);
                map.step_ahead();
                
                canvas.set_draw_color(BG_COLOR);
                canvas.clear();
                draw_sequence_ui(&mut canvas, &sq_hitboxes.0, map.stages(), &colors);
            }
        }
        canvas.set_draw_color(BG_COLOR);
        canvas.fill_rect(map_rect).unwrap();
        draw_map(&mut canvas, &map_rect, &map, &colors);
        canvas.present();
        //std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn add_u8(a: u8, b: u8) -> u8 {
    let c = a as u16 + b as u16;
    if c > u8::max_value() as u16 { u8::max_value() } else { c as u8 }
}

const THRESHOLD: u8 = 100;
fn same_color(a: &Color, b: &Color) -> bool {
    (add_u8(a.r, THRESHOLD) < b.r && a.r > add_u8(b.r, THRESHOLD)) ||
    (add_u8(a.g, THRESHOLD) < b.g && a.g > add_u8(b.g, THRESHOLD)) ||
    (add_u8(a.b, THRESHOLD) < b.b && a.b > add_u8(b.b, THRESHOLD))
}

fn gen_random_color(rng: &mut ThreadRng) -> Color {
    let u8distr: Uniform<u8> = Uniform::from(0..u8::max_value());
    Color::RGB(
        u8distr.sample(rng),
        u8distr.sample(rng),
        u8distr.sample(rng),
    )
}

fn gen_random_unique_color(rng: &mut ThreadRng, colors: &Vec<Color>) -> Color {

    let mut rndc = gen_random_color(rng);
    while colors.iter().chain([BG_COLOR, ANT_COLOR, TXT_COLOR].iter()).any(|c| same_color(c, &rndc)) {
        rndc = gen_random_color(rng)
    }
    rndc
}

fn gen_random_unique_colors(rng: &mut ThreadRng, size: usize) -> Vec<Color> {
    
    let mut v: Vec<Color> = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(gen_random_unique_color(rng, &v));
    }
    v
}

fn draw_map(canvas: &mut Canvas<Window>, area: &Rect, map: &AntMap, colors: &Vec<Color>) {
    let mut side = if area.width() > area.height() { area.height() }
    else { area.width() } as i32 / if map.width() > map.height() { map.width() }
    else { map.height() } as i32;

    if side == 0 { side = 1; }

    let ant = map.ant();
    let mut offset_x = area.left() + (area.width()  as i32 - side * map.width() as i32) / 2;
    if side as u32 * map.width() as u32 > area.width() {
        offset_x -= (ant.0 as i32 - map.width() as i32 / 2) * side;
    }

    let mut offset_y = area.top() + (area.height() as i32 - side * map.height() as i32) / 2;
    if side as u32 * map.height() as u32 > area.height() {
        offset_y -= (ant.1 as i32 - map.height() as i32 / 2) * side;
    }

    for (x, xv) in map.iter().enumerate() {
        let mut last_stage = 0;
        for (y, yv) in xv.iter().enumerate() {
            let p = Point::new(x as i32 * side + offset_x, y as i32 * side + offset_y);
            if *yv != 0 {
                if p.x + side > area.left() && p.x < area.right() &&
                   p.y > area.top() && p.y + side < area.bottom() {

                    if *yv != last_stage {
                        last_stage = *yv;
                        canvas.set_draw_color(colors[*yv as usize - 1]);
                    }
                    canvas.fill_rect(Rect::new(p.x, p.y, side as u32, side as u32)).unwrap();
                }
            }
            if x == ant.0 && y == ant.1 && side > 6 {
                canvas.set_draw_color(ANT_COLOR);
                last_stage = 0;
                draw_ant(canvas, p, side as u32, ant.2);
            }
        }
    }
}

fn draw_ant(canvas: &mut Canvas<Window>, at: Point, side: u32, way: Go) {
    let mut unit = side / 6;
    if unit == 0 { unit = 1; }

    match way {
        Go::Up => {
            canvas.fill_rects(&[
                Rect::new(at.x + unit as i32, at.y + unit as i32, unit, unit),
                Rect::new(at.x + 3 * unit as i32, at.y + unit as i32, unit, unit),
                Rect::new(at.x + 2 * unit as i32, at.y + 2 * unit as i32, unit, 4 * unit),
                Rect::new(at.x + unit as i32, at.y + 3 * unit as i32, 3 * unit, unit),
                Rect::new(at.x + unit as i32, at.y + 5 * unit as i32, 3 * unit, unit),
            ]).unwrap();
        },
        Go::Down => {
            canvas.fill_rects(&[
                Rect::new(at.x + unit as i32, at.y + 4 * unit as i32, unit, unit),
                Rect::new(at.x + 3 * unit as i32, at.y + 4 * unit as i32, unit, unit),
                Rect::new(at.x + 2 * unit as i32, at.y, unit, 4 * unit),
                Rect::new(at.x + unit as i32, at.y, 3 * unit, unit),
                Rect::new(at.x + unit as i32, at.y + 2 * unit as i32, 3 * unit, unit),
            ]).unwrap();
        },
        
        Go::Left => {
            canvas.fill_rects(&[
                Rect::new(at.x + unit as i32, at.y + unit as i32, unit, unit),
                Rect::new(at.x + unit as i32, at.y + 3 * unit as i32, unit, unit),
                Rect::new(at.x + 2 * unit as i32, at.y + 2 * unit as i32, 4 * unit, unit),
                Rect::new(at.x + 3 * unit as i32, at.y + unit as i32, unit, 3 * unit),
                Rect::new(at.x + 5 * unit as i32, at.y + unit as i32, unit, 3 * unit),
            ]).unwrap();
        },
        
        Go::Right => {
            canvas.fill_rects(&[
                Rect::new(at.x + 4 * unit as i32, at.y + 2 * unit as i32, unit, unit),
                Rect::new(at.x + 4 * unit as i32, at.y + 4 * unit as i32, unit, unit),
                Rect::new(at.x, at.y + 3 * unit as i32, 4 * unit, unit),
                Rect::new(at.x, at.y + 2 * unit as i32, unit, 3 * unit),
                Rect::new(at.x + 2 * unit as i32, at.y + 2 * unit as i32, unit, 3 * unit),
            ]).unwrap();
        },
    }
}

fn draw_sequence_ui(canvas: &mut Canvas<Window>, rects: &Vec<Rect>, rots: &Vec<Turn>, colors: &Vec<Color>) {
    for (i, r) in rects.iter().enumerate().take(colors.len()) {

        canvas.set_draw_color(colors[i]);
        canvas.fill_rect(*r).unwrap();

        canvas.set_draw_color(TXT_COLOR);
        match rots[i] {
            Turn::Right => draw_r(canvas, r.top_left(), r.width()),
            Turn::Left => draw_l(canvas, r.top_left(), r.width()),
        }
        
        if i > 0 {
            canvas.set_draw_color(DEL_COLOR);
            canvas.fill_rect(Rect::new(r.right() - r.width() as i32 / 4, r.top(), r.width() / 4, r.height())).unwrap();
        }
    }
    if let Some(r) = rects.last() {
        canvas.set_draw_color(TXT_COLOR);
        canvas.draw_rect(*r).unwrap();
        draw_plus(canvas, r.top_left(), r.width());
    }
}

fn draw_r(canvas: &mut Canvas<Window>, at: Point, side: u32) {
    let mut unit = side / 6;
    if unit == 0 { unit = 1; }

    canvas.fill_rects(&[
        Rect::new(at.x + unit as i32, at.y + unit as i32, unit, unit * 4),
        Rect::new(at.x + 2 * unit as i32, at.y + unit as i32, unit * 1, unit),
        Rect::new(at.x + 2 * unit as i32, at.y + 3 * unit as i32, unit * 1, unit),
        Rect::new(at.x + 3 * unit as i32, at.y + unit as i32, unit, unit * 2),
        Rect::new(at.x + 3 * unit as i32, at.y + 4 * unit as i32, unit, unit),
    ]).unwrap();
}

fn draw_l(canvas: &mut Canvas<Window>, at: Point, side: u32) {
    let mut unit = side / 6;
    if unit == 0 { unit = 1; }

    canvas.fill_rects(&[
        Rect::new(at.x + unit as i32, at.y + unit as i32, unit, unit * 4),
        Rect::new(at.x + unit as i32, at.y + 4 * unit as i32, unit * 3, unit),
    ]).unwrap();
}

fn draw_plus(canvas: &mut Canvas<Window>, at: Point, side: u32) {
    let mut unit = side / 6;
    if unit == 0 { unit = 1; }

    canvas.fill_rects(&[
        Rect::new(at.x + 2 * unit as i32, at.y + unit as i32, unit, 3 * unit),
        Rect::new(at.x + unit as i32, at.y + 2 * unit as i32, unit * 3, unit),
    ]).unwrap();
}

const HSPLIT_PERCENTAGE: u32 = 75;
fn get_map_area(window: &Window) -> Rect {
    let width = window.size().0;
    let height = window.size().1;
    
    if width > height {
        Rect::new((width - height) as i32 / 2, 0, HSPLIT_PERCENTAGE * height / 100, height)
    } else {
        Rect::new(0, (height - width) as i32 / 2, HSPLIT_PERCENTAGE * width / 100, width)
    }
}

fn get_sequence_area(window: &Window) -> Rect {
    let width = window.size().0;
    let height = window.size().1;
    
    if width > height {
        Rect::new((HSPLIT_PERCENTAGE * height / 100 + (width - height) / 2) as i32, 0, (100 - HSPLIT_PERCENTAGE) * height / 100, height)
    } else {
        Rect::new((HSPLIT_PERCENTAGE * width / 100) as i32, (height - width) as i32 / 2, (100 - HSPLIT_PERCENTAGE) * width / 100, width)
    }
}

fn get_sequence_hitboxes(sequence: Rect, array: &Vec<Turn>) -> (Vec<Rect>, Vec<Rect>) {

    let mut v1: Vec<Rect> = Vec::with_capacity(array.len());
    let mut v2: Vec<Rect> = Vec::with_capacity(array.len() - 1);

    let hb_unit = sequence.height() / (array.len() as u32 + 3);
    let wb_unit = sequence.width() / 2;
    let unit = if hb_unit > wb_unit { wb_unit } else { hb_unit };
    let x = (sequence.width() - unit) as i32 / 2 + sequence.left();

    for i in 0..array.len() {
        let y = (i + 1) as i32 * unit as i32 + sequence.top();
        v1.push(Rect::new(x, y, unit, unit));
        
        if i > 0 {
            v2.push(Rect::new(x + unit as i32 - unit as i32 / 4, y, unit / 4, unit));
        }
    }
    v1.push(Rect::new(x, (array.len() + 1) as i32 * unit as i32 + sequence.top(), unit, unit));
    (v1, v2)
}