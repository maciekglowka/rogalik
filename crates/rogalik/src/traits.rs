use crate::{engine::Context, scenes::SceneController};

pub trait Game {
    fn setup(&mut self, context: &mut Context);
    fn resize(&mut self, _context: &mut Context) {}
    fn resume(&mut self, _context: &mut Context) {}
    fn reload_assets(&mut self, _context: &mut Context) {}
}

pub trait Scene {
    type Game: Game;

    /// Triggered when the scene is first entered.
    #[allow(unused_variables)]
    fn enter(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
    /// Triggered when the scene is destroyed (via either `pop` or `switch`)
    #[allow(unused_variables)]
    fn exit(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
    /// Triggered when the scene is temporarily pushed to background
    /// when another scene is pushed on top of it.
    #[allow(unused_variables)]
    fn stop(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
    /// Triggered when the scene is restored from the background
    /// (scene on top of it is popped).
    #[allow(unused_variables)]
    fn restore(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
    fn update(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    );
}

pub(crate) enum SceneChange<T: Game> {
    Pop,
    Push(Box<dyn Scene<Game = T>>),
    Switch(Box<dyn Scene<Game = T>>),
}
