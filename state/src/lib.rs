use sfml::system::Vector2i as Vec2;
use std::collections::HashMap;
use std::fmt::Debug;

pub fn vec2(x: i32, y: i32) -> Vec2 {
    Vec2::new(x, y)
}

////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloorTypeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WallTypeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTypeId(usize);

////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum ToolKind {
    Sword,
    Dagger,
    Scythe,
    Hammer,
    Whip,
    Bow,
    Crossbow,

    Pickaxe,
    Axe,
    Shovel,
}

////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FloorType {
    pub asset: String,
    pub walkable: bool,
}

impl FloorType {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            walkable: true,
        }
    }

    pub fn non_walkable(mut self) -> Self {
        self.walkable = false;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Floor {
    pub type_id: FloorTypeId,
}

impl FloorTypeId {
    pub fn instance(self) -> Floor {
        Floor { type_id: self }
    }
}

////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Breakable {
    pub tool_kind: ToolKind,
    pub hits: usize,
}

#[derive(Debug)]
pub struct WallType {
    pub asset: String,
    pub breakable: Option<Breakable>,
}

impl WallType {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            breakable: None,
        }
    }

    pub fn breakable(mut self, tool_kind: ToolKind, hits: usize) -> Self {
        self.breakable = Some(Breakable { tool_kind, hits });
        self
    }
}

#[derive(Debug, Clone)]
pub struct Wall {
    pub type_id: WallTypeId,
}

impl WallTypeId {
    pub fn instance(self) -> Wall {
        Wall { type_id: self }
    }
}

////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct EntityType {
    pub asset: String,
}

impl EntityType {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(usize);

impl From<EntityId> for usize {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

#[derive(Debug)]
pub struct Entity {
    pub type_id: EntityTypeId,
    pub pos: Vec2,
    pub asset: Option<String>,
}

impl EntityTypeId {
    pub fn instance(self, pos: Vec2) -> Entity {
        Entity {
            type_id: self,
            pos,
            asset: None,
        }
    }
}

impl Entity {
    pub fn asset(mut self, asset: impl Into<String>) -> Self {
        self.asset = Some(asset.into());
        self
    }
}

////////////////////////////////////////////////////////////

pub const PLAYER_ENTITY_TYPE_ID: EntityTypeId = EntityTypeId(0);

#[derive(Debug)]
pub struct State {
    next_floor_type_id: FloorTypeId,
    next_wall_type_id: WallTypeId,
    next_entity_type_id: EntityTypeId,
    next_entity_id: EntityId,

    floor_types: HashMap<FloorTypeId, FloorType>,
    wall_types: HashMap<WallTypeId, WallType>,
    entity_types: HashMap<EntityTypeId, EntityType>,

    floors: HashMap<(i32, i32), Floor>,
    walls: HashMap<(i32, i32), Wall>,
    entities: HashMap<EntityId, Entity>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_floor_type_id: FloorTypeId(0),
            next_wall_type_id: WallTypeId(0),
            next_entity_type_id: EntityTypeId(1),
            next_entity_id: EntityId(0),

            floor_types: HashMap::default(),
            wall_types: HashMap::default(),
            entity_types: HashMap::from([(PLAYER_ENTITY_TYPE_ID, EntityType::new(""))]),

            floors: HashMap::default(),
            walls: HashMap::default(),
            entities: HashMap::default(),
        }
    }
}

impl State {
    pub fn floors(&self) -> &HashMap<(i32, i32), Floor> {
        &self.floors
    }

    pub fn walls(&self) -> &HashMap<(i32, i32), Wall> {
        &self.walls
    }

    pub fn entities(&self) -> &HashMap<EntityId, Entity> {
        &self.entities
    }

    pub fn player_entity_ids(&self) -> impl Iterator<Item = EntityId> + use<'_> {
        self.entities.iter().filter_map(|(&entity_id, entity)| {
            (entity.type_id == PLAYER_ENTITY_TYPE_ID).then_some(entity_id)
        })
    }
}

pub trait ObjectType<Type, TypeId> {
    fn insert_type(&mut self, value: Type) -> TypeId;
    fn get_type(&self, type_id: TypeId) -> &Type;
}

impl ObjectType<FloorType, FloorTypeId> for State {
    fn insert_type(&mut self, value: FloorType) -> FloorTypeId {
        let res = self.next_floor_type_id;
        self.next_floor_type_id.0 += 1;
        self.floor_types.insert(res, value);
        res
    }

    fn get_type(&self, type_id: FloorTypeId) -> &FloorType {
        &self.floor_types[&type_id]
    }
}

impl ObjectType<WallType, WallTypeId> for State {
    fn insert_type(&mut self, value: WallType) -> WallTypeId {
        let res = self.next_wall_type_id;
        self.next_wall_type_id.0 += 1;
        self.wall_types.insert(res, value);
        res
    }

    fn get_type(&self, type_id: WallTypeId) -> &WallType {
        &self.wall_types[&type_id]
    }
}

impl ObjectType<EntityType, EntityTypeId> for State {
    fn insert_type(&mut self, value: EntityType) -> EntityTypeId {
        let res = self.next_entity_type_id;
        self.next_entity_type_id.0 += 1;
        self.entity_types.insert(res, value);
        res
    }

    fn get_type(&self, type_id: EntityTypeId) -> &EntityType {
        &self.entity_types[&type_id]
    }
}

pub trait Place<Tile> {
    fn place(&mut self, pos: impl Into<Vec2>, tile: Tile);
}

impl Place<Floor> for State {
    fn place(&mut self, pos: impl Into<Vec2>, floor: Floor) {
        let Vec2 { x, y } = pos.into();
        self.floors.insert((x, y), floor);
    }
}

impl Place<Wall> for State {
    fn place(&mut self, pos: impl Into<Vec2>, wall: Wall) {
        let Vec2 { x, y } = pos.into();
        self.walls.insert((x, y), wall);
    }
}

impl State {
    pub fn place_rect<Tile: Clone>(
        &mut self,
        min: impl Into<Vec2>,
        max: impl Into<Vec2>,
        tile: Tile,
    ) where
        Self: Place<Tile>,
    {
        let min = min.into();
        let max = max.into();

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                self.place((x, y), tile.clone());
            }
        }
    }

    pub fn place_frame<Tile: Clone>(
        &mut self,
        min: impl Into<Vec2>,
        max: impl Into<Vec2>,
        tile: Tile,
    ) where
        Self: Place<Tile>,
    {
        let min = min.into();
        let max = max.into();

        for y in [min.y, max.y] {
            for x in min.x..=max.x {
                self.place((x, y), tile.clone());
            }
        }

        for x in [min.x, max.x] {
            for y in min.y + 1..max.y {
                self.place((x, y), tile.clone());
            }
        }
    }
}

impl State {
    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        let res = self.next_entity_id;
        self.next_entity_id.0 += 1;
        self.entities.insert(res, entity);
        res
    }
}
