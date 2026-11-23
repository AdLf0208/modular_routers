// Pure router logic for Modular Routers (standalone crate lib, injected as
// `crate::game::modularrouters`). No game access lives here.
//
// Upstream a router is a block entity with nine module slots, a one-slot buffer
// and five upgrade slots. Every `tickRate` ticks it runs its modules in slot
// order: a Puller Mk1 takes items from the container beside it into the buffer,
// a Sender Mk1 pushes them out of the buffer into the first container along its
// facing. With no upgrades that is one operation a second, one item each. The
// numbers below come from `CommonConfig.java` and `ModularRouterBlockEntity.java`
// (see `FINDINGS.md`).

/// A block position, as the glue reads it from the level.
pub type Pos = [i32; 3];

/// One of the six faces, in vanilla's `Direction.values()` order.
pub type Face = usize;

pub const FACE_COUNT: usize = 6;

pub const DOWN: Face = 0;
pub const UP: Face = 1;
pub const NORTH: Face = 2;
pub const SOUTH: Face = 3;
pub const WEST: Face = 4;
pub const EAST: Face = 5;

/// `Direction.values()`'s names, which are also the block's state values.
pub const FACE_NAMES: [&str; FACE_COUNT] = ["down", "up", "north", "south", "west", "east"];

/// `CommonConfig.router.baseTickRate`: a router with no upgrades runs every 20 ticks.
pub const BASE_TICK_RATE: u64 = 20;
/// `CommonConfig.router.ticksPerUpgrade`: each Speed Upgrade shortens the interval by 2.
pub const TICKS_PER_SPEED_UPGRADE: u64 = 2;
/// `CommonConfig.router.hardMinTickRate`: the interval never drops below 2 ticks.
pub const HARD_MIN_TICK_RATE: u64 = 2;
/// `ModularRouterBlockEntity.compileUpgrades` caps the shift at 6 (`1 << 6 = 64`).
pub const MAX_STACK_UPGRADES: u32 = 6;
/// `N_MODULE_SLOTS`: the router's module slots.
pub const MODULE_SLOTS: usize = 9;
/// `N_UPGRADE_SLOTS`.
pub const UPGRADE_SLOTS: usize = 5;
/// `N_BUFFER_SLOTS`: the one buffer slot every module shares.
pub const BUFFER_SLOTS: usize = 1;
/// `CommonConfig.module.sender1BaseRange`: Sender Mk1 reaches 8 blocks.
pub const SENDER_BASE_RANGE: i32 = 8;
/// `CommonConfig.module.sender1MaxRange`.
pub const SENDER_MAX_RANGE: i32 = 16;
/// `CommonConfig.module.puller2BaseRange`: a Puller Mk2 reaches 12 blocks.
pub const PULLER_BASE_RANGE: i32 = 12;
/// `CommonConfig.module.puller2MaxRange`: up to 24 with Range Augments.
pub const PULLER_MAX_RANGE: i32 = 24;
/// `AugmentItem.SLOTS`: every module has four augment slots.
pub const AUGMENT_SLOTS: usize = 4;
/// One Range Up/Down augment moves a module's reach one block, as
/// `ModuleItem.getRangeModifier` counts them (up − down, one each).
pub const RANGE_PER_AUGMENT: i32 = 1;
/// `StackAugment.getMaxAugments`: a module takes six Stack Augments in one slot.
pub const STACK_AUGMENT_LIMIT: u32 = 6;
/// `CompiledModule.getItemsPerTick`: `min(1 << n, 64)` — the most one operation
/// moves, whatever the augment or upgrade counts add up to.
pub const MAX_ITEMS_PER_OPERATION: u32 = 64;
/// `StackAugment.getMaxAugments` names the modules that take **none** of it: their
/// operation is not an item count. Upstream tests the module class; this port has
/// no class for a kind it does not run yet (gap 1), so its rule is the id's.
pub const NO_STACK_AUGMENT: [&str; 7] = [
    "modularrouters:detector_module",
    "modularrouters:extruder_module_1",
    "modularrouters:extruder_module_2",
    "modularrouters:breaker_module",
    "modularrouters:placer_module",
    "modularrouters:fluid_module",
    "modularrouters:fluid_module_2",
];

/// The face's unit vector, as `Direction.normal()` gives it.
pub const fn face_normal(face: Face) -> [i32; 3] {
    match face {
        DOWN => [0, -1, 0],
        UP => [0, 1, 0],
        NORTH => [0, 0, -1],
        SOUTH => [0, 0, 1],
        WEST => [-1, 0, 0],
        EAST => [1, 0, 0],
        _ => [0, 0, 0],
    }
}

/// The position one step from `pos` through `face`.
pub const fn step(pos: Pos, face: Face) -> Pos {
    let [dx, dy, dz] = face_normal(face);
    [pos[0] + dx, pos[1] + dy, pos[2] + dz]
}

/// The face a normal vector names (`BlockHit::face`), or none for a vector that is
/// not one of the six: `Direction.fromNormal`.
pub fn face_of_normal(normal: [i32; 3]) -> Option<Face> {
    (0..FACE_COUNT).find(|face| face_normal(*face) == normal)
}

/// The side of the block at `from` that points most nearly at `to`, which is the
/// face a player standing at `to` would have clicked (`Direction.approximate_nearest`
/// on the vector from the block to the player). None when `to` is that same block.
pub fn side_towards(from: Pos, to: Pos) -> Option<Face> {
    let delta = [
        i64::from(to[0]) - i64::from(from[0]),
        i64::from(to[1]) - i64::from(from[1]),
        i64::from(to[2]) - i64::from(from[2]),
    ];
    let mut best = (0i64, None);
    for face in 0..FACE_COUNT {
        let [x, y, z] = face_normal(face);
        let dot = delta[0] * i64::from(x) + delta[1] * i64::from(y) + delta[2] * i64::from(z);
        if dot > best.0 {
            best = (dot, Some(face));
        }
    }
    best.1
}

/// The four faces a router can point at (`Direction.Plane.HORIZONTAL`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub const ALL: [Facing; 4] = [Facing::North, Facing::East, Facing::South, Facing::West];

    pub const fn name(self) -> &'static str {
        match self {
            Facing::North => "north",
            Facing::East => "east",
            Facing::South => "south",
            Facing::West => "west",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Facing::ALL.into_iter().find(|facing| facing.name() == name)
    }

    pub const fn face(self) -> Face {
        match self {
            Facing::North => NORTH,
            Facing::East => EAST,
            Facing::South => SOUTH,
            Facing::West => WEST,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Facing::North => Facing::South,
            Facing::East => Facing::West,
            Facing::South => Facing::North,
            Facing::West => Facing::East,
        }
    }

    /// `Direction.getClockWise()` on the horizontal plane.
    pub const fn clock_wise(self) -> Self {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    /// `Direction.getCounterClockWise()`.
    pub const fn counter_clock_wise(self) -> Self {
        self.clock_wise().clock_wise().clock_wise()
    }
}

/// `Direction.fromYRot`: the horizontal direction a player yaw looks along
/// (0 = south, 90 = west, -90 = east, 180 = north), as `from_y_rot` gives it.
pub fn facing_from_yaw(yaw: f32) -> Facing {
    // `Direction.fromYRot`: `from2D((int) (yRot / 90 + 0.5) & 3)`, whose 2D
    // order is south, west, north, east.
    let index = ((f64::from(yaw) / 90.0 + 0.5).floor() as i64 & 3) as usize;
    [Facing::South, Facing::West, Facing::North, Facing::East][index]
}

/// `getStateForPlacement`: a placed router faces the opposite of its placer.
pub fn facing_for_placement(yaw: f32) -> Facing {
    facing_from_yaw(yaw).opposite()
}

/// The facing a `facing` state value names; `north` when it names none (`Direction`
/// defaults), so a state read mid-load cannot move a module's target.
pub fn facing_of_state(name: &str) -> Facing {
    Facing::from_name(name).unwrap_or(Facing::North)
}

/// A module's direction relative to the router's facing (upstream
/// `RelativeDirection`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relative {
    None,
    Down,
    Up,
    Left,
    Right,
    Front,
    Back,
}

impl Relative {
    pub const ALL: [Relative; 7] = [
        Relative::None,
        Relative::Down,
        Relative::Up,
        Relative::Left,
        Relative::Right,
        Relative::Front,
        Relative::Back,
    ];

    /// `RelativeDirection.getSerializedName`.
    pub const fn name(self) -> &'static str {
        match self {
            Relative::None => "none",
            Relative::Down => "down",
            Relative::Up => "up",
            Relative::Left => "left",
            Relative::Right => "right",
            Relative::Front => "front",
            Relative::Back => "back",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Relative::ALL
            .into_iter()
            .find(|relative| relative.name() == name)
    }

    /// The absolute face this direction names, or none when it names nothing —
    /// `RelativeDirection.NONE` leaves a directional module without a target.
    pub const fn resolve(self, facing: Facing) -> Option<Face> {
        match self {
            Relative::None => None,
            Relative::Down => Some(DOWN),
            Relative::Up => Some(UP),
            Relative::Left => Some(facing.clock_wise().face()),
            Relative::Right => Some(facing.counter_clock_wise().face()),
            Relative::Front => Some(facing.face()),
            Relative::Back => Some(facing.opposite().face()),
        }
    }
}

/// Which module is installed: one of the three transfer modules this port runs,
/// or any other module id a save or an add-on carries.
///
/// The second case is the **registered** path: `modularrouters:modules` is the
/// item tag the jar ships, and an item in it is a module whatever port
/// implements it. This port keeps such a module — its id, direction and filter
/// — and leaves its behaviour to the mod that owns it; what it must never do is
/// drop it from the router's saved list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleKind {
    /// `modularrouters:puller_module_1`: container → buffer.
    PullerMk1,
    /// `modularrouters:puller_module_2`: a selected container in range → buffer.
    PullerMk2,
    /// `modularrouters:sender_module_1`: buffer → container.
    SenderMk1,
    /// A module this port does not implement, carried by its id.
    Registered(String),
}

impl ModuleKind {
    pub const PULLER: &'static str = "modularrouters:puller_module_1";
    pub const PULLER2: &'static str = "modularrouters:puller_module_2";
    pub const SENDER: &'static str = "modularrouters:sender_module_1";

    /// The kind an item id names: one of the transfer modules this port runs, or a
    /// registered kind that keeps the id unchanged.
    #[must_use]
    pub fn of(item: &str) -> Self {
        match item {
            Self::PULLER => ModuleKind::PullerMk1,
            Self::PULLER2 => ModuleKind::PullerMk2,
            Self::SENDER => ModuleKind::SenderMk1,
            other => ModuleKind::Registered(other.to_string()),
        }
    }

    /// Whether this port runs the module itself rather than only carrying it.
    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(
            self,
            ModuleKind::PullerMk1 | ModuleKind::PullerMk2 | ModuleKind::SenderMk1
        )
    }

    /// Whether the module works on a block the player **selected** rather than the
    /// one beside it (`ModuleItem.isDirectional` is false for a targeted module,
    /// so its relative direction names nothing).
    #[must_use]
    pub fn is_targeted(&self) -> bool {
        matches!(self, ModuleKind::PullerMk2)
    }

    /// The item id this module is.
    #[must_use]
    pub fn item(&self) -> &str {
        match self {
            ModuleKind::PullerMk1 => Self::PULLER,
            ModuleKind::PullerMk2 => Self::PULLER2,
            ModuleKind::SenderMk1 => Self::SENDER,
            ModuleKind::Registered(item) => item,
        }
    }

    /// The reach this module starts with (`IRangedModule.getBaseRange`), or
    /// `None` for a module that is not ranged and therefore has none.
    #[must_use]
    pub fn base_range(&self) -> Option<i32> {
        match self {
            ModuleKind::PullerMk2 => Some(PULLER_BASE_RANGE),
            ModuleKind::SenderMk1 => Some(SENDER_BASE_RANGE),
            ModuleKind::PullerMk1 | ModuleKind::Registered(_) => None,
        }
    }

    /// The longest reach Range Up augments can give it (`getHardMaxRange`).
    #[must_use]
    pub fn max_range(&self) -> Option<i32> {
        match self {
            ModuleKind::PullerMk2 => Some(PULLER_MAX_RANGE),
            ModuleKind::SenderMk1 => Some(SENDER_MAX_RANGE),
            ModuleKind::PullerMk1 | ModuleKind::Registered(_) => None,
        }
    }

    /// How many of `augment` this module accepts in one slot
    /// (`AugmentItem.getMaxAugments`): a Range Up augment is worth
    /// `hardMax − base` of them, a Range Down one `base − 1`, and a module that
    /// is not ranged takes neither. A Stack Augment is `StackAugment`'s own
    /// number — six, or none on a module whose operation is not an item count. An
    /// augment this port does not run is carried one per slot when it is
    /// installed — what it does, and how many of it fit, are its own mod's, so a
    /// saved count of it is kept as written.
    #[must_use]
    pub fn augment_limit(&self, augment: &AugmentKind) -> u32 {
        let limit = match augment {
            AugmentKind::RangeUp => self
                .max_range()
                .zip(self.base_range())
                .map(|(max, base)| max - base),
            AugmentKind::RangeDown => self.base_range().map(|base| base - 1),
            AugmentKind::Stack => self.stack_augment_limit(),
            AugmentKind::Registered(_) => Some(OTHER_PER_SLOT as i32),
        };
        limit.map_or(0, |count| u32::try_from(count.max(0)).unwrap_or(u32::MAX))
    }

    /// `StackAugment.getMaxAugments`: six for a module whose operation moves
    /// items, and none for the seven kinds whose operation is not an item count
    /// (the Detector, both Extruders, the Breaker, the Placer and both Fluid
    /// modules). Upstream tests the module class, so every other module —
    /// including one this build only carries — takes six.
    #[must_use]
    pub fn stack_augment_limit(&self) -> Option<i32> {
        (!NO_STACK_AUGMENT.contains(&self.item()))
            .then(|| i32::try_from(STACK_AUGMENT_LIMIT).unwrap_or(i32::MAX))
    }
}

/// A module's item filter: the ids it accepts, and whether that list is a
/// whitelist (`ModuleFlags.whiteList`, default false = blacklist).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    whitelist: bool,
    items: Vec<String>,
}

/// The most item ids one filter lists. Upstream's filter GUI is a fixed row of
/// slots; this slice's list is what a save can grow, so the bound lives here.
pub const MAX_FILTER_ITEMS: usize = 16;

impl Filter {
    /// The default: blacklist mode with no entries, which `Filter.test` lets
    /// every item through.
    pub const fn any() -> Self {
        Filter {
            whitelist: false,
            items: Vec::new(),
        }
    }

    pub fn new(whitelist: bool, items: Vec<String>) -> Self {
        let mut items = items;
        // The saved list is scanned on every router tick: an edit must not be able
        // to grow it without bound, and a save with more entries keeps the first
        // ones rather than being refused whole.
        items.truncate(MAX_FILTER_ITEMS);
        Filter { whitelist, items }
    }

    pub fn whitelist(&self) -> bool {
        self.whitelist
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `Filter.test`: a whitelist takes only its own entries (and nothing when
    /// empty), a blacklist takes everything but them.
    pub fn accepts(&self, item: &str) -> bool {
        let listed = self.items.iter().any(|entry| entry == item);
        if self.whitelist { listed } else { !listed }
    }
}

/// A module installed in one slot: what it is, where it points, what it takes,
/// the block a targeted one selected, and the augments in its four slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub kind: ModuleKind,
    pub direction: Relative,
    pub filter: Filter,
    /// The selected block of a targeted module (`ModuleTarget`), `None` until the
    /// player sets one — an unselected targeted module does nothing.
    pub target: Option<Target>,
    augments: Augments,
}

impl Module {
    pub fn new(kind: ModuleKind, direction: Relative, filter: Filter) -> Self {
        Module {
            kind,
            direction,
            filter,
            target: None,
            augments: Augments::default(),
        }
    }

    /// The same module with `target` selected on it.
    #[must_use]
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }

    /// The same module with `augments` in its slots, each count cut to the limit
    /// this module gives that augment (a saved list can name more of one than it
    /// takes). An augment this port does not run is left as the save wrote it: its
    /// per-slot limit belongs to its own mod, and the read must not invent one.
    #[must_use]
    pub fn with_augments(mut self, augments: Augments) -> Self {
        let kind = self.kind.clone();
        self.augments = augments.capped(|augment| {
            if augment.is_known() {
                kind.augment_limit(augment)
            } else {
                0
            }
        });
        self
    }

    /// The augments in the module's four slots.
    #[must_use]
    pub fn augments(&self) -> &Augments {
        &self.augments
    }

    /// How many of `augment` this module takes in one slot
    /// (`AugmentItem.getMaxAugments`); zero when it takes none.
    #[must_use]
    pub fn augment_limit(&self, augment: &AugmentKind) -> u32 {
        self.kind.augment_limit(augment)
    }

    /// Add one `augment` if the module has room: false when it takes none of that
    /// augment or its four slots are full.
    #[must_use]
    pub fn add_augment(&mut self, augment: AugmentKind) -> bool {
        let limit = self.augment_limit(&augment);
        self.augments.add(augment, limit)
    }

    /// Take one `augment` out; true when the module had one.
    #[must_use]
    pub fn remove_augment(&mut self, augment: &AugmentKind) -> bool {
        self.augments.remove_one(augment)
    }

    /// The reach this module works at now —
    /// `max(1, min(hardMax, base + range_up − range_down))` for a ranged module
    /// (`IRangedModule.getCurrentRange`), `None` for one that is not ranged.
    #[must_use]
    pub fn current_range(&self) -> Option<i32> {
        let boost = self.augments.range_boost();
        match self.kind {
            ModuleKind::PullerMk2 => Some(puller_range(boost)),
            ModuleKind::SenderMk1 => Some(sender_range(boost)),
            ModuleKind::PullerMk1 | ModuleKind::Registered(_) => None,
        }
    }

    /// How many items one operation of this module moves —
    /// `CompiledModule.getItemsPerTick`: `n > 0 ? min(1 << n, 64) : router's count`,
    /// where `n` is the Stack Augments in its slots and the router's count is what
    /// its Stack Upgrades give it. The augment **overrides** that count; it does
    /// not add to it or multiply it.
    #[must_use]
    pub fn items_per_operation(&self, router_items: u32) -> u32 {
        // The slot limit is six, so the shift can never reach past 64; both bounds
        // are upstream's (`min(1 << n, 64)`), and the shift is clamped rather than
        // trusted because a save can write a count the GUI could not.
        let stacks = self.augments.count(&AugmentKind::Stack);
        if stacks == 0 {
            return router_items;
        }
        (1u32 << stacks.min(STACK_AUGMENT_LIMIT)).min(MAX_ITEMS_PER_OPERATION)
    }

    /// Whether this module works on its selected block rather than the one its
    /// direction names (`ModuleItem.isDirectional`).
    #[must_use]
    pub fn uses_target(&self) -> bool {
        self.kind.is_targeted()
    }
}

/// The block a targeted module works on, as `ModuleTarget` holds it: where it is,
/// which face the player clicked (the side the container is reached through), and
/// the level it stands in (`isSameWorld` refuses another one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub pos: Pos,
    pub face: Face,
    pub dimension: String,
}

impl Target {
    pub fn new(pos: Pos, face: Face, dimension: &str) -> Self {
        Target {
            pos,
            face,
            dimension: dimension.to_string(),
        }
    }

    /// Whether this target stands in the level `dimension` names.
    #[must_use]
    pub fn same_world(&self, dimension: &str) -> bool {
        self.dimension == dimension
    }

    /// The position as the readout and the command spell it (`0,100,-10`).
    #[must_use]
    pub fn at_text(&self) -> String {
        format!("{},{},{}", self.pos[0], self.pos[1], self.pos[2])
    }
}

/// A block position written as `x,y,z` — the `/modularrouters target` spelling.
#[must_use]
pub fn parse_pos(text: &str) -> Option<Pos> {
    let mut parts = text.split(',');
    let x = parts.next()?.trim().parse::<i32>().ok()?;
    let y = parts.next()?.trim().parse::<i32>().ok()?;
    let z = parts.next()?.trim().parse::<i32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([x, y, z])
}

/// A stack in the buffer or in a container slot: its item, its count and the
/// component key that makes two stacks of one item different resources (a named
/// or enchanted diamond is not a plain one).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stack {
    pub item: String,
    pub count: u32,
    pub extra: String,
}

impl Stack {
    /// A plain stack of `item` at `count`, with no components.
    #[must_use]
    pub fn of(item: &str, count: u32) -> Self {
        Self {
            item: item.to_string(),
            count,
            extra: String::new(),
        }
    }

    /// Whether both stacks are the same resource: same item and same components.
    #[must_use]
    pub fn same_resource(&self, other: &Self) -> bool {
        self.item == other.item && self.extra == other.extra
    }
}

/// One readable slot of a container: its number, stack and whether the side may
/// take from it (`native_container_slots`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub slot: usize,
    pub item: String,
    pub count: u32,
    /// The component key, as [`Stack::extra`].
    pub extra: String,
}

impl Slot {
    pub fn new(slot: usize, item: &str, count: u32) -> Self {
        Slot {
            slot,
            item: item.to_string(),
            count,
            extra: String::new(),
        }
    }

    /// The same slot, holding a stack that carries components.
    #[must_use]
    pub fn with_extra(mut self, extra: &str) -> Self {
        self.extra = extra.to_string();
        self
    }

    /// Whether this slot holds the resource `stack` is.
    #[must_use]
    fn is(&self, stack: &Stack) -> bool {
        self.item == stack.item && self.extra == stack.extra
    }
}

/// `UpgradeItem.getInstalledStackLimit` for a Speed Upgrade: nine fit in one slot.
pub const SPEED_PER_SLOT: u32 = 9;
/// …for a Stack Upgrade: six fit in one slot, which is the `1 << min(6, …)` cap.
pub const STACK_PER_SLOT: u32 = 6;
/// …and for every other upgrade item, one.
pub const OTHER_PER_SLOT: u32 = 1;

/// An upgrade installed in one of the router's five slots: one of the two this
/// port runs, or any other id a save or another mod carries.
///
/// The second case is the **registered** path, as [`ModuleKind::Registered`] is
/// for modules: an add-on's upgrade keeps its id and its slot, and what it does
/// stays its own mod's business.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeKind {
    /// `modularrouters:speed_upgrade`: two ticks less per operation, nine a slot.
    Speed,
    /// `modularrouters:stack_upgrade`: one more doubling of the items moved.
    Stack,
    /// An upgrade this port does not run, carried by its id.
    Registered(String),
}

impl UpgradeKind {
    pub const SPEED: &'static str = "modularrouters:speed_upgrade";
    pub const STACK: &'static str = "modularrouters:stack_upgrade";

    /// The kind an item id names: one of the two upgrades this port runs, or a
    /// registered kind that keeps the id unchanged.
    #[must_use]
    pub fn of(item: &str) -> Self {
        match item {
            Self::SPEED => UpgradeKind::Speed,
            Self::STACK => UpgradeKind::Stack,
            other => UpgradeKind::Registered(other.to_string()),
        }
    }

    /// The upgrade a `/modularrouters upgrade … <kind>` word names.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "speed" => Some(UpgradeKind::Speed),
            "stack" => Some(UpgradeKind::Stack),
            _ => None,
        }
    }

    /// Whether this port runs the upgrade itself rather than only carrying it.
    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(self, UpgradeKind::Speed | UpgradeKind::Stack)
    }

    /// The item id this upgrade is.
    #[must_use]
    pub fn item(&self) -> &str {
        match self {
            UpgradeKind::Speed => Self::SPEED,
            UpgradeKind::Stack => Self::STACK,
            UpgradeKind::Registered(item) => item,
        }
    }

    /// How many of this upgrade fit in one of the five slots.
    #[must_use]
    pub fn per_slot(&self) -> u32 {
        match self {
            UpgradeKind::Speed => SPEED_PER_SLOT,
            UpgradeKind::Stack => STACK_PER_SLOT,
            UpgradeKind::Registered(_) => OTHER_PER_SLOT,
        }
    }
}

/// One of the router's five upgrade slots: which upgrade, and how many of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upgrade {
    pub kind: UpgradeKind,
    pub count: u32,
}

impl Upgrade {
    /// A slot holding `count` of `kind`, its kind's limit applied. `None` for a
    /// count of zero: an empty slot is no slot at all.
    #[must_use]
    pub fn new(kind: UpgradeKind, count: u32) -> Option<Self> {
        if count == 0 {
            return None;
        }
        Some(Upgrade {
            count: count.min(kind.per_slot()),
            kind,
        })
    }
}

/// The router's five upgrade slots (`N_UPGRADE_SLOTS`), as the upgrades in them.
///
/// One kind per slot and at most that kind's limit, as upstream's `UpgradeHandler`
/// holds them; a save naming more slots than the router has is cut rather than
/// trusted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Upgrades {
    slots: Vec<Upgrade>,
}

impl Upgrades {
    /// The slots a saved list names, at most [`UPGRADE_SLOTS`] of them.
    #[must_use]
    pub fn new(slots: Vec<Upgrade>) -> Self {
        Upgrades {
            slots: slots.into_iter().take(UPGRADE_SLOTS).collect(),
        }
    }

    #[must_use]
    pub fn slots(&self) -> &[Upgrade] {
        &self.slots
    }

    /// How many slots are in use.
    #[must_use]
    pub fn used(&self) -> usize {
        self.slots.len()
    }

    /// How many of `kind` the router holds, summed over the slots holding it.
    #[must_use]
    pub fn count(&self, kind: &UpgradeKind) -> u32 {
        self.slots
            .iter()
            .filter(|slot| slot.kind.item() == kind.item())
            .map(|slot| slot.count)
            .sum()
    }

    /// The Speed Upgrades installed (`getUpgradeCount(SPEED_UPGRADE)`).
    #[must_use]
    pub fn speed(&self) -> u32 {
        self.count(&UpgradeKind::Speed)
    }

    /// The Stack Upgrades installed (`getUpgradeCount(STACK_UPGRADE)`).
    #[must_use]
    pub fn stack(&self) -> u32 {
        self.count(&UpgradeKind::Stack)
    }

    /// Add one `kind`: the first slot of that kind with room, else a free slot.
    /// False when five slots are all held by kinds at their limits.
    #[must_use]
    pub fn add(&mut self, kind: UpgradeKind) -> bool {
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|slot| slot.kind.item() == kind.item() && slot.count < slot.kind.per_slot())
        {
            slot.count += 1;
            return true;
        }
        if self.slots.len() >= UPGRADE_SLOTS {
            return false;
        }
        self.slots.push(Upgrade { kind, count: 1 });
        true
    }

    /// Take one `kind` out, from the last slot holding it; true when the router
    /// had one. The slot goes when that empties it.
    #[must_use]
    pub fn remove_one(&mut self, kind: &UpgradeKind) -> bool {
        let Some(index) = self
            .slots
            .iter()
            .rposition(|slot| slot.kind.item() == kind.item())
        else {
            return false;
        };
        self.slots[index].count -= 1;
        if self.slots[index].count == 0 {
            self.slots.remove(index);
        }
        true
    }
}

/// An augment installed in one of a module's four slots: a Range Up, Range Down or
/// Stack augment, or any other id a save or another mod carries.
///
/// The last case is the **registered** path, as [`ModuleKind::Registered`] is for
/// modules and [`UpgradeKind::Registered`] for upgrades: an add-on's augment keeps
/// its id and its slot, and what it does stays its own mod's business.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AugmentKind {
    /// `modularrouters:range_up_augment`: one block more of reach.
    RangeUp,
    /// `modularrouters:range_down_augment`: one block less.
    RangeDown,
    /// `modularrouters:stack_augment`: how many items one operation moves,
    /// **instead of** the count the router's Stack Upgrades give it.
    Stack,
    /// An augment this port does not run, carried by its id.
    Registered(String),
}

impl AugmentKind {
    pub const RANGE_UP: &'static str = "modularrouters:range_up_augment";
    pub const RANGE_DOWN: &'static str = "modularrouters:range_down_augment";
    pub const STACK: &'static str = "modularrouters:stack_augment";

    /// The kind an item id names: one of the three augments this port runs, or a
    /// registered kind that keeps the id unchanged.
    #[must_use]
    pub fn of(item: &str) -> Self {
        match item {
            Self::RANGE_UP => AugmentKind::RangeUp,
            Self::RANGE_DOWN => AugmentKind::RangeDown,
            Self::STACK => AugmentKind::Stack,
            other => AugmentKind::Registered(other.to_string()),
        }
    }

    /// The augment a `/modularrouters augment … <kind>` word names.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "range_up" => Some(AugmentKind::RangeUp),
            "range_down" => Some(AugmentKind::RangeDown),
            "stack" => Some(AugmentKind::Stack),
            _ => None,
        }
    }

    /// Whether this port runs the augment itself rather than only carrying it.
    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(
            self,
            AugmentKind::RangeUp | AugmentKind::RangeDown | AugmentKind::Stack
        )
    }

    /// The item id this augment is.
    #[must_use]
    pub fn item(&self) -> &str {
        match self {
            AugmentKind::RangeUp => Self::RANGE_UP,
            AugmentKind::RangeDown => Self::RANGE_DOWN,
            AugmentKind::Stack => Self::STACK,
            AugmentKind::Registered(item) => item,
        }
    }

    /// The name the readout and the command use (`range_up`).
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            AugmentKind::RangeUp => "range_up",
            AugmentKind::RangeDown => "range_down",
            AugmentKind::Stack => "stack",
            AugmentKind::Registered(item) => item,
        }
    }
}

/// One of a module's four augment slots: which augment, and how many of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Augment {
    pub kind: AugmentKind,
    pub count: u32,
}

impl Augment {
    /// A slot holding `count` of `kind`. `None` for a count of zero: an empty slot
    /// is no slot at all.
    #[must_use]
    pub fn new(kind: AugmentKind, count: u32) -> Option<Self> {
        (count > 0).then_some(Augment { kind, count })
    }
}

/// A module's four augment slots (`AugmentItem.SLOTS`), as the augments in them.
///
/// One kind per slot, as upstream's `AugmentHandler` holds them (`isItemValid`
/// refuses the same augment in a second slot), and each slot's count cut to the
/// limit the module gives that augment. A save naming more slots than a module
/// has is cut rather than trusted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Augments {
    slots: Vec<Augment>,
}

impl Augments {
    /// The slots a saved list names, at most [`AUGMENT_SLOTS`] of them.
    #[must_use]
    pub fn new(slots: Vec<Augment>) -> Self {
        Augments {
            slots: slots.into_iter().take(AUGMENT_SLOTS).collect(),
        }
    }

    #[must_use]
    pub fn slots(&self) -> &[Augment] {
        &self.slots
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// How many slots are in use.
    #[must_use]
    pub fn used(&self) -> usize {
        self.slots.len()
    }

    /// How many of `kind` the module holds, summed over the slots holding it.
    #[must_use]
    pub fn count(&self, kind: &AugmentKind) -> u32 {
        self.slots
            .iter()
            .filter(|slot| slot.kind.item() == kind.item())
            .map(|slot| slot.count)
            .sum()
    }

    /// The reach the Range Augments add (or take away): one block each, Range Up
    /// minus Range Down (`ModuleItem.getRangeModifier`).
    #[must_use]
    pub fn range_boost(&self) -> i32 {
        let up = i32::try_from(self.count(&AugmentKind::RangeUp)).unwrap_or(i32::MAX);
        let down = i32::try_from(self.count(&AugmentKind::RangeDown)).unwrap_or(i32::MAX);
        up.saturating_sub(down).saturating_mul(RANGE_PER_AUGMENT)
    }

    /// The same slots with every count cut to `limit_of` (the module's limit for
    /// that augment). A limit of zero — the module takes none of it, or it is an
    /// augment this port does not run — keeps the saved count: reading a save must
    /// not lose an entry, and `add` refuses any more of it.
    #[must_use]
    pub fn capped(self, limit_of: impl Fn(&AugmentKind) -> u32) -> Self {
        let slots = self
            .slots
            .into_iter()
            .map(|mut slot| {
                let limit = limit_of(&slot.kind);
                if limit > 0 && slot.count > limit {
                    slot.count = limit;
                }
                slot
            })
            .collect();
        Augments { slots }
    }

    /// Add one `kind` up to `limit` per slot: the slot already holding it with
    /// room, else a free slot. False when the module takes none of it, its slot
    /// holds `limit`, or the four slots are full.
    #[must_use]
    pub fn add(&mut self, kind: AugmentKind, limit: u32) -> bool {
        if limit == 0 {
            return false;
        }
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|slot| slot.kind.item() == kind.item() && slot.count < limit)
        {
            slot.count += 1;
            return true;
        }
        if self.slots.len() >= AUGMENT_SLOTS
            || self
                .slots
                .iter()
                .any(|slot| slot.kind.item() == kind.item())
        {
            return false;
        }
        self.slots.push(Augment { kind, count: 1 });
        true
    }

    /// Take one `kind` out, from the last slot holding it; true when the module
    /// had one. The slot goes when that empties it.
    #[must_use]
    pub fn remove_one(&mut self, kind: &AugmentKind) -> bool {
        let Some(index) = self
            .slots
            .iter()
            .rposition(|slot| slot.kind.item() == kind.item())
        else {
            return false;
        };
        self.slots[index].count -= 1;
        if self.slots[index].count == 0 {
            self.slots.remove(index);
        }
        true
    }
}

/// The interval between two operations, given the router's Speed Upgrades
/// (`compileUpgrades`: `max(2, 20 − 2 × speed)`).
pub fn tick_rate(speed_upgrades: u32) -> u64 {
    BASE_TICK_RATE
        .saturating_sub(TICKS_PER_SPEED_UPGRADE.saturating_mul(u64::from(speed_upgrades)))
        .max(HARD_MIN_TICK_RATE)
}

/// The items one operation moves, given the router's Stack Upgrades
/// (`1 << min(6, stacks)`).
pub fn items_per_operation(stack_upgrades: u32) -> u32 {
    1 << stack_upgrades.min(MAX_STACK_UPGRADES)
}

/// The router's own phase inside the interval, from its position: routers on one
/// level spread over the interval instead of all running on the same tick
/// (upstream picks a random counter per router when it loads).
pub fn phase_of(pos: Pos, tick_rate: u64) -> u64 {
    if tick_rate == 0 {
        return 0;
    }
    let sum = i64::from(pos[0]) + i64::from(pos[1]) + i64::from(pos[2]);
    sum.rem_euclid(tick_rate as i64) as u64
}

/// Whether this tick is the router's operation tick.
pub fn is_operation_tick(game_time: u64, phase: u64, tick_rate: u64) -> bool {
    tick_rate != 0 && game_time.wrapping_add(phase).is_multiple_of(tick_rate)
}

/// A Sender Mk1's reach: `max(1, min(16, 8 + range_up − range_down))`
/// (`IRangedModule.getCurrentRange` with `sender1BaseRange`/`sender1MaxRange`).
pub fn sender_range(boost: i32) -> i32 {
    (SENDER_BASE_RANGE + boost).clamp(1, SENDER_MAX_RANGE)
}

/// A Puller Mk2's reach: `max(1, min(24, 12 + range_up − range_down))`
/// (`IRangedModule.getCurrentRange` with `puller2BaseRange`/`puller2MaxRange`).
pub fn puller_range(boost: i32) -> i32 {
    (PULLER_BASE_RANGE + boost).clamp(1, PULLER_MAX_RANGE)
}

/// Whether `target` is within `range` blocks of `from`, as
/// `CompiledModule.isTargetValid` asks: `distSqr <= range * range`, so the reach
/// is a sphere and a diagonal target is out sooner than a straight one.
pub fn in_range(from: Pos, target: Pos, range: i32) -> bool {
    let [dx, dy, dz] = [
        i64::from(target[0]) - i64::from(from[0]),
        i64::from(target[1]) - i64::from(from[1]),
        i64::from(target[2]) - i64::from(from[2]),
    ];
    let distance = dx * dx + dy * dy + dz * dz;
    let reach = i64::from(range).max(0);
    distance <= reach * reach
}

/// The positions a sender's scan tries, nearest first (`CompiledSenderModule1`).
pub fn scan_positions(from: Pos, face: Face, range: i32) -> Vec<Pos> {
    let mut positions = Vec::new();
    let mut current = from;
    for _ in 0..range.max(0) {
        current = step(current, face);
        positions.push(current);
    }
    positions
}

/// How much of `stack` the buffer can still take: a stack of the same resource
/// (item and components) is topped up to its max size, anything else needs an
/// empty buffer.
pub fn buffer_room(buffer: Option<&Stack>, stack: &Stack, max_stack: u32) -> u32 {
    match buffer {
        None => max_stack,
        Some(held) if held.same_resource(stack) => max_stack.saturating_sub(held.count),
        Some(_) => 0,
    }
}

/// What one puller operation takes: the source slot and the count.
///
/// `CompiledModule.transferToRouter`/`findItemToPull`: a buffer stack the filter
/// accepts is topped up with more of the same resource (item **and**
/// components); an empty buffer takes the first acceptable stack, scanning the
/// slots in order; a buffer stack the filter refuses stops the operation. Never
/// more than `items` or the room left.
pub fn pick_pull(
    slots: &[Slot],
    filter: &Filter,
    buffer: Option<&Stack>,
    max_stack: impl Fn(&str) -> u32,
    items: u32,
) -> Option<(usize, u32)> {
    if items == 0 {
        return None;
    }
    let wanted = match buffer {
        // A router whose buffer holds an item the filter refuses pulls nothing.
        Some(stack) if !filter.accepts(&stack.item) => return None,
        Some(stack) => stack.clone(),
        None => {
            let slot = slots.iter().find(|slot| filter.accepts(&slot.item))?;
            Stack {
                item: slot.item.clone(),
                count: 0,
                extra: slot.extra.clone(),
            }
        }
    };
    let room = buffer_room(buffer, &wanted, max_stack(&wanted.item));
    if room == 0 {
        return None;
    }
    slots
        .iter()
        .find(|slot| slot.is(&wanted))
        .map(|slot| (slot.slot, items.min(room).min(slot.count)))
        .filter(|(_, count)| *count > 0)
}

/// The buffer after a pull put `count` of the resource `taken` in it.
pub fn buffer_after_pull(buffer: Option<&Stack>, taken: &Stack, count: u32) -> Stack {
    let count = match buffer {
        Some(stack) if stack.same_resource(taken) => stack.count.saturating_add(count),
        _ => count,
    };
    Stack {
        item: taken.item.clone(),
        count,
        extra: taken.extra.clone(),
    }
}

/// What one sender operation tries to move: the item and the count, never more
/// than the buffer holds or `items`. None when the filter refuses the buffer.
pub fn pick_send(buffer: Option<&Stack>, filter: &Filter, items: u32) -> Option<(String, u32)> {
    let stack = buffer?;
    if items == 0 || stack.count == 0 || !filter.accepts(&stack.item) {
        return None;
    }
    Some((stack.item.clone(), items.min(stack.count)))
}

/// The buffer after `sent` items left it; none when it is emptied.
pub fn buffer_after_send(buffer: &Stack, sent: u32) -> Option<Stack> {
    let left = buffer.count.saturating_sub(sent);
    (left > 0).then(|| Stack {
        count: left,
        ..buffer.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_follow_vanillas_direction_order() {
        assert_eq!(FACE_NAMES, ["down", "up", "north", "south", "west", "east"]);
        assert_eq!(FACE_NAMES.len(), FACE_COUNT);
        assert_eq!(FACE_NAMES[EAST], "east");
        assert_eq!(face_normal(NORTH), [0, 0, -1]);
        assert_eq!(face_normal(EAST), [1, 0, 0]);
        assert_eq!(step([4, 5, 6], UP), [4, 6, 6]);
        assert_eq!(step([4, 5, 6], WEST), [3, 5, 6]);
    }

    #[test]
    fn the_router_runs_every_twenty_ticks_and_two_less_per_speed_upgrade() {
        assert_eq!(tick_rate(0), BASE_TICK_RATE, "upstream baseTickRate 20");
        assert_eq!(tick_rate(1), 18);
        assert_eq!(tick_rate(4), 12);
        assert_eq!(
            tick_rate(9),
            HARD_MIN_TICK_RATE,
            "upstream hardMinTickRate 2"
        );
        assert_eq!(tick_rate(100), HARD_MIN_TICK_RATE);
    }

    #[test]
    fn one_operation_moves_one_item_and_stack_upgrades_double_it() {
        assert_eq!(
            items_per_operation(0),
            1,
            "upstream itemsPerTick starts at 1"
        );
        assert_eq!(items_per_operation(1), 2);
        assert_eq!(items_per_operation(6), 64);
        assert_eq!(items_per_operation(20), 64, "capped at 1 << 6");
    }

    #[test]
    fn an_operation_falls_on_the_routers_own_phase() {
        // The router at (1, 100, 0) has phase 101 % 20 == 1.
        let phase = phase_of([1, 100, 0], 20);
        assert_eq!(phase, 1);
        assert!(!is_operation_tick(10, phase, 20));
        assert!(is_operation_tick(19, phase, 20));
        assert!(!is_operation_tick(20, phase, 20));
        assert!(is_operation_tick(39, phase, 20));
        // A faster router keeps its own phase inside the shorter interval.
        let fast = phase_of([0, 100, 0], 2);
        assert_eq!(fast, 0);
        assert!(is_operation_tick(10, fast, 2));
        assert!(!is_operation_tick(11, fast, 2));
        assert!(is_operation_tick(12, fast, 2));
    }

    #[test]
    fn a_placed_router_faces_the_player() {
        // Vanilla `Direction.fromYRot`: 0 south, 90 west, -90 east, 180 north.
        assert_eq!(facing_from_yaw(0.0), Facing::South);
        assert_eq!(facing_from_yaw(90.0), Facing::West);
        assert_eq!(facing_from_yaw(-90.0), Facing::East);
        assert_eq!(facing_from_yaw(180.0), Facing::North);
        assert_eq!(facing_from_yaw(-180.0), Facing::North);
        // …and the placed block faces the opposite way.
        assert_eq!(facing_for_placement(0.0), Facing::North);
        assert_eq!(facing_for_placement(180.0), Facing::South);
        assert_eq!(facing_for_placement(-90.0), Facing::West);
        for name in ["north", "east", "south", "west"] {
            assert_eq!(Facing::from_name(name).map(Facing::name), Some(name));
        }
        assert_eq!(Facing::from_name("up"), None);
        assert_eq!(facing_of_state("east"), Facing::East);
        assert_eq!(facing_of_state(""), Facing::North);
    }

    #[test]
    fn a_modules_direction_turns_with_the_router() {
        let router = Facing::South;
        assert_eq!(Relative::Front.resolve(router), Some(SOUTH));
        assert_eq!(Relative::Back.resolve(router), Some(NORTH));
        assert_eq!(
            Relative::Left.resolve(router),
            Some(WEST),
            "left is getClockWise"
        );
        assert_eq!(Relative::Right.resolve(router), Some(EAST));
        assert_eq!(Relative::Up.resolve(router), Some(UP));
        assert_eq!(Relative::Down.resolve(router), Some(DOWN));
        assert_eq!(
            Relative::None.resolve(router),
            None,
            "no direction, no target"
        );
        // Around the compass: north → east → south → west.
        assert_eq!(Relative::Left.resolve(Facing::North), Some(EAST));
        assert_eq!(Relative::Right.resolve(Facing::North), Some(WEST));
        for name in ["none", "down", "up", "left", "right", "front", "back"] {
            assert_eq!(Relative::from_name(name).map(Relative::name), Some(name));
        }
        assert_eq!(Relative::from_name("sideways"), None);
    }

    #[test]
    fn an_unknown_module_id_is_a_registered_kind_that_keeps_its_id() {
        // The two transfer modules this port runs…
        let puller = ModuleKind::of("modularrouters:puller_module_1");
        assert_eq!(puller, ModuleKind::PullerMk1);
        assert!(puller.is_known());
        assert_eq!(puller.item(), "modularrouters:puller_module_1");
        let sender = ModuleKind::of("modularrouters:sender_module_1");
        assert_eq!(sender, ModuleKind::SenderMk1);
        assert!(sender.is_known());
        // …and anything else is registered by id, not rejected: an add-on's
        // module and a newer save's module both reach the router this way.
        for id in [
            "mekanisticrouters:chemical_puller_module",
            "modularrouters:vacuum_module",
            "example:foreign_module",
        ] {
            let kind = ModuleKind::of(id);
            assert_eq!(kind, ModuleKind::Registered(id.to_string()));
            assert!(!kind.is_known(), "{id} is not this port's to run");
            assert_eq!(kind.item(), id, "the id is kept verbatim");
        }
    }

    #[test]
    fn a_module_keeps_its_kind_direction_and_filter() {
        let module = Module::new(
            ModuleKind::of("example:foreign_module"),
            Relative::Left,
            Filter::new(true, vec!["minecraft:stone".to_string()]),
        );
        assert_eq!(module.kind.item(), "example:foreign_module");
        assert_eq!(module.direction, Relative::Left);
        assert!(module.filter.whitelist());
        assert_eq!(module.clone(), module);
    }

    #[test]
    fn an_empty_filter_is_a_blacklist_that_accepts_every_item() {
        let any = Filter::any();
        assert!(!any.whitelist());
        assert!(any.is_empty());
        assert!(any.accepts("minecraft:stone"));
        assert!(any.accepts("modularrouters:blank_module"));
    }

    #[test]
    fn a_filter_list_is_capped() {
        let items: Vec<String> = (0..MAX_FILTER_ITEMS + 8)
            .map(|n| format!("minecraft:item_{n}"))
            .collect();
        let filter = Filter::new(true, items);
        assert_eq!(
            filter.items().len(),
            MAX_FILTER_ITEMS,
            "an unbounded list is rebuilt on every edit and scanned on every router tick"
        );
        assert!(filter.accepts("minecraft:item_0"));
        assert!(!filter.accepts("minecraft:item_20"));
    }

    #[test]
    fn a_filter_takes_only_what_its_list_says() {
        let whitelist = Filter::new(true, vec!["minecraft:stone".to_string()]);
        assert!(whitelist.accepts("minecraft:stone"));
        assert!(!whitelist.accepts("minecraft:dirt"));
        // A whitelist with no entries matches nothing at all.
        assert!(!Filter::new(true, Vec::new()).accepts("minecraft:stone"));
        let blacklist = Filter::new(false, vec!["minecraft:dirt".to_string()]);
        assert!(!blacklist.accepts("minecraft:dirt"));
        assert!(blacklist.accepts("minecraft:stone"));
    }

    #[test]
    fn a_puller_tops_up_the_buffers_own_item_first() {
        let slots = [
            Slot::new(0, "minecraft:dirt", 5),
            Slot::new(1, "minecraft:stone", 10),
        ];
        let buffer = Stack {
            item: "minecraft:stone".to_string(),
            count: 60,
            extra: String::new(),
        };
        // Room left for the buffer's stone is 4, so only 4 are taken.
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&buffer), |_| 64, 1),
            Some((1, 1))
        );
        // With a full buffer stack nothing is pulled.
        let full = Stack {
            item: "minecraft:stone".to_string(),
            count: 64,
            extra: String::new(),
        };
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&full), |_| 64, 1),
            None
        );
    }

    #[test]
    fn a_puller_skips_slots_the_filter_refuses_and_respects_the_room() {
        let slots = [
            Slot::new(0, "minecraft:dirt", 64),
            Slot::new(1, "minecraft:stone", 64),
        ];
        let stone = Filter::new(true, vec!["minecraft:stone".to_string()]);
        assert_eq!(pick_pull(&slots, &stone, None, |_| 64, 8), Some((1, 8)));
        // A small max stack caps the take.
        assert_eq!(pick_pull(&slots, &stone, None, |_| 16, 8), Some((1, 8)));
        let one = Stack {
            item: "minecraft:stone".to_string(),
            count: 15,
            extra: String::new(),
        };
        assert_eq!(
            pick_pull(&slots, &stone, Some(&one), |_| 16, 8),
            Some((1, 1))
        );
        // A buffer stack the filter refuses stops the operation.
        let dirt = Stack {
            item: "minecraft:dirt".to_string(),
            count: 1,
            extra: String::new(),
        };
        assert_eq!(pick_pull(&slots, &stone, Some(&dirt), |_| 64, 8), None);
        // No acceptable slot, no operation.
        assert_eq!(
            pick_pull(&slots, &Filter::new(true, Vec::new()), None, |_| 64, 8),
            None
        );
    }

    #[test]
    fn a_puller_treats_one_item_with_different_components_as_two_resources() {
        // Slot 0 holds a named diamond, slot 1 a plain one.
        let named = "{display:{Name:'\"Kept\"'}}";
        let slots = [
            Slot::new(0, "minecraft:diamond", 5).with_extra(named),
            Slot::new(1, "minecraft:diamond", 5),
        ];
        // An empty buffer takes the first acceptable resource: the named one.
        assert_eq!(
            pick_pull(&slots, &Filter::any(), None, |_| 64, 8),
            Some((0, 5))
        );
        // A buffer of the named diamond tops up from slot 0 only, never from
        // slot 1 (which would replace the buffer's components).
        let buffer = Stack {
            item: "minecraft:diamond".to_string(),
            count: 60,
            extra: named.to_string(),
        };
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&buffer), |_| 64, 8),
            Some((0, 4))
        );
        // A full named stack pulls nothing, even with plain diamonds beside it.
        let full = Stack {
            count: 64,
            ..buffer.clone()
        };
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&full), |_| 64, 8),
            None
        );
        // And a pull onto an empty buffer keeps the resource's components.
        let taken = Stack {
            count: 5,
            ..buffer.clone()
        };
        let after = buffer_after_pull(None, &taken, 5);
        assert_eq!(after.extra, named, "the buffer keeps the taken components");
    }

    #[test]
    fn the_buffer_round_trips_a_pull_and_a_send() {
        let after_pull = buffer_after_pull(None, &Stack::of("minecraft:stone", 1), 3);
        assert_eq!(
            after_pull,
            Stack {
                item: "minecraft:stone".to_string(),
                count: 3,
                extra: String::new(),
            }
        );
        let topped = buffer_after_pull(Some(&after_pull), &Stack::of("minecraft:stone", 1), 2);
        assert_eq!(topped.count, 5);
        assert_eq!(
            buffer_after_send(&topped, 2).map(|stack| stack.count),
            Some(3)
        );
        assert_eq!(buffer_after_send(&topped, 5), None);
        assert_eq!(buffer_after_send(&topped, 9), None, "never negative");
    }

    #[test]
    fn the_buffer_has_room_only_for_its_own_item() {
        assert_eq!(buffer_room(None, &Stack::of("minecraft:stone", 1), 64), 64);
        let stone = Stack {
            item: "minecraft:stone".to_string(),
            count: 60,
            extra: String::new(),
        };
        assert_eq!(
            buffer_room(Some(&stone), &Stack::of("minecraft:stone", 1), 64),
            4
        );
        assert_eq!(
            buffer_room(Some(&stone), &Stack::of("minecraft:dirt", 1), 64),
            0
        );
        let full = Stack {
            item: "minecraft:stone".to_string(),
            count: 64,
            extra: String::new(),
        };
        assert_eq!(
            buffer_room(Some(&full), &Stack::of("minecraft:stone", 1), 64),
            0
        );
    }

    #[test]
    fn a_sender_moves_one_item_per_operation_until_the_buffer_is_empty() {
        let stone = Stack {
            item: "minecraft:stone".to_string(),
            count: 3,
            extra: String::new(),
        };
        assert_eq!(
            pick_send(Some(&stone), &Filter::any(), 1),
            Some(("minecraft:stone".to_string(), 1))
        );
        assert_eq!(
            pick_send(Some(&stone), &Filter::any(), 8),
            Some(("minecraft:stone".to_string(), 3)),
            "never more than the buffer holds"
        );
        assert_eq!(pick_send(None, &Filter::any(), 8), None);
        let refused = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        assert_eq!(pick_send(Some(&stone), &refused, 8), None);
    }

    #[test]
    fn a_sender_reaches_eight_blocks_and_at_most_sixteen() {
        assert_eq!(sender_range(0), SENDER_BASE_RANGE);
        assert_eq!(sender_range(4), 12);
        assert_eq!(sender_range(8), SENDER_MAX_RANGE);
        assert_eq!(sender_range(-20), 1, "never shorter than one block");
        assert_eq!(
            scan_positions([0, 100, 0], EAST, 3),
            vec![[1, 100, 0], [2, 100, 0], [3, 100, 0]]
        );
        assert!(scan_positions([0, 100, 0], UP, 0).is_empty());
    }

    #[test]
    fn the_two_upgrades_this_port_runs_keep_their_slot_limit_and_a_foreign_one_is_carried() {
        assert_eq!(
            UpgradeKind::of("modularrouters:speed_upgrade"),
            UpgradeKind::Speed
        );
        assert_eq!(
            UpgradeKind::of("modularrouters:stack_upgrade"),
            UpgradeKind::Stack
        );
        assert!(UpgradeKind::of("modularrouters:speed_upgrade").is_known());
        assert_eq!(UpgradeKind::Speed.item(), "modularrouters:speed_upgrade");
        // `UpgradeItem.getInstalledStackLimit`: nine Speed in a slot, six Stack.
        assert_eq!(UpgradeKind::Speed.per_slot(), 9);
        assert_eq!(UpgradeKind::Stack.per_slot(), 6);
        // Any other upgrade item keeps its id and fills its slot once.
        let carried = UpgradeKind::of("example:fast_upgrade");
        assert_eq!(
            carried,
            UpgradeKind::Registered("example:fast_upgrade".to_string())
        );
        assert!(!carried.is_known());
        assert_eq!(carried.per_slot(), 1);
        // The command's two words name the two this build runs.
        assert_eq!(UpgradeKind::from_name("speed"), Some(UpgradeKind::Speed));
        assert_eq!(UpgradeKind::from_name("stack"), Some(UpgradeKind::Stack));
        assert_eq!(UpgradeKind::from_name("blast"), None);
    }

    #[test]
    fn an_upgrade_slot_holds_one_kind_up_to_that_kinds_limit() {
        let mut upgrades = Upgrades::default();
        for _ in 0..SPEED_PER_SLOT {
            assert!(upgrades.add(UpgradeKind::Speed));
        }
        assert_eq!(upgrades.speed(), SPEED_PER_SLOT);
        assert_eq!(upgrades.used(), 1, "nine Speed upgrades share one slot");
        // The tenth Speed upgrade opens a second slot.
        assert!(upgrades.add(UpgradeKind::Speed));
        assert_eq!(upgrades.speed(), 10);
        assert_eq!(upgrades.used(), 2);
        // Stack upgrades go in a slot of their own, six to it.
        for _ in 0..STACK_PER_SLOT {
            assert!(upgrades.add(UpgradeKind::Stack));
        }
        assert_eq!(upgrades.stack(), STACK_PER_SLOT);
        assert_eq!(upgrades.used(), 3);
        // The counts are what the two formulas read.
        assert_eq!(tick_rate(upgrades.speed()), 2, "10 speed hits the floor");
        assert_eq!(
            items_per_operation(upgrades.stack()),
            64,
            "6 stack is the cap"
        );
    }

    #[test]
    fn five_full_slots_hold_forty_five_speed_upgrades_and_refuse_a_sixth_slot() {
        let mut upgrades = Upgrades::default();
        for _ in 0..(SPEED_PER_SLOT as usize * UPGRADE_SLOTS) {
            assert!(upgrades.add(UpgradeKind::Speed));
        }
        assert_eq!(upgrades.used(), UPGRADE_SLOTS, "five slots, nine each");
        assert_eq!(upgrades.speed(), 45);
        assert_eq!(
            tick_rate(upgrades.speed()),
            HARD_MIN_TICK_RATE,
            "45 Speed cannot go below the hard minimum of 2 ticks"
        );
        assert!(
            !upgrades.add(UpgradeKind::Speed),
            "a sixth slot does not exist"
        );
        assert!(!upgrades.add(UpgradeKind::Stack), "no slot is free for it");
        assert_eq!(upgrades.speed(), 45);
    }

    #[test]
    fn removing_an_upgrade_takes_one_and_frees_the_slot_it_empties() {
        let mut upgrades = Upgrades::new(vec![
            Upgrade::new(UpgradeKind::Speed, 2).expect("two speed upgrades"),
            Upgrade::new(UpgradeKind::Stack, 1).expect("one stack upgrade"),
        ]);
        assert!(upgrades.remove_one(&UpgradeKind::Speed));
        assert_eq!(upgrades.speed(), 1);
        assert_eq!(upgrades.used(), 2, "the slot still holds one");
        assert!(upgrades.remove_one(&UpgradeKind::Speed));
        assert_eq!(upgrades.used(), 1, "an emptied slot is no slot");
        assert!(!upgrades.remove_one(&UpgradeKind::Speed), "nothing is left");
        assert!(upgrades.remove_one(&UpgradeKind::Stack));
        assert_eq!(upgrades.stack(), 0);
        assert!(upgrades.slots().is_empty());
    }

    #[test]
    fn a_saved_upgrade_list_is_cut_to_five_slots_and_to_each_kinds_limit() {
        let upgrades = Upgrades::new(vec![
            Upgrade::new(UpgradeKind::Speed, 40).expect("40 saved, 9 kept"),
            Upgrade::new(UpgradeKind::Stack, 40).expect("40 saved, 6 kept"),
            Upgrade::new(UpgradeKind::of("example:fast_upgrade"), 4).expect("4 saved, 1 kept"),
            Upgrade::new(UpgradeKind::Speed, 1).expect("one"),
            Upgrade::new(UpgradeKind::Speed, 1).expect("one"),
            Upgrade::new(UpgradeKind::Speed, 1).expect("one"),
        ]);
        assert_eq!(upgrades.used(), UPGRADE_SLOTS, "the sixth entry is cut");
        assert_eq!(upgrades.slots()[0].count, SPEED_PER_SLOT);
        assert_eq!(upgrades.slots()[1].count, STACK_PER_SLOT);
        assert_eq!(
            upgrades.slots()[2].count,
            1,
            "an upgrade this port does not run fills its slot once"
        );
        assert_eq!(upgrades.count(&UpgradeKind::of("example:fast_upgrade")), 1);
        // A slot holds at least one: an empty entry is not a slot at all.
        assert_eq!(Upgrade::new(UpgradeKind::Speed, 0), None);
    }

    #[test]
    fn a_ranged_puller_reaches_twelve_blocks_and_at_most_twenty_four() {
        // `CommonConfig.module.puller2BaseRange` / `puller2MaxRange`, read through
        // `IRangedModule.getCurrentRange`: `max(1, min(24, 12 + boost))`.
        assert_eq!(puller_range(0), PULLER_BASE_RANGE);
        assert_eq!(puller_range(0), 12, "upstream puller2BaseRange 12");
        assert_eq!(puller_range(4), 16, "a Range Up augment adds one each");
        assert_eq!(
            puller_range(12),
            PULLER_MAX_RANGE,
            "upstream puller2MaxRange 24"
        );
        assert_eq!(puller_range(40), 24, "never past the hard maximum");
        assert_eq!(puller_range(-20), 1, "never shorter than one block");
    }

    #[test]
    fn a_target_is_in_range_by_euclidean_distance() {
        // `CompiledModule.isTargetValid`: `distSqr <= range * range`, so the reach
        // is a sphere and a diagonal target at 9,9 (12.7 blocks) is out at 12.
        let router = [0, 100, 0];
        assert!(
            in_range(router, [0, 100, -12], 12),
            "twelve straight out is in"
        );
        assert!(!in_range(router, [0, 100, -13], 12), "thirteen is out");
        assert!(
            in_range(router, [7, 100, -7], 12),
            "9.9 blocks on the diagonal"
        );
        assert!(
            !in_range(router, [9, 100, -9], 12),
            "12.7 blocks on the diagonal"
        );
        assert!(in_range(router, [0, 112, 0], 12), "up as well as across");
        assert!(!in_range(router, [0, 113, 0], 12));
        // The rule is `distSqr <= range²`, so the block itself is inside a zero
        // reach; `puller_range` never goes below one anyway.
        assert!(in_range(router, [0, 100, 0], 0));
    }

    #[test]
    fn the_ranged_puller_is_a_targeted_module_not_a_directional_one() {
        let puller2 = ModuleKind::of(ModuleKind::PULLER2);
        assert_eq!(puller2, ModuleKind::PullerMk2);
        assert!(puller2.is_known(), "this build runs it");
        assert_eq!(puller2.item(), ModuleKind::PULLER2);
        assert!(
            puller2.is_targeted(),
            "its block is the one the player selected"
        );
        assert!(
            !ModuleKind::PullerMk1.is_targeted(),
            "the Mk1 takes the block beside it, in its direction"
        );
        assert!(!ModuleKind::SenderMk1.is_targeted());
        assert!(!ModuleKind::of("example:foreign_module").is_targeted());
        assert_eq!(
            ModuleKind::of(ModuleKind::PULLER2).item(),
            "modularrouters:puller_module_2"
        );
    }

    #[test]
    fn a_target_keeps_its_position_face_and_world() {
        // Upstream's `ModuleTarget` is a `GlobalPos` plus the clicked face, and
        // `isSameWorld` refuses a target in another level.
        let target = Target::new([0, 100, -10], NORTH, "minecraft:the_end");
        assert_eq!(target.pos, [0, 100, -10]);
        assert_eq!(target.face, NORTH);
        assert!(target.same_world("minecraft:the_end"));
        assert!(!target.same_world("minecraft:overworld"));
        assert_eq!(target.at_text(), "0,100,-10");
        // The command spells a position the same way the readout does.
        assert_eq!(parse_pos("0,100,-10"), Some([0, 100, -10]));
        assert_eq!(parse_pos(" 1 , -60 , 4 "), Some([1, -60, 4]));
        assert_eq!(parse_pos("0,100"), None);
        assert_eq!(parse_pos("0,100,ten"), None);
        assert_eq!(parse_pos("none"), None);
    }

    #[test]
    fn a_module_carries_a_selected_target_beside_its_direction() {
        let target = Target::new([0, 100, -10], NORTH, "minecraft:the_end");
        let module = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_target(target.clone());
        assert_eq!(module.target, Some(target));
        assert!(module.uses_target());
        // A module with no target selects nothing: a targeted module does not fall
        // back to its direction, so it does nothing instead of scanning.
        let bare = Module::new(ModuleKind::PullerMk2, Relative::Front, Filter::any());
        assert_eq!(bare.target, None);
        assert!(
            bare.uses_target(),
            "still a targeted module, just unselected"
        );
        assert!(
            !Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).uses_target(),
            "the Mk1's target is the block its direction names"
        );
    }

    #[test]
    fn a_click_names_a_face_and_a_selected_blocks_side_faces_the_router() {
        // `BlockHit::face` is a normal vector: the six faces map back to their
        // index in `Direction.ALL`, and anything else names none.
        assert_eq!(face_of_normal([0, -1, 0]), Some(DOWN));
        assert_eq!(face_of_normal([0, 1, 0]), Some(UP));
        assert_eq!(face_of_normal([0, 0, -1]), Some(NORTH));
        assert_eq!(face_of_normal([0, 0, 1]), Some(SOUTH));
        assert_eq!(face_of_normal([-1, 0, 0]), Some(WEST));
        assert_eq!(face_of_normal([1, 0, 0]), Some(EAST));
        assert_eq!(face_of_normal([1, 1, 0]), None);
        // The command is told where a selected block is, not which face was
        // clicked: it uses the side facing the router, as a player at the router
        // would have clicked. The dominant axis wins a diagonal, and a block is
        // not beside itself.
        assert_eq!(side_towards([0, 100, -10], [0, 100, 0]), Some(SOUTH));
        assert_eq!(side_towards([0, 100, 10], [0, 100, 0]), Some(NORTH));
        assert_eq!(side_towards([10, 100, 0], [0, 100, 0]), Some(WEST));
        assert_eq!(
            side_towards([0, 100, -10], [4, 100, -7]),
            Some(EAST),
            "4 east against 3 south: the dominant axis wins"
        );
        assert_eq!(side_towards([0, 100, 0], [0, 100, 0]), None);
    }

    #[test]
    fn a_foreign_upgrade_survives_the_slot_list_it_is_carried_in() {
        let mut upgrades = Upgrades::new(vec![
            Upgrade::new(UpgradeKind::of("example:fast_upgrade"), 1).expect("one foreign upgrade"),
        ]);
        assert!(upgrades.add(UpgradeKind::Speed));
        assert_eq!(upgrades.slots().len(), 2);
        assert_eq!(upgrades.slots()[0].kind.item(), "example:fast_upgrade");
        assert_eq!(upgrades.speed(), 1);
    }

    #[test]
    fn a_range_augment_moves_the_reach_one_block_each_up_to_the_modules_limit() {
        // `AugmentItem`/`RangeAugments` and `ModuleItem.getRangeModifier`: the
        // reach is `getCurrentRange` = `max(1, min(hardMax, base + up − down))`,
        // and one augment is one block.
        assert_eq!(
            AugmentKind::of("modularrouters:range_up_augment"),
            AugmentKind::RangeUp
        );
        assert_eq!(
            AugmentKind::of("modularrouters:range_down_augment"),
            AugmentKind::RangeDown
        );
        assert!(AugmentKind::RangeUp.is_known());
        assert_eq!(
            AugmentKind::RangeUp.item(),
            "modularrouters:range_up_augment"
        );
        assert_eq!(AugmentKind::RangeUp.name(), "range_up");
        assert_eq!(
            AugmentKind::from_name("range_down"),
            Some(AugmentKind::RangeDown)
        );
        assert_eq!(AugmentKind::from_name("mimic"), None);
        // An augment this port does not run keeps its id, and fills its slot once.
        let carried = AugmentKind::of("example:range_augment");
        assert_eq!(
            carried,
            AugmentKind::Registered("example:range_augment".to_string())
        );
        assert!(!carried.is_known());
        assert_eq!(carried.name(), "example:range_augment");

        // `RangeAugments.getMaxAugments`: up = hardMax − base, down = base − 1.
        // The Mk2 is 12/24, the Sender Mk1 8/16, and a module that is not ranged
        // (`IRangedModule`) takes none of either.
        assert_eq!(
            ModuleKind::PullerMk2.augment_limit(&AugmentKind::RangeUp),
            12
        );
        assert_eq!(
            ModuleKind::PullerMk2.augment_limit(&AugmentKind::RangeDown),
            11
        );
        assert_eq!(
            ModuleKind::SenderMk1.augment_limit(&AugmentKind::RangeUp),
            8
        );
        assert_eq!(
            ModuleKind::SenderMk1.augment_limit(&AugmentKind::RangeDown),
            7
        );
        assert_eq!(
            ModuleKind::PullerMk1.augment_limit(&AugmentKind::RangeUp),
            0
        );
        assert_eq!(
            ModuleKind::Registered("example:ranged_module".to_string())
                .augment_limit(&AugmentKind::RangeUp),
            0,
            "a module this port does not run does not declare a reach here"
        );
        assert_eq!(
            ModuleKind::PullerMk2.augment_limit(&AugmentKind::of("example:range_augment")),
            1,
            "a foreign augment is carried one per slot"
        );

        // The module's own reach, as the tick and the readout ask for it.
        let mk2 = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any());
        assert_eq!(mk2.current_range(), Some(12));
        let boosted = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeUp, 4).expect("four range up"),
            ]));
        assert_eq!(
            boosted.current_range(),
            Some(16),
            "four augments, four blocks"
        );
        let reduced = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeDown, 1).expect("one range down"),
            ]));
        assert_eq!(reduced.current_range(), Some(11));
        // The hard maximum and the one-block floor are the range rule's, not the
        // augment's: twelve up is 24, and eleven down is 1.
        let capped = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeUp, 12).expect("twelve range up"),
            ]));
        assert_eq!(capped.current_range(), Some(PULLER_MAX_RANGE));
        let floored = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeDown, 11).expect("eleven range down"),
            ]));
        assert_eq!(floored.current_range(), Some(1));
        // The sender is ranged too, and its own augments move its scan.
        let sender = Module::new(ModuleKind::SenderMk1, Relative::Back, Filter::any());
        assert_eq!(sender.current_range(), Some(SENDER_BASE_RANGE));
        let sender_up = sender.clone().with_augments(Augments::new(vec![
            Augment::new(AugmentKind::RangeUp, 8).expect("eight range up"),
        ]));
        assert_eq!(sender_up.current_range(), Some(SENDER_MAX_RANGE));
        // A module that is not ranged has no reach to report or move.
        assert_eq!(
            Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).current_range(),
            None
        );
    }

    #[test]
    fn the_four_augment_slots_hold_one_kind_each_up_to_the_modules_limit() {
        let mut mk2 = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any());
        // `AugmentItem.SLOTS` is four, and `AugmentHandler.isItemValid` refuses the
        // same augment in a second slot: twelve Range Up augments share slot 0 and
        // the thirteenth is refused, not put in a slot of its own.
        for _ in 0..12 {
            assert!(mk2.add_augment(AugmentKind::RangeUp));
        }
        assert_eq!(mk2.augments().count(&AugmentKind::RangeUp), 12);
        assert_eq!(mk2.augments().used(), 1, "one slot holds all twelve");
        assert!(
            !mk2.add_augment(AugmentKind::RangeUp),
            "its limit is twelve"
        );
        assert_eq!(mk2.current_range(), Some(24));
        // A second kind opens a second slot; eleven down is its own limit.
        for _ in 0..11 {
            assert!(mk2.add_augment(AugmentKind::RangeDown));
        }
        assert_eq!(mk2.augments().used(), 2);
        assert!(!mk2.add_augment(AugmentKind::RangeDown));
        // Two foreign augments fill the last two slots; a fifth kind has nowhere.
        assert!(mk2.add_augment(AugmentKind::of("example:a_augment")));
        assert!(mk2.add_augment(AugmentKind::of("example:b_augment")));
        assert_eq!(mk2.augments().used(), AUGMENT_SLOTS);
        assert!(!mk2.add_augment(AugmentKind::of("example:c_augment")));
        // A module that is not ranged takes no Range augment at all.
        let mut mk1 = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any());
        assert!(!mk1.add_augment(AugmentKind::RangeUp));
        assert_eq!(mk1.augments().used(), 0);
        // Removing takes one out and an emptied slot is no slot.
        assert!(mk2.remove_augment(&AugmentKind::RangeDown));
        assert_eq!(mk2.augments().count(&AugmentKind::RangeDown), 10);
        for _ in 0..10 {
            assert!(mk2.remove_augment(&AugmentKind::RangeDown));
        }
        assert!(!mk2.remove_augment(&AugmentKind::RangeDown), "nothing left");
        assert_eq!(mk2.augments().used(), 3);
    }

    #[test]
    fn a_saved_augment_list_is_cut_to_four_slots_and_to_the_modules_limit() {
        // A save can name more than the module has: fifteen Range Up augments in
        // one slot are cut to the Mk2's twelve (24 blocks, the hard maximum), a
        // fifth slot is cut, and a count of zero is not a slot.
        let saved = Augments::new(vec![
            Augment::new(AugmentKind::RangeUp, 15).expect("fifteen saved"),
            Augment::new(AugmentKind::of("example:a_augment"), 4).expect("four saved"),
            Augment::new(AugmentKind::of("example:b_augment"), 1).expect("one"),
            Augment::new(AugmentKind::of("example:c_augment"), 1).expect("one"),
            Augment::new(AugmentKind::of("example:d_augment"), 1).expect("the fifth slot is cut"),
        ]);
        let module =
            Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any()).with_augments(saved);
        assert_eq!(module.augments().used(), AUGMENT_SLOTS);
        assert_eq!(module.augments().count(&AugmentKind::RangeUp), 12);
        assert_eq!(
            module
                .augments()
                .count(&AugmentKind::of("example:a_augment")),
            4,
            "an augment this port does not run keeps the count the save gave it: its \
             per-slot limit is its own mod's, so the read must not invent one"
        );
        assert_eq!(module.current_range(), Some(PULLER_MAX_RANGE));
        assert_eq!(Augment::new(AugmentKind::RangeUp, 0), None);
        assert!(Augments::default().is_empty());
    }

    #[test]
    fn a_stack_augment_overrides_the_routers_items_per_operation() {
        // `CompiledModule.getItemsPerTick`: `n > 0 ? min(1 << n, 64) : router.getItemsPerTick()`.
        // The augment **replaces** the count the router's Stack Upgrades give it; it
        // does not multiply it, which is the one augment whose number is not a range.
        assert_eq!(
            AugmentKind::of("modularrouters:stack_augment"),
            AugmentKind::Stack
        );
        assert!(AugmentKind::Stack.is_known());
        assert_eq!(AugmentKind::Stack.item(), "modularrouters:stack_augment");
        assert_eq!(AugmentKind::Stack.name(), "stack");
        assert_eq!(AugmentKind::from_name("stack"), Some(AugmentKind::Stack));

        let with_stack = |count: u32| {
            Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_augments(
                Augments::new(vec![
                    Augment::new(AugmentKind::Stack, count).expect("a stack augment slot"),
                ]),
            )
        };
        // One Stack Upgrade in the router reads `2 item(s) every 20 ticks`; two Stack
        // Augments on the module make the operation four, not `2 × 4`.
        assert_eq!(items_per_operation(1), 2, "the router's own count");
        assert_eq!(with_stack(2).items_per_operation(items_per_operation(1)), 4);
        // The same two augments with no upgrade at all: still four. The augment is
        // what moves them, not the upgrade.
        assert_eq!(with_stack(2).items_per_operation(items_per_operation(0)), 4);
        // Six augments are the cap: `min(1 << 6, 64)` is 64.
        assert_eq!(with_stack(6).items_per_operation(1), 64);
        assert_eq!(with_stack(1).items_per_operation(1), 2);
        // No augment: the router's count is what the operation moves, unchanged.
        let plain = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any());
        assert_eq!(plain.items_per_operation(1), 1);
        assert_eq!(plain.items_per_operation(64), 64);
        // The Sender moves the augmented count too (`CompiledSenderModule1`).
        let sender = Module::new(ModuleKind::SenderMk1, Relative::Back, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::Stack, 2).expect("two stack augments"),
            ]));
        assert_eq!(sender.items_per_operation(1), 4);
    }

    #[test]
    fn a_stack_augment_takes_six_per_slot_and_none_on_the_modules_that_count_no_items() {
        // `StackAugment.getMaxAugments`: six, except the Detector, both Extruders, the
        // Breaker, the Placer and both Fluid modules — their operation is not an item
        // count. Those kinds are gap 1 in this port (carried by id, never run), so the
        // rule is the module **id**'s; every other module, including one this port
        // only carries, takes upstream's six.
        for known in [
            ModuleKind::PullerMk1,
            ModuleKind::PullerMk2,
            ModuleKind::SenderMk1,
        ] {
            assert_eq!(
                known.augment_limit(&AugmentKind::Stack),
                6,
                "{} takes six",
                known.item()
            );
        }
        for id in [
            "modularrouters:detector_module",
            "modularrouters:extruder_module_1",
            "modularrouters:extruder_module_2",
            "modularrouters:breaker_module",
            "modularrouters:placer_module",
            "modularrouters:fluid_module",
            "modularrouters:fluid_module_2",
        ] {
            let kind = ModuleKind::of(id);
            assert_eq!(
                kind.augment_limit(&AugmentKind::Stack),
                0,
                "{id} takes none"
            );
            let mut module = Module::new(kind, Relative::Front, Filter::any());
            assert!(!module.add_augment(AugmentKind::Stack), "{id}");
            assert_eq!(module.augments().used(), 0);
        }
        assert_eq!(
            ModuleKind::Registered("example:other_module".to_string())
                .augment_limit(&AugmentKind::Stack),
            6,
            "upstream's rule lists the classes that take none, so the rest take six"
        );
        // A Stack augment is not a Range augment: it does not touch a reach.
        let mut mk2 = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any());
        assert!(mk2.add_augment(AugmentKind::Stack));
        assert_eq!(mk2.current_range(), Some(PULLER_BASE_RANGE));
    }

    #[test]
    fn a_saved_stack_augment_count_is_cut_to_the_modules_six() {
        // The slot holds at most six, so a save naming nine reads as six — and the
        // shift never reaches a count the slot's limit cannot give it.
        let saved = Augments::new(vec![
            Augment::new(AugmentKind::Stack, 9).expect("nine saved"),
        ]);
        let mut module =
            Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_augments(saved);
        assert_eq!(module.augments().count(&AugmentKind::Stack), 6);
        assert_eq!(module.items_per_operation(1), MAX_ITEMS_PER_OPERATION);
        assert!(module.remove_augment(&AugmentKind::Stack));
        assert_eq!(module.augments().count(&AugmentKind::Stack), 5);
        assert_eq!(module.items_per_operation(1), 32);
        assert_eq!(module.augment_limit(&AugmentKind::Stack), 6);
    }
}
