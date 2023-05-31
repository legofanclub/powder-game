use fps_counter::FPSCounter;
use image::{Rgb, RgbImage};
use nannou::image::{self, DynamicImage};
use nannou::prelude::*;

const SCALE: f32 = 3.0;
const HEIGHT: usize = (150.0 * SCALE) as usize;
const WIDTH: usize = (267.0 * SCALE) as usize;
const BLOCK_SIZE: usize = 1;
const ACCELERATION_DUE_TO_GRAVITY: i32 = 1;
const GRAVITY_DIRECTION_VECTOR: (i32, i32) = (0, -1);

struct Model {
    map: Vec<Vec<BlockKind>>,
    pressed: bool,
    current_mouse_position: Vec2,
    frame_parity: bool,
    current_block_kind: BlockKind,
    fps: FPSCounter,
    fps_result: usize,
}

use nannou::rand;

#[derive(Clone, Copy)]
enum BlockKind {
    Empty,
    Concrete(Block),
    Steel(Block),
    Sand(Block),
    Water(Block),
}

#[derive(Clone, Copy)]
struct Block {
    velocity_x: i32,
    velocity_y: i32,
    life_time: f32,
}

impl Block {
    fn new() -> Self {
        Block {
            velocity_x: 0,
            velocity_y: 0,
            life_time: -1.0,
        }
    }
}

// impl BlockKind::Concrete {
//     const MY_STATIC: i32 = 123  ;
// }

// Block{color: Rgb([194, 178, 128]), should_fall: true; density: 3}

impl BlockKind {
    fn should_fall(&self) -> bool {
        match self {
            BlockKind::Concrete(_) => true,
            BlockKind::Steel(_) => false,
            BlockKind::Sand(_) => true,
            BlockKind::Water(_) => true,
            _ => false,
        }
    }

    fn density(&self) -> i32 {
        match self {
            BlockKind::Concrete(_) => 4,
            BlockKind::Steel(_) => 5,
            BlockKind::Sand(_) => 3,
            BlockKind::Water(_) => 2,
            _ => 0,
        }
    }

    fn color(&self) -> Rgb<u8> {
        match self {
            BlockKind::Concrete(_) => Rgb([90, 90, 90]),
            BlockKind::Steel(_) => Rgb([208, 212, 214]),
            BlockKind::Sand(_) => Rgb([194, 178, 128]),
            BlockKind::Water(_) => Rgb([0, 0, 255]),
            _ => Rgb([0, 0, 0]),
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

        put_pixel(x, y, self.color(), img);
    }

    fn update(&self, map: &mut [Vec<BlockKind>], j: usize, i: usize, frame_parity: bool) {
        self.update_block_velocity(map, i, j);
        self.move_block(map, i, j, frame_parity);
    }

    fn move_block(&self, map: &mut [Vec<BlockKind>], i: usize, j: usize, frame_parity: bool) {
        match self {
            BlockKind::Concrete(_) | BlockKind::Steel(_) => {
                let block_below = map[i + 1][j];

                if self.should_fall() && self.density() > block_below.density() {
                    let temp = map[i][j];
                    map[i][j] = block_below;
                    map[i + 1][j] = temp;
                }
            }
            BlockKind::Sand(_) => {
                let block_below = map[i + 1][j];
                let block_below_left = if j > 0 {
                    map[i + 1][j - 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                let block_below_right = if j < WIDTH - 1 {
                    map[i + 1][j + 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                if self.should_fall() && self.density() > block_below.density() {
                    // fall down
                    let temp = map[i][j];
                    map[i][j] = block_below;
                    map[i + 1][j] = temp;
                } else if self.should_fall() && self.density() > block_below_left.density() {
                    // fall down left
                    let temp = map[i][j];
                    map[i][j] = block_below_left;
                    map[i + 1][j - 1] = temp;
                } else if self.should_fall() && self.density() > block_below_right.density() {
                    // fall down right
                    let temp = map[i][j];
                    map[i][j] = block_below_right;
                    map[i + 1][j + 1] = temp;
                }
            }
            BlockKind::Water(_) => {
                let block_below = map[i + 1][j];
                let block_below_left = if j > 0 {
                    map[i + 1][j - 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                let block_below_right = if j < WIDTH - 1 {
                    map[i + 1][j + 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                let block_left = if j > 0 {
                    map[i][j - 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                let block_right = if j < WIDTH - 1 {
                    map[i][j + 1]
                } else {
                    BlockKind::Steel(Block::new())
                };

                if self.should_fall() && self.density() > block_below.density() {
                    // fall down
                    let temp = map[i][j];
                    map[i][j] = block_below;
                    map[i + 1][j] = temp;
                } else if self.should_fall() && self.density() > block_below_left.density() {
                    // fall down left
                    let temp = map[i][j];
                    map[i][j] = block_below_left;
                    map[i + 1][j - 1] = temp;
                } else if self.should_fall() && self.density() > block_below_right.density() {
                    // fall down right
                    let temp = map[i][j];
                    map[i][j] = block_below_right;
                    map[i + 1][j + 1] = temp;
                } else if self.should_fall()
                    && self.density() > block_left.density()
                    && frame_parity
                {
                    // move left
                    let temp = map[i][j];
                    map[i][j] = block_left;
                    map[i][j - 1] = temp;
                } else if self.should_fall()
                    && self.density() > block_right.density()
                    && !frame_parity
                {
                    // move right
                    let temp = map[i][j];
                    map[i][j] = block_right;
                    map[i][j + 1] = temp;
                }
            }
            _ => {}
        }
    }

    fn update_block_velocity(&self, map: &mut [Vec<BlockKind>], i: usize, j: usize) {
        todo!()
    }
}

impl Model {
    fn new_map() -> Vec<Vec<BlockKind>> {
        let mut outer = Vec::new();
        for _ in 0..HEIGHT {
            let mut inner = Vec::new();
            for _ in 0..WIDTH {
                match (rand::random::<f32>() * 100.0) as i32 {
                    0..=15 => {
                        inner.push(BlockKind::Concrete(Block::new()));
                    }
                    40..=40 => {
                        inner.push(BlockKind::Steel(Block::new()));
                    }
                    50..=64 => {
                        inner.push(BlockKind::Sand(Block::new()));
                    }
                    65..=80 => {
                        inner.push(BlockKind::Water(Block::new()));
                    }
                    _ => {
                        inner.push(BlockKind::Empty);
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
            pressed: false,
            current_mouse_position: vec2(0.0, 0.0),
            frame_parity: false,
            current_block_kind: BlockKind::Sand(Block::new()),
            fps: FPSCounter::new(),
            fps_result: 0,
        }
    }

    fn update(&mut self) {
        process_mouse(self);

        for i in (0..self.map.len() - 1).rev() {
            for j in 0..self.map[0].len() {
                let current_block = self.map[i][j];
                current_block.update(&mut self.map, j, i, self.frame_parity);
            }
        }

        self.frame_parity = !self.frame_parity;
        self.fps_result = self.fps.tick();
    }
}

fn process_mouse(model: &mut Model) {
    if model.pressed {
        model.map[(-model.current_mouse_position[1] + (HEIGHT as f32 / 2.0)) as usize]
            [(model.current_mouse_position[0] + (WIDTH as f32 / 2.0)) as usize] =
            model.current_block_kind;
        // println!(
        //     "added water at x: {} y: {} block x: {} block y: {}",
        //     model.current_mouse_position[0],
        //     model.current_mouse_position[1],
        //     ((model.current_mouse_position[0] + 400.5) / 3.0) as usize,
        //     (model.current_mouse_position[0] / 3.0) as usize
        // )
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .size((WIDTH * BLOCK_SIZE) as u32, (HEIGHT * BLOCK_SIZE) as u32)
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
        RgbImage::new((WIDTH * BLOCK_SIZE) as u32, (HEIGHT * BLOCK_SIZE) as u32);

    // let texture = wgpu::Texture::load_from_image_buffer(app, &img);

    for (i, row) in model.map.iter().enumerate() {
        for (j, block) in row.iter().enumerate() {
            block.draw(j, i, &mut img);
        }
    }

    let texture = wgpu::Texture::from_image(app, &DynamicImage::ImageRgb8(img));
    draw.texture(&texture);
    draw.text(model.fps_result.to_string().as_str());
    draw.to_frame(app, &frame).unwrap();
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
            // println!("{:?}", event);
            model.current_mouse_position = pos;
            if model.pressed {
                // todo add blocks between the last known mouse position while pressed and the current mouse position

                // println!("mapped");
                // model.map[0][50] = BlockKind::Water;
                model.map[(-model.current_mouse_position[1] + (HEIGHT as f32 / 2.0)) as usize]
                    [(model.current_mouse_position[0] + (WIDTH as f32 / 2.0)) as usize] =
                    model.current_block_kind;
            }
        }
        MousePressed(_button) => {
            model.pressed = true;
        }
        MouseReleased(_button) => {
            model.pressed = false;
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
