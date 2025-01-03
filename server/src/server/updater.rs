use crate::Plugin;
use signals::PlayerSignal;
use state::State;

#[derive(Default)]
pub struct ServerUpdater {
    pub(crate) state: State,
    pub plugins: Vec<Plugin>,
}

impl ServerUpdater {
    pub(crate) fn handle_signal(&mut self, signal: PlayerSignal) {
        println!("received {:?}!", signal);
    }
}
