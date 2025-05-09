pub trait Game {
    fn setup(&mut self, context: &mut super::Context);
    fn resize(&mut self, _context: &mut super::Context) {}
    fn resume(&mut self, _context: &mut super::Context) {}
    fn reload_assets(&mut self, _context: &mut super::Context) {}
}

pub trait Scene {
    type Game: Game;
    fn enter(&mut self, _game: &mut Self::Game, _context: &mut super::Context) {}
    fn exit(&mut self, _game: &mut Self::Game, _context: &mut super::Context) {}
    fn restore(&mut self, _game: &mut Self::Game, _context: &mut super::Context) {}
    fn update(
        &mut self,
        game: &mut Self::Game,
        context: &mut super::Context,
    ) -> Option<SceneChange<Self::Game>>;
}

pub enum SceneChange<T: Game> {
    Pop,
    Push(Box<dyn Scene<Game = T>>),
    Switch(Box<dyn Scene<Game = T>>),
}
