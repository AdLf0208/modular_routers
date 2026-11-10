// Right-clicking a router with a module installs it (`native_use_item_on`,
// API 13): upstream's path is the router GUI, which this slice does not carry.
fn observe_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::modularrouters_state::on_use_item_on(game, stack, pos, claimed);
}
