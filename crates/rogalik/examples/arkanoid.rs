use rogalik::prelude::*;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const PIXEL_SCALE: u32 = 4;

const SPRITE_SIZE: f32 = 8.;

const BLOCK_WIDTH: f32 = 32.;
const BLOCK_HEIGHT: f32 = 8.;
const BLOCKS_HORIZONTAL: usize = 4;
const BLOCKS_VERTICAL: usize = 4;

const PADDLE_WIDTH: f32 = 32.;
const PADDLE_HEIGHT: f32 = 8.;
const PADDLE_OFFSET: f32 = 8.;
const PADDLE_MOVE: f32 = 4.;

const BALL_SIZE: f32 = 8.;
const BALL_MOVE: f32 = 2.;

const PADDLE_SPRITE: usize = 0;
const BLOCK_SPRITE: usize = 1;
const BALL_SPRITE: usize = 2;

const MAX_LIVES: usize = 3;

#[derive(Default)]
struct GameState {
    lives: usize,
    width: u32,
    height: u32,
    paddle_origin: Vector2f,
    ball_origin: Option<Vector2f>,
    ball_velocity: Vector2f,
    blocks: [[bool; BLOCKS_VERTICAL]; BLOCKS_HORIZONTAL],
}
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load sprite texture
        let sprite_texture = Some(
            context
                .graphics
                .load_texture("examples/arkanoid/sprites.png"),
        );
        // Create sprite material
        context.graphics.load_material(
            "sprites",
            MaterialParams {
                atlas: Some(AtlasParams {
                    cols: 4,
                    rows: 1,
                    padding: None,
                }),
                diffuse_texture: sprite_texture,
                ..Default::default()
            },
        );

        // Load bitmap font
        context
            .graphics
            .load_font("pixel", "examples/font.png", 16, 16, Some((11., 7.)), None);

        // Create camera
        context.graphics.create_camera(1., Vector2f::ZERO);
    }

    fn resize(&mut self, context: &mut Context) {
        // Handle resize
        self.width = context.get_physical_size().x as u32 / PIXEL_SCALE;
        self.height = context.get_physical_size().y as u32 / PIXEL_SCALE;
        // Set pixel upscaling
        context
            .graphics
            .set_rendering_resolution(self.width, self.height);

        // Center the camera
        context
            .graphics
            .get_current_camera_mut()
            .set_target(0.5 * Vector2f::new(self.width as f32, self.height as f32));
    }
}

/// Main game loop scene
struct GameScene;
impl Scene for GameScene {
    type Game = GameState;

    /// Initialize game state
    fn enter(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
        // Reset blocks
        for col in game.blocks.iter_mut() {
            for block in col.iter_mut() {
                *block = true;
            }
        }

        // Reset paddle and the ball
        game.paddle_origin = Vector2f::new(0., PADDLE_OFFSET);
        game.ball_origin = None;

        // Reset player
        game.lives = MAX_LIVES;
    }

    fn update(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
        update_ball(game);
        draw_world(&game, context);
        draw_status(&game, context);
        handle_input(game, context);

        // Loose condition
        if game.lives == 0 {
            scenes.switch(Box::new(EndScene("Game Over".to_string())));
        }
        // Win condition
        if !game
            .blocks
            .iter()
            .map(|col| col.iter())
            .flatten()
            .any(|b| *b)
        {
            scenes.switch(Box::new(EndScene("Congratulations!".to_string())));
        }
    }
}

/// GameOver / Win scene
struct EndScene(String);
impl Scene for EndScene {
    type Game = GameState;
    fn update(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
        let bounds = context.graphics.get_current_camera().get_bounds();
        let center = 0.5 * (bounds.0 + bounds.1);
        let width = context.graphics.text_dimensions("pixel", &self.0, 9.).x;
        let _ = context.graphics.draw_text(
            "pixel",
            &self.0,
            center - Vector2f::new(0.5 * width, 0.),
            10,
            9.,
            SpriteParams::default(),
        );

        // Reset game on key press
        if context.input.is_key_down(rogalik::input::KeyCode::Space) {
            scenes.switch(Box::new(GameScene));
        }
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("Arkanoid".to_string())
        .with_physical_size(WIDTH, HEIGHT)
        .build(GameState::default(), Box::new(GameScene));
    engine.run();
}

fn update_ball(game: &mut GameState) {
    let Some(origin) = game.ball_origin.as_mut() else {
        return;
    };

    *origin += game.ball_velocity;
    let origin = game.ball_origin.unwrap();

    // Void
    if origin.y + BALL_SIZE <= 0. {
        game.ball_origin = None;
        game.lives -= 1;
        return;
    }

    // Wall collisions
    if origin.y + BALL_SIZE >= game.height as f32 {
        game.ball_velocity.y *= -1.;
    }
    if origin.x <= 0. || origin.x + BALL_SIZE >= game.width as f32 {
        game.ball_velocity.x *= -1.;
    }

    // Paddle collision
    if origin.y >= PADDLE_OFFSET
        && origin.y <= PADDLE_OFFSET + PADDLE_HEIGHT
        && origin.x <= game.paddle_origin.x + PADDLE_WIDTH
        && origin.x >= game.paddle_origin.x - BALL_SIZE
    {
        game.ball_velocity.y = game.ball_velocity.y.abs();
    }

    // Block collisions
    for cx in 0..=1 {
        for cy in 0..=1 {
            let v = origin + BALL_SIZE * Vector2f::new(cx as f32, cy as f32);

            let Some((bx, by)) = get_block_at(get_blocks_origin(game), v) else {
                continue;
            };

            if !game.blocks[bx][by] {
                continue;
            };

            game.blocks[bx][by] = false;
            game.ball_velocity.y *= -1.;
        }
    }
}

fn draw_world(game: &GameState, context: &mut Context) {
    draw_paddle(game.paddle_origin, context);
    if let Some(origin) = game.ball_origin {
        draw_ball(origin, context);
    }
    draw_blocks(game, context);
}

fn draw_paddle(origin: Vector2f, context: &mut Context) {
    let _ = context.graphics.draw_atlas_sprite(
        "sprites",
        PADDLE_SPRITE,
        origin,
        0,
        Vector2f::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        SpriteParams {
            slice: Some((2, Vector2f::splat(SPRITE_SIZE))),
            ..Default::default()
        },
    );
}

fn draw_ball(origin: Vector2f, context: &mut Context) {
    let _ = context.graphics.draw_atlas_sprite(
        "sprites",
        BALL_SPRITE,
        origin,
        0,
        Vector2f::splat(BALL_SIZE),
        SpriteParams::default(),
    );
}

fn draw_blocks(game: &GameState, context: &mut Context) {
    let origin = get_blocks_origin(game);

    for (x, col) in game.blocks.iter().enumerate() {
        for (y, block) in col.iter().enumerate() {
            if !block {
                continue;
            }
            let _ = context.graphics.draw_atlas_sprite(
                "sprites",
                BLOCK_SPRITE,
                origin + Vector2f::new(x as f32 * BLOCK_WIDTH, y as f32 * BLOCK_HEIGHT),
                0,
                Vector2f::new(BLOCK_WIDTH, BLOCK_HEIGHT),
                SpriteParams {
                    slice: Some((2, Vector2f::splat(SPRITE_SIZE))),
                    ..Default::default()
                },
            );
        }
    }
}

fn draw_status(game: &GameState, context: &mut Context) {
    let bounds = context.graphics.get_current_camera().get_bounds();
    let _ = context.graphics.draw_text(
        "pixel",
        &"*".repeat(game.lives),
        bounds.0,
        10,
        9.,
        SpriteParams::default(),
    );
}

fn handle_input(game: &mut GameState, context: &mut Context) {
    // Paddle movement
    if context
        .input
        .is_key_down(rogalik::input::KeyCode::ArrowLeft)
    {
        game.paddle_origin.x = 0.0_f32.max(game.paddle_origin.x - PADDLE_MOVE);
    }
    if context
        .input
        .is_key_down(rogalik::input::KeyCode::ArrowRight)
    {
        game.paddle_origin.x =
            (game.width as f32 - PADDLE_WIDTH).min(game.paddle_origin.x + PADDLE_MOVE);
    }

    // Ball release
    if game.ball_origin.is_none() && context.input.is_key_pressed(rogalik::input::KeyCode::Space) {
        game.ball_origin = Some(Vector2f::new(
            game.paddle_origin.x + 0.5 * (PADDLE_WIDTH - BALL_SIZE),
            game.paddle_origin.y + PADDLE_HEIGHT,
        ));
        game.ball_velocity = Vector2f::splat(BALL_MOVE);
    }
}

fn get_blocks_origin(game: &GameState) -> Vector2f {
    Vector2f::new(
        0.5 * (game.width as f32 - (BLOCKS_HORIZONTAL as f32 * BLOCK_WIDTH)),
        game.height as f32 - (BLOCKS_VERTICAL + 1) as f32 * BLOCK_HEIGHT,
    )
}

fn get_block_at(blocks_origin: Vector2f, mut position: Vector2f) -> Option<(usize, usize)> {
    position -= blocks_origin;
    let x = (position.x / BLOCK_WIDTH).floor();
    let y = (position.y / BLOCK_HEIGHT).floor();

    if x < 0. || y < 0. || x >= BLOCKS_HORIZONTAL as f32 || y >= BLOCKS_VERTICAL as f32 {
        return None;
    }

    Some((x as usize, y as usize))
}
