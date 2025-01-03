use state::*;

#[expect(clippy::allow_attributes)]
#[allow(unused)]
pub struct WallTypes {
    pub bedrock: WallTypeId,
    pub planks: WallTypeId,
}

impl WallTypes {
    pub fn new(state: &mut State) -> Self {
        Self {
            bedrock: state.insert_type(WallType::new("bedrock")),
            planks: state.insert_type(WallType::new("planks").breakable(ToolKind::Axe, 2)),
        }
    }
}
