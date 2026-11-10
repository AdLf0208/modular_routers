// Right-clicking a router reports its state (`native_right_click_block`,
// API 21): upstream opens the router screen, which this slice does not carry.
// The clicked face is unused: a router has no per-face controls here.
fn observe_right_click_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    let _ = face;
    crate::game::modularrouters_state::on_right_click_block(game, pos, canceled);
}
