use super::ServerUpdater;
use crate::Client;
use state::{EntityId, ObjectType, State};
use std::{
    io,
    mem::take,
    net::{TcpListener, TcpStream},
};

fn make_init_signal(state: &State) -> signals::ClientUpdate {
    // FIXME
    let grass_asset_id = signals::AssetId(0);
    let bedrock_asset_id = signals::AssetId(1);
    let suisei_asset_id = signals::AssetId(2);
    let ougi_asset_id = signals::AssetId(3);

    let asset_to_id = |asset: &str| match asset {
        "grass" => grass_asset_id,
        "bedrock" => bedrock_asset_id,
        "skins/suisei" => suisei_asset_id,
        "skins/ougi" => ougi_asset_id,
        _ => panic!("no asset for {asset}"),
    };

    let assets = Box::new([
        (
            grass_asset_id,
            Box::new(*include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/tiles/floors/grass.png"
            ))) as _,
        ),
        (
            bedrock_asset_id,
            Box::new(*include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/tiles/walls/bedrock.png"
            ))) as _,
        ),
        (
            suisei_asset_id,
            Box::new(*include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/skins/suisei.png"
            ))) as _,
        ),
        (
            ougi_asset_id,
            Box::new(*include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../assets/skins/ougi.png"
            ))) as _,
        ),
    ]);

    let floors = state
        .floors()
        .iter()
        .map(|(&(x, y), floor)| {
            let asset_id = asset_to_id(&state.get_type(floor.type_id).asset);

            let pos = signals::Position::new(x, y);
            let tile = signals::Tile { asset_id };
            (pos, tile)
        })
        .collect();

    let walls = state
        .walls()
        .iter()
        .map(|(&(x, y), wall)| {
            let asset_id = asset_to_id(&state.get_type(wall.type_id).asset);

            let pos = signals::Position::new(x, y);
            let tile = signals::Tile { asset_id };
            (pos, tile)
        })
        .collect();

    let entities = state
        .entities()
        .iter()
        .map(|(&entity_id, entity)| {
            let asset =
                (entity.asset.as_ref()).unwrap_or_else(|| &state.get_type(entity.type_id).asset);
            let asset_id = asset_to_id(asset);

            (signals::EntityId(entity_id.into()), signals::Entity {
                asset_id,
                pos: signals::Position {
                    x: entity.pos.x,
                    y: entity.pos.y,
                },
            })
        })
        .collect();

    signals::ClientUpdate {
        assets,
        floors,
        walls,
        entities,
    }
}

struct Player {
    username: String,
    client: Option<Client>,
    entity_id: EntityId,
}

pub struct ServerConnector {
    listener: TcpListener,
    unauthorized_clients: Vec<TcpStream>,
    // FIXME `HashMap<EntityId, Player \ entity_id>` or `BTreeSet<String, Player \ username>`?
    players: Vec<Player>,
}

impl ServerConnector {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            unauthorized_clients: Vec::default(),
            players: Vec::default(),
        }
    }

    pub fn accept_all_unathorized(&mut self) {
        while let Ok((stream, _)) = self.listener.accept() {
            self.unauthorized_clients.push(stream);
        }
    }

    pub fn try_auth_all(&mut self, state: &State) {
        self.unauthorized_clients = take(&mut self.unauthorized_clients)
            .into_iter()
            .filter_map(|mut stream| match bincode::deserialize_from(&mut stream) {
                Ok(signals::Auth { username }) => {
                    let entity_id = self.get_or_pick_entity(&username, state);

                    {
                        let update = make_init_signal(state);
                        bincode::serialize_into(&mut stream, &signals::Authorized {
                            player_entity_id: signals::EntityId(entity_id.into()),
                            update,
                        })
                        .ok()?;
                    }

                    println!("{} (re)joined as {:?}!", username, entity_id);

                    self.players.push(Player {
                        username,
                        client: Some(Client::new(stream)),
                        entity_id,
                    });
                    None
                }

                Err(_) => Some(stream),
            })
            .collect();
    }

    pub fn handle_clients(&mut self, updater: &mut ServerUpdater) {
        for player in &mut self.players {
            let Some(client) = &mut player.client else {
                continue;
            };

            match client.read_signal() {
                Ok(signal) => {
                    updater.handle_signal(signal);
                }

                Err(error) => match *error {
                    bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {}

                    bincode::ErrorKind::Io(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::ConnectionReset
                                | io::ErrorKind::ConnectionAborted
                                | io::ErrorKind::UnexpectedEof
                        ) =>
                    {
                        println!("disconnected!");
                        player.client = None;
                    }

                    _ => println!("error: {:?}!", error),
                },
            }
        }
    }

    fn get_or_pick_entity(&self, username: &str, state: &State) -> EntityId {
        if let Some(entity_id) = self.get_player_entity(username) {
            return entity_id;
        }

        let mut free_entity_ids = state.player_entity_ids().filter(move |&player_entity_id| {
            self.players
                .iter()
                .all(move |player| player.entity_id != player_entity_id)
        });

        // FIXME remove `expect`
        free_entity_ids
            .next()
            .expect("no player entities to assign")
    }

    fn get_player_entity(&self, username: &str) -> Option<EntityId> {
        self.players
            .iter()
            .find_map(|player| (player.username == username).then_some(player.entity_id))
    }
}
