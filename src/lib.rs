//! Modular Routers for Horizon: a router block entity with nine module slots,
//! five upgrade slots and a one-slot buffer that runs its installed modules on
//! its operation tick.
//!
//! Upstream (`desht/ModularRouters`) a router is a block entity with nine module
//! slots, five upgrade slots and a one-slot buffer; every `tickRate` ticks it
//! runs its modules in slot order, and the upgrades in its slots set that rate and
//! how many items one operation moves. This slice carries the router, its two Mk1
//! transfer modules and the two upgrades those numbers are about: a **Puller Mk1**
//! takes items from the container beside it into the buffer, a **Puller Mk2**
//! takes from the container the player selects up to twelve blocks away, a
//! **Sender Mk1** pushes them out of the buffer into the first container along its
//! facing, a **Speed Upgrade** shortens the interval by two ticks, a **Stack
//! Upgrade** doubles the items per operation, and the **Range Augments** in a
//! module's four augment slots move that module's reach one block each. With no
//! upgrades that is one operation a second, one item each.
//!
//! The rules are what [`runtime`] decides (tick rate, items per operation, the
//! relative module direction, the filter, the pull and send operations);
//! [`support`] is the game side (the block entity's Part, its container, the
//! module tick, the install and readout paths and `/modularrouters`), and this
//! file is the one `impl Hooks`.

pub mod runtime;

mod support;

use mod_api::{
    self, ComponentTypesEvent, Flow, Hooks, HostSide, RegisterCommandsEvent, Registrar,
    RegistrationContext, RightClickBlockEvent, RunModCommandEvent, ServerHost,
};

/// The mod keeps no state of its own: every router's modules, upgrades and
/// buffer live in its own block entity.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the router's block entity type, its one-slot container (the
    /// buffer, reachable from every side) and the Part that gives it a facing, a
    /// tick and the per-tick module run.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        support::declare(registrar);
    }

    /// Declare the component a targeted module carries its selected block in
    /// (`ModDataComponents.MODULE_TARGET_LIST` upstream). A component patch naming
    /// a type the game does not know is not a value an item can hold, so the id is
    /// registered before any content loads.
    fn component_types(&self, _context: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(support::target_component().to_string());
    }

    /// Right-clicking a router with a module or an upgrade installs it, and a
    /// sneak right-click with an augment installs it into the first module that
    /// takes it — upstream's GUI insert, which this port replaces with a click.
    fn right_click_block(&self, host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        support::right_click(host, event)
    }

    /// `/modularrouters info|install|remove|direction|filter|upgrade|augment|target`:
    /// the router and module screens' controls, as the harness and a player without
    /// the screens use them.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::declare_commands(event.commands);
    }

    /// Run the eight usages [`Hooks::register_commands`] declares, after the game
    /// has parsed and permission-checked them.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }
}
