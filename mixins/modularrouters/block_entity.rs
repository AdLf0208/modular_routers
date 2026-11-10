// A placed router declares a block entity, which saves its modules and buffer
// (`native_block_entity`, API 15).
fn observe_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::modularrouters_state::on_block_entity(block, entity);
}
