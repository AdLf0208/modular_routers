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
/// and — for a targeted module — the block the player selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub kind: ModuleKind,
    pub direction: Relative,
    pub filter: Filter,
    /// The selected block of a targeted module (`ModuleTarget`), `None` until the
    /// player sets one — an unselected targeted module does nothing.
    pub target: Option<Target>,
}

impl Module {
    pub fn new(kind: ModuleKind, direction: Relative, filter: Filter) -> Self {
        Module {
            kind,
            direction,
            filter,
            target: None,
        }
    }

    /// The same module with `target` selected on it.
    #[must_use]
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
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
}
