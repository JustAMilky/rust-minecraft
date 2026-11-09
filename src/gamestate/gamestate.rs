use winit::{
    event_loop::ActiveEventLoop, keyboard::KeyCode
};

pub struct GameState {

}

impl GameState {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(
            Self {}
        )
    }

    pub fn update(&mut self) {
        // todo!();
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}