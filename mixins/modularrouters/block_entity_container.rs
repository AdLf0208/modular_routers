// The buffer is the block's item container on every side (API 16): hoppers,
// comparators and breaking the block reach it as upstream's item handler does.
fn observe_block_entity_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::modularrouters_state::on_container(block, container);
}
