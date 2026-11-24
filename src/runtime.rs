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
/// `CommonConfig.module.sender2BaseRange`: a Sender Mk2 reaches 24 blocks.
pub const SENDER2_BASE_RANGE: i32 = 24;
/// `CommonConfig.module.sender2MaxRange`: up to 48 with Range Augments.
pub const SENDER2_MAX_RANGE: i32 = 48;
/// `CommonConfig.module.puller2BaseRange`: a Puller Mk2 reaches 12 blocks.
pub const PULLER_BASE_RANGE: i32 = 12;
/// `CommonConfig.module.puller2MaxRange`: up to 24 with Range Augments.
pub const PULLER_MAX_RANGE: i32 = 24;
/// `CommonConfig.module.vacuumBaseRange`: the Vacuum Module sweeps 6 blocks in
/// every direction around it (or `range + 1` blocks out when it is given one).
pub const VACUUM_BASE_RANGE: i32 = 6;
/// `CommonConfig.module.vacuumMaxRange`: up to 12 with Range Augments.
pub const VACUUM_MAX_RANGE: i32 = 12;
/// `FlingerModule.MIN_SPEED` / `MAX_SPEED`: the speed a flinger's item leaves
/// with, in blocks per tick (`Vec3.scale(speed)` on the unit aim vector).
pub const FLINGER_MIN_SPEED: f32 = 0.0;
pub const FLINGER_MAX_SPEED: f32 = 20.0;
/// `FlingerModule.MIN_PITCH` / `MAX_PITCH`: degrees added to the throw's base
/// pitch (0 for a horizontal direction, ±90 for up/down).
pub const FLINGER_MIN_PITCH: f32 = -90.0;
pub const FLINGER_MAX_PITCH: f32 = 90.0;
/// `FlingerModule.MIN_YAW` / `MAX_YAW`: degrees added to the direction's own
/// quadrant (±60 of the module's aim, so a fling cannot be aimed backwards).
pub const FLINGER_MIN_YAW: f32 = -60.0;
pub const FLINGER_MAX_YAW: f32 = 60.0;
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
/// `RegulatorAugment.getMaxAugments`: one Regulator Augment per module.
pub const REGULATOR_LIMIT: u32 = 1;
/// `RegulatorAugment.getMaxAugments` names the modules that take **none** of it:
/// the Detector and the Extruder Mk2. Every other module — including the ones this
/// build only carries — takes one, because upstream's rule is a list of classes
/// that take none.
pub const NO_REGULATOR_AUGMENT: [&str; 2] = [
    "modularrouters:detector_module",
    "modularrouters:extruder_module_2",
];
/// `IPickaxeUser.DEFAULT_PICK`: the pickaxe a Breaker Module starts with when
/// the recipe (or a save) did not put one in its slot — one stone pickaxe.
pub const DEFAULT_PICKAXE: &str = "minecraft:stone_pickaxe";

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

/// Which module is installed: one of the four transfer modules this port runs,
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
    /// `modularrouters:sender_module_1`: buffer → container along its facing.
    SenderMk1,
    /// `modularrouters:sender_module_2`: buffer → the selected container in range.
    SenderMk2,
    /// `modularrouters:void_module`: destroys the filtered stack in the buffer.
    Void,
    /// `modularrouters:dropper_module`: throws the filtered stack in the buffer into
    /// the block its direction names.
    Dropper,
    /// `modularrouters:vacuum_module`: fills the buffer from the dropped items in
    /// the box it sweeps.
    Vacuum,
    /// `modularrouters:flinger_module`: the Dropper Module with a velocity — the
    /// same filtered stack thrown into the block its direction names, given the
    /// module's saved speed, pitch and yaw.
    Flinger,
    /// `modularrouters:placer_module`: places one item out of the buffer as a block
    /// in the adjacent block its direction names.
    Placer,
    /// `modularrouters:breaker_module`: breaks the block its direction names with
    /// its saved pickaxe, putting the drops that pass its filter into the buffer.
    Breaker,
    /// A module this port does not implement, carried by its id.
    Registered(String),
}

impl ModuleKind {
    pub const PULLER: &'static str = "modularrouters:puller_module_1";
    pub const PULLER2: &'static str = "modularrouters:puller_module_2";
    pub const SENDER: &'static str = "modularrouters:sender_module_1";
    pub const SENDER2: &'static str = "modularrouters:sender_module_2";
    pub const VOID: &'static str = "modularrouters:void_module";
    pub const DROPPER: &'static str = "modularrouters:dropper_module";
    pub const VACUUM: &'static str = "modularrouters:vacuum_module";
    pub const FLINGER: &'static str = "modularrouters:flinger_module";
    pub const PLACER: &'static str = "modularrouters:placer_module";
    pub const BREAKER: &'static str = "modularrouters:breaker_module";

    /// The kind an item id names: one of the modules this port runs, or a
    /// registered kind that keeps the id unchanged.
    #[must_use]
    pub fn of(item: &str) -> Self {
        match item {
            Self::PULLER => ModuleKind::PullerMk1,
            Self::PULLER2 => ModuleKind::PullerMk2,
            Self::SENDER => ModuleKind::SenderMk1,
            Self::SENDER2 => ModuleKind::SenderMk2,
            Self::VOID => ModuleKind::Void,
            Self::DROPPER => ModuleKind::Dropper,
            Self::VACUUM => ModuleKind::Vacuum,
            Self::FLINGER => ModuleKind::Flinger,
            Self::PLACER => ModuleKind::Placer,
            Self::BREAKER => ModuleKind::Breaker,
            other => ModuleKind::Registered(other.to_string()),
        }
    }

    /// Whether this port runs the module itself rather than only carrying it.
    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(
            self,
            ModuleKind::PullerMk1
                | ModuleKind::PullerMk2
                | ModuleKind::SenderMk1
                | ModuleKind::SenderMk2
                | ModuleKind::Void
                | ModuleKind::Dropper
                | ModuleKind::Vacuum
                | ModuleKind::Flinger
                | ModuleKind::Placer
                | ModuleKind::Breaker
        )
    }

    /// Whether the module moves items **out of the buffer into a container** (a
    /// sender): its regulation amount is the floor the target is topped up to,
    /// where a puller's or a void's is what is left in the source.
    #[must_use]
    pub fn is_sender(&self) -> bool {
        matches!(self, ModuleKind::SenderMk1 | ModuleKind::SenderMk2)
    }

    /// Whether the module works on a block the player **selected** rather than the
    /// one beside it (`ModuleItem.isDirectional` is false for a targeted module,
    /// so its relative direction names nothing).
    #[must_use]
    pub fn is_targeted(&self) -> bool {
        matches!(self, ModuleKind::PullerMk2 | ModuleKind::SenderMk2)
    }

    /// The item id this module is.
    #[must_use]
    pub fn item(&self) -> &str {
        match self {
            ModuleKind::PullerMk1 => Self::PULLER,
            ModuleKind::PullerMk2 => Self::PULLER2,
            ModuleKind::SenderMk1 => Self::SENDER,
            ModuleKind::SenderMk2 => Self::SENDER2,
            ModuleKind::Void => Self::VOID,
            ModuleKind::Dropper => Self::DROPPER,
            ModuleKind::Vacuum => Self::VACUUM,
            ModuleKind::Flinger => Self::FLINGER,
            ModuleKind::Placer => Self::PLACER,
            ModuleKind::Breaker => Self::BREAKER,
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
            ModuleKind::SenderMk2 => Some(SENDER2_BASE_RANGE),
            ModuleKind::Vacuum => Some(VACUUM_BASE_RANGE),
            ModuleKind::PullerMk1
            | ModuleKind::Void
            | ModuleKind::Dropper
            | ModuleKind::Flinger
            | ModuleKind::Placer
            | ModuleKind::Breaker
            | ModuleKind::Registered(_) => None,
        }
    }

    /// The longest reach Range Up augments can give it (`getHardMaxRange`).
    #[must_use]
    pub fn max_range(&self) -> Option<i32> {
        match self {
            ModuleKind::PullerMk2 => Some(PULLER_MAX_RANGE),
            ModuleKind::SenderMk1 => Some(SENDER_MAX_RANGE),
            ModuleKind::SenderMk2 => Some(SENDER2_MAX_RANGE),
            ModuleKind::Vacuum => Some(VACUUM_MAX_RANGE),
            ModuleKind::PullerMk1
            | ModuleKind::Void
            | ModuleKind::Dropper
            | ModuleKind::Flinger
            | ModuleKind::Placer
            | ModuleKind::Breaker
            | ModuleKind::Registered(_) => None,
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
            AugmentKind::Regulator => self.regulator_limit(),
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

    /// `RegulatorAugment.getMaxAugments`: one on every module except the Detector
    /// and the Extruder Mk2, which take none (their operation does not read an
    /// inventory count). Upstream tests the module class, so every other module —
    /// including one this build only carries — takes one.
    #[must_use]
    pub fn regulator_limit(&self) -> Option<i32> {
        (!NO_REGULATOR_AUGMENT.contains(&self.item()))
            .then(|| i32::try_from(REGULATOR_LIMIT).unwrap_or(i32::MAX))
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

/// `CompiledBreakerModule.MatchType`: whether a Breaker's filter tests each
/// drop (`item`) or the block itself (`block`). The serialized names are the
/// enum's `getSerializedName`; the variants are named `By…` so they cannot be
/// mistaken for the SDK's `Item`/`Block` types when the package's uses table is
/// compared across builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    ByItem,
    ByBlock,
}

impl MatchType {
    /// The type named by its serialized form, or `None` for anything else.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "item" => Some(Self::ByItem),
            "block" => Some(Self::ByBlock),
            _ => None,
        }
    }

    /// The serialized name (`MatchType.getSerializedName`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ByItem => "item",
            Self::ByBlock => "block",
        }
    }
}

/// `BreakerSettings` plus the pickaxe a Breaker Module breaks with: the two
/// fields upstream keeps in the module item's `BREAKER_SETTINGS` and `PICKAXE`
/// components. A saved entry without either reads as this default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Break {
    pub pickaxe: String,
    pub match_type: MatchType,
}

impl Break {
    /// The settings a module carries before a player changes them.
    #[must_use]
    pub fn new(pickaxe: impl Into<String>, match_type: MatchType) -> Self {
        Self {
            pickaxe: pickaxe.into(),
            match_type,
        }
    }
}

impl Default for Break {
    fn default() -> Self {
        Self::new(DEFAULT_PICKAXE, MatchType::ByItem)
    }
}

/// A module installed in one slot: what it is, where it points, what it takes,
/// the block a targeted one selected, the augments in its four slots and the
/// amount a Regulator Augment makes it leave in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub kind: ModuleKind,
    pub direction: Relative,
    pub filter: Filter,
    /// The selected block of a targeted module (`ModuleTarget`), `None` until the
    /// player sets one — an unselected targeted module does nothing.
    pub target: Option<Target>,
    augments: Augments,
    /// `ModuleSettings.regulatorAmount`: what a Regulator Augment makes the
    /// operation leave in the source. Zero, and inert without the augment.
    regulation: u32,
    /// `FlingerSettings`: the flight settings of a Flinger Module, carried by
    /// every module and read only by that kind.
    flinger: Flinger,
    /// `BreakerSettings` and the saved pickaxe, carried by every module and read
    /// only by that kind.
    breaker: Break,
}

impl Module {
    pub fn new(kind: ModuleKind, direction: Relative, filter: Filter) -> Self {
        Module {
            kind,
            direction,
            filter,
            target: None,
            augments: Augments::default(),
            regulation: 0,
            flinger: Flinger::DEFAULT,
            breaker: Break::default(),
        }
    }

    /// The module's saved pickaxe and match type (`BreakerSettings`).
    #[must_use]
    pub fn breaker(&self) -> &Break {
        &self.breaker
    }

    /// The same module saving the given pickaxe and match type.
    #[must_use]
    pub fn with_breaker(mut self, breaker: Break) -> Self {
        self.breaker = breaker;
        self
    }

    /// Update the saved pickaxe and match type (the module screen's two fields).
    pub fn set_breaker(&mut self, breaker: Break) {
        self.breaker = breaker;
    }

    /// The module's saved flight settings (`FlingerSettings`); the default
    /// (0, 0, 0) on a module that is not a flinger, where they are inert.
    #[must_use]
    pub fn flinger(&self) -> Flinger {
        self.flinger
    }

    /// The same module saving `flinger` — the validated triple, as
    /// [`Flinger::validated`] leaves it.
    #[must_use]
    pub fn with_flinger(mut self, flinger: Flinger) -> Self {
        self.flinger = flinger.validated();
        self
    }

    /// Set the saved flight settings (the module screen's three fields).
    pub fn set_flinger(&mut self, flinger: Flinger) {
        self.flinger = flinger.validated();
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

    /// The amount a Regulator Augment makes this module leave in the source
    /// (`ModuleSettings.regulatorAmount`), as the module saves it. It has no
    /// effect until a Regulator Augment is in one of its slots.
    #[must_use]
    pub fn regulation(&self) -> u32 {
        self.regulation
    }

    /// The same module saving `amount` as its regulation amount.
    #[must_use]
    pub fn with_regulation(mut self, amount: u32) -> Self {
        self.regulation = amount;
        self
    }

    /// Set the saved regulation amount (the module screen's text field).
    pub fn set_regulation(&mut self, amount: u32) {
        self.regulation = amount;
    }

    /// `CompiledModule.getRegulationAmount`: the saved amount **only while a
    /// Regulator Augment is installed**, zero otherwise. The augment is the switch;
    /// without it the saved number is carried but ignored.
    #[must_use]
    pub fn regulation_amount(&self) -> u32 {
        if self.augments.count(&AugmentKind::Regulator) > 0 {
            self.regulation
        } else {
            0
        }
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
            ModuleKind::SenderMk2 => Some(sender2_range(boost)),
            ModuleKind::Vacuum => Some(vacuum_range(boost)),
            ModuleKind::PullerMk1
            | ModuleKind::Void
            | ModuleKind::Dropper
            | ModuleKind::Flinger
            | ModuleKind::Placer
            | ModuleKind::Breaker
            | ModuleKind::Registered(_) => None,
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
    pub(crate) fn is(&self, stack: &Stack) -> bool {
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

/// An augment installed in one of a module's four slots: a Range Up, Range Down,
/// Stack or Regulator augment, or any other id a save or another mod carries.
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
    /// `modularrouters:regulator_augment`: make the saved regulation amount count,
    /// so a pull leaves that many of the item in the source.
    Regulator,
    /// An augment this port does not run, carried by its id.
    Registered(String),
}

impl AugmentKind {
    pub const RANGE_UP: &'static str = "modularrouters:range_up_augment";
    pub const RANGE_DOWN: &'static str = "modularrouters:range_down_augment";
    pub const STACK: &'static str = "modularrouters:stack_augment";
    pub const REGULATOR: &'static str = "modularrouters:regulator_augment";

    /// The kind an item id names: one of the four augments this port runs, or a
    /// registered kind that keeps the id unchanged.
    #[must_use]
    pub fn of(item: &str) -> Self {
        match item {
            Self::RANGE_UP => AugmentKind::RangeUp,
            Self::RANGE_DOWN => AugmentKind::RangeDown,
            Self::STACK => AugmentKind::Stack,
            Self::REGULATOR => AugmentKind::Regulator,
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
            "regulator" => Some(AugmentKind::Regulator),
            _ => None,
        }
    }

    /// Whether this port runs the augment itself rather than only carrying it.
    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(
            self,
            AugmentKind::RangeUp
                | AugmentKind::RangeDown
                | AugmentKind::Stack
                | AugmentKind::Regulator
        )
    }

    /// The item id this augment is.
    #[must_use]
    pub fn item(&self) -> &str {
        match self {
            AugmentKind::RangeUp => Self::RANGE_UP,
            AugmentKind::RangeDown => Self::RANGE_DOWN,
            AugmentKind::Stack => Self::STACK,
            AugmentKind::Regulator => Self::REGULATOR,
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
            AugmentKind::Regulator => "regulator",
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

/// A Sender Mk2's reach: `max(1, min(48, 24 + range_up − range_down))`
/// (`IRangedModule.getCurrentRange` with `sender2BaseRange`/`sender2MaxRange`).
pub fn sender2_range(boost: i32) -> i32 {
    (SENDER2_BASE_RANGE + boost).clamp(1, SENDER2_MAX_RANGE)
}

/// A Puller Mk2's reach: `max(1, min(24, 12 + range_up − range_down))`
/// (`IRangedModule.getCurrentRange` with `puller2BaseRange`/`puller2MaxRange`).
pub fn puller_range(boost: i32) -> i32 {
    (PULLER_BASE_RANGE + boost).clamp(1, PULLER_MAX_RANGE)
}

/// A Vacuum Module's reach: `max(1, min(12, 6 + range_up − range_down))`
/// (`IRangedModule.getCurrentRange` with `vacuumBaseRange`/`vacuumMaxRange`), the
/// half-width of the box it sweeps.
pub fn vacuum_range(boost: i32) -> i32 {
    (VACUUM_BASE_RANGE + boost).clamp(1, VACUUM_MAX_RANGE)
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
///
/// `keep` is `getRegulationAmount()`: a source whose total of the wanted resource
/// minus `items` would fall below it refuses the operation **whole** (upstream's
/// `count.getInt(stack) - nToTake >= getRegulationAmount()`), so the pull leaves at
/// least that many behind instead of taking a partial stack. Zero is the old
/// behaviour.
pub fn pick_pull(
    slots: &[Slot],
    filter: &Filter,
    buffer: Option<&Stack>,
    max_stack: impl Fn(&str) -> u32,
    items: u32,
    keep: u32,
) -> Option<(usize, u32)> {
    if items == 0 {
        return None;
    }
    // The source's total for one resource, over every slot that holds it — the
    // count upstream's `CountedItemStacks` keeps for the regulation check.
    let total = |wanted: &Stack| -> u32 {
        slots
            .iter()
            .filter(|slot| slot.is(wanted))
            .map(|slot| slot.count)
            .sum()
    };
    // `count - nToTake >= regulation`: taking `items` must still leave `keep`.
    let leaves_enough = |wanted: &Stack| total(wanted).saturating_sub(items) >= keep;
    let wanted = match buffer {
        // A router whose buffer holds an item the filter refuses pulls nothing.
        Some(stack) if !filter.accepts(&stack.item) => return None,
        Some(stack) => {
            let stack = stack.clone();
            // A buffer stack the regulation would starve also stops the operation
            // rather than being filled from the source down to its floor.
            if !leaves_enough(&stack) {
                return None;
            }
            stack
        }
        None => {
            // Scan for the first acceptable stack that still leaves `keep` behind;
            // upstream skips a stack at the floor and tries the next slot.
            let mut found = None;
            for slot in slots {
                if !filter.accepts(&slot.item) {
                    continue;
                }
                let candidate = Stack {
                    item: slot.item.clone(),
                    count: 0,
                    extra: slot.extra.clone(),
                };
                if leaves_enough(&candidate) {
                    found = Some(candidate);
                    break;
                }
            }
            found?
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

/// What one sender operation may move: the module's items-per-operation, capped
/// by the room a Regulator Augment's amount leaves in the target.
///
/// `CompiledSenderModule1.execute`: with `getRegulationAmount() > 0` it counts the
/// target's matching items (up to the amount) and sends
/// `min(getItemsPerTick, amount − existing)`, moving nothing when that is not
/// positive. `amount = 0` is exactly the pre-augment behaviour.
#[must_use]
pub fn send_quota(items: u32, amount: u32, existing: u32) -> u32 {
    if amount == 0 {
        return items;
    }
    items.min(amount.saturating_sub(existing))
}

/// What one void operation destroys: this many of the buffered stack, never more
/// than the buffer holds or `items`, and never the `keep` the module's saved
/// regulation amount reserves.
///
/// `CompiledVoidModule.execute` is `min(getItemsPerTick, count −
/// getRegulationAmount())` and nothing at all when that is not positive; the
/// filter's test comes first, so a stack the filter refuses stays whole. `keep`
/// is `Module::regulation_amount()`, which is zero unless a Regulator Augment is
/// installed.
pub fn pick_void(buffer: Option<&Stack>, filter: &Filter, items: u32, keep: u32) -> Option<u32> {
    let stack = buffer?;
    if items == 0 || stack.count == 0 || !filter.accepts(&stack.item) {
        return None;
    }
    let to_void = items.min(stack.count.saturating_sub(keep));
    (to_void > 0).then_some(to_void)
}

/// What one Dropper operation throws: this many of the buffered stack, never more
/// than the buffer holds or `items`, and never the `keep` the module's saved
/// regulation amount reserves.
///
/// `CompiledDropperModule.execute` computes exactly what
/// [`pick_void`] does — `min(getItemsPerTick, count − getRegulationAmount())`,
/// nothing when that is not positive, the filter's test first — and only what
/// happens to the stack differs: it is spawned as an item entity instead of
/// destroyed. `keep` is `Module::regulation_amount()`, zero without the augment.
#[must_use]
pub fn pick_drop(buffer: Option<&Stack>, filter: &Filter, items: u32, keep: u32) -> Option<u32> {
    pick_void(buffer, filter, items, keep)
}

/// What one Placer operation places: **one** item out of the buffer, and nothing
/// at all when the buffer is empty, the filter refuses its item, or the buffer is
/// already down to `keep`.
///
/// `CompiledPlacerModule.execute` is the shortest operation in the mod: it checks
/// `isRegulationOK(router, false)`, `peekBuffer`s **one** item, tests the filter,
/// tries to place it as a block, and only then `extractBuffer`s it. It never calls
/// `getItemsPerTick`, which is the same rule as `StackAugment.getMaxAugments`
/// giving it none: a placer places one block per operation whatever the router's
/// Stack Upgrades or a foreign save's augment count say. The count is therefore
/// not a parameter here — the operation cannot be told to move more.
///
/// `keep` is `Module::regulation_amount()`, which is zero unless a Regulator
/// Augment is installed; `isRegulationOK(router, false)` is `keep < count`, so the
/// buffer's floor is the amount.
#[must_use]
pub fn pick_place(buffer: Option<&Stack>, filter: &Filter, keep: u32) -> Option<u32> {
    let stack = buffer?;
    if stack.count == 0 || !filter.accepts(&stack.item) || stack.count <= keep {
        return None;
    }
    Some(1)
}

/// The harvest tier of a Breaker's saved tool (`BlockUtil.tryBreakBlock`'s
/// `pickaxe.getItem() instanceof TieredItem t ? t.getTier() : Tiers.STONE`).
/// Horizon does not expose an item's tier, so the six vanilla pickaxes are named
/// and everything else — a stick, or an add-on's pickaxe — falls back to stone,
/// which is exactly upstream's branch for an item that is not a `TieredItem`.
#[must_use]
pub fn tool_tier(pickaxe: &str) -> &'static str {
    match pickaxe {
        "minecraft:wooden_pickaxe" => "wooden",
        "minecraft:stone_pickaxe" => "stone",
        "minecraft:iron_pickaxe" => "iron",
        "minecraft:golden_pickaxe" => "gold",
        "minecraft:diamond_pickaxe" => "diamond",
        "minecraft:netherite_pickaxe" => "netherite",
        _ => "stone",
    }
}

/// `CompiledModule.isRegulationOK(router, true)`: with no amount the break is
/// always allowed, and with one it needs an amount **above** the buffer's count.
/// This is the inbound half — the opposite of the placer's `amount < count`, so
/// a buffer already at the amount stops a breaker where it would let a placer go.
#[must_use]
pub fn break_allowed(buffer_count: u32, keep: u32) -> bool {
    keep == 0 || keep > buffer_count
}

/// What one Breaker operation decides, mirroring `BlockUtil.tryBreakBlock`:
///
/// - in `ByBlock` mode the filter tests the **block item**; a refusal means no
///   break at all, and every actual drop is taken. This is the field the artefact
///   calls `getMatchType() == BLOCK`.
/// - in `ByItem` mode the filter tests each **drop**. The block is broken when
///   there were no drops, or when at least one drop passed the filter; the drops
///   that passed are what go into the buffer. The one exception is upstream's
///   explicit guard `!matchByBlock && allDrops.isEmpty() && !filter.isEmpty() &&
///   filter.isWhiteList()`: an empty whitelist with no drops refuses the break.
///   With the default (empty) blacklist and no drops — glass — the break still
///   happens and yields nothing.
///
/// `Some` means "break"; the vector is what may be collected into the buffer.
#[must_use]
pub fn break_decision(
    drops: &[Stack],
    filter: &Filter,
    match_type: MatchType,
    block_item: &str,
) -> Option<Vec<Stack>> {
    match match_type {
        MatchType::ByBlock => filter.accepts(block_item).then(|| drops.to_vec()),
        MatchType::ByItem => {
            if drops.is_empty() {
                return if !filter.is_empty() && filter.whitelist() {
                    None
                } else {
                    Some(Vec::new())
                };
            }
            let passing: Vec<Stack> = drops
                .iter()
                .filter(|drop| filter.accepts(&drop.item))
                .cloned()
                .collect();
            (!passing.is_empty()).then_some(passing)
        }
    }
}

/// Where a Dropper's item is spawned: the middle of the block one step along
/// `face`, moved `0.2` blocks back toward the router. `CompiledDropperModule`
/// builds its `ItemEntity` at `target.pos + 0.5 + 0.2 × target.face`, and the
/// target block's face is the side the router is on — so the item appears just
/// inside the adjacent block, nearest the router, never in the router's own space.
#[must_use]
pub fn drop_point(from: Pos, face: Face) -> [f64; 3] {
    let [x, y, z] = step(from, face);
    let [dx, dy, dz] = face_normal(face);
    [
        f64::from(x) + 0.5 - 0.2 * f64::from(dx),
        f64::from(y) + 0.5 - 0.2 * f64::from(dy),
        f64::from(z) + 0.5 - 0.2 * f64::from(dz),
    ]
}

/// A Flinger Module's saved flight settings (`FlingerSettings`): how fast its
/// item leaves, and the pitch and yaw offsets added to the direction's own aim.
///
/// The field order and ranges are `FlingerModule`'s (`speed` 0–20, `pitch`
/// −90–90, `yaw` −60–60); a module with the default (0, 0, 0) throws exactly as a
/// Dropper does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flinger {
    /// `FlingerSettings.speed`, in blocks per tick.
    pub speed: f32,
    /// `FlingerSettings.pitch`, degrees in −90–90 added to the base pitch.
    pub pitch: f32,
    /// `FlingerSettings.yaw`, degrees in −60–60 added to the base yaw.
    pub yaw: f32,
}

impl Flinger {
    /// `FlingerSettings.DEFAULT`: no push and no offset, a dropper's throw.
    pub const DEFAULT: Flinger = Flinger {
        speed: 0.0,
        pitch: 0.0,
        yaw: 0.0,
    };

    /// The settings as written, before [`Flinger::validated`] is applied.
    #[must_use]
    pub const fn new(speed: f32, pitch: f32, yaw: f32) -> Self {
        Self { speed, pitch, yaw }
    }

    /// Whether every setting is inside its range. `FlingerSettings.validateSetting`
    /// is `val >= min && val <= max && !Float.isInfinite(val)`, so a NaN is out of
    /// range too (both comparisons are false).
    #[must_use]
    pub fn is_valid(self) -> bool {
        in_flinger_range(self.speed, FLINGER_MIN_SPEED, FLINGER_MAX_SPEED)
            && in_flinger_range(self.pitch, FLINGER_MIN_PITCH, FLINGER_MAX_PITCH)
            && in_flinger_range(self.yaw, FLINGER_MIN_YAW, FLINGER_MAX_YAW)
    }

    /// `FlingerSettings.validate`: a triple with **any** field out of range is not
    /// used at all — the module falls back to `DEFAULT` for all three, never
    /// keeping the two that fit.
    #[must_use]
    pub fn validated(self) -> Self {
        if self.is_valid() { self } else { Self::DEFAULT }
    }
}

impl Default for Flinger {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One fling setting inside its range (`FlingerSettings.validateSetting`).
fn in_flinger_range(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max && value.is_finite()
}

/// `yawFromFacing`: the base yaw a direction's quadrant gives, in degrees —
/// east 0 (the +X axis), north 90 (−Z), west 180, south 270. A vertical face has
/// no quadrant of its own and reads 0, as upstream's `default -> 0` does.
#[must_use]
pub fn yaw_of_face(face: Face) -> f32 {
    match face {
        EAST => 0.0,
        NORTH => 90.0,
        WEST => 180.0,
        SOUTH => 270.0,
        _ => 0.0,
    }
}

/// The velocity a flinger gives its item — `CompiledFlingerModule.setupItemVelocity`.
///
/// `face` is the module's absolute direction (the block the item is thrown into),
/// and `router_facing` is the router block's own facing: upstream's base yaw for
/// an **up or down** throw is `yawFromFacing(routerFacing)`, so a tilted vertical
/// fling leans the way its router points, where a horizontal throw's base yaw is
/// the direction's own quadrant. The base pitch is `±90` for up/down and 0
/// otherwise; the settings are added to both, and the unit aim is scaled by the
/// speed. `x = cos(yaw)·cos(pitch)`, `y = sin(pitch)`,
/// `z = −sin(yaw)·cos(pitch)` — east is +X and north is −Z.
#[must_use]
pub fn fling_velocity(
    face: Face,
    router_facing: Facing,
    yaw: f32,
    pitch: f32,
    speed: f32,
) -> [f64; 3] {
    let (base_pitch, base_yaw) = match face {
        UP | DOWN => (
            if face == UP { 90.0 } else { -90.0 },
            yaw_of_face(router_facing.face()),
        ),
        _ => (0.0, yaw_of_face(face)),
    };
    let yaw = f64::from(base_yaw + yaw).to_radians();
    let pitch = f64::from(base_pitch + pitch).to_radians();
    let speed = f64::from(speed);
    [
        yaw.cos() * pitch.cos() * speed,
        pitch.sin() * speed,
        -(yaw.sin() * pitch.cos()) * speed,
    ]
}

/// One dropped item a Vacuum Module can see, as its tick reads it: the stack the
/// item entity holds, and whether it still carries a pickup delay.
///
/// `CompiledVacuumModule.handleItemMode` skips `!item.isAlive()` and
/// `(!fastPickup && item.hasPickUpDelay())`; the Fast Pickup Augment is a later
/// gap, so a delayed item is skipped here too — a fresh drop is not collected in
/// its first half-second.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    pub stack: Stack,
    /// `ItemEntity.hasPickUpDelay`.
    pub delayed: bool,
}

impl Dropped {
    /// A plain dropped stack: an item id and a count, no components.
    #[must_use]
    pub fn of(item: &str, count: u32, delayed: bool) -> Self {
        Self {
            stack: Stack::of(item, count),
            delayed,
        }
    }

    /// The same dropped stack carrying components, as [`Stack::extra`] spells them.
    #[must_use]
    pub fn with_extra(mut self, extra: &str) -> Self {
        self.stack.extra = extra.to_string();
        self
    }
}

/// What one Vacuum operation does: how many items it takes out of each dropped
/// stack (by index, in the order the tick read them) and the buffer afterwards.
///
/// An empty `takes` is no operation at all — the buffer's load is unchanged and
/// nothing in the world moved, which is upstream's `toPickUp < getItemsPerTick`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vacuum {
    /// `(candidate index, items taken)`, in the order they were taken.
    pub takes: Vec<(usize, u32)>,
    /// The buffer after the operation, or `None` when it stays empty.
    pub buffer: Option<Stack>,
}

impl Vacuum {
    /// Whether the operation moved anything (`return toPickUp < getItemsPerTick`).
    #[must_use]
    pub fn worked(&self) -> bool {
        !self.takes.is_empty()
    }

    /// How many items the operation takes out of the candidate at `index`.
    #[must_use]
    pub fn taken(&self, index: usize) -> u32 {
        self.takes
            .iter()
            .find(|(candidate, _)| *candidate == index)
            .map_or(0, |(_, count)| *count)
    }
}

/// The centre of the box a Vacuum Module sweeps
/// (`CompiledVacuumModule.setupTargets`): the router's own block with no
/// direction — "operates in all directions by default" — and `range + 1` blocks
/// along the direction when it has one, so a directed vacuum sweeps in front of
/// the router only and never its own block.
#[must_use]
pub fn vacuum_centre(from: Pos, face: Option<Face>, range: i32) -> Pos {
    let Some(face) = face else {
        return from;
    };
    let [dx, dy, dz] = face_normal(face);
    let offset = range + 1;
    [
        from[0] + dx * offset,
        from[1] + dy * offset,
        from[2] + dz * offset,
    ]
}

/// The box a Vacuum Module sweeps: `new AABB(centre).inflate(range)` — the centre
/// block's own unit cube grown by the reach in every direction, as `entities_in`
/// wants it (`min`/`max` of a box whose bounds are the block's edges).
#[must_use]
pub fn vacuum_box(centre: Pos, range: i32) -> ([f64; 3], [f64; 3]) {
    let reach = f64::from(range.max(0));
    (
        [
            f64::from(centre[0]) - reach,
            f64::from(centre[1]) - reach,
            f64::from(centre[2]) - reach,
        ],
        [
            f64::from(centre[0]) + 1.0 + reach,
            f64::from(centre[1]) + 1.0 + reach,
            f64::from(centre[2]) + 1.0 + reach,
        ],
    )
}

/// The most entity handles one Vacuum operation inspects, however many drops the
/// box holds. The pin resolves a handle by searching the level's dropped-item list
/// (`Host::item_entity`, then `ItemEntityRef::stack`), so one operation costs at
/// most this many searches instead of one per drop in the pile.
pub const VACUUM_CANDIDATE_LIMIT: usize = 64;

/// One Vacuum operation in progress (`CompiledVacuumModule.handleItemMode`), fed
/// the dropped items one candidate at a time.
///
/// Resolving a candidate is the expensive step — the pin's SDK searches the
/// level's dropped-item list for each one — so the operation decides **before** it
/// looks: [`VacuumOp::start`] answers `None` when the buffer can take nothing (it
/// is full, it is at the module's regulation amount, or it holds an item the
/// filter refuses), and the tick then reads no entities at all; and
/// [`VacuumOp::next_candidate`] hands out an index only while the operation still
/// wants one, so a filled pickup budget or a pile past [`VACUUM_CANDIDATE_LIMIT`]
/// ends the loop rather than the list's length.
///
/// Three tests decide a candidate, in upstream's order: it carries no pickup
/// delay, it is **the buffer's own resource** while the buffer holds anything
/// (`ItemStack.isSameItemSameComponents` — so a non-empty buffer collects only its
/// own item and components, not "the first acceptable entity"), and the module's
/// filter accepts it. The room is `min(max_stack, keep) − buffered`, where `keep`
/// is the Regulator Augment's saved amount and zero without it; the budget is
/// `items` items in the whole operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VacuumOp {
    /// The buffer after what the operation has taken so far.
    plan: Vacuum,
    /// Items still to take this operation.
    budget: u32,
    /// Candidates still to inspect (`VACUUM_CANDIDATE_LIMIT`).
    inspect_left: usize,
    /// How many candidate indices have been handed out.
    next: usize,
    /// `Module::regulation_amount`: zero without the Regulator Augment.
    keep: u32,
}

impl VacuumOp {
    /// The operation's start, or `None` when it can take nothing — a full buffer,
    /// a buffer at the module's amount, an item its filter refuses, or an
    /// operation that moves no items. `None` means the tick must not read the
    /// level: there is nothing a resolved drop could change.
    ///
    /// `max_stack` is the **buffer's own** item's `getMaxStackSize` when the
    /// buffer holds something (the only resource it can collect); with an empty
    /// buffer it is not used, and [`VacuumOp::offer`] takes the candidate's.
    #[must_use]
    pub fn start(
        buffer: Option<&Stack>,
        filter: &Filter,
        keep: u32,
        items: u32,
        max_stack: u32,
    ) -> Option<Self> {
        if items == 0 {
            return None;
        }
        let (plan, budget) = match buffer {
            Some(held) => {
                // A non-empty buffer collects only its own resource, so a filter
                // that refuses the buffer's item leaves nothing to collect.
                if !filter.accepts(&held.item) {
                    return None;
                }
                // `isBufferFull()` and `spaceInRouter`: the amount, where a
                // Regulator Augment gives one, is a **cap** on the buffer.
                let cap = max_stack.max(1);
                let room = if keep > 0 { cap.min(keep) } else { cap };
                let budget = items.min(room.saturating_sub(held.count));
                if budget == 0 {
                    return None;
                }
                (
                    Vacuum {
                        takes: Vec::new(),
                        buffer: Some(held.clone()),
                    },
                    budget,
                )
            }
            None => (Vacuum::default(), items),
        };
        Some(Self {
            plan,
            budget,
            inspect_left: VACUUM_CANDIDATE_LIMIT,
            next: 0,
            keep,
        })
    }

    /// Whether the operation still wants another candidate: it has items left to
    /// take and inspections left to spend. A refused candidate does not spend the
    /// item budget, so a box of a thousand drops the filter refuses costs exactly
    /// [`VACUUM_CANDIDATE_LIMIT`] inspections and stops.
    #[must_use]
    pub fn wants(&self) -> bool {
        self.budget > 0 && self.inspect_left > 0
    }

    /// The index of the next candidate to resolve, or `None` when the loop must
    /// stop: the pickup budget is filled, the inspection cap is spent, or there
    /// are no handles left. The caller resolves **only** the index it is handed.
    pub fn next_candidate(&mut self, available: usize) -> Option<usize> {
        if !self.wants() || self.next >= available {
            return None;
        }
        let index = self.next;
        self.next += 1;
        self.inspect_left -= 1;
        Some(index)
    }

    /// Offer the resolved candidate at `index`: how many of it the operation takes
    /// (zero is a refusal), with the buffer updated as if those items were in it.
    /// The caller writes the ground stack and credits the buffer for the answer.
    pub fn offer(
        &mut self,
        index: usize,
        dropped: &Dropped,
        filter: &Filter,
        max_stack: u32,
    ) -> Option<u32> {
        if !self.wants() || dropped.delayed || dropped.stack.count == 0 {
            return None;
        }
        // The buffer's item is the operation's subject: `bufferStack.isEmpty() ||
        // ItemStack.isSameItemSameComponents(stackOnGround, bufferStack)`.
        if let Some(held) = &self.plan.buffer
            && !held.same_resource(&dropped.stack)
        {
            return None;
        }
        if !filter.accepts(&dropped.stack.item) {
            return None;
        }
        let cap = max_stack.max(1);
        let room = if self.keep > 0 {
            cap.min(self.keep)
        } else {
            cap
        };
        let buffered = self.plan.buffer.as_ref().map_or(0, |held| held.count);
        let take = self
            .budget
            .min(room.saturating_sub(buffered))
            .min(dropped.stack.count);
        if take == 0 {
            return None;
        }
        self.budget -= take;
        match &mut self.plan.buffer {
            Some(held) => held.count += take,
            None => {
                self.plan.buffer = Some(Stack {
                    count: take,
                    ..dropped.stack.clone()
                });
            }
        }
        self.plan.takes.push((index, take));
        Some(take)
    }

    /// The operation's plan: what it took and the buffer afterwards.
    #[must_use]
    pub fn finish(self) -> Vacuum {
        self.plan
    }
}

/// The buffer after `sent` items left it — a send, or the items a Void Module
/// destroyed; none when it is emptied.
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
            pick_pull(&slots, &Filter::any(), Some(&buffer), |_| 64, 1, 0),
            Some((1, 1))
        );
        // With a full buffer stack nothing is pulled.
        let full = Stack {
            item: "minecraft:stone".to_string(),
            count: 64,
            extra: String::new(),
        };
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&full), |_| 64, 1, 0),
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
        assert_eq!(pick_pull(&slots, &stone, None, |_| 64, 8, 0), Some((1, 8)));
        // A small max stack caps the take.
        assert_eq!(pick_pull(&slots, &stone, None, |_| 16, 8, 0), Some((1, 8)));
        let one = Stack {
            item: "minecraft:stone".to_string(),
            count: 15,
            extra: String::new(),
        };
        assert_eq!(
            pick_pull(&slots, &stone, Some(&one), |_| 16, 8, 0),
            Some((1, 1))
        );
        // A buffer stack the filter refuses stops the operation.
        let dirt = Stack {
            item: "minecraft:dirt".to_string(),
            count: 1,
            extra: String::new(),
        };
        assert_eq!(pick_pull(&slots, &stone, Some(&dirt), |_| 64, 8, 0), None);
        // No acceptable slot, no operation.
        assert_eq!(
            pick_pull(&slots, &Filter::new(true, Vec::new()), None, |_| 64, 8, 0),
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
            pick_pull(&slots, &Filter::any(), None, |_| 64, 8, 0),
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
            pick_pull(&slots, &Filter::any(), Some(&buffer), |_| 64, 8, 0),
            Some((0, 4))
        );
        // A full named stack pulls nothing, even with plain diamonds beside it.
        let full = Stack {
            count: 64,
            ..buffer.clone()
        };
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&full), |_| 64, 8, 0),
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
    fn the_targeted_sender_reaches_twenty_four_blocks_and_forty_eight_with_augments() {
        // `SenderModule2 implements IRangedModule, ITargetedModule`: the jar's Mk2
        // inserts into a block the player selected (its direction names nothing),
        // so it is targeted like the Puller Mk2 and its base reach is
        // `sender2BaseRange` 24 (`sender2MaxRange` 48 with Range Augments).
        let sender2 = ModuleKind::of(ModuleKind::SENDER2);
        assert_eq!(sender2, ModuleKind::SenderMk2);
        assert!(sender2.is_known(), "this build runs it");
        assert_eq!(sender2.item(), "modularrouters:sender_module_2");
        assert!(
            sender2.is_targeted(),
            "its container is the one the player selected"
        );
        assert!(
            !ModuleKind::SenderMk1.is_targeted(),
            "the Mk1 scans along its direction"
        );
        assert_eq!(SENDER2_BASE_RANGE, 24, "CommonConfig sender2BaseRange");
        assert_eq!(SENDER2_MAX_RANGE, 48, "CommonConfig sender2MaxRange");
        assert_eq!(sender2_range(0), SENDER2_BASE_RANGE);
        assert_eq!(sender2_range(1), 25, "one Range Up augment is one block");
        assert_eq!(sender2_range(-23), 1, "never shorter than one block");
        assert_eq!(sender2_range(30), SENDER2_MAX_RANGE, "capped at 48");
        // `RangeAugments.getMaxAugments`: up = hardMax − base, down = base − 1.
        assert_eq!(sender2.augment_limit(&AugmentKind::RangeUp), 24);
        assert_eq!(sender2.augment_limit(&AugmentKind::RangeDown), 23);
        // The module's own reach, as the tick and the readout ask for it.
        let module = Module::new(ModuleKind::SenderMk2, Relative::None, Filter::any());
        assert!(module.uses_target());
        assert_eq!(module.current_range(), Some(SENDER2_BASE_RANGE));
        let boosted = module.clone().with_augments(Augments::new(vec![
            Augment::new(AugmentKind::RangeUp, 1).expect("one range up"),
        ]));
        assert_eq!(boosted.current_range(), Some(25));
        let capped = module.clone().with_augments(Augments::new(vec![
            Augment::new(AugmentKind::RangeUp, 24).expect("twenty-four range up"),
        ]));
        assert_eq!(capped.current_range(), Some(SENDER2_MAX_RANGE));
        // The reach rule is the one the Mk2 already uses (`isTargetValid`).
        assert!(!in_range([80, 100, 0], [80, 100, -25], 24));
        assert!(in_range([80, 100, 0], [80, 100, -25], 25));
    }

    #[test]
    fn a_sender_tops_the_target_up_to_the_regulation_amount() {
        // `CompiledSenderModule1.execute`: with `getRegulationAmount() > 0` the send
        // is `min(getItemsPerTick, amount − existing)`, where `existing` counts the
        // target's matching items (capped at the amount) and nothing at all moves
        // once the target holds the amount.
        assert_eq!(send_quota(8, 0, 0), 8, "no amount: the module's own count");
        assert_eq!(send_quota(8, 10, 6), 4, "ten wanted, six there: four go");
        assert_eq!(
            send_quota(1, 10, 6),
            1,
            "never more than items per operation"
        );
        assert_eq!(send_quota(8, 3, 0), 3, "never more than the amount");
        assert_eq!(send_quota(8, 10, 10), 0, "the target holds the amount");
        assert_eq!(send_quota(8, 10, 14), 0, "and more than the amount");
        // The amount is the *target's* floor on a sender and the *source's* on a
        // puller, which is what `/modularrouters regulate`'s reply names.
        assert!(ModuleKind::SenderMk1.is_sender() && ModuleKind::SenderMk2.is_sender());
        assert!(!ModuleKind::PullerMk1.is_sender() && !ModuleKind::PullerMk2.is_sender());
        assert!(!ModuleKind::Void.is_sender());
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
        // count. The Placer ships (slice 15), so its refusal is shown in game on the
        // module itself; the other six kinds are gap 1 (carried by id, never run), so
        // for them the rule is the module **id**'s; every other module, including one
        // this port only carries, takes upstream's six.
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
    fn a_regulator_augment_makes_a_pull_leave_its_amount_in_the_source() {
        // `CompiledModule.getRegulationAmount` is the module's saved amount **only
        // when it holds a Regulator Augment**; `findItemToPull` then refuses an
        // operation whose `nToTake` would drop the source's total below it.
        assert_eq!(
            AugmentKind::of("modularrouters:regulator_augment"),
            AugmentKind::Regulator
        );
        assert!(AugmentKind::Regulator.is_known());
        assert_eq!(
            AugmentKind::Regulator.item(),
            "modularrouters:regulator_augment"
        );
        assert_eq!(AugmentKind::Regulator.name(), "regulator");
        assert_eq!(
            AugmentKind::from_name("regulator"),
            Some(AugmentKind::Regulator)
        );

        let without =
            Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_regulation(3);
        assert_eq!(
            without.regulation_amount(),
            0,
            "no Regulator augment: getRegulationAmount is 0 whatever the module saves"
        );
        let mut module = without.clone().with_augments(Augments::new(vec![
            Augment::new(AugmentKind::Regulator, 1).expect("one regulator augment"),
        ]));
        assert_eq!(module.regulation_amount(), 3);
        assert_eq!(module.regulation(), 3);
        module.set_regulation(0);
        assert_eq!(module.regulation_amount(), 0);

        // Twelve stone in one slot, one item per operation, leave three: nine moves
        // and then nothing, so the chest keeps exactly three (the game assertion's
        // 12 -> 9 in the buffer / 3 in the source).
        let mut slots = vec![Slot::new(0, "minecraft:stone", 12)];
        let mut moved = 0;
        while let Some((slot, count)) = pick_pull(&slots, &Filter::any(), None, |_| 64, 1, 3) {
            let held = slots
                .iter_mut()
                .find(|held| held.slot == slot)
                .expect("the slot the pull named");
            held.count -= count;
            moved += count;
        }
        assert_eq!(moved, 9);
        assert_eq!(slots[0].count, 3);

        // The rule is "the total would still be at least N after taking `items`",
        // not "take at most total − N": a source of six with four per operation
        // cannot satisfy `6 − 4 >= 3`, so the whole operation is refused.
        assert_eq!(
            pick_pull(
                &[Slot::new(0, "minecraft:stone", 6)],
                &Filter::any(),
                None,
                |_| 64,
                4,
                3
            ),
            None,
            "six minus four is two, below the three to leave, so nothing is taken"
        );
        assert_eq!(
            pick_pull(
                &[Slot::new(0, "minecraft:stone", 7)],
                &Filter::any(),
                None,
                |_| 64,
                4,
                3
            ),
            Some((0, 4))
        );
        // The count is over the source's slots: two stacks of six are twelve.
        let split = [
            Slot::new(0, "minecraft:stone", 6),
            Slot::new(1, "minecraft:stone", 6),
        ];
        assert_eq!(
            pick_pull(&split, &Filter::any(), None, |_| 64, 1, 3),
            Some((0, 1))
        );
        // With the amount at zero the behaviour is exactly what it was before the
        // augment existed (`keep = 0` never refuses), and the buffered resource is
        // counted the same way.
        assert_eq!(
            pick_pull(&slots, &Filter::any(), None, |_| 64, 1, 0),
            Some((0, 1))
        );
        let buffered = Stack::of("minecraft:stone", 1);
        assert_eq!(
            pick_pull(&slots, &Filter::any(), Some(&buffered), |_| 64, 1, 3),
            None,
            "three left in the source is the floor for the buffered resource too"
        );
    }

    #[test]
    fn a_regulator_augment_fills_one_slot_and_none_on_the_detector_and_the_second_extruder() {
        // `RegulatorAugment.getMaxAugments`: one on every module except the
        // Detector and the Extruder Mk2, which take none (their operation does not
        // read an inventory count).
        for known in [
            ModuleKind::PullerMk1,
            ModuleKind::PullerMk2,
            ModuleKind::SenderMk1,
        ] {
            assert_eq!(
                known.augment_limit(&AugmentKind::Regulator),
                1,
                "{} takes one",
                known.item()
            );
        }
        for id in [
            "modularrouters:detector_module",
            "modularrouters:extruder_module_2",
        ] {
            let kind = ModuleKind::of(id);
            assert_eq!(
                kind.augment_limit(&AugmentKind::Regulator),
                0,
                "{id} takes none"
            );
            let mut module = Module::new(kind, Relative::Front, Filter::any());
            assert!(!module.add_augment(AugmentKind::Regulator), "{id}");
            assert_eq!(module.augments().used(), 0);
        }
        // Only those two: upstream's list does not name the Extruder Mk1, and a
        // module this port only carries is not a class that takes none.
        assert_eq!(
            ModuleKind::of("modularrouters:extruder_module_1")
                .augment_limit(&AugmentKind::Regulator),
            1
        );
        assert_eq!(
            ModuleKind::Registered("example:other_module".to_string())
                .augment_limit(&AugmentKind::Regulator),
            1,
            "upstream tests the classes that take none, so the rest take one"
        );
        let mut module = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any());
        assert!(module.add_augment(AugmentKind::Regulator));
        assert!(
            !module.add_augment(AugmentKind::Regulator),
            "one per module"
        );
        assert_eq!(module.augments().used(), 1);
        assert!(module.remove_augment(&AugmentKind::Regulator));
        assert_eq!(module.regulation_amount(), 0);
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

    #[test]
    fn a_void_module_eats_the_buffer_down_to_its_saved_amount() {
        // `CompiledVoidModule.execute`: `min(getItemsPerTick, count − getRegulationAmount())`,
        // and nothing at all when that is not positive.
        assert_eq!(
            ModuleKind::of("modularrouters:void_module"),
            ModuleKind::Void
        );
        assert!(ModuleKind::Void.is_known());
        assert_eq!(ModuleKind::Void.item(), "modularrouters:void_module");
        assert!(
            !ModuleKind::Void.is_targeted(),
            "it acts on the router's own buffer, not a block"
        );

        let eight = Stack::of("minecraft:stone", 8);
        assert_eq!(pick_void(Some(&eight), &Filter::any(), 1, 0), Some(1));
        assert_eq!(
            pick_void(Some(&eight), &Filter::any(), 64, 0),
            Some(8),
            "never more than the buffer holds"
        );
        assert_eq!(
            pick_void(Some(&eight), &Filter::any(), 64, 3),
            Some(5),
            "a floor of three leaves three"
        );
        assert_eq!(
            pick_void(Some(&eight), &Filter::any(), 1, 3),
            Some(1),
            "the operation's own count still applies above the floor"
        );
        // A buffer at or below the floor is left whole: `toVoid <= 0` is no operation.
        let three = Stack::of("minecraft:stone", 3);
        assert_eq!(pick_void(Some(&three), &Filter::any(), 64, 3), None);
        assert_eq!(pick_void(Some(&three), &Filter::any(), 64, 5), None);
        assert_eq!(pick_void(None, &Filter::any(), 64, 0), None);
        assert_eq!(
            pick_void(Some(&eight), &Filter::any(), 0, 0),
            None,
            "a zero-count operation"
        );
    }

    #[test]
    fn a_void_module_only_eats_what_its_filter_accepts() {
        let stone = Stack::of("minecraft:stone", 8);
        let only_dirt = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        assert_eq!(
            pick_void(Some(&stone), &only_dirt, 64, 0),
            None,
            "a whitelist that does not list stone refuses the whole buffer"
        );
        let not_dirt = Filter::new(false, vec!["minecraft:dirt".to_string()]);
        assert_eq!(pick_void(Some(&stone), &not_dirt, 64, 0), Some(8));
        assert_eq!(
            pick_void(Some(&stone), &Filter::new(true, Vec::new()), 64, 0),
            None,
            "an empty whitelist accepts nothing"
        );
    }

    #[test]
    fn a_dropper_module_throws_the_filtered_buffer_down_to_its_saved_amount() {
        // `CompiledDropperModule.execute`: `min(getItemsPerTick, count −
        // getRegulationAmount())`, and nothing at all when that is not positive — the
        // Void Module's arithmetic with the stack thrown into the world instead of
        // destroyed, so the saved amount is the buffer's floor.
        assert_eq!(
            ModuleKind::of("modularrouters:dropper_module"),
            ModuleKind::Dropper
        );
        assert!(ModuleKind::Dropper.is_known());
        assert_eq!(ModuleKind::Dropper.item(), ModuleKind::DROPPER);
        assert!(
            !ModuleKind::Dropper.is_targeted(),
            "it drops through its direction, not a block the player selected"
        );
        assert!(
            !ModuleKind::Dropper.is_sender(),
            "its amount is the buffer's floor, not the target's"
        );
        let dropper = Module::new(ModuleKind::Dropper, Relative::Front, Filter::any());
        assert_eq!(
            dropper.current_range(),
            None,
            "a dropper is not ranged: it reaches the block beside it"
        );

        let eight = Stack::of("minecraft:stone", 8);
        assert_eq!(pick_drop(Some(&eight), &Filter::any(), 1, 0), Some(1));
        assert_eq!(
            pick_drop(Some(&eight), &Filter::any(), 64, 0),
            Some(8),
            "never more than the buffer holds"
        );
        assert_eq!(
            pick_drop(Some(&eight), &Filter::any(), 64, 3),
            Some(5),
            "a floor of three leaves three in the buffer"
        );
        assert_eq!(
            pick_drop(Some(&eight), &Filter::any(), 1, 3),
            Some(1),
            "the operation's own count still applies above the floor"
        );
        // `nItems <= 0` is no operation: a buffer at or below the floor stays whole.
        let three = Stack::of("minecraft:stone", 3);
        assert_eq!(pick_drop(Some(&three), &Filter::any(), 64, 3), None);
        assert_eq!(pick_drop(Some(&three), &Filter::any(), 64, 5), None);
        assert_eq!(pick_drop(None, &Filter::any(), 64, 0), None);
        assert_eq!(pick_drop(Some(&eight), &Filter::any(), 0, 0), None);
    }

    #[test]
    fn a_dropper_module_only_throws_what_its_filter_accepts() {
        let stone = Stack::of("minecraft:stone", 8);
        let only_dirt = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        assert_eq!(
            pick_drop(Some(&stone), &only_dirt, 64, 0),
            None,
            "a whitelist that does not list stone throws nothing"
        );
        let not_dirt = Filter::new(false, vec!["minecraft:dirt".to_string()]);
        assert_eq!(pick_drop(Some(&stone), &not_dirt, 64, 0), Some(8));
        assert_eq!(
            pick_drop(Some(&stone), &Filter::new(true, Vec::new()), 64, 0),
            None,
            "an empty whitelist accepts nothing"
        );
    }

    #[test]
    fn a_droppers_item_lands_in_the_block_its_direction_names() {
        // `CompiledDropperModule.execute` spawns the item at the **target block's**
        // position plus `0.5 + 0.2 × face`, where the face is that block's side toward
        // the router — so the item lands just inside the adjacent block, nearest the
        // router, and one block (never two) away.
        assert_eq!(step([90, 100, 0], NORTH), [90, 100, -1]);
        let north = drop_point([90, 100, 0], NORTH);
        assert_eq!(north[0], 90.5, "the target block's middle x");
        assert_eq!(north[1], 100.5, "the target block's middle y");
        assert!(
            (north[2] - (-0.3)).abs() < 1e-9,
            "the target block's z from its router-facing side: {}",
            north[2]
        );
        let up = drop_point([90, 100, 0], UP);
        assert_eq!((up[0], up[2]), (90.5, 0.5));
        assert!(
            (up[1] - 101.3).abs() < 1e-9,
            "above the router, not below: {}",
            up[1]
        );
        let down = drop_point([90, 100, 0], DOWN);
        assert!(
            (down[1] - 99.7).abs() < 1e-9,
            "below the router: {}",
            down[1]
        );
        let east = drop_point([90, 100, 0], EAST);
        assert!(
            (east[0] - 91.3).abs() < 1e-9,
            "east of the router: {}",
            east[0]
        );
        assert_eq!(east[2], 0.5);
        // The module's direction is the router's own when it says `front`: a
        // north-facing router throws into the block north of it.
        assert_eq!(
            step(
                [90, 100, 0],
                Relative::Front.resolve(Facing::North).expect("front")
            ),
            [90, 100, -1]
        );
    }

    #[test]
    fn a_flinger_module_is_a_dropper_with_its_own_flight_settings() {
        // `FlingerModule extends DropperModule` and `CompiledFlingerModule
        // extends CompiledDropperModule`: the only thing it changes is the
        // velocity it gives the item it throws, so its operation, filter, count
        // and regulation amount are the dropper's (`pick_drop`).
        assert_eq!(
            ModuleKind::of("modularrouters:flinger_module"),
            ModuleKind::Flinger
        );
        assert!(ModuleKind::Flinger.is_known());
        assert_eq!(ModuleKind::Flinger.item(), ModuleKind::FLINGER);
        assert!(
            !ModuleKind::Flinger.is_targeted(),
            "it throws through its direction, not a block the player selected"
        );
        assert!(
            !ModuleKind::Flinger.is_sender(),
            "its amount is the buffer's floor, not the target's"
        );
        let flinger = Module::new(ModuleKind::Flinger, Relative::Front, Filter::any());
        assert_eq!(
            flinger.current_range(),
            None,
            "a flinger is not ranged: it throws into the block beside it"
        );
        assert_eq!(flinger.flinger(), Flinger::DEFAULT);
        assert_eq!(
            ModuleKind::Flinger.augment_limit(&AugmentKind::Regulator),
            1,
            "a flinger takes the Regulator Augment (upstream's list of the two kinds that take none)"
        );
        assert_eq!(
            ModuleKind::Flinger.augment_limit(&AugmentKind::Stack),
            6,
            "and six Stack Augments, its operation being an item count"
        );

        // The operation itself is the dropper's rule, unchanged.
        let eight = Stack::of("minecraft:stone", 8);
        assert_eq!(pick_drop(Some(&eight), &Filter::any(), 64, 3), Some(5));
        assert_eq!(
            pick_drop(
                Some(&eight),
                &Filter::new(true, vec!["minecraft:dirt".to_string()]),
                64,
                0
            ),
            None
        );

        // `FlingerSettings.validate`: in range keeps every field, out of range
        // (or infinite, or NaN) resets **all three** to the default.
        let asked = Flinger::new(1.0, 0.0, 0.0);
        assert_eq!(asked.validated(), asked);
        assert_eq!(
            Flinger::new(20.0, 90.0, 60.0).validated(),
            Flinger::new(20.0, 90.0, 60.0)
        );
        assert_eq!(Flinger::new(-0.1, 0.0, 0.0).validated(), Flinger::DEFAULT);
        assert_eq!(Flinger::new(20.1, 0.0, 0.0).validated(), Flinger::DEFAULT);
        assert_eq!(Flinger::new(1.0, -90.1, 0.0).validated(), Flinger::DEFAULT);
        assert_eq!(Flinger::new(1.0, 90.1, 0.0).validated(), Flinger::DEFAULT);
        assert_eq!(Flinger::new(1.0, 0.0, -60.1).validated(), Flinger::DEFAULT);
        assert_eq!(Flinger::new(1.0, 0.0, 60.1).validated(), Flinger::DEFAULT);
        assert_eq!(
            Flinger::new(1.0, 0.0, 0.0).validated(),
            Flinger::new(1.0, 0.0, 0.0),
            "one field out of range must not be the only one reset"
        );
        assert_eq!(
            Flinger::new(f32::INFINITY, 0.0, 0.0).validated(),
            Flinger::DEFAULT
        );
        assert_eq!(
            Flinger::new(f32::NAN, 0.0, 0.0).validated(),
            Flinger::DEFAULT
        );

        // The settings a module saves travel with it.
        let configured = flinger.with_flinger(Flinger::new(3.5, -15.0, 45.0));
        assert_eq!(configured.flinger(), Flinger::new(3.5, -15.0, 45.0));
        let mut set = configured;
        set.set_flinger(Flinger::new(2.0, 0.0, 0.0));
        assert_eq!(set.flinger().speed, 2.0);
    }

    #[test]
    fn a_flingers_item_leaves_along_its_aim_not_the_routers() {
        // `CompiledFlingerModule.setupItemVelocity`: base yaw is the module's own
        // absolute direction (`yawFromFacing`: east 0, north 90, west 180, south
        // 270) and base pitch is 0 — except for an UP/DOWN throw, whose base pitch
        // is ±90 and whose base yaw is the **router's** facing, so a tilted
        // vertical fling leans the way the router points.
        let east = fling_velocity(EAST, Facing::North, 0.0, 0.0, 1.0);
        assert!((east[0] - 1.0).abs() < 1e-12, "speed 1 east: {east:?}");
        assert!(east[1].abs() < 1e-12 && east[2].abs() < 1e-12);
        let north = fling_velocity(NORTH, Facing::East, 0.0, 0.0, 1.0);
        assert!(north[0].abs() < 1e-12 && north[1].abs() < 1e-12);
        assert!((north[2] + 1.0).abs() < 1e-12, "north is −Z: {north:?}");
        let west = fling_velocity(WEST, Facing::North, 0.0, 0.0, 1.0);
        assert!((west[0] + 1.0).abs() < 1e-12, "west is −X: {west:?}");
        let south = fling_velocity(SOUTH, Facing::North, 0.0, 0.0, 1.0);
        assert!((south[2] - 1.0).abs() < 1e-12, "south is +Z: {south:?}");

        // The speed scales the whole vector; zero is a dropper's drop.
        let fast = fling_velocity(EAST, Facing::North, 0.0, 0.0, 20.0);
        assert!((fast[0] - 20.0).abs() < 1e-12);
        assert_eq!(fling_velocity(EAST, Facing::North, 0.0, 0.0, 0.0), [0.0; 3]);

        // Pitch tilts the throw: 90 is straight up, −90 straight down.
        let up = fling_velocity(EAST, Facing::North, 0.0, 90.0, 1.0);
        assert!(up[0].abs() < 1e-12 && up[2].abs() < 1e-12);
        assert!((up[1] - 1.0).abs() < 1e-12, "pitch 90 is up: {up:?}");
        let down = fling_velocity(EAST, Facing::North, 0.0, -90.0, 1.0);
        assert!((down[1] + 1.0).abs() < 1e-12, "pitch −90 is down: {down:?}");

        // Yaw swings it: +60 turns 60° toward north (−Z).
        let swung = fling_velocity(EAST, Facing::North, 60.0, 0.0, 1.0);
        assert!((swung[0] - 0.5).abs() < 1e-6, "cos 60 east: {swung:?}");
        assert!(
            (swung[2] + 0.866_025_4).abs() < 1e-6,
            "sin 60 north: {swung:?}"
        );

        // An UP throw's base pitch is +90 (straight up at pitch 0) and its base
        // yaw is the router's facing; the same rig with pitch −30 leans that way
        // (60° above the horizon), so the direction it leans is the router's.
        let straight_up = fling_velocity(UP, Facing::North, 0.0, 0.0, 1.0);
        assert!((straight_up[1] - 1.0).abs() < 1e-12);
        let tilted_north = fling_velocity(UP, Facing::North, 0.0, -30.0, 1.0);
        assert!(
            (tilted_north[1] - 0.866_025_4).abs() < 1e-6,
            "60° above the horizon: {tilted_north:?}"
        );
        assert!(
            (tilted_north[2] + 0.5).abs() < 1e-6,
            "a tilted up fling leans the way its router faces (north): {tilted_north:?}"
        );
        let tilted_east = fling_velocity(UP, Facing::East, 0.0, -30.0, 1.0);
        assert!(
            (tilted_east[0] - 0.5).abs() < 1e-6 && tilted_east[2].abs() < 1e-6,
            "the same fling on an east-facing router leans east: {tilted_east:?}"
        );
        let straight_down = fling_velocity(DOWN, Facing::North, 0.0, 0.0, 1.0);
        assert!((straight_down[1] + 1.0).abs() < 1e-12);

        // The spawn point is the dropper's: the target block's middle, 0.2 back
        // toward the router, so a flinger at (145,100,0) throws from x=146.3.
        let spawn = drop_point([145, 100, 0], EAST);
        assert!((spawn[0] - 146.3).abs() < 1e-9, "{spawn:?}");
        assert!((spawn[1] - 100.5).abs() < 1e-9);
        assert!((spawn[2] - 0.5).abs() < 1e-9);
    }

    /// The dropped items one operation can see: a plain stack, or none.
    fn ground(item: &str, count: u32) -> Dropped {
        Dropped::of(item, count, false)
    }

    /// Run one operation over `dropped` the way the tick does — a candidate is
    /// resolved **only** when the operation hands out its index — and answer the
    /// plan with how many candidates it resolved. Resolving one is a search of the
    /// level's dropped-item list in the pin's SDK, so that count is the
    /// operation's cost.
    fn sweep(
        buffer: Option<&Stack>,
        dropped: &[Dropped],
        filter: &Filter,
        items: u32,
        keep: u32,
        max_stack: u32,
    ) -> (Vacuum, usize) {
        let Some(mut op) = VacuumOp::start(buffer, filter, keep, items, max_stack) else {
            return (Vacuum::default(), 0);
        };
        let mut resolved = 0;
        while let Some(index) = op.next_candidate(dropped.len()) {
            resolved += 1;
            op.offer(index, &dropped[index], filter, max_stack);
        }
        (op.finish(), resolved)
    }

    #[test]
    fn a_vacuum_module_fills_its_buffer_from_the_items_around_it() {
        // `CompiledVacuumModule.handleItemMode`: the buffer's own count is the
        // operation's budget (`getItemsPerTick`), one candidate at a time, and a
        // stack that runs out leaves the world.
        assert_eq!(
            ModuleKind::of("modularrouters:vacuum_module"),
            ModuleKind::Vacuum
        );
        assert!(ModuleKind::Vacuum.is_known());
        assert_eq!(ModuleKind::Vacuum.item(), ModuleKind::VACUUM);
        assert!(
            !ModuleKind::Vacuum.is_targeted(),
            "the player selects no block: it collects what lies in its reach"
        );
        assert!(!ModuleKind::Vacuum.is_sender());

        let eight = ground("minecraft:stone", 8);
        let (plan, _) = sweep(None, std::slice::from_ref(&eight), &Filter::any(), 1, 0, 64);
        assert_eq!(plan.takes, vec![(0, 1)], "one item per operation");
        assert_eq!(plan.buffer, Some(Stack::of("minecraft:stone", 1)));
        assert!(plan.worked());

        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&eight),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert_eq!(plan.takes, vec![(0, 8)], "never more than the stack holds");
        assert_eq!(plan.buffer, Some(Stack::of("minecraft:stone", 8)));

        // The budget is shared over the candidates, in the order they are seen.
        let (plan, _) = sweep(
            None,
            &[ground("minecraft:stone", 2), ground("minecraft:stone", 8)],
            &Filter::any(),
            3,
            0,
            64,
        );
        assert_eq!(plan.takes, vec![(0, 2), (1, 1)]);
        assert_eq!(plan.buffer, Some(Stack::of("minecraft:stone", 3)));

        // A fresh drop carries a pickup delay and is not collectable yet (`!fastPickup
        // && item.hasPickUpDelay()` skips it; the Fast Pickup augment is gap 3).
        let delayed = Dropped::of("minecraft:stone", 8, true);
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&delayed),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert!(plan.takes.is_empty(), "a delayed item is skipped");
        assert_eq!(plan.buffer, None);
        assert!(!plan.worked(), "nothing moved is no operation");

        // `Filter.test` comes after the item test: a whitelist that does not name
        // the dropped item collects nothing, and an empty whitelist accepts none.
        let only_dirt = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        let (plan, _) = sweep(None, std::slice::from_ref(&eight), &only_dirt, 64, 0, 64);
        assert!(plan.takes.is_empty());
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&eight),
            &Filter::new(true, Vec::new()),
            64,
            0,
            64,
        );
        assert!(plan.takes.is_empty(), "an empty whitelist accepts nothing");
        let not_dirt = Filter::new(false, vec!["minecraft:dirt".to_string()]);
        let (plan, _) = sweep(None, std::slice::from_ref(&eight), &not_dirt, 64, 0, 64);
        assert_eq!(plan.takes, vec![(0, 8)], "a blacklist of dirt takes stone");
    }

    #[test]
    fn a_vacuum_module_only_collects_its_buffered_item_and_respects_its_amount() {
        // `bufferStack.isEmpty() || ItemStack.isSameItemSameComponents(stackOnGround,
        // bufferStack)`: a buffer that already holds something decides what the
        // operation is **for** — it is not "the first acceptable entity".
        let stone = Stack::of("minecraft:stone", 4);
        let (plan, _) = sweep(
            Some(&stone),
            &[ground("minecraft:stone", 8), ground("minecraft:dirt", 8)],
            &Filter::any(),
            64,
            0,
            64,
        );
        assert_eq!(
            plan.takes,
            vec![(0, 8)],
            "only the stack the buffer already holds is collected"
        );
        assert_eq!(plan.buffer, Some(Stack::of("minecraft:stone", 12)));

        let dirt = Stack::of("minecraft:dirt", 1);
        let (plan, _) = sweep(
            Some(&dirt),
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert!(plan.takes.is_empty(), "a buffer of dirt takes no stone");
        assert_eq!(plan.buffer, Some(dirt));

        // One item with different components is a different resource, as the pull's
        // `Slot::is` already treats it (`ItemStack.isSameItemSameComponents`).
        let named = Stack {
            item: "minecraft:stone".to_string(),
            count: 1,
            extra: "hex:0a".to_string(),
        };
        let plain = ground("minecraft:stone", 8);
        let (plan, _) = sweep(
            Some(&named),
            std::slice::from_ref(&plain),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert!(plan.takes.is_empty(), "components are part of the identity");

        // `spaceInRouter` with a Regulator Augment's amount: `min(maxStack, amount) −
        // inRouter`. The amount is a **cap** on the buffer, not a floor.
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            3,
            64,
        );
        assert_eq!(plan.takes, vec![(0, 3)], "the amount caps the buffer");
        assert_eq!(plan.buffer, Some(Stack::of("minecraft:stone", 3)));

        let two = Stack::of("minecraft:stone", 2);
        let (plan, _) = sweep(
            Some(&two),
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            3,
            64,
        );
        assert_eq!(
            plan.takes,
            vec![(0, 1)],
            "a buffer at two has room for one of an amount of three"
        );
        let full = Stack::of("minecraft:stone", 3);
        let (plan, resolved) = sweep(
            Some(&full),
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            3,
            64,
        );
        assert!(plan.takes.is_empty(), "a buffer at the amount has no room");
        assert_eq!(resolved, 0, "and then the operation reads no entity");

        // Without the augment the item's own max stack is the cap, and a full buffer
        // is no operation at all (`isBufferFull`).
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            0,
            16,
        );
        assert_eq!(plan.takes, vec![(0, 8)], "an item whose max stack is 16");
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            u32::MAX,
            0,
            4,
        );
        assert_eq!(plan.takes, vec![(0, 4)], "never past the max stack");
        let one = Stack::of("minecraft:stone", 1);
        let (plan, resolved) = sweep(
            Some(&one),
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            64,
            0,
            1,
        );
        assert!(
            plan.takes.is_empty(),
            "a full one-item buffer is no operation"
        );
        assert_eq!(resolved, 0);

        // A zero budget moves nothing, as `getItemsPerTick` can only be 0 on a
        // module whose count a save cut to nothing.
        let (plan, _) = sweep(
            None,
            std::slice::from_ref(&ground("minecraft:stone", 8)),
            &Filter::any(),
            0,
            0,
            64,
        );
        assert!(plan.takes.is_empty());
    }

    #[test]
    fn a_vacuum_with_no_room_in_its_buffer_resolves_no_dropped_item() {
        // Found in review (NikitaMGrimm, 2026-11-23): the vacuum used to resolve and
        // store every nearby drop before asking whether the buffer had room, and the
        // pin's SDK searches the level's dropped-item list for each resolution —
        // so a full buffer next to a crowded pile stalled a server tick for nothing.
        // The operation must decide **before** it reads the world: zero resolutions.
        let eight = ground("minecraft:stone", 8);
        let full = Stack::of("minecraft:stone", 64);
        let (plan, resolved) = sweep(
            Some(&full),
            std::slice::from_ref(&eight),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert_eq!(resolved, 0, "a full buffer resolves no entity at all");
        assert_eq!(plan, Vacuum::default());
        assert!(VacuumOp::start(Some(&full), &Filter::any(), 0, 64, 64).is_none());

        // The same for a buffer at the module's regulation amount (`spaceInRouter`
        // is `min(maxStack, amount) − inRouter`, so the amount of three is full).
        let at_amount = Stack::of("minecraft:stone", 3);
        let (_, resolved) = sweep(
            Some(&at_amount),
            std::slice::from_ref(&eight),
            &Filter::any(),
            64,
            3,
            64,
        );
        assert_eq!(resolved, 0, "a buffer at its amount resolves nothing");

        // A buffer whose own item the filter refuses can collect nothing either: a
        // non-empty buffer takes only its own resource.
        let dirt = Stack::of("minecraft:dirt", 1);
        let only_stone = Filter::new(true, vec!["minecraft:stone".to_string()]);
        let (_, resolved) = sweep(
            Some(&dirt),
            std::slice::from_ref(&eight),
            &only_stone,
            64,
            0,
            64,
        );
        assert_eq!(
            resolved, 0,
            "a refused buffer item leaves nothing to collect"
        );

        // An operation that moves no items reads nothing either.
        assert!(VacuumOp::start(None, &Filter::any(), 0, 0, 64).is_none());

        // The control: one item of room does read, and takes it.
        let nearly = Stack::of("minecraft:stone", 63);
        let (plan, resolved) = sweep(
            Some(&nearly),
            std::slice::from_ref(&eight),
            &Filter::any(),
            64,
            0,
            64,
        );
        assert_eq!(resolved, 1, "one item of room resolves the first candidate");
        assert_eq!(plan.takes, vec![(0, 1)]);
    }

    #[test]
    fn a_vacuum_operation_stops_reading_drops_once_its_budget_is_filled() {
        // The second half of the same review point: inspection is bounded per
        // operation, and the loop stops at the pickup budget instead of running to
        // the end of the list.
        let eight = ground("minecraft:stone", 8);
        let (plan, resolved) = sweep(
            None,
            &[eight.clone(), eight.clone(), eight.clone()],
            &Filter::any(),
            1,
            0,
            64,
        );
        assert_eq!(
            resolved, 1,
            "a budget of one stops after the first successful take"
        );
        assert_eq!(plan.takes, vec![(0, 1)]);

        // A budget of eight is filled by the first stack of eight, so the rest of
        // the pile is never resolved.
        let (plan, resolved) = sweep(
            None,
            &[eight.clone(), eight.clone()],
            &Filter::any(),
            8,
            0,
            64,
        );
        assert_eq!(resolved, 1);
        assert_eq!(plan.takes, vec![(0, 8)]);

        // Refused candidates spend inspections, not the item budget, so the cap —
        // not the pile — bounds the operation.
        let delayed: Vec<Dropped> = (0..200)
            .map(|_| Dropped::of("minecraft:stone", 8, true))
            .collect();
        let (plan, resolved) = sweep(None, &delayed, &Filter::any(), 64, 0, 64);
        assert_eq!(
            resolved, VACUUM_CANDIDATE_LIMIT,
            "one operation inspects at most its cap, however many drops lie there"
        );
        assert!(plan.takes.is_empty());
    }

    #[test]
    fn a_vacuum_module_reaches_six_blocks_all_around_and_one_block_further_with_a_direction() {
        // `CompiledVacuumModule.setupTargets`: `offset = dir == NONE ? 0 :
        // getRange() + 1`, and the entity box is `new AABB(centre).inflate(range)`.
        // So NONE centres the box on the router's own block — "operates in all
        // directions by default" — and a direction centres it one block past the
        // reach, leaving the router out of the box.
        let vacuum = Module::new(ModuleKind::Vacuum, Relative::None, Filter::any());
        assert_eq!(vacuum.current_range(), Some(6), "vacuumBaseRange 6");
        assert_eq!(ModuleKind::Vacuum.base_range(), Some(VACUUM_BASE_RANGE));
        assert_eq!(ModuleKind::Vacuum.max_range(), Some(VACUUM_MAX_RANGE));
        assert_eq!(vacuum_range(0), 6);
        assert_eq!(vacuum_range(6), 12, "six Range Up augments reach 12");
        assert_eq!(vacuum_range(20), 12, "the hard maximum holds");
        assert_eq!(vacuum_range(-9), 1, "never less than one");
        assert_eq!(
            ModuleKind::Vacuum.augment_limit(&AugmentKind::RangeUp),
            6,
            "hardMax − base"
        );
        assert_eq!(
            ModuleKind::Vacuum.augment_limit(&AugmentKind::RangeDown),
            5,
            "base − 1"
        );

        assert_eq!(vacuum_centre([10, 100, 0], None, 6), [10, 100, 0]);
        assert_eq!(
            vacuum_centre([10, 100, 0], Relative::Front.resolve(Facing::North), 6),
            [10, 100, -7],
            "a direction centres the box range + 1 blocks out"
        );
        let (min, max) = vacuum_box([10, 100, 0], 6);
        assert_eq!(min, [4.0, 94.0, -6.0]);
        assert_eq!(max, [17.0, 107.0, 7.0]);
        // The reach is the box's face: a dropped item's block is inside while its box
        // meets the box, and one block further out is not (proven in game by phase U).
        let (min, max) = vacuum_box([10, 100, 0], 1);
        assert_eq!(min, [9.0, 99.0, -1.0]);
        assert_eq!(max, [12.0, 102.0, 2.0]);
    }

    #[test]
    fn a_placer_module_places_one_item_at_a_time_down_to_its_saved_amount() {
        // `CompiledPlacerModule.execute`: `isRegulationOK(router, false)` first, then
        // `peekBuffer(1)` and the filter's test, and `extractBuffer(1)` only after the
        // block went in. `getItemsPerTick` is never read, so one item moves whatever
        // the router's Stack Upgrades say.
        assert_eq!(
            ModuleKind::of("modularrouters:placer_module"),
            ModuleKind::Placer
        );
        assert!(ModuleKind::Placer.is_known());
        assert_eq!(ModuleKind::Placer.item(), ModuleKind::PLACER);
        assert!(
            !ModuleKind::Placer.is_targeted(),
            "it places beside the router through its direction, not into a selected block"
        );
        assert!(
            !ModuleKind::Placer.is_sender(),
            "its amount is the buffer's floor, not a target's"
        );
        let placer = Module::new(ModuleKind::Placer, Relative::Front, Filter::any());
        assert_eq!(
            placer.current_range(),
            None,
            "a placer is not ranged: it places one block beside the router"
        );

        let eight = Stack::of("minecraft:stone", 8);
        assert_eq!(pick_place(Some(&eight), &Filter::any(), 0), Some(1));
        assert_eq!(
            pick_place(Some(&Stack::of("minecraft:stone", 64)), &Filter::any(), 0),
            Some(1),
            "one item per operation however many the buffer holds"
        );
        assert_eq!(
            pick_place(Some(&eight), &Filter::any(), 3),
            Some(1),
            "a floor of three still places one while the buffer is above it"
        );
        assert_eq!(
            pick_place(Some(&Stack::of("minecraft:stone", 1)), &Filter::any(), 0),
            Some(1),
            "the last item goes too"
        );
        // `isRegulationOK(router, false)` is `amount < count`: a buffer at or below the
        // floor is no operation at all.
        let three = Stack::of("minecraft:stone", 3);
        assert_eq!(pick_place(Some(&three), &Filter::any(), 3), None);
        assert_eq!(pick_place(Some(&three), &Filter::any(), 8), None);
        assert_eq!(pick_place(None, &Filter::any(), 0), None);
    }

    #[test]
    fn a_placer_module_places_only_what_its_filter_accepts() {
        let stone = Stack::of("minecraft:stone", 8);
        let only_dirt = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        assert_eq!(
            pick_place(Some(&stone), &only_dirt, 0),
            None,
            "a whitelist that does not list stone places nothing"
        );
        let not_dirt = Filter::new(false, vec!["minecraft:dirt".to_string()]);
        assert_eq!(pick_place(Some(&stone), &not_dirt, 0), Some(1));
        assert_eq!(
            pick_place(Some(&stone), &Filter::new(true, Vec::new()), 0),
            None,
            "an empty whitelist accepts nothing"
        );
    }

    #[test]
    fn a_placer_takes_no_stack_augment_and_one_regulator_augment() {
        // `StackAugment.getMaxAugments` names the Placer; `RegulatorAugment` does not.
        // Until this slice the placer was a carried id (gap 1), so the refusal was
        // unit-tested only (`blocked(module-kind)`); the kind exists now, so the same
        // rule is what the in-game leg shows.
        let placer = Module::new(ModuleKind::Placer, Relative::Front, Filter::any());
        assert_eq!(placer.augment_limit(&AugmentKind::Stack), 0);
        assert_eq!(placer.augment_limit(&AugmentKind::Regulator), 1);

        let mut module = placer.clone();
        assert!(!module.add_augment(AugmentKind::Stack));
        assert_eq!(module.augments().used(), 0);
        assert!(module.add_augment(AugmentKind::Regulator));
        assert_eq!(module.augments().used(), 1);
        assert_eq!(
            module.regulation_amount(),
            0,
            "the saved amount starts at zero"
        );
        module.set_regulation(3);
        assert_eq!(module.regulation_amount(), 3);

        // The router's own count is not the placer's either: `getItemsPerTick` is not
        // read by this module, so the operation stays one item.
        assert_eq!(placer.items_per_operation(64), 64);
        assert_eq!(
            pick_place(Some(&Stack::of("minecraft:stone", 64)), &Filter::any(), 0),
            Some(1)
        );
    }

    #[test]
    fn a_placers_block_is_the_one_its_direction_names() {
        // `CompiledModule.setupTargets`: a directional module's target is the block
        // beside the router in its absolute facing, and a `NONE` direction has **no**
        // target at all — the module then has nothing to place into.
        let placer = Module::new(ModuleKind::Placer, Relative::Front, Filter::any());
        assert_eq!(
            placer.direction.resolve(Facing::North),
            Some(NORTH),
            "dir=front on a north-facing router is north"
        );
        assert_eq!(
            Module::new(ModuleKind::Placer, Relative::Up, Filter::any())
                .direction
                .resolve(Facing::North),
            Some(UP)
        );
        assert_eq!(
            Module::new(ModuleKind::Placer, Relative::None, Filter::any())
                .direction
                .resolve(Facing::North),
            None,
            "dir=none has no target, so nothing is placed"
        );
        assert_eq!(step([201, 100, 0], NORTH), [201, 100, -1]);
        assert_eq!(step([201, 100, 0], UP), [201, 101, 0]);
    }

    #[test]
    fn a_breaker_breaks_only_when_a_drop_passes_the_filter() {
        // `BlockUtil.tryBreakBlock`: `allDrops` is prepared with the pickaxe, then
        // partitioned by the filter; the block is destroyed only when `drops.empty`
        // or some drop passed the test. In ITEM mode the filter tests the drops.
        assert_eq!(
            ModuleKind::of("modularrouters:breaker_module"),
            ModuleKind::Breaker
        );
        assert!(ModuleKind::Breaker.is_known());
        assert!(!ModuleKind::Breaker.is_targeted());
        assert!(!ModuleKind::Breaker.is_sender());
        assert_eq!(ModuleKind::Breaker.item(), ModuleKind::BREAKER);
        assert_eq!(
            Module::new(ModuleKind::Breaker, Relative::Front, Filter::any()).current_range(),
            None,
            "a breaker is not ranged"
        );

        let cobble = vec![Stack::of("minecraft:cobblestone", 1)];
        assert_eq!(
            break_decision(
                &cobble,
                &Filter::any(),
                MatchType::ByItem,
                "minecraft:stone"
            ),
            Some(cobble.clone()),
            "the jar's default blacklist takes stone's cobble drop"
        );
        let only_dirt = Filter::new(true, vec!["minecraft:dirt".to_string()]);
        assert_eq!(
            break_decision(&cobble, &only_dirt, MatchType::ByItem, "minecraft:stone"),
            None,
            "a whitelist that does not list the drop refuses the break"
        );
        let not_cobble = Filter::new(false, vec!["minecraft:cobblestone".to_string()]);
        assert_eq!(
            break_decision(&cobble, &not_cobble, MatchType::ByItem, "minecraft:stone"),
            None,
            "a blacklist naming the drop refuses the break"
        );
        let only_cobble = Filter::new(true, vec!["minecraft:cobblestone".to_string()]);
        assert_eq!(
            break_decision(&cobble, &only_cobble, MatchType::ByItem, "minecraft:stone"),
            Some(cobble.clone()),
            "an ITEM-mode whitelist naming the drop breaks and takes it"
        );
    }

    #[test]
    fn a_breaker_with_no_drops_breaks_on_a_blacklist_but_not_a_whitelist() {
        // `!matchByBlock && allDrops.isEmpty() && !filter.isEmpty() &&
        // filter.isWhiteList()` is the one explicit refusal in `tryBreakBlock`;
        // otherwise `allDrops.isEmpty()` alone lets the break through. That is the
        // artefact's "glass is broken but yields nothing" case.
        let none: Vec<Stack> = Vec::new();
        assert_eq!(
            break_decision(&none, &Filter::any(), MatchType::ByItem, "minecraft:glass"),
            Some(Vec::new()),
            "no drops + the empty blacklist: break, nothing into the buffer"
        );
        assert_eq!(
            break_decision(
                &none,
                &Filter::new(true, vec!["minecraft:dirt".to_string()]),
                MatchType::ByItem,
                "minecraft:glass"
            ),
            None,
            "no drops + a non-empty whitelist: no break at all"
        );
    }

    #[test]
    fn a_breakers_pickaxe_tier_falls_back_to_stone_for_non_tools() {
        // `BlockUtil.tryBreakBlock`: `pickaxe.getItem() instanceof TieredItem t ?
        // t.getTier() : Tiers.STONE`. A stick, or a mod's pickaxe whose tier this
        // port cannot read, is therefore a stone-tier tool — the artefact's own
        // fallback, not an invented one.
        assert_eq!(tool_tier("minecraft:wooden_pickaxe"), "wooden");
        assert_eq!(tool_tier("minecraft:stone_pickaxe"), "stone");
        assert_eq!(tool_tier("minecraft:iron_pickaxe"), "iron");
        assert_eq!(tool_tier("minecraft:golden_pickaxe"), "gold");
        assert_eq!(tool_tier("minecraft:diamond_pickaxe"), "diamond");
        assert_eq!(tool_tier("minecraft:netherite_pickaxe"), "netherite");
        assert_eq!(tool_tier("minecraft:stick"), "stone");
        assert_eq!(tool_tier("example:mod_pickaxe"), "stone");
    }

    #[test]
    fn a_block_match_tests_the_block_item_not_the_drops() {
        assert_eq!(MatchType::from_name("item"), Some(MatchType::ByItem));
        assert_eq!(MatchType::from_name("block"), Some(MatchType::ByBlock));
        assert_eq!(MatchType::from_name("stone"), None);
        assert_eq!(MatchType::ByItem.name(), "item");
        assert_eq!(MatchType::ByBlock.name(), "block");

        let cobble = vec![Stack::of("minecraft:cobblestone", 1)];
        let only_stone = Filter::new(true, vec!["minecraft:stone".to_string()]);
        assert_eq!(
            break_decision(&cobble, &only_stone, MatchType::ByBlock, "minecraft:stone"),
            Some(cobble.clone()),
            "BLOCK mode tests the block item, which is stone"
        );
        let only_cobble = Filter::new(true, vec!["minecraft:cobblestone".to_string()]);
        assert_eq!(
            break_decision(&cobble, &only_cobble, MatchType::ByBlock, "minecraft:stone"),
            None,
            "BLOCK mode refuses a drop-name filter on the same stone"
        );
        assert_eq!(
            break_decision(&cobble, &only_cobble, MatchType::ByItem, "minecraft:stone"),
            Some(cobble.clone()),
            "ITEM mode is the opposite of BLOCK on the same stone"
        );
    }

    #[test]
    fn a_breakers_inbound_floor_requires_amount_above_the_buffer() {
        // `isRegulationOK(router, true)` is `amount > count` (the opposite of the
        // placer's `amount < count`): a buffer already at the amount breaks nothing.
        assert!(break_allowed(0, 0));
        assert!(
            break_allowed(3, 4),
            "an amount above the buffer permits a break"
        );
        assert!(
            !break_allowed(4, 4),
            "a buffer at the amount breaks nothing"
        );
        assert!(
            !break_allowed(8, 4),
            "a buffer above the amount breaks nothing"
        );

        let breaker = Module::new(ModuleKind::Breaker, Relative::Front, Filter::any());
        assert_eq!(
            breaker.breaker().pickaxe,
            "minecraft:stone_pickaxe",
            "IPickaxeUser.DEFAULT_PICK"
        );
        assert_eq!(breaker.breaker().match_type, MatchType::ByItem);
        assert_eq!(breaker.augment_limit(&AugmentKind::Stack), 0);
        assert_eq!(breaker.augment_limit(&AugmentKind::Regulator), 1);
        assert_eq!(
            break_decision(
                &[],
                &Filter::any(),
                breaker.breaker().match_type,
                "minecraft:glass"
            ),
            Some(Vec::new())
        );
    }
}
