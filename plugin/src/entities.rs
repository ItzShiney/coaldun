use state::*;

#[expect(clippy::allow_attributes)]
#[allow(unused)]
pub struct EntityTypes {
    pub skeleton: EntityTypeId,
}

impl EntityTypes {
    pub fn new(state: &mut State) -> Self {
        Self {
            skeleton: state.insert_type(EntityType::new("skeleton")),
        }
    }
}
