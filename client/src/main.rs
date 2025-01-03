use sfml::graphics::Color;
use sfml::graphics::Font;
use sfml::graphics::Rect;
use sfml::graphics::RenderTarget;
use sfml::graphics::RenderWindow;
use sfml::graphics::Sprite;
use sfml::graphics::Texture;
use sfml::graphics::Transformable;
use sfml::graphics::View;
use sfml::system::Vector2;
use sfml::system::Vector2i;
use sfml::system::Vector2u;
use sfml::window;
use sfml::window::ContextSettings;
use sfml::window::Event;
use sfml::window::Key;
use sfml::SfBox;
use sfml::SfError;
use signals::AssetData;
use signals::AssetId;
use signals::Auth;
use signals::Authorized;
use signals::ClientUpdate;
use signals::Entity;
use signals::EntityId;
use signals::PlayerSignal;
use signals::Position;
use signals::Tile;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::str::FromStr;
use std::time::Duration;

mod logger;

use logger::*;

const TILE_SIZE: u32 = 12;
const FIELD_SIZE: Vector2u = Vector2u::new(39, 25);
const BG_COLOR: Color = Color::rgb(0x11, 0x0a, 0x03);

struct Asset {
    texture: SfBox<Texture>,
}

impl TryFrom<AssetData> for Asset {
    type Error = SfError;

    fn try_from(data: AssetData) -> Result<Self, Self::Error> {
        let texture = {
            let mut res = Texture::new()?;
            res.load_from_memory(&data, Rect::default())?;
            res
        };

        Ok(Self { texture })
    }
}

#[derive(Default)]
struct State {
    assets: HashMap<AssetId, Asset>,
    floors: HashMap<Position, Tile>,
    walls: HashMap<Position, Tile>,
    entities: HashMap<EntityId, Entity>,
}

impl State {
    fn update(
        &mut self,
        ClientUpdate {
            assets,
            floors,
            walls,
            entities,
        }: ClientUpdate,
    ) {
        self.assets
            .extend(assets.into_vec().into_iter().filter_map(|(id, data)| {
                let asset = data.try_into().ok()?;
                Some((id, asset))
            }));

        self.floors.extend(floors);
        self.walls.extend(walls);
        self.entities.extend(entities);
    }
}

struct Client {
    window: SfBox<RenderWindow>,
    logger: Logger,
    state: State,
}

impl Client {
    fn new() -> Self {
        let window_size = FIELD_SIZE * TILE_SIZE * 2;
        let mut window = RenderWindow::new(
            (window_size.x, window_size.y),
            "CD Combat Test",
            window::Style::CLOSE,
            &ContextSettings::default(),
        )
        .unwrap();

        let view_size = window_size.as_other::<f32>() / 2.;
        window.set_view(&View::from_rect(Rect::from_vecs(
            -view_size / 2. + Vector2::new(TILE_SIZE, TILE_SIZE).as_other::<f32>() / 2.,
            view_size,
        )));

        let logger = Logger::new({
            let path = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/font.ttf"));
            Font::from_memory_static(path).unwrap()
        });

        Self {
            window,
            logger,
            state: State::default(),
        }
    }

    fn run(&mut self, auth: Auth, server_addr: SocketAddr) {
        const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

        'reload: loop {
            self.logger.clear();

            self.window.clear(BG_COLOR);
            self.window.display();

            self.state = State::default();
            let maybe_connected = TcpStream::connect_timeout(&server_addr, CONNECT_TIMEOUT)
                .map_err(|error| error.to_string())
                .and_then(|mut stream| {
                    let Authorized {
                        player_entity_id,
                        update,
                    } = bincode::serialize_into(&mut stream, &auth)
                        .and_then(|_| bincode::deserialize_from(&mut stream))
                        .map_err(|error| error.to_string())?;

                    self.state.update(update);

                    Ok((stream, player_entity_id))
                });

            let (mut stream, _player_entity_id) = match maybe_connected {
                Ok(connected) => connected,

                Err(error) => {
                    self.logger.push(error.to_string());

                    while self.window.is_open() {
                        while let Some(event) = self.window.poll_event() {
                            match event {
                                Event::Closed => self.window.close(),

                                Event::KeyPressed { code, ctrl, .. } => {
                                    if ctrl && code == Key::R {
                                        continue 'reload;
                                    }
                                }

                                _ => {}
                            }
                        }

                        self.window.clear(BG_COLOR);
                        self.window.draw(&self.logger);
                        self.window.display();
                    }
                    break 'reload;
                }
            };

            while self.window.is_open() {
                while let Some(event) = self.window.poll_event() {
                    match event {
                        Event::Closed => self.window.close(),

                        Event::KeyPressed { code, ctrl, .. } => {
                            if ctrl && code == Key::R {
                                match bincode::serialize_into(
                                    &mut stream,
                                    &PlayerSignal::ReloadServer,
                                ) {
                                    Ok(_) => continue 'reload,

                                    Err(error) => {
                                        self.logger.push_if_unique(error.to_string());
                                    }
                                }
                            }
                        }

                        _ => {}
                    }
                }

                self.window.clear(BG_COLOR);
                self.draw();
                self.window.display();
            }

            break;
        }
    }

    fn draw(&mut self) {
        let posed_textures = {
            let tiles = (self.state.floors.iter())
                .chain(&self.state.walls)
                .map(|(&pos, tile)| (pos, tile.asset_id));

            let entities =
                (self.state.entities.values()).map(|entity| (entity.pos, entity.asset_id));

            tiles.chain(entities)
        }
        .filter_map(|(pos, asset_id)| match self.state.assets.get(&asset_id) {
            Some(Asset { texture }) => Some((pos, texture)),

            None => {
                self.logger
                    .push_if_unique(format!("no asset for {:?}", asset_id));
                None
            }
        });

        for (Position { x, y }, texture) in posed_textures {
            let mut sprite = Sprite::with_texture(texture);
            sprite
                .set_position(Vector2i::new(x * TILE_SIZE as i32, y * TILE_SIZE as i32).as_other());
            self.window.draw(&sprite);
        }

        self.window.draw(&self.logger);
    }
}

fn main() {
    let server_addr = match fs::read_to_string("addr.txt") {
        Ok(addr) => SocketAddr::from_str(&addr).unwrap(),

        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080)
        }

        Err(error) => panic!("{}", error),
    };

    let auth = {
        let username = fs::read_to_string("auth.txt").unwrap();
        Auth { username }
    };

    Client::new().run(auth, server_addr);
}
