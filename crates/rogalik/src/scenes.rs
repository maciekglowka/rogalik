use crate::{
    engine::Context,
    traits::{Game, Scene, SceneChange},
};

const EMPTY_STACK_MSG: &str = "Scene stack is empty!";

pub struct SceneController<T: Game>(Option<SceneChange<T>>);
impl<T: Game> SceneController<T> {
    fn new() -> Self {
        Self(None)
    }
    pub fn pop(&mut self) {
        self.0 = Some(SceneChange::Pop);
    }
    pub fn push(&mut self, scene: Box<dyn Scene<Game = T>>) {
        self.0 = Some(SceneChange::Push(scene));
    }
    pub fn switch(&mut self, scene: Box<dyn Scene<Game = T>>) {
        self.0 = Some(SceneChange::Switch(scene));
    }
}

pub struct SceneManager<T> {
    scenes: Vec<Box<dyn Scene<Game = T>>>,
    initialized: bool,
}
impl<T> SceneManager<T> {
    pub fn new(scene: Box<dyn Scene<Game = T>>) -> Self {
        Self {
            scenes: vec![scene],
            initialized: false,
        }
    }
}
impl<T: Game> SceneManager<T> {
    /// Push new scecne to the stack
    pub(crate) fn push(&mut self, scene: Box<dyn Scene<Game = T>>) {
        self.scenes.push(scene);
    }
    /// Pop current scene. Panics if scene stack is empty.
    pub(crate) fn pop(&mut self) -> Box<dyn Scene<Game = T>> {
        self.scenes.pop().expect(EMPTY_STACK_MSG)
    }
    /// Replace current (top) scene. Panics if scene stack is empty.
    pub(crate) fn switch(&mut self, scene: Box<dyn Scene<Game = T>>) -> Box<dyn Scene<Game = T>> {
        let prev = self.scenes.pop().expect(EMPTY_STACK_MSG);
        self.push(scene);
        prev
    }
    pub(crate) fn current_mut(&mut self) -> &mut Box<dyn Scene<Game = T>> {
        self.scenes.last_mut().expect(EMPTY_STACK_MSG)
    }
    pub(crate) fn initialize(
        &mut self,
        game: &mut T,
        context: &mut Context,
        controller: &mut SceneController<T>,
    ) {
        self.initialized = true;
        self.current_mut().enter(game, context, controller);
    }
}

pub fn update_scenes<T: Game>(
    scene_manager: &mut SceneManager<T>,
    game: &mut T,
    context: &mut Context,
) {
    let mut controller = SceneController::new();

    match scene_manager.initialized {
        // TODO find a better way to enter first scene on start?
        false => {
            scene_manager.initialize(game, context, &mut controller);
        }
        true => {
            scene_manager
                .current_mut()
                .update(game, context, &mut controller);
        }
    }

    while let Some(change) = controller.0.take() {
        match change {
            SceneChange::Pop => {
                scene_manager.pop().exit(game, context, &mut controller);
                scene_manager
                    .current_mut()
                    .restore(game, context, &mut controller);
            }
            SceneChange::Push(mut scene) => {
                scene.enter(game, context, &mut controller);
                scene_manager.push(scene);
            }
            SceneChange::Switch(new_scene) => {
                scene_manager
                    .switch(new_scene)
                    .exit(game, context, &mut controller);
                scene_manager
                    .current_mut()
                    .enter(game, context, &mut controller);
            }
        }
    }
}
