use crate::{
    traits::{Game, Scene, SceneResult},
    Context,
};

pub struct SceneManager<T> {
    scenes: Vec<Box<dyn Scene<Game = T>>>,
}
impl<T> SceneManager<T> {
    pub fn new() -> Self {
        Self { scenes: Vec::new() }
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
}

pub fn update_scenes<T: Game>(
    scene_manager: &mut SceneManager<T>,
    game: &mut T,
    context: &mut Context,
) {
    let mut scene_result = SceneResult::None;
    if let Some(scene) = scene_manager.current_mut() {
        scene_result = scene.update(game, context);
    }
    match scene_result {
        SceneResult::None => (),
        SceneResult::Pop => {
            if let Some(mut scene) = scene_manager.pop() {
                scene.exit(game, context);
            }
        }
        SceneResult::Push(mut scene) => {
            scene.enter(game, context);
            scene_manager.push(scene);
        }
        SceneResult::Switch(mut new_scene) => {
            new_scene.enter(game, context);
            if let Some(mut old_scene) = scene_manager.switch(new_scene) {
                old_scene.exit(game, context)
            }
        }
    };
}
