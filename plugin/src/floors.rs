use state::*;

#[expect(clippy::allow_attributes)]
#[allow(unused)]
pub struct FloorTypes {
    pub grass: FloorTypeId,
    pub planks: FloorTypeId,
}

impl FloorTypes {
    pub fn new(state: &mut State) -> Self {
        Self {
            grass: state.insert_type(FloorType::new("grass")),
            planks: state.insert_type(FloorType::new("planks")),
        }
    }
}
