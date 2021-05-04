use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Drawable, DrawParam, Rect, BlendMode};

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("newtonian_rustsody", "Fire").build().expect("context and event loop");

    let my_game = MyGame::new(&mut ctx);

    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    angle: f32,
    dt: std::time::Duration,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame { angle: 0f32, dt: std::time::Duration::new(0, 0) }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = ggez::timer::delta(ctx);

        self.angle += 0.001;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::Color { r: 0f32, g: 0f32, b: 0f32, a: 1f32});

        let text = graphics::Text::new(format!("Hello ggez! dt = {}ns", self.dt.as_nanos()));

        graphics::draw(ctx, &text, graphics::DrawParam::default().rotation(self.angle)).unwrap();

        graphics::present(ctx)
    }
}
