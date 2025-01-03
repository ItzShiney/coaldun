use crate::Plugin;

mod connector;
mod updater;

pub use connector::*;
pub use updater::*;

// FIXME `pub(crate)`s
pub struct Server {
    pub(crate) connector: ServerConnector,
    pub(crate) updater: ServerUpdater,
}

impl Server {
    pub fn new(connector: ServerConnector, updater: ServerUpdater) -> Self {
        Self { connector, updater }
    }

    pub fn push_plugin(&mut self, plugin: Plugin) {
        self.updater.plugins.push(plugin);
    }
}
