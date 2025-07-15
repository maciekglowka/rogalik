use crate::{engine::Context, scenes::SceneController};

pub trait Game {
    fn setup(&mut self, context: &mut Context);
    fn resize(&mut self, _context: &mut Context) {}
    fn resume(&mut self, _context: &mut Context) {}
    fn reload_assets(&mut self, _context: &mut Context) {}
}

pub trait Scene {
    type Game: Game;

    #[allow(unused_variables)]
    fn enter(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
    #[allow(unused_variables)]
    fn exit(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        scenes: &mut SceneController<Self::Game>,
    ) {
    }
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
