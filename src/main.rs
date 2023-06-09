use fps_counter::FPSCounter;
use image::{Rgb, RgbImage};
use nannou::image::{self, DynamicImage};
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

const SCALE: f32 = 2.0;
const GRID_HEIGHT: usize = (150.0 * SCALE) as usize;
const GRID_WIDTH: usize = (267.0 * SCALE) as usize;
const BLOCK_SIZE: usize = 2;
const SCREEN_HEIGHT: u32 = (GRID_HEIGHT * BLOCK_SIZE) as u32;
const SCREEN_WIDTH: u32 = (GRID_WIDTH * BLOCK_SIZE) as u32;

// const UP: (i8, i8) = (0, -1);
const DOWN: (i8, i8) = (0, 1);
const LEFT: (i8, i8) = (-1, 0);
const RIGHT: (i8, i8) = (1, 0);
// const UP_LEFT: (i8, i8) = (-1, -1);
// const UP_RIGHT: (i8, i8) = (1, -1);
const DOWN_LEFT: (i8, i8) = (-1, 1);
const DOWN_RIGHT: (i8, i8) = (1, 1);

const ACCELERATION_DUE_TO_GRAVITY: i8 = 1;

struct Model {
    map: Vec<Vec<Option<Block>>>,
    pressed_left: bool,
    current_mouse_position: Vec2,
    frame_parity: bool,
    fps: FPSCounter,
    fps_result: usize,
    egui: Egui,
    settings: Settings,
}

struct Settings {
    brush_size: u32,
    fill_type: BlockKind,
}

use nannou::rand::{self, Rng};

#[derive(Clone, Copy, PartialEq)]
struct Block {
    block_kind: BlockKind,
    velocity_x: i8,
    velocity_y: i8,
    life_time: i8,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum BlockKind {
    Concrete,
    Steel,
    Sand,
    Water,
    Wood,
    Fire,
}

impl Block {
    fn new(block_kind: BlockKind) -> Self {
        match block_kind {
            BlockKind::Fire => Block {
                block_kind,
                velocity_x: 0,
                velocity_y: 0,
                life_time: 60,
            },
            _ => Block {
                block_kind,
                velocity_x: 0,
                velocity_y: 0,
                life_time: -1,
            },
        }
    }
}

impl Block {
    fn update(&mut self, map: &mut [Vec<Option<Block>>], x: usize, y: usize, frame_parity: bool) {
        self.update_block_velocity(map, y, x);
        self.move_block(map, y, x, frame_parity);
        self.handle_lifecycle(map, y, x);
    }

    fn update_block_velocity(&mut self, map: &mut [Vec<Option<Block>>], y: usize, x: usize) {
        if (self.velocity_y as i32 + ACCELERATION_DUE_TO_GRAVITY as i32) < 127 {
            map[y][x].as_mut().unwrap().velocity_y +=
                ACCELERATION_DUE_TO_GRAVITY * rand::thread_rng().gen_range(0..2);
        }

        // below is out of bounds or has a block in it
        if Block::get_cell(map, x, y + 1).is_none()
            || Block::get_cell(map, x, y + 1).unwrap().is_some()
        {
            map[y][x].as_mut().unwrap().velocity_y = 1;
        }
    }

    /** returns a (possibly empty) cell, None if cell is unreachable */
    fn get_cell(map: &[Vec<Option<Block>>], x: usize, y: usize) -> Option<Option<Block>> {
        if y < GRID_HEIGHT && x < GRID_WIDTH {
            Some(map[y][x])
        } else {
            None
        }
    }

    fn move_block(&self, map: &mut [Vec<Option<Block>>], y: usize, x: usize, frame_parity: bool) {
        if !self.block_kind.affected_by_gravity() {
            return;
        }

        if !self.velocity_based_move(map, y, x, frame_parity) {
            // make a simple move if the velocity based move doesn't change the position of the block
            self.simple_rules_move(map, (x, y), frame_parity);
        }
    }

    fn velocity_based_move(
        &self,
        map: &mut [Vec<Option<Block>>],
        y: usize,
        x: usize,
        _frame_parity: bool,
    ) -> bool {
        let desired_position = (x + self.velocity_x as usize, y + self.velocity_y as usize);
        self.long_move(map, (x, y), desired_position)
    }

    fn long_move(
        &self,
        map: &mut [Vec<Option<Block>>],
        current_position: (usize, usize),
        desired_position: (usize, usize),
    ) -> bool {
        let mut best_position = None;

        for position in Self::get_positions_iterator(current_position, desired_position) {
            if position_is_empty(position, map) {
                best_position = Some(position);
            } else {
                // break on the closest invalid position or filled block
                break;
            }
        }

        if let Some(best_position) = best_position {
            Self::swap_cells(map, current_position, best_position);
            true
        } else {
            false
        }
    }

    /// iterator of coordinates in line from start coordinate to end coordinate
    fn get_positions_iterator(
        start: (usize, usize),
        end: (usize, usize),
    ) -> impl Iterator<Item = (usize, usize)> {
        let x_diff = end.0 as i32 - start.0 as i32;
        let y_diff = end.1 as i32 - start.1 as i32;

        let mut x_diff_sign: i32 = 1;
        if x_diff < 0 {
            x_diff_sign = -1;
        }

        let mut y_diff_sign: i32 = 1;
        if y_diff < 0 {
            y_diff_sign = -1;
        }

        let (bigger_difference, smaller_difference, x_main);

        if x_diff.abs() > y_diff.abs() {
            bigger_difference = x_diff;
            smaller_difference = y_diff;
            x_main = true;
        } else {
            bigger_difference = y_diff;
            smaller_difference = x_diff;
            x_main = false;
        };

        let ratio: f32 = smaller_difference as f32 / bigger_difference as f32;

        let mut i = 1;
        std::iter::from_fn(move || {
            if i <= bigger_difference.abs() {
                let current_i = i;
                i += 1;
                if x_main {
                    Some((
                        (start.0 as i32 + current_i * x_diff_sign) as usize,
                        (start.1 as i32 + (current_i as f32 * y_diff_sign as f32 * ratio) as i32)
                            as usize,
                    ))
                } else {
                    Some((
                        (start.0 as i32 + (current_i as f32 * x_diff_sign as f32 * ratio) as i32)
                            as usize,
                        start.1 + current_i as usize * y_diff_sign as usize,
                    ))
                }
            } else {
                None
            }
        })
    }

    fn swap_cells(
        map: &mut [Vec<Option<Block>>],
        (x_1, y_1): (usize, usize),
        (x_2, y_2): (usize, usize),
    ) {
        let temp = map[y_1][x_1];
        map[y_1][x_1] = map[y_2][x_2];
        map[y_2][x_2] = temp;
    }

    fn simple_rules_move(
        &self,
        map: &mut [Vec<Option<Block>>],
        (x, y): (usize, usize),
        frame_parity: bool,
    ) {
        for mut move_group in self.block_kind.directions_to_fall() {
            // 'randomize' order of move group
            if frame_parity {
                move_group.reverse();
            }

            for direction in move_group {
                if self.try_simple_move(map, direction, (x, y)) {
                    return;
                }
            }
        }
    }

    fn try_simple_move(
        &self,
        map: &mut [Vec<Option<Block>>],
        direction: (i8, i8),
        (current_x, current_y): (usize, usize),
    ) -> bool {
        let mut x_new = (current_x as i32 + direction.0 as i32) as usize;
        let y_new = (current_y as i32 + direction.1 as i32) as usize;

        // if the move is a slide, use sliding speed and do a long move
        if y_new == current_y && self.block_kind.sliding_speed() > 0 {
            x_new = ((current_x as i32)
                + (direction.0 as i32 * self.block_kind.sliding_speed() as i32))
                as usize;
            return self.long_move(map, (current_x, current_y), (x_new, y_new));
        }

        let new_cell = Block::get_cell(map, x_new, y_new);
        match new_cell {
            None => false, // cell is off the map
            Some(cell) => {
                if cell.is_none() || cell.unwrap().block_kind.density() < self.block_kind.density()
                {
                    Block::swap_cells(map, (current_x, current_y), (x_new, y_new));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn handle_lifecycle(&mut self, map: &mut [Vec<Option<Block>>], y: usize, x: usize) {
        if !self.block_kind.has_lifecycle() {
            return;
        }

        map[y][x].as_mut().unwrap().life_time -= 1;

        if self.life_time < 1 {
            map[y][x] = None;
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

// returns true if position is valid and empty
fn position_is_empty((x, y): (usize, usize), map: &[Vec<Option<Block>>]) -> bool {
    if y < GRID_HEIGHT && x < GRID_WIDTH && y > 0 && x > 0 {
        matches!(map[y][x], None)
    } else {
        false
    }
}

impl BlockKind {
    fn density(&self) -> i32 {
        match self {
            BlockKind::Concrete => 4,
            BlockKind::Steel => 5,
            BlockKind::Sand => 3,
            BlockKind::Water => 2,
            BlockKind::Wood => 5,
            BlockKind::Fire => 0,
        }
    }

    fn color(&self) -> Rgb<u8> {
        match self {
            BlockKind::Concrete => Rgb([90, 90, 90]),
            BlockKind::Steel => Rgb([208, 212, 214]),
            BlockKind::Sand => Rgb([194, 178, 128]),
            BlockKind::Water => Rgb([0, 0, 255]),
            BlockKind::Wood => Rgb([58, 29, 0]),
            BlockKind::Fire => Rgb([255, 0, 0]),
        }
    }

    fn sliding_speed(&self) -> i8 {
        match self {
            BlockKind::Concrete => 0,
            BlockKind::Steel => 0,
            BlockKind::Sand => 0,
            BlockKind::Water => 10,
            BlockKind::Wood => 0,
            BlockKind::Fire => 0,
        }
    }

    // the outer vec is ordered from 'do first' to 'do last'
    // each inner vec is a group of directions that could/should be executed in any order
    fn directions_to_fall(&self) -> Vec<Vec<(i8, i8)>> {
        match self {
            BlockKind::Concrete => vec![vec![DOWN]],
            BlockKind::Steel => vec![],
            BlockKind::Sand => vec![vec![DOWN], vec![DOWN_LEFT, DOWN_RIGHT]],
            BlockKind::Water => vec![vec![DOWN], vec![DOWN_LEFT, DOWN_RIGHT], vec![LEFT, RIGHT]],
            BlockKind::Wood => vec![],
            BlockKind::Fire => vec![],
        }
    }

    fn affected_by_gravity(&self) -> bool {
        match self {
            BlockKind::Concrete => true,
            BlockKind::Steel => false,
            BlockKind::Sand => true,
            BlockKind::Water => true,
            BlockKind::Wood => false,
            BlockKind::Fire => false,
        }
    }

    fn has_lifecycle(&self) -> bool {
        match self {
            BlockKind::Concrete => false,
            BlockKind::Steel => false,
            BlockKind::Sand => false,
            BlockKind::Water => false,
            BlockKind::Wood => false,
            BlockKind::Fire => true,
        }
    }
}

impl Model {
    fn new_map() -> Vec<Vec<Option<Block>>> {
        let mut outer = Vec::new();
        for _ in 0..GRID_HEIGHT {
            let mut inner = Vec::new();
            for _ in 0..GRID_WIDTH {
                // match (rand::random::<f32>() * 100.0) as i32 {
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
                // _ => {
                inner.push(None);
                // }
                // }
            }
            outer.push(inner);
        }
        outer
    }

    fn new(egui: Egui) -> Self {
        Self {
            map: Self::new_map(),
            pressed_left: false,
            current_mouse_position: vec2(0.0, 0.0),
            frame_parity: false,
            fps: FPSCounter::new(),
            fps_result: 0,
            egui,
            settings: Settings {
                brush_size: 4,
                fill_type: BlockKind::Sand,
            },
        }
    }

    fn update(&mut self) {
        process_mouse(self);

        self.gui();
        self.update_world();

        self.frame_parity = !self.frame_parity;
        self.fps_result = self.fps.tick();
    }

    fn update_world(&mut self) {
        for i in (0..self.map.len() - 1).rev() {
            // the if else block is to iterate from right to left and left to right on different frame parity
            if self.frame_parity {
                for j in 0..self.map[0].len() {
                    let current_block = &mut self.map[i][j];
                    match current_block {
                        None => {}
                        Some(mut block) => block.update(&mut self.map, j, i, self.frame_parity),
                    }
                }
            } else {
                for j in (0..self.map[0].len()).rev() {
                    let current_block = &mut self.map[i][j];
                    match current_block {
                        None => {}
                        Some(mut block) => block.update(&mut self.map, j, i, self.frame_parity),
                    }
                }
            }
        }
    }

    fn gui(&mut self) {
        let egui = &mut self.egui;
        let ctx = egui.begin_frame();

        egui::Window::new("Settings").show(&ctx, |ui| {
            // brush size slider
            ui.label("Brush Size:");
            ui.add(egui::Slider::new(&mut self.settings.brush_size, 1..=40));

            // dropdown to select material type
            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", self.settings.fill_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.fill_type, BlockKind::Sand, "Sand");
                    ui.selectable_value(&mut self.settings.fill_type, BlockKind::Water, "Water");
                    ui.selectable_value(
                        &mut self.settings.fill_type,
                        BlockKind::Concrete,
                        "Concrete",
                    );
                    ui.selectable_value(&mut self.settings.fill_type, BlockKind::Steel, "Steel");
                    ui.selectable_value(&mut self.settings.fill_type, BlockKind::Wood, "Wood");
                    ui.selectable_value(&mut self.settings.fill_type, BlockKind::Fire, "Fire");
                });
        });
    }
}

fn process_mouse(model: &mut Model) {
    if model.pressed_left {
        brush(model, model.settings.fill_type);
    }
}

fn brush(model: &mut Model, kind: BlockKind) {
    let size: u32 = model.settings.brush_size;
    for r in 0..size {
        for i in 0..720 {
            let x = r as f32 * (i as f32 * PI / 360.0).cos();
            let y = r as f32 * (i as f32 * PI / 360.0).sin();
            model.map[(((-model.current_mouse_position[1] + (SCREEN_HEIGHT as f32 / 2.0)) as usize
                / BLOCK_SIZE) as f32
                + y) as usize][(((model.current_mouse_position[0]
                + (SCREEN_WIDTH as f32 / 2.0))
                as usize
                / BLOCK_SIZE) as f32
                + x) as usize] = Some(Block::new(kind));
        }
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .event(event)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    Model::new(egui)
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(PLUM);
    let draw = app.draw();

    let mut img: image::ImageBuffer<Rgb<u8>, Vec<u8>> = RgbImage::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    for (i, row) in model.map.iter().enumerate() {
        for (j, block) in row.iter().enumerate() {
            match block {
                None => {}
                Some(block) => {
                    block.draw(j, i, &mut img);
                }
            }
        }
    }

    let texture = wgpu::Texture::from_image(app, &DynamicImage::ImageRgb8(img));
    draw.texture(&texture);
    draw_paintbrush(&draw, model);
    draw.text(model.fps_result.to_string().as_str());
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn draw_paintbrush(draw: &Draw, model: &Model) {
    let r: u32 = model.settings.brush_size * BLOCK_SIZE as u32;
    draw.ellipse()
        .color(WHITE)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .w(r as f32 * 1.8)
        .h(r as f32 * 1.8)
        .x(model.current_mouse_position[0])
        .y(model.current_mouse_position[1] - BLOCK_SIZE as f32);
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
                brush(model, model.settings.fill_type);
            }
        }
        MousePressed(button) => {
            if button == MouseButton::Left {
                model.pressed_left = true;
            }
        }
        MouseReleased(button) => {
            if button == MouseButton::Left {
                model.pressed_left = false;
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
