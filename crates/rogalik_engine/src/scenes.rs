use crate::{
    traits::{Game, Scene, SceneChange},
    Context,
};

pub struct SceneManager<T> {
    scenes: Vec<Box<dyn Scene<Game = T>>>,
    initialized: bool,
}
impl<T> SceneManager<T> {
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
            initialized: false,
        }
    }
}
impl<T: Game> SceneManager<T> {
    pub fn push(&mut self, scene: Box<dyn Scene<Game = T>>) {
        self.scenes.push(scene);
    }
    pub fn pop(&mut self) -> Option<Box<dyn Scene<Game = T>>> {
        self.scenes.pop()
    }
    pub fn switch(&mut self, scene: Box<dyn Scene<Game = T>>) -> Option<Box<dyn Scene<Game = T>>> {
        let prev = self.scenes.pop();
        self.push(scene);
        prev
    }
    pub fn current_mut(&mut self) -> Option<&mut Box<dyn Scene<Game = T>>> {
        self.scenes.last_mut()
    }
    pub fn initialize(&mut self, game: &mut T, context: &mut Context) {
        self.initialized = true;
        self.current_mut()
            .expect("No scene found!")
            .enter(game, context);
    }
}

pub fn update_scenes<T: Game>(
    scene_manager: &mut SceneManager<T>,
    game: &mut T,
    context: &mut Context,
) {
    // TODO find a better way to enter first scene on start?
    if !scene_manager.initialized {
        scene_manager.initialize(game, context);
        return;
    }

    let mut scene_result = None;
    if let Some(scene) = scene_manager.current_mut() {
        scene_result = scene.update(game, context);
    }
    match scene_result {
        None => (),
        Some(SceneChange::Pop) => {
            if let Some(mut scene) = scene_manager.pop() {
                scene.exit(game, context);
            }
            if let Some(scene) = scene_manager.current_mut() {
                scene.restore(game, context);
            }
        }
        Some(SceneChange::Push(mut scene)) => {
            scene.enter(game, context);
            scene_manager.push(scene);
        }
        Some(SceneChange::Switch(new_scene)) => {
            if let Some(mut old_scene) = scene_manager.switch(new_scene) {
                old_scene.exit(game, context)
            }
            if let Some(new) = scene_manager.current_mut() {
                new.enter(game, context)
            };
        }
    };
}
