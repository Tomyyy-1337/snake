use std::collections::VecDeque;

use nannou::{event::Update, rand::{thread_rng, Rng}, time::DurationF64, App, Frame};

fn main() {
    nannou::app(Model::new).update(Model::update).run();
}

struct Model {
    snake: Snake,
    timer: f32,
    bot: bool,
    highscore: u32,
    running: bool,
}

impl Model {
    fn new(app: &nannou::App) -> Self {
        app.new_window()
            .size(512, 512)
            .view(Model::view)
            .build()
            .unwrap();

        Model {
            snake: Snake::new((-20, -20, 20, 20)),
            timer: 0.0,
            bot: true,
            highscore: 0,
            running: true,
        }

    }

    pub fn update(app: &App, model: &mut Model, update: Update) {
        app.keys.down.iter().for_each(|key| {
            match key {
                nannou::event::Key::Up => {
                    model.snake.direction = Direction::Up;
                }
                nannou::event::Key::Down => {
                    model.snake.direction = Direction::Down;
                }
                nannou::event::Key::Left => {
                    model.snake.direction = Direction::Left;
                }
                nannou::event::Key::Right => {
                    model.snake.direction = Direction::Right;
                }
                nannou::event::Key::Return => {
                    model.bot = !model.bot;
                }
                nannou::event::Key::Space => {
                    model.running = !model.running;
                }
                nannou::event::Key::R => {
                    model.snake = Snake::new((-20, -20, 20, 20));
                }
                _ => {}
            }
        });
        
        if !model.running {
            return;
        }
        
        model.timer += update.since_last.secs() as f32;
        if model.timer < 0.02 {
            return;
        }


        if model.bot {
            let snake_len = model.snake.body.len();
            let height = model.snake.borders.3 - model.snake.borders.1 - 4;
            let colls = snake_len / height as usize + 2;

            if model.snake.apple.0 >= model.snake.body.front().unwrap().0 && model.snake.body.front().unwrap().0 < model.snake.borders.2 - colls as i32 {
                if let Some(direction) = model.snake.direction_to_apple() {
                    model.snake.direction = direction;
                } else{
                    model.snake.direction = model.snake.path_direction();
                }
            } else {
                if model.snake.body.front().unwrap().0 < model.snake.borders.2 - colls as i32 && model.snake.body.front().unwrap().1 < model.snake.borders.3 - 2 {
                    model.snake.direction = Direction::Right;
                } else {
                    model.snake.direction = model.snake.path_direction();
                }
            }
        }
        model.snake.step();

        if model.snake.body.len() as u32 > model.highscore {
            model.highscore = model.snake.body.len() as u32;
            println!("Highscore: {}", model.highscore);
        }

        model.timer = 0.0;
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
                    let color = nannou::color::ORANGE;
                    draw.rect()
                        .x_y(x, y)
                        .w_h(sqare_size, sqare_size)
                        .z(0.0)
                        .color(color);
                };
            }
        }

        for (i, (x, y)) in model.snake.body.iter().enumerate() {
            let (x, y) = model.to_screen_coords(size, *x, *y);

            let min_green = 100;
            let max_green = 255;
            let green = min_green as f32 + (max_green - min_green) as f32 * ((model.snake.body.len() - i) as f32 / model.snake.body.len() as f32);
            let color = nannou::color::rgb(20, green as u8, 20);

            draw.rect()
                .x_y(x, y)
                .w_h(sqare_size, sqare_size)
                .z(1.0)
                .color(color);
        }

        let (x, y) = model.to_screen_coords(size, model.snake.apple.0, model.snake.apple.1);
        draw.rect()
            .x_y(x, y)
            .w_h(sqare_size, sqare_size)
            .z(1.0)
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
        }
    }

    fn path_direction(&self) -> Direction {
        let &(x, y) = self.body.front().unwrap();

        if y == self.borders.3 - 2 && x > self.borders.0 + 1 {
            return Direction::Left;
        }

        if x % 2 != self.borders.0 % 2 {
            if y == self.borders.1 + 1 {
                return Direction::Right;
            } else {
                return Direction::Down;
            }
        } else {
            if y == self.borders.3 - 3 && x < self.borders.2 - 2 {
                return Direction::Right;
            } else {
                return Direction::Up;
            }
        }
    }

    fn direction_to_apple(&self) -> Option<Direction> {
        // rank directions
        let mut directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        directions.sort_unstable_by_key(|a| {
            let (x, y) = self.body.front().unwrap();
            let (x_apple, y_apple) = self.apple;
            let (x, y) = match a {
                Direction::Up => (*x, y + 1),
                Direction::Down => (*x, y - 1),
                Direction::Left => (x - 1, *y),
                Direction::Right => (x + 1, *y),
            };
            let dist = |x, y, x_apple, y_apple| (x as i32 - x_apple as i32).abs() + (y as i32 - y_apple as i32).abs();
            dist(x, y, x_apple, y_apple)
        });

        directions.into_iter().find(|direction| {
            let (x, y) = self.body.front().unwrap();
            let (x, y) = match direction {
                Direction::Up => (*x, y + 1),
                Direction::Down => (*x, y - 1),
                Direction::Left => (x - 1, *y),
                Direction::Right => (x + 1, *y),
            };
            let mut clone = Snake {
                body: self.body.clone(),
                direction: *direction,
                borders: self.borders,
                apple: self.apple,
            };
            !self.body.contains(&(x, y)) && x > self.borders.0 && x < self.borders.2 && y > self.borders.1 && y < self.borders.3 - 1 && clone.step()
        })
    }


    fn step(&mut self) -> bool {
        if self.body.len() == 0 || self.body.len() as i32 == (self.borders.2 - self.borders.0 - 2) * (self.borders.3 - self.borders.1 - 2) {
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
                if !self.body.contains(&(random_x, random_y)) {
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
    
}

 