use fps_counter::FPSCounter;
use image::{Rgb, RgbImage};
use nannou::image::{self, DynamicImage};
use nannou::prelude::*;

const SCALE: f32 = 2.0;
const GRID_HEIGHT: usize = (150.0 * SCALE) as usize;
const GRID_WIDTH: usize = (267.0 * SCALE) as usize;
const BLOCK_SIZE: usize = 2;
const HEIGHT: u32 = (GRID_HEIGHT*BLOCK_SIZE) as u32;
const WIDTH: u32 = (GRID_WIDTH*BLOCK_SIZE) as u32;

const PAINTBRUSH_SIZE: u32 = 5;

const ACCELERATION_DUE_TO_GRAVITY: i8 = 1;
// const GRAVITY_DIRECTION_VECTOR: (i32, i32) = (0, -1);

struct Model {
    map: Vec<Vec<Option<Block>>>,
    pressed_left: bool,
    pressed_right: bool,
    current_mouse_position: Vec2,
    frame_parity: bool,
    current_block_kind: Option<Block>,
    fps: FPSCounter,
    fps_result: usize,
}

use nannou::rand;

#[derive(Clone, Copy)]
#[derive(PartialEq)]
// enum Option {
//     Empty,
//     Filled(Block),
// }

struct Block {
    block_kind: BlockKind,
    velocity_x: i8,
    velocity_y: i8,
    life_time: i8,
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
enum BlockKind {
    Concrete,
    Steel,
    Sand,
    Water,
}

impl Block {
    fn new(block_kind: BlockKind) -> Self {
        Block {
            block_kind,
            velocity_x: 0,
            velocity_y: 0,
            life_time: -1,
        }
    }
}

// Block{color: Rgb([194, 178, 128]), should_fall: true; density: 3}

impl Block {
    fn update(&mut self, map: &mut [Vec<Option<Block>>], x: usize, y: usize, frame_parity: bool) {
        self.update_block_velocity(map, y, x);
        self.move_block(map, y, x, frame_parity);
    }

    fn update_block_velocity(&mut self, map: &mut [Vec<Option<Block>>], y: usize, x: usize) {
        if self.velocity_y < 127 {
            self.velocity_y += ACCELERATION_DUE_TO_GRAVITY;
        }

        if Block::get_cell(map, x, y+1).is_none() || Block::get_cell(map, x, y+1).unwrap().is_some(){
            self.velocity_y = 1;
        }
    }

    /** returns a (possibly empty) cell, None if cell is unreachable */
    fn get_cell(map: & [Vec<Option<Block>>], x: usize, y: usize) -> Option<Option<Block>>{
        if y < GRID_HEIGHT && x < GRID_WIDTH {
            Some(map[y][x])
        } else {
            None
        }
    }
    
    fn try_to_fall(map: &mut [Vec<Option<Block>>], (x_new, y_new): (usize, usize), current_block: &Block, (current_x, current_y): (usize, usize)) -> bool {
        let new_cell = Block::get_cell(map, x_new, y_new);
        match new_cell {
            None => false,
            Some(cell) => {
                if cell.is_none() || cell.unwrap().block_kind.density() < current_block.block_kind.density() {
                    // fall down
                    Block::swap_cells(map, (current_x,current_y), (x_new, y_new));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn swap_cells(map: &mut [Vec<Option<Block>>], (x_1, y_1): (usize, usize), (x_2, y_2): (usize, usize)){
        let temp = map[y_1][x_1];
        map[y_1][x_1] = map[y_2][x_2];
        map[y_2][x_2] = temp;
    }
    
    fn move_block(&self, map: &mut [Vec<Option<Block>>], y: usize, x: usize, frame_parity: bool) {
        match self.block_kind {
            BlockKind::Concrete => {
                for new_tup in [(x,y+1)] {
                    match Block::try_to_fall(map, new_tup, self, (x,y)) {
                        true => break,
                        false => {},
                    }
                } 
            }
            BlockKind::Sand => {
                // problem: have 3 potential spots
                // they could either be:
                // unavailable (off the screen)
                // empty
                // have a block in them

                // if l_b is available -> try to move block there (will move if empty or if density is appropriate) else move on to checking the next blocks

                // alternate option (could work if we only used the tuple notation (x,y))
                
                let places_to_fall = if frame_parity {
                    [(x,y+1),(x-1,y+1),(x+1,y+1)]
                } else {
                    [(x,y+1),(x+1,y+1),(x-1,y+1)]   
                };

                // calculate desired next positions by taking the possible next directions and multiplying them by the current velocity
                let desired_next_positions = places_to_fall.map(|(x,y)| (x + self.velocity_x as usize, y+ self.velocity_y as usize));
                for new_tup in desired_next_positions {
                    match Block::try_to_fall(map, new_tup, self, (x,y)) {
                        true => break,
                        false => {},
                    }
                }           
            }
            BlockKind::Water => {
                let places_to_fall = if frame_parity {
                    [(x,y+1),(x-1,y+1),(x+1,y+1),(x+1,y),(x-1,y)]
                } else {
                    [(x,y+1),(x+1,y+1),(x-1,y+1),(x-1*self.block_kind.sliding_speed()as usize,y),(x+1*self.block_kind.sliding_speed()as usize,y)]
                };
                
                let desired_next_positions = places_to_fall.map(|(x,y)| (x + self.velocity_x as usize, y+ self.velocity_y as usize));
                for new_tup in desired_next_positions {
                    match Block::try_to_fall(map, new_tup, self, (x,y)) {
                        true => break,
                        false => {},
                    }
                }           
            }
            _ => {}
        }
    }

    fn draw(&self, x: usize, y: usize, img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let x = x;
        let y = y;

        fn put_pixel(
            x: usize,
            y: usize,
            color: Rgb<u8>,
            img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>,
        ) {
            for i in 0..(BLOCK_SIZE as u32) {
                for j in 0..(BLOCK_SIZE as u32) {
                    img.put_pixel(
                        (x * BLOCK_SIZE) as u32 + j,
                        (y * BLOCK_SIZE) as u32 + i,
                        color,
                    );
                }
            }
        }

        put_pixel(x, y, self.block_kind.color(), img);
    }
}

impl BlockKind {
    fn density(&self) -> i32 {
        match self {
            BlockKind::Concrete => 4,
            BlockKind::Steel => 5,
            BlockKind::Sand => 3,
            BlockKind::Water => 2,
            // _ => 0,
        }
    }

    fn color(&self) -> Rgb<u8> {
        match self {
            BlockKind::Concrete => Rgb([90, 90, 90]),
            BlockKind::Steel => Rgb([208, 212, 214]),
            BlockKind::Sand => Rgb([194, 178, 128]),
            BlockKind::Water => Rgb([0, 0, 255]),
        }
    }

    fn sliding_speed(&self) -> i8 {
        match self {
            BlockKind::Concrete => 0,
            BlockKind::Steel => 0,
            BlockKind::Sand => 0,
            BlockKind::Water => 10,
        }
    }
}

impl Model {
    fn new_map() -> Vec<Vec<Option<Block>>> {
        let mut outer = Vec::new();
        for _ in 0..GRID_HEIGHT {
            let mut inner = Vec::new();
            for _ in 0..GRID_WIDTH {
                match (rand::random::<f32>() * 100.0) as i32 {
                    // 0..=15 => {
                    //     inner.push(Some(Block::new(BlockKind::Concrete)));
                    // }
                    // 40..=40 => {
                    //     inner.push(Some(Block::new(BlockKind::Steel)));
                    // }
                    // 50..=64 => {
                    //     inner.push(Some(Block::new(BlockKind::Sand)));
                    // }
                    // 65..=80 => {
                    //     inner.push(Some(Block::new(BlockKind::Water)));
                    // }
                    _ => {
                        inner.push(None);
                    }
                }
            }
            outer.push(inner);
        }
        outer
    }

    fn new() -> Self {
        Self {
            map: Self::new_map(),
            pressed_left: false,
            pressed_right: false,
            current_mouse_position: vec2(0.0, 0.0),
            frame_parity: false,
            current_block_kind: Some(Block::new(BlockKind::Sand)),
            fps: FPSCounter::new(),
            fps_result: 0,
        }
    }

    fn update(&mut self) {
        process_mouse(self);

        for i in (0..self.map.len() - 1).rev() {
            for j in 0..self.map[0].len() {
                let current_block = self.map[i][j];
                match current_block {
                    None => {},
                    Some(mut block) => {
                        (&mut block).update(&mut self.map, j, i, self.frame_parity)
                    }
                }
                
            }
        }

        self.frame_parity = !self.frame_parity;
        self.fps_result = self.fps.tick();
    }
}



fn process_mouse(model: &mut Model) {
    if model.pressed_left {
        brush(model, BlockKind::Sand);
    } else if model.pressed_right {
        brush(model, BlockKind::Water)
    }
}

fn brush(model: &mut Model, kind: BlockKind) {
    let size: u32 = PAINTBRUSH_SIZE;
    for r in 0..size{
        for i in 0..720 {
            let x = r as f32 * (i as f32 * PI/360.0).cos();
            let y = r as f32 * (i as f32 * PI/360.0).sin();
            model.map[(((-model.current_mouse_position[1] + (HEIGHT as f32 / 2.0)) as usize / BLOCK_SIZE) as f32 + y) as usize]
                        [(((model.current_mouse_position[0] + (WIDTH as f32 / 2.0)) as usize / BLOCK_SIZE) as f32 + x) as usize] =
                        Some(Block::new(kind));
        }
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(WIDTH, HEIGHT)
        .event(event)
        .view(view)
        .build()
        .unwrap();
    Model::new()
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(PLUM);
    let draw = app.draw();

    let mut img: image::ImageBuffer<Rgb<u8>, Vec<u8>> =
    RgbImage::new(WIDTH, HEIGHT);

    for (i, row) in model.map.iter().enumerate() {
        for (j, block) in row.iter().enumerate() {
            match block {
                None => {},
                Some(block) => {
                    block.draw(j, i, &mut img);
                }
            }
        }
    }

    let texture = wgpu::Texture::from_image(app, &DynamicImage::ImageRgb8(img));
    draw.texture(&texture);
    draw_paintbrush(&draw, &model);
    draw.text(model.fps_result.to_string().as_str());
    draw.to_frame(app, &frame).unwrap();
}

fn draw_paintbrush(draw: &Draw, model: &Model) {
    let r: u32 = PAINTBRUSH_SIZE * BLOCK_SIZE as u32;
    draw.ellipse().color(WHITE).no_fill().stroke(WHITE).stroke_weight(1.0).w(r as f32 * 1.8).h(r as f32 * 1.8).x(model.current_mouse_position[0]).y(model.current_mouse_position[1]-BLOCK_SIZE as f32);
}

// We can also update the application based on events received by the window like key presses and
// mouse movement here.
fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    // Print events as they occur to the console
    // We can `match` on the event to do something different depending on the kind of event.
    match event {
        // Keyboard events
        KeyPressed(_key) => {
            model.map = Model::new_map();
        }
        KeyReleased(_key) => {}
        ReceivedCharacter(_char) => {}

        // Mouse events
        MouseMoved(pos) => {
            model.current_mouse_position = pos;
            if model.pressed_left {
                brush(model, BlockKind::Sand);
            } else if model.pressed_right {
                brush(model, BlockKind::Water)
            }
        }
        MousePressed(button) => {
            match button {
                MouseButton::Left => {
                    model.pressed_left = true;
                }
                MouseButton::Right => {
                    model.pressed_right = true;
                }
                _ => {}
            }
            
        }
        MouseReleased(button) => {
            match button {
                MouseButton::Left => {
                    model.pressed_left = false;
                }
                MouseButton::Right => {
                    model.pressed_right = false;
                }
                _ => {}
            }
        }
        MouseWheel(_amount, _phase) => {}
        MouseEntered => {}
        MouseExited => {}

        // Touch events
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}

        // Window events
        Moved(_pos) => {}
        Resized(_size) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
}
