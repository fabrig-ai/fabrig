//! Store requests and terminals (protocol/store/README.md, section 3 and protocol/store/loads.md and protocol/store/transcripts.md and protocol/store/secrets.md and protocol/store/opening.md).
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{self as core, RestoreRange, reads};
use jig_domain_notes as notes;
use skein_io::file;
use skein_io::kernel::Fd;
use skein_lib::{Queue, Secret, Token};

/// What the engine's protocol layer asks of the component. Each request but
/// `Commit` names its asker's token; each has one terminal event.
#[derive(Debug)]
pub enum Request {
    /// Take a serving checkpoint at its ordered cut (protocol/store/checkpoints.md, 3).
    Checkpoint { owner: Token, draw: [u8; 16] },
    /// Hold a checkpoint's directory lock until Close (checkpoints.md, 6).
    HoldCheckpoint { owner: Token, root: Root, name: crate::CheckpointName },
    /// Remove a checkpoint under its lock, without opening the deployment.
    RemoveCheckpoint { owner: Token, root: Root, name: crate::CheckpointName },
    /// Rehearse a stopped deployment over a caller-owned scratch root (opening.md 5).
    Rehearse { owner: Token, directory: Root, scratch: Root, installation: [u8; 16] },
    /// Name the previous generation again, before the active one's first own commit.
    Back { owner: Token, root: Root },
    /// End a rehearsal and empty its scratch root, close a checkpoint-only open,
    /// or release this owner's checkpoint reader lock.
    Close { owner: Token },
    /// Open both stores in the deployment's directory (protocol/store/opening.md); the
    /// component's own rows and the core's header are checked before
    /// `Opened`. In skein-kv's memory mode, the worlds' component needs none.
    /// `installation` is the identifier the shell drew for this start
    /// (configuration.md 7): an empty store keeps it in its first commit; a
    /// store that holds one answers its own instead.
    Open { owner: Token, root: Root, installation: [u8; 16], purpose: Purpose },
    /// Activate the copy opened pending, once every earlier commit is durable
    /// (protocol/store/opening.md 2, step 7). Once, after `Opened`; terminal `Switched` or `OpenRefused`.
    Switch { owner: Token },
    /// The owner's answer to one page handed out at open (protocol/store/jobs.md, 3).
    Job { owner: Token, answer: Result<crate::JobAnswer, crate::JobRefusal> },
    /// Commit `number` (protocol/store/README.md 5): its writes, in order, one skein-kv
    /// commit with the same number. Terminal: `Committed`, `Failed` or
    /// `FrameTooLarge`; a refusal stops the engine.
    Commit { number: u64, writes: Box<[Write]> },
    /// A page of one restart step's ranges (protocol/store/loads.md 2), from `after`.
    Restore { owner: Token, range: Restore, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A page of a child's range (protocol/store/loads.md 1), from `after`.
    Load { owner: Token, range: Load, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A party's read (protocol/store/loads.md 3): its scope, position, direction and bounds.
    Read { owner: Token, range: reads::Range },
    /// One record by key.
    Get { owner: Token, key: Get },
    /// A transcript's turns (protocol/store/transcripts.md 3).
    Transcript { owner: Token, read: TranscriptRead, rows: u32, bytes: u64 },
    /// The secrets' store (protocol/store/secrets.md).
    Secrets { owner: Token, request: Secrets },
    /// A written site's answer (protocol/store/opening.md 3), with its machine handed back.
    Site { owner: Token, patcher: Box<skein_codec::patch::Patcher>, result: Result<(), skein_codec::patch::Halt> },
}

/// Whether open may upgrade or make a deployment (protocol/store/checkpoints.md, 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Serve,
    Checkpoint,
}

/// Why a stopped deployment cannot be checkpointed by this build (checkpoints.md, 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unready {
    Empty,
    Upgrade,
}

/// One write of a commit (protocol/store/README.md 5).
#[derive(Debug)]
pub enum Write {
    /// The start's component-owned wires row, from `Store::wires` (protocol/store/records.md 3).
    Wires,
    /// The worker's last accepted opening (protocol/store/records.md 3).
    Opening(WorkerOpening),
    /// Copy the bytes the map holds under `from` to `to` in this commit (protocol/store/README.md 5).
    Copy { from: core::Key, to: core::Key },
    /// jig's save: a put of its encoded key and record; a turn's save
    /// becomes a frame in its task's transcript and puts of its places.
    Save(core::Record),
    /// jig's erase.
    Erase(core::Key),
    /// A task's transcript dropped whole (protocol/store/transcripts.md 5): its places erased,
    /// its file removed before the commit is submitted.
    DropTranscript { task: u64 },
    /// Drop a task history log whole (jig/docs/design/protocol/store/logs.md, 3 and 9).
    DropHistory { task: u64 },
    /// The application's put: its key's bytes (without the tag) and its value's.
    Put { key: Box<[u8]>, value: Box<[u8]> },
    /// The application's erase.
    Remove { key: Box<[u8]> },
}

/// What a restart step restores.
#[derive(Clone, Debug)]
pub enum Restore {
    /// jig's ranges, in order: `RestoreRange::LIVE` for `LoadCore`, the
    /// census for `LoadInbox`. Rows are handed up typed; the census's are
    /// the ended tasks its index rows name.
    Jig(Box<[RestoreRange]>),
    /// A connector's live intervals, relative to the application's tag.
    Application(Box<[skein_kv::Range]>),
}

/// A child's range to load.
#[derive(Clone, Debug)]
pub enum Load {
    /// The notes child's entry or a scope's lines.
    Notes(notes::Range),
    /// The application's intervals, relative to its tag.
    Application(Box<[skein_kv::Range]>),
}

/// A record to get.
#[derive(Clone, Debug)]
pub enum Get {
    Jig(core::Key),
    /// The application's key, relative to its tag.
    Application(Box<[u8]>),
}

/// A transcript read (protocol/store/transcripts.md 3). Offsets are the component's, given
/// and taken back; none reads past what is committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TranscriptRead {
    /// A resuming run's current window, from the turn the core names as its
    /// start; a window that starts at another turn is `Unread`.
    Window { task: u64, attempt: u64, turn: u32, after: Option<u64> },
    /// One turn, walked to from its attempt's first turn.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// A page of a task's turns from either end; a fresh run's tail is
    /// `backwards: true, after: None`.
    Page { task: u64, backwards: bool, after: Option<u64> },
}

/// Whose secret (protocol/store/secrets.md).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SecretOwner {
    /// A sign-in's digest, by the sign-in's number.
    SignIn,
    /// A service's credential's digest, by the service's number.
    Service,
    /// The application's own record, under its tag, by its number.
    Application,
    /// A signed-in source's record or an LLM key's value, by the source's number.
    Source,
    /// The deployment's 32-byte public identifiers' key, at number 0 (protocol/store/secrets.md).
    PublicNames,
}

/// The first number of the application's range that jig numbers itself,
/// for the values clients' requests carry (protocol/store/secrets.md); the application
/// numbers its own values below it.
pub const CARRIED: u64 = 1 << 63;

/// The first number of values held in memory for this run alone (protocol/store/secrets.md).
pub const HELD: u64 = CARRIED + (1 << 62);

/// What a secrets row holds, as its owner gives it and gets it back (protocol/store/secrets.md).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SecretValue {
    /// A credential digest; a service's expiry is `u64::MAX`.
    Digest {
        expiry: u64,
        digest: [u8; 32],
    },
    /// A signed-in source's `SavedToken`, in skein-oauth's own form.
    Record(Secret),
    Key(Secret),
    Carried {
        kind: CarriedKind,
        value: Secret,
    },
    /// The application's own family's bytes, below CARRIED.
    Application(Secret),
    /// A value held for this run alone, never written.
    Held(Secret),
}
/// What a request's carried value is (protocol/store/secrets.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CarriedKind {
    Key,
    Value,
    Moved,
}

/// A request to the secrets' store (protocol/store/secrets.md).
#[derive(Debug)]
pub enum Secrets {
    Keep {
        owner: SecretOwner,
        number: u64,
        value: SecretValue,
    },
    /// Keep `value` in place of the value whose digest is `replaces`, none
    /// when nothing is held: `Kept` once durable, or `Stale`, nothing
    /// written, when another value is held, durable or on its way.
    Replace {
        owner: SecretOwner,
        number: u64,
        value: SecretValue,
        replaces: Option<[u8; 32]>,
    },
    /// Atomically move a carried value into its owner's row, conditional on
    /// the target's digest as in `Replace`. Missing carried values are stale.
    Adopt {
        carried: u64,
        owner: SecretOwner,
        number: u64,
        replaces: Option<[u8; 32]>,
    },
    /// Adopt a decoded value, conditional on both the carried file's digest
    /// and the destination's digest. Both rows change in one commit (protocol/store/secrets.md).
    AdoptValue {
        carried: u64,
        holds: [u8; 32],
        owner: SecretOwner,
        number: u64,
        value: SecretValue,
        replaces: Option<[u8; 32]>,
    },
    /// Hold in memory, never written, under the next number from HELD (protocol/store/secrets.md).
    /// Carried at once, nothing committed; Stale at the held-values bound.
    Hold {
        value: Secret,
    },
    /// Keep a value a client's request carried, under the next number of
    /// the application's carried range, from [`CARRIED`] up (protocol/store/secrets.md):
    /// `Carried` once durable, the next number moved in the same commit;
    /// `Stale`, nothing written, when that number already holds a value.
    Carry {
        kind: CarriedKind,
        value: Secret,
    },
    Erase {
        owner: SecretOwner,
        number: u64,
    },
    Get {
        owner: SecretOwner,
        number: u64,
    },
    Load {
        owner: SecretOwner,
        after: Option<u64>,
    },
}

#[derive(Debug)]
pub struct SecretRow {
    pub owner: SecretOwner,
    pub number: u64,
    pub value: SecretValue,
}

/// Where a page stopped: the interval of the request's list, and the last
/// key it returned (protocol/store/loads.md 1: the next page starts just past it).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cursor {
    pub interval: u32,
    pub after: Box<[u8]>,
}

/// One restored or loaded row.
#[derive(Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::large_enum_variant,
        reason = "bounded jig records travel by value; Row queues price their full size"
    )
)]
pub enum Row {
    /// jig's record, decoded.
    Jig(core::Record),
    /// The application's row, its key relative to its tag.
    Application { key: Box<[u8]>, value: Box<[u8]> },
}

/// Whose stored version or layout disagrees with this build (protocol/store/records.md 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Settings,
    Component,
    Core,
    Tasks,
    People,
    Notes,
    Accounts,
    Secrets,
    SkeinOauth,
    SmithTranscript,
    Application,
}

/// What the check before the switch found (protocol/store/opening.md 2, step 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Check {
    /// A row of the restart's ranges that does not load within the deployment's limits.
    Load { key: Box<[u8]> },
    /// A file in the copy's secrets' store that is not private.
    Private { name: Box<[u8]> },
}
impl Check {
    /// Its refusal's words, in pieces with the deployment's directory between
    /// them, as `OpenRefusal::words` gives every refusal's (protocol/store/opening.md 1).
    fn words(&self) -> &'static [&'static str] {
        match self {
            Self::Load { .. } => &[
                "The check before the switch found a record the restart could not load within the deployment's limits in the copy of the store this start patched in ",
                ", and the generation it patched from stays active, unchanged. Every start of this build checks a fresh copy the same way: start the build before it again, which opens that generation as it was.",
            ],
            Self::Private { .. } => &[
                "The check before the switch found a file in the secrets' store of the copy this start patched in ",
                " that is not private to the service's user, and the generation it patched from stays active, unchanged. A start makes its copy's files private, so another process or user changed one: make sure nothing else writes in ",
                ", and start again.",
            ],
        }
    }
}

/// Why a store refuses to open, saying which (protocol/store/records.md, 3, protocol/store/README.md, 5, protocol/store/opening.md).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OpenRefusal {
    /// This open would make or upgrade a deployment (checkpoints.md, 5).
    Unready { why: Unready },
    /// A held layout cannot be read by this binary (protocol/store/layouts.md, 5.3).
    Layout { family: Box<str>, value_type: u16, id: u16, values: Option<u64>, why: Unplanned },
    /// The format row names another version of `family`.
    Format { family: Family, found: u32, wanted: u32 },
    /// The format row names another deployment than the header.
    Deployment,
    /// skein-kv's recovered last commit is not the header's.
    Numbers { store: u64, header: u64 },
    /// A file failed, or a store did not recover.
    Failed,
    /// A store from an older build is left untouched (protocol/store/README.md, 2, and protocol/store/opening.md, 1).
    OlderBuild { why: Older },
    /// The deployment's generations refuse (protocol/store/opening.md 1).
    Generations { why: Generations },
    /// A level past the binary's (5.3); `found: None` for a component
    /// version past the binary's, its levels unread.
    Ahead { owner: crate::Owner, found: Option<u32>, wanted: Option<u32> },
    /// The binary's owner sets do not check (protocol/store/opening.md, 3).
    Set { owner: crate::Owner },
    /// A job refused its page or answered outside its bounds (protocol/store/jobs.md, 3).
    Job { owner: crate::Owner, job: Box<str>, key: Option<Box<[u8]>>, why: Box<str> },
    /// The check before the switch failed (10.2, step 5); the generation
    /// patched from stays active, unchanged.
    Check { why: Check },
}
/// Why a layout cannot be planned (protocol/store/opening.md, 1.8).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unplanned {
    Underived(skein_codec::layout::Underived),
    Text,
    Undeclared,
    Retired,
}

/// Join the bridge into a static piece, keeping deployment slots unchanged.
macro_rules! bridge_words {
    ($before:expr, $after:expr) => {{
        #[expect(clippy::indexing_slicing, reason = "const concatenation indexes within each measured piece")]
        const BYTES: [u8; $before.len() + crate::FLOOR_BRIDGE.len() + $after.len()] = {
            let mut bytes = [0; $before.len() + crate::FLOOR_BRIDGE.len() + $after.len()];
            let parts = [$before.as_bytes(), crate::FLOOR_BRIDGE.as_bytes(), $after.as_bytes()];
            let mut part = 0_usize;
            let mut at = 0_usize;
            while part < parts.len() {
                let mut i = 0_usize;
                while i < parts[part].len() {
                    bytes[at] = parts[part][i];
                    at = at.checked_add(1).expect("measured text");
                    i = i.checked_add(1).expect("measured piece");
                }
                part = part.checked_add(1).expect("three pieces");
            }
            bytes
        };
        #[expect(clippy::disallowed_methods, reason = "const concatenation of UTF-8 literals and FLOOR_BRIDGE")]
        const TEXT: &str = match ::core::str::from_utf8(&BYTES) {
            Ok(text) => text,
            Err(_) => panic!("UTF-8 pieces"),
        };
        TEXT
    }};
}

/// A refusal whose fix moves the generations (protocol/store/README.md 2), `why` saying
/// which version or build.
macro_rules! older {
    ($why:literal) => {
        older!(" holds a store from an older build, ", $why)
    };
    ($clause:literal, $why:literal) => {
        &[
            "",
            bridge_words!(concat!($clause, $why, ", and this start has changed nothing in it. No build translates a store from before the baseline, build `"), "` included, so every start refuses this one until it is moved. To start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of "),
            " together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.",
        ]
    };
}

impl Family {
    fn version_words(self) -> &'static [&'static str] {
        match self {
            Family::Settings => {
                older!(" holds a store whose ", "settings family is at another version than this build's")
            }
            Family::Component => {
                older!(" holds a store whose ", "component family is at another version than this build's")
            }
            Family::Core => older!(" holds a store whose ", "core family is at another version than this build's"),
            Family::Tasks => {
                older!(" holds a store whose ", "tasks family is at another version than this build's")
            }
            Family::People => {
                older!(" holds a store whose ", "people family is at another version than this build's")
            }
            Family::Notes => {
                older!(" holds a store whose ", "notes family is at another version than this build's")
            }
            Family::Accounts => {
                older!(" holds a store whose ", "accounts family is at another version than this build's")
            }
            Family::Secrets => {
                older!(" holds a store whose ", "secrets family is at another version than this build's")
            }
            Family::SkeinOauth => {
                older!(" holds a store whose ", "kept skein-oauth records are at another version than this build's")
            }
            Family::SmithTranscript => {
                older!(" holds a store whose ", "kept smith turns are at another version than this build's")
            }
            Family::Application => {
                older!(" holds a store whose ", "application family is at another version than this build's")
            }
        }
    }
}

impl OpenRefusal {
    /// The refusal's text in pieces, in order: the deployment's directory goes
    /// between each two consecutive pieces, wherever protocol/store/README.md has `<deployment>`.
    /// No allocation, no formatting.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "one match keeps every open refusal's operator instructions together")]
    pub fn words(&self) -> &'static [&'static str] {
        match self {
            Self::Unready { why: Unready::Empty } => &[
                "",
                " holds no deployment to take a checkpoint of: no `active` names a generation, and nothing in it has changed.",
            ],
            Self::Unready { why: Unready::Upgrade } => &[
                "The store in ",
                " needs an upgrade before it serves, so this build takes no checkpoint of it, and nothing in it has changed. Take it with the build that served it, or start this build, which upgrades it, and take one while it serves.",
            ],
            Self::Format { family, .. } => family.version_words(),
            Self::Deployment => &[
                "The store in ",
                " names another deployment than its header does, so its files come from two deployments, and this start has changed nothing in it. Restore ",
                " whole from one backup, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of ",
                " together, and start again: nothing is deleted, and the next start makes a new, empty store.",
            ],
            Self::Numbers { .. } => &[
                "The store in ",
                " does not end at the commit its header records, so a file was lost, cut or restored apart from the others, and this start has changed nothing in it. Restore ",
                " whole from one backup, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of ",
                " together, and start again: nothing is deleted, and the next start makes a new, empty store.",
            ],
            Self::Layout { .. } => &[
                "",
                " holds values at a layout this build cannot read, and this start has changed nothing in it. Start the build before it again, or a later build whose schema converts that layout, by its defaults and former names or by a written job.",
            ],
            Self::Failed => &[
                "A file in ",
                " could not be read or written, or its store could not be recovered, while this start opened it, and the start stopped there. Check the disk's space and the directory's permissions, and start again. If the next start fails the same way, restore ",
                " from a backup.",
            ],
            Self::OlderBuild { why: Older::Floor } => &[
                "",
                bridge_words!(
                    " holds a store older than this build opens: it has no floor, or one below this build's, and this start has changed nothing in it. Start build `",
                    "` on it and let it open, then this build."
                ),
            ],
            Self::OlderBuild { why: Older::Unregistered } => &[
                "",
                bridge_words!(
                    " holds a store older than this build opens: it has no registry of its layouts, and this start has changed nothing in it. Start build `",
                    "` on it and let it open, then this build."
                ),
            ],
            Self::OlderBuild { why: Older::Unleveled } => &[
                "",
                bridge_words!(
                    " holds a store from before the baseline, the first build whose stores later builds upgrade, and this start has changed nothing in it. No build translates a store from before the baseline, build `",
                    "` included, so every start refuses this one until it is moved. To start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of "
                ),
                " together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.",
            ],
            Self::OlderBuild { why: Older::Behind } => &[
                "",
                bridge_words!(
                    " holds a store from an older build, at patch levels behind this build's, and this start has changed nothing in it. This build keeps no patch: start build `",
                    "` on it and let it open, which brings it to this build's levels, then start this build."
                ),
            ],
            Self::Ahead { .. } => &[
                "",
                " holds a store from a later build, at patch levels past this build's, and this start has changed nothing in it. No patch runs backwards: start the later build again, or restore ",
                " from a backup this build made.",
            ],
            Self::Set { .. } => &[
                "This build's owner sets do not check, and this start has changed nothing in ",
                ". The build is wrong: start the build before it again.",
            ],
            Self::Check { why } => why.words(),
            Self::OlderBuild { why: Older::Layout(family) } => match family {
                Family::Settings => older!("whose settings family's layout differs from this build's"),
                Family::Component => older!("whose component family's layout differs from this build's"),
                Family::Core => older!("whose core family's layout differs from this build's"),
                Family::Tasks => older!("whose tasks family's layout differs from this build's"),
                Family::People => older!("whose people family's layout differs from this build's"),
                Family::Notes => older!("whose notes family's layout differs from this build's"),
                Family::Accounts => older!("whose accounts family's layout differs from this build's"),
                Family::Secrets => older!("whose secrets family's layout differs from this build's"),
                Family::SkeinOauth => older!("whose kept skein-oauth records' layout differs from this build's"),
                Family::SmithTranscript => older!("whose kept smith turns' layout differs from this build's"),
                Family::Application => older!("whose application family's layout differs from this build's"),
            },
            Self::OlderBuild { why: Older::Directory } => &[
                "",
                bridge_words!(
                    " holds a store from an older build, in the layout before generations (`store/`, `secrets/` or `transcripts/` at its top), and this start has changed nothing in it. No build translates a store from before generations, build `",
                    "` included, so every start refuses this one until it is moved. To start afresh, stop the service, move `store/`, `secrets/` and `transcripts/` out of "
                ),
                " together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.",
            ],
            Self::Job { .. } => &[
                "A job this build runs at open refused a record while this start upgraded the store in ",
                ", and the generation it upgraded from stays active, unchanged. Every start of this build refuses the same record: start the build before it again, which opens that generation as it was.",
            ],
            Self::Generations { why: Generations::Busy } => &[
                "Another process has ",
                " open, and this start has changed nothing in it. Stop that process first, then start again.",
            ],
            Self::Generations { why: Generations::Unreadable } => &[
                "The `active` file in ",
                " does not name a generation, and this start has changed nothing in it. Restore ",
                " from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
            ],
            Self::Generations { why: Generations::Missing { .. } } => &[
                "The generation `active` names in ",
                " has lost its `store/` or `secrets/`, and this start has changed nothing in it. Restore ",
                " from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
            ],
            Self::Generations { why: Generations::Unnamed } => &[
                "",
                " holds generations that no `active` names, and this start has changed nothing in it. Name one again from a backup of ",
                ", or write `active` to name the generation under `generations/` to open, and start again.",
            ],
            Self::Generations { why: Generations::Checkpoints } => &[
                "",
                " holds checkpoints but no `active` names a generation, and this start has changed nothing in it. Restore ",
                " from a backup or from one of its checkpoints, or, to start afresh, move `checkpoints/` out of ",
                ", and start again.",
            ],
        }
    }
}
/// Which older build (protocol/store/records.md, 3, and protocol/store/opening.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Older {
    /// A store before an owner's baseline, which records no patch level (5.3).
    Unleveled,
    /// No floor row, or one below this build's floor (protocol/store/opening.md, 1.8).
    Floor,
    /// A level behind the binary's (protocol/store/records.md, 3).
    Behind,
    /// A store holding values and no registry (protocol/store/layouts.md, 5.1).
    Unregistered,
    /// The component's own row does not decode at its version (protocol/store/records.md, 3).
    Layout(Family),
    /// The layout before generations, with no active file (protocol/store/README.md 2).
    Directory,
}
/// A refusal while finding the deployment's generation (protocol/store/opening.md 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Generations {
    /// Another open holds the deployment lock.
    Busy,
    /// Active is not decimal digits and one newline within 21 bytes.
    Unreadable,
    /// Generations stand that no active names, beyond a cut first start.
    Unnamed,
    /// Checkpoints stand but no active names a generation.
    Checkpoints,
    /// The active generation's maps do not both stand as directories.
    Missing { generation: u64 },
}

/// The first record refused by a rehearsal's kept text (protocol/store/opening.md, 5).
#[derive(Debug)]
pub enum Undecoded {
    Record { map: crate::KeptMap, key: Box<[u8]> },
    Frame { transcript: Box<str>, offset: u64 },
}

/// Why a commit failed (protocol/store/README.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommitFailure {
    /// skein-kv refused it at its entrance, carrying its refusal whole.
    Refused(skein_kv::Refusal),
    /// skein-kv's log write or sync failed or stalled; the outcome is uncertain.
    Failed,
    /// skein-kv answered under another root commit's owner.
    Owner { owner: u64 },
    /// A transcript's file step or the writer's bookkeeping failed.
    Transcript,
    /// The store's own checks refused it before skein-kv saw it.
    Check(CommitCheck),
    /// Another commit's failure stopped it while queued or waiting on frames.
    Stopped,
}

/// The store's check that refused a commit (protocol/store/README.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommitCheck {
    /// Not the store's next number, or at or before its last durable one.
    Number,
    /// The store isn't open.
    Closed,
    /// The queue behind transcripts' syncs is full.
    Queue,
    /// More writes than the root's bound, `Commits::writes`.
    Writes,
    /// Wires, a dropped transcript or a turn while a patch switch is pending.
    Switching,
    /// A key, record, turn frame or first commit's own rows past codec limits.
    Encoding,
    /// A copy's first key is missing, or its read failed.
    Copy,
    /// A map read needed by the commit failed or did not decode.
    Reads,
    /// More operations, places or keys than the commit's bounds hold.
    Capacity,
    /// Operations past the store's commit bytes, `Limits::commit`.
    Bytes,
}

/// What the component answers.
#[derive(Debug)]
pub enum Event {
    Checkpointed {
        owner: Token,
        checkpoint: Checkpointed,
    },
    CheckpointHeld {
        owner: Token,
    },
    CheckpointRemoved {
        owner: Token,
    },
    CheckpointRefused {
        owner: Token,
        why: CheckpointRefusal,
    },
    Rehearsing {
        owner: Token,
        wires: Option<Box<[KeptWire]>>,
        openings: Box<[WorkerOpening]>,
    },
    Decoded {
        owner: Token,
        records: u64,
    },
    Undecodable {
        owner: Token,
        family: Box<str>,
        version: u32,
        at: Undecoded,
    },
    Back {
        owner: Token,
        to: u64,
    },
    BackRefused {
        owner: Token,
        why: BackRefusal,
    },
    Closed {
        owner: Token,
        removed: bool,
    },
    /// Both stores are open and checked; `last` is the last durable commit.
    /// `installation` is the installation's identifier (protocol/store/README.md 4.4): the
    /// kept one, or, in an empty store, the drawn one its first commit keeps.
    Opened {
        owner: Token,
        last: u64,
        installation: [u8; 16],
        pending: bool,
    },
    /// The pending copy's manifest is durable and active has been replaced (10.2).
    Switched {
        owner: Token,
    },
    OpenRefused {
        owner: Token,
        why: OpenRefusal,
    },
    /// Commit `number` is durable, and every earlier one; `fill` is the
    /// map's bytes (skein-kv's `used`), for the core's high-water mark.
    Committed {
        number: u64,
        fill: u64,
    },
    /// The map's named census, after Opened or a commit's answer (protocol/store/README.md, 6.3).
    Census {
        census: Box<jig_domain_core_vocabulary::documents::StoreCensus>,
    },
    /// Commit `number` failed or was refused; the engine stops (protocol/store/README.md 5).
    Failed {
        number: u64,
        why: CommitFailure,
    },
    /// Commit refused before any of its data IO: this turn's complete encoded
    /// frame exceeds the page byte bound (protocol/store/README.md, 5, and protocol/store/transcripts.md, 2). In-process only.
    FrameTooLarge {
        number: u64,
        task: u64,
        bytes: u32,
        limit: u32,
    },
    /// A start's page (protocol/store/loads.md 1, 2): rows typed, an owner missing from the
    /// map handed up as a link to make; `next` is none once the step's ranges are done.
    Page {
        owner: Token,
        items: Box<[crate::restore::Item]>,
        next: Option<Cursor>,
    },
    /// A start's page that could not be read: the request's terminal; the engine stops.
    PageRefused {
        owner: Token,
        failure: crate::restore::Failure,
    },
    /// A load's page; `next` is none once the range is done.
    Loaded {
        owner: Token,
        rows: Box<[Row]>,
        next: Option<Cursor>,
    },
    /// A read's page, busy, or gone.
    Read {
        owner: Token,
        result: reads::Loaded,
    },
    Got {
        owner: Token,
        row: Option<Row>,
    },
    /// A transcript's turns; `next` is the offset to continue from after
    /// a nonempty prefix. An empty page is terminal (protocol/store/transcripts.md 3).
    Turns {
        owner: Token,
        turns: Box<[core::TurnRecord]>,
        next: Option<u64>,
        /// A window's page whose one turn is past the read's room (protocol/store/transcripts.md, 3):
        /// `turns` holds that turn alone, its fields read and its `transcript`
        /// empty, and this is its transcript's length.
        cut: Option<u32>,
    },
    /// A transcript read failed: a frame's checksum, a window at another
    /// turn, a dropped transcript, or no complete turn fitting the direct
    /// read's room while committed bytes remain (protocol/store/transcripts.md 3, 5).
    Unread {
        owner: Token,
    },
    /// A secrets request refused until this start's mark is durable (protocol/store/records.md 3).
    SecretsNotReady {
        owner: Token,
    },
    Kept {
        owner: Token,
    },
    /// A carried value kept, durable, under the application's `number`.
    Carried {
        owner: Token,
        number: u64,
    },
    /// A replace that named another value than the one held, or a carry
    /// whose next number holds one.
    Stale {
        owner: Token,
    },
    Erased {
        owner: Token,
    },
    Secret {
        owner: Token,
        value: Option<SecretValue>,
    },
    SecretsLoaded {
        owner: Token,
        rows: Box<[SecretRow]>,
        next: Option<u64>,
    },
    /// A stored row did not decode: the request's terminal; the engine
    /// stops (protocol/store/loads.md 1).
    Corrupt {
        owner: Token,
    },
    /// The shell sets this record's key in `of`'s Context, runs its generated
    /// written function over `patcher.parts()`, and answers `Request::Site`
    /// (protocol/store/opening.md 3). Application keys are relative to their tag; every
    /// other owner's keys are the map's whole keys. While serving, a frame's
    /// key is its task's places key followed by its eight-byte big-endian
    /// file offset (protocol/store/transcripts.md 6).
    Site {
        owner: Token,
        of: crate::Owner,
        key: Box<[u8]>,
        site: skein_codec::patch::Site,
        at: skein_codec::value::Node,
        patcher: Box<skein_codec::patch::Patcher>,
    },
    /// A job page, routed to its owner (protocol/store/jobs.md, 3).
    Job {
        owner: Token,
        of: crate::Owner,
        job: Box<str>,
        page: crate::JobPage,
    },
    /// A job's last answer and done row are durable.
    Ran {
        owner: Token,
        of: crate::Owner,
        job: Box<str>,
        rows: u64,
        commits: u64,
        counts: Box<[crate::JobCount]>,
    },
}

/// A wire the deployment's last start kept (protocol/store/opening.md 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeptWire {
    pub name: Box<str>,
    pub version: u32,
}

/// A worker's last authenticated opening (protocol/store/opening.md 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WorkerOpening {
    pub worker: Box<str>,
    pub wires: Box<[OpenedWire]>,
    pub output: u32,
    pub terms: Box<[OpenedTerm]>,
    pub at: skein_lib::Wall,
}

/// A wire's lowest version and ordered layouts (protocol/store/opening.md 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpenedWire {
    pub name: Box<str>,
    pub lowest: u16,
    pub layouts: Box<[[u8; 32]]>,
}

/// An opening term's largest supported value (protocol/store/opening.md 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpenedTerm {
    pub kind: u16,
    pub largest: u32,
}

/// Why the active generation cannot go back (protocol/store/opening.md 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackRefusal {
    Busy,
    NoPrevious,
    Committed { manifest: u64, last: u64 },
    Unreadable,
}

/// A directory adopted by the service (skein/docs/design/runtime/io/README.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Root {
    pub token: Token,
    pub fd: Fd,
}

/// The result of taking a checkpoint (protocol/store/checkpoints.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Checkpointed {
    pub name: crate::CheckpointName,
    pub secrets: u64,
    pub linked: u32,
    pub copied: u32,
    pub bytes: u64,
    pub paused: u64,
    pub took: u64,
    pub held: u64,
    pub removed: u32,
    pub skipped: u32,
}

/// Checkpoint refusals do not stop the serving store (protocol/store/checkpoints.md, 3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CheckpointRefusal {
    InProgress,
    Unavailable,
    Files,
    Unrecognized { path: Box<str> },
    Vanished { path: Box<str> },
    Failed { path: Box<str>, error: skein_io::kernel::Error },
    InUse,
    Missing,
}
impl CheckpointRefusal {
    #[must_use]
    pub fn words(&self) -> Box<str> {
        match self {
            Self::InProgress => Box::from("checkpoint in progress"),
            Self::Unavailable => Box::from("the store is not serving"),
            Self::Files => Box::from("more than 65536 files to take"),
            Self::Unrecognized { path } => {
                alloc::fmt::format(format_args!("an unrecognized file, {path}")).into_boxed_str()
            }
            Self::Vanished { path } => {
                alloc::fmt::format(format_args!("a selected file vanished, {path}")).into_boxed_str()
            }
            Self::Failed { path, error } => {
                alloc::fmt::format(format_args!("a file failed, {path}: {error:?}")).into_boxed_str()
            }
            Self::InUse => Box::from("in use: its lock is held"),
            Self::Missing => Box::from("no such checkpoint"),
        }
    }
}

/// Independent file drivers (protocol/store/README.md, section 8).
#[derive(Debug)]
pub struct Below {
    pub store: Queue<file::Request>,
    pub secrets: Queue<file::Request>,
    pub transcripts: Queue<file::Request>,
    pub checkpoints: Queue<file::Request>,
}

/// A file terminal and its descriptor, read by the service alone.
#[derive(Debug)]
pub struct Arrived {
    pub event: file::Event,
    pub fd: Option<Fd>,
}

#[derive(Debug)]
pub enum Io {
    Store(Arrived),
    Secrets(Arrived),
    Transcripts(Arrived),
    Checkpoints(Arrived),
}
