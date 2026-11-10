// `/modularrouters …` is registered at the command hook's head.
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::modularrouters_state::on_register_commands(commands);
}
