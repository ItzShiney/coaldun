//! Плагин должен только задавать тайлы, предметы, сущности и пр., а также предоставлять `init` и хендлеры ивентов
//! Все хендлеры принимают `State`, но не `Server`, поскольку не нуждаются в верификации своих действий
//! Но всё равно могут попросить её от `State`, т.к. методы для верификации находятся там, а не у `Server`

use state::*;

mod entities;
mod floors;
mod walls;

use entities::*;
use floors::*;
use walls::*;

#[expect(dead_code)]
pub struct Plugin {
    floors: FloorTypes,
    walls: WallTypes,
    entities: EntityTypes,
}

impl Plugin {
    fn new(state: &mut State) -> Self {
        let floors = FloorTypes::new(state);
        let walls = WallTypes::new(state);
        let entities = EntityTypes::new(state);

        {
            let min = vec2(-19, -12);
            let max = vec2(19, 12);
            let center = (min + max) / 2;

            state.place_rect(min, max, floors.grass.instance());
            state.place_frame(min, max, walls.bedrock.instance());

            state.spawn(
                PLAYER_ENTITY_TYPE_ID
                    .instance(center - vec2(1, 0))
                    .asset("skins/suisei"),
            );
            state.spawn(
                PLAYER_ENTITY_TYPE_ID
                    .instance(center + vec2(1, 0))
                    .asset("skins/ougi"),
            );
        }

        Self {
            floors,
            walls,
            entities,
        }
    }

    #[unsafe(no_mangle)]
    pub const extern "Rust" fn handle_event(&mut self, _state: &mut State) {}
}

#[unsafe(no_mangle)]
pub extern "Rust" fn init(state: &mut State) -> Box<Plugin> {
    Box::new(Plugin::new(state))
}

#[unsafe(no_mangle)]
pub extern "Rust" fn uninit(_: Box<Plugin>) {}
