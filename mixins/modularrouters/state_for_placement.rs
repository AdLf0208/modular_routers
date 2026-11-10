// `getStateForPlacement`: a placed router faces the opposite of its placer
// (`native_state_for_placement`, API 18).
fn observe_state_for_placement(
    level: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    crate::game::modularrouters_state::on_state_for_placement(level, state, context, answer);
}
