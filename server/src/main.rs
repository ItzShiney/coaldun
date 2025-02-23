//! Сервер должен только обрабатывать запросы клиентов, проверять их допустимость (верифицировать) и выполнять
//! Допустимость проверяется через методы у `State`, у самого сервера их не должно быть

use libloader::libloading::os::windows::Library;
use std::{env, net::TcpListener};

mod client;
mod plugin;
mod server;

use client::*;
use plugin::*;
use server::*;

const PLUGIN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/release/plugin.dll");

fn main() {
    let library = unsafe { Library::new(PLUGIN_PATH).unwrap() };

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut server = Server::new(ServerConnector::new(listener), ServerUpdater::default());

    // FIXME call `init_field` manually later instead of passing state directly into `Plugin::new`
    {
        let plugin = Plugin::new(library, &mut server.updater.state).unwrap();
        server.push_plugin(plugin);
    }

    println!("server started!");
    loop {
        server.connector.try_auth_all(&server.updater.state);
        server.connector.accept_all_unathorized();
        server.connector.handle_clients(&mut server.updater);
    }
}
