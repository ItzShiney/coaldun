use serde::Deserialize;
use sfml::graphics::Color;
use sfml::graphics::Font;
use sfml::graphics::Rect;
use sfml::graphics::RenderTarget;
use sfml::graphics::RenderWindow;
use sfml::graphics::Sprite;
use sfml::graphics::Texture;
use sfml::graphics::Transformable;
use sfml::graphics::View;
use sfml::system::Vector2i;
use sfml::system::Vector2u;
use sfml::window;
use sfml::window::ContextSettings;
use sfml::window::Event;
use sfml::window::Key;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::mem;
use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::Instant;

mod logger;

pub use logger::*;

const TILE_SIZE: u32 = 12;
const FIELD_SIZE: Vector2u = Vector2u::new(39, 25);
const FIELD_MAX: Vector2u = Vector2u::new(FIELD_SIZE.x - 1, FIELD_SIZE.y - 1);
const BG_COLOR: Color = Color::rgb(0x11, 0x0a, 0x03);

const TICK_SPEED: Duration = Duration::from_micros(1_000_000 / 60);
const MOVEMENT_COOLDOWN_TICKS: usize = 10;

macro_rules! asseted_id {
    ( $Ident:ident, $path:literal ) => {
        #[derive(Debug, Deserialize, Clone, Eq)]
        #[serde(from = "String")]
        pub struct $Ident {
            pub ty: String,
            pub asset: String,
        }

        impl PartialEq for $Ident {
            fn eq(&self, other: &Self) -> bool {
                self.ty == other.ty
            }
        }

        impl std::hash::Hash for $Ident {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.ty.hash(state);
            }
        }

        impl From<String> for $Ident {
            fn from(ty: String) -> Self {
                let asset = format!("{}/{}", $path, ty);
                Self { ty, asset }
            }
        }
    };
}

#[derive(Debug, Deserialize)]
struct FloorConfig {
    #[serde(default)]
    #[expect(dead_code)]
    pub obstacle: bool,

    #[serde(default)]
    pub model: ModelConfig,
}

#[derive(Debug, Deserialize)]
pub struct WallConfig {
    pub breakable: Option<Breakable>,

    #[serde(default)]
    pub model: ModelConfig,
}

#[derive(Debug, Deserialize)]
pub struct Breakable {
    pub tool: Tool,
    pub power: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    Pickaxe,
    Axe,
    Shovel,
}

#[derive(Debug, Deserialize)]
struct TilesConfig {
    floors: HashMap<AssetedFloorType, FloorConfig>,
    walls: HashMap<AssetedWallType, WallConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EntityConfig {
    #[serde(default)]
    pub model: ModelConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    #[default]
    SameAsType,

    None,

    #[serde(untagged)]
    Custom(String),
}

impl ModelConfig {
    pub const fn asset<'s>(&'s self, ty: &'s String) -> Option<&'s String> {
        match self {
            Self::SameAsType => Some(ty),
            Self::None => None,
            Self::Custom(asset) => Some(asset),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "[u32; 2]")]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

impl TilePos {
    pub const fn vec(self) -> Vector2u {
        Vector2u::new(self.x, self.y)
    }
}

impl From<TilePos> for Vector2u {
    fn from(value: TilePos) -> Self {
        value.vec()
    }
}

impl From<Vector2u> for TilePos {
    fn from(vec: Vector2u) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<[u32; 2]> for TilePos {
    fn from([x, y]: [u32; 2]) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default)]
struct Field {
    floors: BTreeMap<TilePos, AssetedFloorType>,
    walls: BTreeMap<TilePos, AssetedWallType>,
    entities: Vec<Entity>,
}

impl Field {
    pub fn insert(&mut self, pos: TilePos, ty: AssetedTileType) {
        match ty {
            AssetedTileType::Floor(ty) => _ = self.floors.insert(pos, ty),
            AssetedTileType::Wall(ty) => _ = self.walls.insert(pos, ty),
        }
    }
}

#[derive(Debug)]
struct Entity {
    pub ty: AssetedEntityType,
    pub pos: TilePos,
    pub data: EntityData,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct EntityData {
    pub player_controller: Option<PlayerController>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerController {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,

    #[serde(skip)]
    pub cooldown_reset_frame: usize,
}

asseted_id!(AssetedFloorType, "tiles/floors");
asseted_id!(AssetedWallType, "tiles/walls");
asseted_id!(AssetedEntityType, "entities");

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
enum AssetedTileType {
    Floor(AssetedFloorType),
    Wall(AssetedWallType),
}

#[derive(Debug, Deserialize)]
struct Config {
    tiles: TilesConfig,
    entities: HashMap<AssetedEntityType, EntityConfig>,
    #[serde(rename = "assets")]
    additional_assets: Vec<String>,
    #[serde(rename = "init")]
    init_actions: Vec<InitAction>,
}

#[derive(Debug, Deserialize)]
enum InitAction {
    Place {
        ty: AssetedTileType,
        #[serde(flatten)]
        poses: TilePoses,
    },

    Spawn {
        ty: AssetedEntityType,
        #[serde(flatten)]
        poses: TilePoses,
        #[serde(default)]
        data: EntityData,
    },
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum TilePoses {
    #[serde(rename = "pos")]
    Pos(TilePos),

    #[serde(untagged)]
    Rect {
        min: TilePos,
        #[serde(flatten)]
        end: RectEnd,
        #[serde(default)]
        hollow: bool,
    },
}

impl TilePoses {
    pub fn for_each(self, mut f: impl FnMut(TilePos)) {
        match self {
            Self::Pos(pos) => f(pos),

            Self::Rect { min, end, hollow } => {
                let max = end.max(min);
                if hollow {
                    for x in min.x..=max.x {
                        f([x, min.y].into());
                        f([x, max.y].into());
                    }

                    for y in min.y + 1..max.y {
                        f([min.x, y].into());
                        f([max.x, y].into());
                    }
                } else {
                    for x in min.x..=max.x {
                        for y in min.y..=max.y {
                            f([x, y].into());
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RectEnd {
    Max(TilePos),
    Size(TilePos),
}

impl RectEnd {
    fn max(self, min: TilePos) -> TilePos {
        match self {
            Self::Max(max) => max,
            Self::Size(size) => TilePos::from([
                min.x + size.x.saturating_sub(1),
                min.y + size.y.saturating_sub(1),
            ]),
        }
    }
}

#[expect(clippy::allow_attributes)]
#[allow(clippy::too_many_lines)]
fn main() {
    let window_size = FIELD_SIZE * TILE_SIZE * 2;
    let mut window = RenderWindow::new(
        (window_size.x, window_size.y),
        "CD Combat Test",
        window::Style::CLOSE,
        &ContextSettings::default(),
    )
    .unwrap();

    window.set_view(&View::from_rect(Rect::new(
        0.,
        0.,
        window_size.x as f32 / 2.,
        window_size.y as f32 / 2.,
    )));

    let mut logger = Logger::new({
        let path = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/font.ttf"));
        Font::from_memory_static(path).unwrap()
    });

    'reload_config: loop {
        let mut field = Field::default();
        let config = {
            let mut config = {
                let file = File::open("config.yaml").unwrap();
                serde_yaml::from_reader::<_, Config>(file).unwrap()
            };

            for init_action in mem::take(&mut config.init_actions) {
                match init_action {
                    InitAction::Place { ty, poses } => {
                        poses.for_each(|pos| field.insert(pos, ty.clone()));
                    }

                    InitAction::Spawn { ty, poses, data } => {
                        poses.for_each(|pos| {
                            field.entities.push(Entity {
                                ty: ty.clone(),
                                pos,
                                data: data.clone(),
                            });
                        });
                    }
                }
            }

            config
        };

        let assets = {
            let floors = config
                .tiles
                .floors
                .iter()
                .filter_map(|(ty, floor_config)| floor_config.model.asset(&ty.asset));

            let walls = config
                .tiles
                .walls
                .iter()
                .filter_map(|(ty, wall_config)| wall_config.model.asset(&ty.asset));

            let entities = config
                .entities
                .iter()
                .filter_map(|(ty, entity_config)| entity_config.model.asset(&ty.asset));
            let additional = config.additional_assets.iter();

            floors.chain(walls).chain(entities).chain(additional)
        }
        .filter_map(|asset| {
            let Ok(texture) = Texture::from_file(&format!("assets/{asset}.png")) else {
                logger.push(format!("failed to load {asset}"));
                return None;
            };

            Some((asset, texture))
        })
        .collect::<HashMap<_, _>>();

        let mut tick = 0_usize;
        let mut tick_timer = Instant::now();

        while window.is_open() {
            while let Some(event) = window.poll_event() {
                match event {
                    Event::Closed => window.close(),

                    Event::KeyPressed { code, ctrl, .. } => {
                        if ctrl && code == Key::R {
                            continue 'reload_config;
                        }
                    }

                    _ => {}
                }
            }

            if tick_timer.elapsed() >= TICK_SPEED {
                tick += 1;
                tick_timer = Instant::now();

                for player in &mut field.entities {
                    let Some(player_controller) = &mut player.data.player_controller else {
                        continue;
                    };
                    if tick < player_controller.cooldown_reset_frame {
                        continue;
                    }

                    let dpos = Vector2i::new(
                        i32::from(player_controller.right.is_pressed())
                            - i32::from(player_controller.left.is_pressed()),
                        i32::from(player_controller.down.is_pressed())
                            - i32::from(player_controller.up.is_pressed()),
                    );

                    let new_pos = {
                        let res = player.pos.vec().as_other::<i32>() + dpos;
                        let x = (res.x.max(0_i32) as u32).min(FIELD_MAX.x);
                        let y = (res.y.max(0_i32) as u32).min(FIELD_MAX.y);
                        TilePos { x, y }
                    };

                    if player.pos != new_pos {
                        player.pos = new_pos;
                        player_controller.cooldown_reset_frame = tick + MOVEMENT_COOLDOWN_TICKS;
                    }
                }
            }

            window.clear(BG_COLOR);

            let posed_assets = {
                let floors = field.floors.iter().map(|(&pos, ty)| (pos, &ty.asset));
                let walls = field.walls.iter().map(|(&pos, ty)| (pos, &ty.asset));
                let entities = field.entities.iter().map(|entity| {
                    let asset = entity.data.model.as_ref().unwrap_or(&entity.ty.asset);
                    (entity.pos, asset)
                });

                floors.chain(walls).chain(entities)
            };

            for (pos, asset) in posed_assets {
                let Some(texture) = assets.get(asset) else {
                    logger.push(format!("missing texture for {asset}"));
                    continue;
                };

                let mut sprite = Sprite::with_texture(texture);
                sprite.set_position((pos.vec() * TILE_SIZE).as_other());
                window.draw(&sprite);
            }

            window.draw(&logger);

            window.display();
        }

        break;
    }
}
