// One server tick of one router (`native_block_entity_tick`, API 15/17): on the
// router's operation tick, run its modules.
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::modularrouters_state::on_tick(game, pos, entity);
}
