// The router ticks on the server, as upstream's `EntityBlock.getTicker`
// (`native_block_entity_ticks`, API 15).
fn observe_block_entity_ticks(block: &str, ticks: &mut bool) {
    crate::game::modularrouters_state::on_block_entity_ticks(block, ticks);
}
