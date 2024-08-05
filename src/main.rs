use std::{collections::VecDeque, hint::unreachable_unchecked};

use nannou::{event::Update, glam::Vec2, rand::{self, thread_rng, Rng}, time::DurationF64, App, Frame};
use rayon::iter::{ParallelBridge, ParallelIterator};

fn main() {
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    nannou::app(Model::new)
        .update(Model::update)
        .loop_mode(nannou::LoopMode::refresh_sync())
        .run();
}

struct Model {
    snake: Snake,
    timer: f32,
    bot: bool,
    highscore: u32,
    running: bool,
    speed: f32,
    size: u32,
    key_cooldown: f32,
}

impl Model {
    fn new(app: &nannou::App) -> Self {
        app.new_window()
            .size(800, 800)
            .view(Model::view)
            .build()
            .unwrap();

        Model {
            snake: Snake::new((-6, -6, 6, 6)),
            timer: 0.0,
            bot: true,
            highscore: 0,
            running: true,
            speed: 0.1,
            size: 6,
            key_cooldown: 0.0,
        }

    }

    pub fn update(app: &App, model: &mut Model, update: Update) {
        handle_keyboard_input(model, update, app);
        
        if !model.running {
            return;
        }
        let step_time = 1.0 / 90 as f32 / model.speed;
        model.timer += update.since_last.secs() as f32;

        if model.speed < 1.0 {
            if model.timer < step_time {
                return;
            }
        }

        if model.snake.body.len() == 0 {
            model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
            model.timer = -10.0;
        }


        if model.bot {
            for _ in 0..(model.speed as usize).max(1) {
                let dir = Snake::bot_move(model);
                model.snake.direction = dir;
                if !model.snake.step() {
                    model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
                    model.timer = -3.0;
                }
            }
        } else {
            if !model.snake.step() {
                model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
                model.timer = -3.0;
            }
        }

        if model.snake.body.len() as u32 > model.highscore {
            model.highscore = model.snake.body.len() as u32;
            println!("Highscore: {}", model.highscore);
        }

        model.timer -= step_time;
    }

    pub fn view(app: &App, model: &Model, frame: Frame) {
        let draw = app.draw();

        draw.background().color(nannou::color::BLACK);

        let size = app.window_rect().w().min(app.window_rect().h());

        let sqare_size = size / (model.snake.borders.2 - model.snake.borders.0) as f32;
        for i in model.snake.borders.0..model.snake.borders.2 {
            for j in model.snake.borders.1..model.snake.borders.3 {
                let (x, y) = model.to_screen_coords(size, i, j);
                if i == model.snake.borders.0 || i == model.snake.borders.2 - 1 || j == model.snake.borders.1 || j == model.snake.borders.3 - 1 {
                    let color = nannou::color::rgb(0.2, 0.4, 0.2);
                    draw.rect()
                        .x_y(x, y)
                        .w_h(sqare_size * 0.9, sqare_size * 0.9)
                        .z(0.0)
                        .color(color);
                } else {
                    let color = if (i + j + size as i32 * size as i32) % 2 == 0 {
                        nannou::color::rgb(0.02, 0.2, 0.02)
                    } else {
                        nannou::color::rgb(0.04, 0.1, 0.04)
                    };
                    draw.rect()
                        .x_y(x, y)
                        .w_h(sqare_size * 0.9, sqare_size * 0.9)
                        .z(0.0)
                        .color(color);
                }
            }
        }

        for i in 0..model.snake.direction_path.len() {
            let current = model.snake.direction_path.get(i).unwrap();
            let x = model.snake.borders.0 + 1 + i as i32 % (model.snake.borders.2 - model.snake.borders.0 - 2);
            let y = model.snake.borders.1 + 1 + i as i32 / (model.snake.borders.2 - model.snake.borders.0 - 2);
            let (next_x, next_y) = match current {
                Direction::Up => (x, y + 1),
                Direction::Down => (x, y - 1),
                Direction::Left => (x - 1, y),
                Direction::Right => (x + 1, y),
            };
            let square_size = size;

            let start = model.to_screen_coords(square_size, x, y);
            let end = model.to_screen_coords(square_size, next_x, next_y);

            draw.line()
                .start(Vec2::new(start.0, start.1))
                .end(Vec2::new(end.0, end.1))
                .color(nannou::color::rgba(1.0, 1.0, 1.0, 0.3));
        }

        let base = 0.55f32.powf(1.0 / model.snake.body.len() as f32).max(0.94);
        let size_mult = (model.snake.body.len() as f32 / 200.0).max(0.6).min(0.87);
        for i in 0..model.snake.body.len() as usize - 1 {
            let (x, y) = model.snake.body.get(i).unwrap();
            let (x, y) = model.to_screen_coords(size, *x, *y);
            let (n_x, n_y) = model.snake.body.get(i + 1).unwrap();
            let (n_x, n_y) = model.to_screen_coords(size, *n_x, *n_y);

            let min_green = 150;
            let max_green = 255;
            let green = min_green as f32 + (max_green - min_green) as f32 * ((model.snake.body.len() - i - 1) as f32 / model.snake.body.len() as f32);
            let color = nannou::color::rgb(20, green as u8, 20);
            let width = base.powi(i as i32) * sqare_size * size_mult;

            draw.line()
                .start(Vec2::new(x, y))
                .end(Vec2::new(n_x, n_y))
                .color(color)
                .z(model.snake.body.len() as f32 - i as f32 )
                .stroke_weight(width * base);

            let green = min_green as f32 + (max_green - min_green) as f32 * ((model.snake.body.len() - i) as f32 / model.snake.body.len() as f32);
            let color = nannou::color::rgb(20, green as u8, 20);
            draw.ellipse()
                .x_y(x, y)
                .w_h(width, width)
                .z(model.snake.body.len() as f32 - i as f32 + 0.5)
                .color(color);
        }
        let (x, y) = model.snake.body.back().unwrap();
        let (x, y) = model.to_screen_coords(size, *x, *y);
        let size = base.powi(model.snake.body.len() as i32 - 1) * sqare_size;
        let min_green = 150;
        let max_green = 255;
        let green = min_green as f32 + (max_green - min_green) as f32 * ((1) as f32 / model.snake.body.len() as f32);
        let color = nannou::color::rgb(20, green as u8, 20);
        draw.ellipse()
            .x_y(x, y)
            .w_h(size * size_mult, size * size_mult)
            .z(1.0)
            .color(color);

        draw.ellipse()
            .x_y(model.snake.apple.0 as f32 * sqare_size + sqare_size / 2.0, model.snake.apple.1 as f32 * sqare_size + sqare_size / 2.0)
            .w_h(sqare_size * 0.7, sqare_size * 0.7)
            .z(1.5)
            .color(nannou::color::RED);

        draw.to_frame(app, &frame).unwrap();
    }

    fn to_screen_coords(&self, size: f32, x: i32, y: i32) -> (f32, f32) {
        let sqare_size = size / (self.snake.borders.2 - self.snake.borders.0) as f32;
        let x = x as f32 * sqare_size + sqare_size / 2.0;
        let y = y as f32 * sqare_size + sqare_size / 2.0;
        (x, y)
    }


}



fn handle_keyboard_input(model: &mut Model, update: Update, app: &App) {
    model.key_cooldown -= update.since_last.secs() as f32;
    app.keys.down.iter().for_each(|key| {
        match key {
            nannou::event::Key::W => {
                model.snake.direction = Direction::Up;
            }
            nannou::event::Key::S => {
                model.snake.direction = Direction::Down;
            }
            nannou::event::Key::A => {
                model.snake.direction = Direction::Left;
            }
            nannou::event::Key::D => {
                model.snake.direction = Direction::Right;
            }
            nannou::event::Key::Up => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.speed *= 1.2;
                model.key_cooldown = 0.1;
            }
            nannou::event::Key::Down => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.speed /= 1.2;
                model.key_cooldown = 0.1;
            }
            nannou::event::Key::Right => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.size += 2;
                model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
                model.key_cooldown = 0.1;
            }
            nannou::event::Key::Left => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.size = (model.size - 2).max(4); 

                model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
                model.key_cooldown = 0.1;
            }
            nannou::event::Key::Return => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.bot = !model.bot;
                model.key_cooldown = 0.2;
            }
            nannou::event::Key::Space => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.running = !model.running;
                model.key_cooldown = 0.2;
            }
            nannou::event::Key::R => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                model.snake = Snake::new((-(model.size as i32), -(model.size as i32), model.size as i32, model.size as i32));
                model.key_cooldown = 0.1;
            }
            nannou::event::Key::F11 => {
                if model.key_cooldown > 0.0 {
                    return;
                }
                let win_id = app.window_id();
                let window = app.window(win_id).unwrap();
                window.set_fullscreen(!window.is_fullscreen());
                model.key_cooldown = 0.1;
            }
            _ => {}
        }
    });
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
struct Snake {
    body: VecDeque<(i32, i32)>,
    direction: Direction,
    borders: (i32, i32, i32, i32),
    apple: (i32, i32),
    direction_path: Vec<Direction>,
}

impl Snake {
    fn new(borders: (i32, i32, i32, i32)) -> Self {
        let mut body = VecDeque::new();
        body.push_back((0, 0));
        Snake {
            body,
            direction: Direction::Up,
            borders,
            apple: (1, 1),
            direction_path: Snake::init_path_direction((borders.2 - borders.0 - 2) as u32),
        }
    }


    fn init_path_direction(size: u32) -> Vec<Direction> {
        let mut path = (0..size).flat_map(|y |
            (0..size).map(move |x| {
                if y == size - 1 && x > 0 {
                    return Direction::Left;
                }
        
                if x % 2 == 0 {
                    if y == 0 {
                        return Direction::Right;
                    } else {
                        return Direction::Down;
                    }
                } else {
                    if y == size - 2 && x < size - 1 {
                        return Direction::Right;
                    } else {
                        return Direction::Up;
                    }
                }
            })
        ).collect::<Vec<Direction>>();    

        let num_iterations = (size * size) as u64;
        
        let progress = indicatif::ProgressBar::new(num_iterations);
        progress.inc(0);
        let offsets = [-1, -(size as i32) + 1, -(size as i32), -(size as i32) - 1, 1, size as i32 - 1, size as i32 + 1, size as i32];

        for _ in 0..num_iterations {
            progress.inc(1);
            let tmp_path = (0..)
                .par_bridge()
                .map(|_| {
                    let random_index_1 = thread_rng().gen_range(0..path.len());
                    let random_index_2 = (random_index_1 as i32 + offsets[thread_rng().gen_range(0..8)]).min(path.len() as i32 - 1).max(0) as usize;
                    let random_index_3 = thread_rng().gen_range(0..path.len());
                    let random_index_4 = (random_index_3 as i32 + offsets[thread_rng().gen_range(0..8)]).min(path.len() as i32 - 1).max(0) as usize;

                    let random_dir_1 = Snake::random_dir(path[random_index_1]);
                    let random_dir_2 = Snake::random_dir(path[random_index_2]);
                    let random_dir_3 = Snake::random_dir(path[random_index_3]);
                    let random_dir_4 = Snake::random_dir(path[random_index_4]);
                    (random_index_1, random_index_2, random_index_3, random_index_4, random_dir_1, random_dir_2, random_dir_3, random_dir_4)
                })
                .find_any(|(random_index1, random_index2, random_index3, random_index4, random_dir1, random_dir2, random_dir3, random_dir4)| {
                    let mut x = 0;
                    let mut y = 0;
                    let mut path_len = 0;
                    let mut seen: Vec<bool> = vec![false; path.len()];
                    let mut new_path = path.clone();
                    new_path[*random_index1] = *random_dir1;
                    new_path[*random_index2] = *random_dir2;
                    new_path[*random_index3] = *random_dir3;
                    new_path[*random_index4] = *random_dir4;
                    loop {
                        if seen[x as usize + y as usize * size as usize] {
                            break;
                        }
                        seen[x as usize + y as usize * size as usize] = true;
                        match new_path[x as usize + y as usize * size as usize] {
                            Direction::Up => y += 1,
                            Direction::Down => y -= 1,
                            Direction::Left => x -= 1,
                            Direction::Right => x += 1,
                        }
                        path_len += 1;
                        if (x == 0 && y == 0) || x < 0 || x >= size as i32 || y < 0 || y >= size as i32 {
                            break;
                        }
                    }
                    
                    path_len == new_path.len() && x == 0 && y == 0
                }).unwrap();
            
            let (random_index1, random_index2, random_index3, random_index4, random_dir1, random_dir2, random_dir3, random_dir4) = tmp_path;
            path[random_index1] = random_dir1;
            path[random_index2] = random_dir2;
            path[random_index3] = random_dir3;
            path[random_index4] = random_dir4;
        }


        path
    }

    fn path_direction(&self, x: i32, y: i32) -> Direction {
        self.direction_path[(y - self.borders.0 - 1) as usize * (self.borders.3 - self.borders.1 - 2) as usize + (x - self.borders.1 - 1) as usize]
    }

    fn free_path_len(&self, base_x: i32, base_y: i32, max_len: u32) -> u32 {
        let (mut x, mut y) = (base_x, base_y);
        let mut len = 0;
        let snakelen = self.body.len();
        let mut apples = 0;
        loop {
            match self.path_direction(x, y) {
                Direction::Up => y += 1,
                Direction::Down => y -= 1,
                Direction::Left => x -= 1,
                Direction::Right => x += 1,
            }
            if x == self.apple.0 && y == self.apple.1 {
                apples += 1;
            }
            len += 1;
            let collision_index = self.body.iter().take(snakelen - len + apples).position(|&(x_body, y_body)| x_body == x && y_body == y); 
            if collision_index.is_some() || len >= max_len as usize || (base_x == x && base_y == y) {
                break;
            }
        }
        len as u32
    }

    fn path_len(&self, x: i32,  y: i32) -> u32 {
        let (mut x, mut y) = (x,y);
        let mut len = 0;
        while x != self.apple.0 || y != self.apple.1 {
            match self.path_direction(x, y) {
                Direction::Up => y += 1,
                Direction::Down => y -= 1,
                Direction::Left => x -= 1,
                Direction::Right => x += 1,
            }
            len += 1;
        }
        len
    }

    fn step(&mut self) -> bool {
        if self.body.len() == 0 || self.body.len() as i32 == (self.borders.2 - self.borders.0 - 2) * (self.borders.3 - self.borders.1 - 2) - 1 {
            return false;
        }
        let (x, y) = self.body.front().unwrap();
        let (x, y) = match self.direction {
            Direction::Up => (*x, y + 1),
            Direction::Down => (*x, y - 1),
            Direction::Left => (x - 1, *y),
            Direction::Right => (x + 1, *y),
        };
        if x == self.apple.0 && y == self.apple.1 {
            let (x,y) = loop {
                let random_x = thread_rng().gen_range(self.borders.0 + 1..self.borders.2 - 1);
                let random_y = thread_rng().gen_range(self.borders.1 + 1..self.borders.3 - 1);
                if !self.body.contains(&(random_x, random_y)) && !(random_x == x && random_y == y) {
                    break (random_x, random_y);
                }
            };
            self.apple = (x, y);
        } else {
            self.body.pop_back();
        }
        if self.body.contains(&(x, y)) || x <= self.borders.0 || x >= self.borders.2 - 1 || y <= self.borders.1 || y >= self.borders.3 - 1 {
            return false;
        }
        self.body.push_front((x, y));
        true
    }

    fn random_dir(dir: Direction) -> Direction {
        match dir {
            Direction::Up => {
                match thread_rng().gen_range(0..3) {
                    0 => Direction::Left,
                    1 => Direction::Right,
                    2 => Direction::Up,
                    _ => unsafe { unreachable_unchecked() }
                }
            }
            Direction::Down => {
                match thread_rng().gen_range(0..3) {
                    0 => Direction::Left,
                    1 => Direction::Right,
                    2 => Direction::Up,
                    _ => unsafe { unreachable_unchecked() }
                }
            }
            Direction::Left => {
                match thread_rng().gen_range(0..3) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Right,
                    _ => unsafe { unreachable_unchecked() }
                }
            }
            Direction::Right => {
                match thread_rng().gen_range(0..3) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    _ => unsafe { unreachable_unchecked() }
                }
            }
        }
    }

    fn bot_move(model: &mut Model) -> Direction {
        let &(x, y) = model.snake.body.front().unwrap();
        let mut dir = model.snake.path_direction(x, y);
        let mut path_len = model.snake.path_len(x, y);
        let snake_len = model.snake.body.len() as u32;
        
        if y < model.snake.borders.3 - 2
        && x < model.snake.borders.2 - 2
        && model.snake.path_len(x, y + 1) + 1 < path_len
        && !model.snake.body.contains(&(x, y + 1))
        && snake_len <= model.snake.free_path_len(x, y + 1, snake_len) {
            path_len = model.snake.path_len(x, y + 1) + 1;
            dir = Direction::Up;
        }
        if y > model.snake.borders.1 + 1
        && x < model.snake.borders.2 - 2
        && model.snake.path_len(x, y - 1) + 1 < path_len
        && !model.snake.body.contains(&(x, y - 1))
        && snake_len <= model.snake.free_path_len(x, y - 1, snake_len) {
            path_len = model.snake.path_len(x, y - 1) + 1;
            dir = Direction::Down;
        }

        if x > model.snake.borders.0 + 1
        && y < model.snake.borders.3 - 2
        && model.snake.path_len(x - 1, y) + 1 < path_len
        && !model.snake.body.contains(&(x - 1, y))
        && snake_len <= model.snake.free_path_len(x - 1, y, snake_len) {
            path_len = model.snake.path_len(x - 1, y) + 1;
            dir = Direction::Left;
        }   

        if x < model.snake.borders.2 - 2
        && y < model.snake.borders.3 - 2
        && model.snake.path_len(x + 1, y) + 1 < path_len
        && !model.snake.body.contains(&(x + 1, y))
        && snake_len <= model.snake.free_path_len(x + 1, y, snake_len) {
            dir = Direction::Right;
        }
        dir 
    }
}

