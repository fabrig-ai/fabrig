//! The engine's lines and handover io (temper/docs/design/protocol/services.md, 7;
//! jig/docs/design/domain/engine/settings-store.md, 1).
use temper_engine_service::{CommitStop, End, RouteStop, Stop};
use temper_shell::directories::Directories;

pub fn read_handover(
    read: &temper_engine_service::HandoverRead,
) -> Result<Option<skein_lib::Secret>, temper_engine_service::HandoverUnread> {
    read_handover_with(read, |path, max| skein_shell::read_private_file(path, max, skein_shell::effective_user()))
}

fn read_handover_with(
    read: &temper_engine_service::HandoverRead,
    read_private: impl FnOnce(&std::path::Path, u32) -> Result<skein_lib::Secret, skein_shell::Unread>,
) -> Result<Option<skein_lib::Secret>, temper_engine_service::HandoverUnread> {
    use temper_engine_service::HandoverUnread;
    if read.forge == 0 {
        make_playground_key(super::path(&read.path))
            .map_err(|why| HandoverUnread::Kernel(why.raw_os_error().expect("key io returns an errno")))?;
    }
    match read_private(super::path(&read.path), read.max) {
        Ok(value) => {
            let bytes = value.expose();
            let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
            Ok(Some(skein_lib::Secret::new(bytes.into())))
        }
        Err(skein_shell::Unread::Absent) => Ok(None),
        Err(skein_shell::Unread::NotAFile) => Err(HandoverUnread::NotAFile),
        Err(skein_shell::Unread::TooLarge { .. }) => Err(HandoverUnread::TooLarge),
        Err(skein_shell::Unread::Unsafe(why)) => Err(HandoverUnread::Unsafe(why)),
        Err(skein_shell::Unread::Kernel(error)) => Err(HandoverUnread::Kernel(temper_shell::directories::errno(error))),
    }
}
/// The scripted account's key is made before its read and the playground's group start
/// (temper/docs/design/protocol/configuration/providers.md, 4).
fn make_playground_key(path: &std::path::Path) -> std::io::Result<()> {
    use std::{
        fs,
        io::Write as _,
        os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _},
    };
    let directory = path.parent().expect("the playground key is in demo");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(directory)?;
    let random = skein_shell::crypto_seed().map_err(std::io::Error::from_raw_os_error)?;
    let mut key = Vec::with_capacity(65);
    for byte in random.expose() {
        const HEX: &[u8] = b"0123456789abcdef";
        key.push(HEX[usize::from(byte >> 4)]);
        key.push(HEX[usize::from(byte & 15)]);
    }
    key.push(b'\n');
    let key = skein_lib::Secret::new(key.into_boxed_slice());
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(why),
    }
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    file.write_all(key.expose())
}

pub fn remove_handover(remove: &temper_engine_service::HandoverRemoval) -> Result<(), i32> {
    match std::fs::remove_file(super::path(&remove.path)) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(why.raw_os_error().expect("unlink returns an errno")),
    }
}

/// The composition's listening note, with the client-listen boundary kept beside its words.
pub fn listening_line(service: &mut temper_engine_service::Service, binary: &str) -> Option<(String, bool)> {
    let note = service.listening().pop()?;
    let clients = matches!(
        &note,
        temper_engine_service::Note::Listening { listener: temper_engine_service::Listening::Clients, .. }
    );
    Some((note_line(binary, note), clients))
}
pub fn next_note_line(service: &mut temper_engine_service::Service, binary: &str) -> Option<String> {
    service.notes().pop().map(|note| note_line(binary, note))
}
#[must_use]
pub fn service_ending(
    service: &temper_engine_service::Service,
    binary: &str,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    ending_with_binary(binary, service.end(), failed, directories, store, service.unsettled())
}

/// The terminal stderr and status shared by the shell and its real-loop host.
#[must_use]
pub fn ending_with_binary(
    binary: &str,
    end: Option<&End>,
    output_failed: bool,
    _directories: &Directories,
    store: &[u8],
    unsettled: &[temper_engine_service::Unsettled],
) -> (String, u8) {
    let mut lines = match end {
        Some(End::Stopped(why)) => match why {
            Stop::Start(found) => problems(
                binary,
                &temper_engine_service::startup::Problems::jig(found.clone(), &jig_protocol_config::Path::root()),
            ),
            Stop::LimitRestore { name, kind, configured, held } => format!(
                "{binary} serve: {} (setting {}): its figure is {configured}, but the store holds {held}; put limits-held in the data directory to start at the stock\n",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(kind.name())
            ),
            Stop::CodecRestore { bound, kind, row } => codec_restore(binary, bound, *kind, row.as_ref()),
            Stop::Room(refusal) => {
                format!("{binary} serve: restart refused: {}\n", String::from_utf8_lossy(&refusal.words()))
            }
            Stop::RestorePage { range, failure } => {
                format!("{binary} serve: restore {}: {}\n", restore_kind(*range), page_failure(failure.as_ref()))
            }
            Stop::StartRefused { phase, failure } => {
                let during = phase.map_or_else(|| "unknown phase".into(), start_phase);
                format!("{binary} serve: restart refused during {during}: {}\n", start_failure(failure))
            }
            Stop::ForgeFit { phase, failure } => {
                let during = phase.map_or_else(|| "unknown phase".into(), start_phase);
                let family = match failure.family {
                    Some(kind) => format!(" {kind:?}"),
                    None => String::new(),
                };
                format!(
                    "{binary} serve: restart refused during {during}: forge{family} {:?}, held {}, additional {}, limit {}\n",
                    failure.set, failure.held, failure.additional, failure.limit
                )
            }
            Stop::BranchWrites(name, count) => branch_writes(binary, name, *count),
            Stop::ForgeLink { phase, link } => {
                let during = phase.map_or_else(|| "unknown phase".into(), start_phase);
                format!(
                    "{binary} serve: restart refused during {during}: forge {:?} {:?} links missing {:?}\n",
                    link.kind, link.key, link.need
                )
            }
            Stop::Setting(key) => {
                let words = temper_engine_domain_forge::settings::retired_words(*key);
                format!("{binary} serve: restart refused: {}\n", String::from_utf8_lossy(&words))
            }
            Stop::Binding => format!("{binary} serve: deployment namespace binding refused\n"),
            Stop::StoreOpen { why } => {
                let deployment = super::path(store);
                return store_refusal(
                    binary,
                    temper_engine_service::refused_start(why, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::StoreRow { row } => {
                let deployment = super::path(store);
                return store_refusal(
                    binary,
                    temper_engine_service::refused_row_start(row, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::FrameTooLarge { number, task, bytes, limit } => format!(
                "{binary} serve: commit {number} refused: task {task}'s transcript frame, {bytes} bytes, past the store's page of {limit} bytes\n"
            ),
            Stop::Journal { why } => {
                let why = match why {
                    temper_engine_service::JournalStop::Refused { commit, sent, durable } => {
                        format!("the store answered commit {commit}, with {sent} sent and {durable} durable")
                    }
                    temper_engine_service::JournalStop::PastEdge { writes } => {
                        format!("a decision past the store's {writes} writes")
                    }
                    temper_engine_service::JournalStop::Counter => "the commit counter at its end".into(),
                };
                format!("{binary} serve: the engine's journal stopped: {why}\n")
            }
            Stop::CommitFailed { number, why } => {
                format!("{binary} serve: commit {number} failed: {}\n", commit_why(*number, *why))
            }
            Stop::Hooks => format!("{binary} serve: webhooks listener failed\n"),
            Stop::Listener { listener, address, error } => listener_failed_line(binary, *listener, *address, *error),
        },
        None | Some(End::Drained | End::Aborted) => String::new(),
    };
    if end == Some(&End::Drained) {
        use std::fmt::Write as _;
        for write in unsettled {
            let (entry, task) = (write.entry, write.task);
            writeln!(
                lines,
                "{binary} serve: stopped before task {task}'s forge write {entry} settled; the next start settles it"
            )
            .expect("format stop diagnostic");
        }
    }
    (lines, u8::from(output_failed || end != Some(&End::Drained)))
}

fn branch_writes(binary: &str, name: &temper_engine_domain_forge::Name, count: usize) -> String {
    let temper_engine_domain_forge::What::Branch(parts) = &name.what else {
        unreachable!("only branch records are converted")
    };
    let branch = parts.iter().map(|part| String::from_utf8_lossy(part)).collect::<Vec<_>>().join("/");
    format!(
        "{binary} serve: restart refused during Link: branch {branch} (forge {}, repository {}) needs {count} write entries, limit {}\n",
        name.forge,
        name.repository,
        temper_engine_domain_forge::WRITES
    )
}

fn codec_restore(
    binary: &str,
    bound: &jig_protocol_store::codec::Lowered,
    kind: temper_engine_service::startup::LimitKind,
    row: Option<&jig_protocol_store::restore::RowSite>,
) -> String {
    let site = match row {
        Some(row) => format!(", key {:02x?}", row.key),
        None => String::new(),
    };
    format!(
        "{binary} serve: {} (setting {}): its figure is {}, but the store holds a record that needs {}{site}; put limits-held in the data directory to start at the stock\n",
        String::from_utf8_lossy(bound.name),
        String::from_utf8_lossy(kind.name()),
        bound.configured,
        bound.held
    )
}

/// Bounded causes without record contents (temper/docs/design/protocol/store-versions.md, 1).
fn commit_why(number: u64, why: CommitStop) -> String {
    use jig_protocol_store::{CommitCheck, CommitFailure};
    use skein_kv::{Exhausted, Large, Refusal};
    match why {
        CommitStop::Store(CommitFailure::Refused(Refusal::Full(Exhausted::Budget { used, budget, projected }))) => {
            format!(
                "the store's map is full ({} of {} bytes; this commit needs {})",
                grouped(used),
                grouped(budget),
                grouped(projected)
            )
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::Full(Exhausted::Entries { capacity }))) => {
            format!("the store's map is full ({} entries)", grouped(u64::from(capacity)))
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::Full(Exhausted::Numbers))) => {
            "the store's commit numbers are at their end".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Key))) => {
            "the store refused it: a key past the key limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Value))) => {
            "the store refused it: a value past the value limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Ops))) => {
            "the store refused it: more operations than the commit limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Frame))) => {
            "the store refused it: a frame past the commit limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Page))) => {
            "the store refused it: a page past the page limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Range))) => {
            "the store refused it: a range that ends before it starts".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Record))) => {
            "the store refused it: a log's record past the record limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::TooLarge(Large::Log))) => {
            "the store refused it: a log past its length limit".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::Logs)) => {
            "the store refused it: a log's operation its log rule refuses".into()
        }
        CommitStop::Store(CommitFailure::Refused(Refusal::Busy)) => "the store refused it: its queue is full".into(),
        CommitStop::Store(CommitFailure::Refused(Refusal::Unavailable)) => {
            "the store refused it: the store is not open".into()
        }
        CommitStop::Store(CommitFailure::Failed) => {
            "the store's log failed a write or a sync, so its outcome is uncertain".into()
        }
        CommitStop::Store(CommitFailure::Owner { owner }) => {
            format!("the store committed root commit {owner} as its commit {number}")
        }
        CommitStop::Store(CommitFailure::Transcript) => "a file step of its transcripts failed".into(),
        CommitStop::Store(CommitFailure::Check(CommitCheck::Number)) => {
            "the store's own check refused it: its number is not the store's next".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Closed)) => {
            "the store's own check refused it: the store is not open".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Queue)) => {
            "the store's own check refused it: the queue behind transcripts' syncs is full".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Writes)) => {
            "the store's own check refused it: more writes than the root's bound".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Switching)) => {
            "the store's own check refused it: an upgrade's switch is pending".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Encoding)) => {
            "the store's own check refused it: a key, record or frame past its codec's limits".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Copy)) => {
            "the store's own check refused it: a copy's first key is not in the map".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Reads)) => {
            "the store's own check refused it: a read of the map it needs failed".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Capacity)) => {
            "the store's own check refused it: more operations than the commit's bounds hold".into()
        }
        CommitStop::Store(CommitFailure::Check(CommitCheck::Bytes)) => {
            "the store's own check refused it: its operations past the store's commit bytes".into()
        }
        CommitStop::Store(CommitFailure::Stopped) => "the store had stopped at another commit's failure".into(),
        CommitStop::Route(RouteStop::Open) => {
            "the engine's store routing stopped: the store refused to open while the engine ran".into()
        }
        CommitStop::Route(RouteStop::Secrets) => {
            "the engine's store routing stopped: the store's secrets were not ready".into()
        }
        CommitStop::Route(RouteStop::Cursor) => {
            "the engine's store routing stopped: a load's cursor did not translate or advance".into()
        }
        CommitStop::Route(RouteStop::Page) => {
            "the engine's store routing stopped: a load's page could not be measured or cut".into()
        }
    }
}

fn grouped(figure: u64) -> String {
    let digits = figure.to_string();
    let mut grouped = String::new();
    for (position, digit) in digits.chars().enumerate() {
        if position != 0 && (digits.len() - position).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

pub(crate) fn store_refusal(binary: &str, (lines, exit): (Box<[u8]>, u8)) -> (String, u8) {
    use std::fmt::Write as _;
    let lines = String::from_utf8(lines.into_vec()).expect("refusal is UTF-8");
    let mut named = String::new();
    for line in lines.lines() {
        writeln!(named, "{binary} serve: {}", line.strip_prefix("temper serve: ").expect("service refusal prefix"))
            .expect("writing into a string");
    }
    (named, exit)
}

fn restore_kind(kind: temper_engine_service::RestoreRangeKind) -> &'static str {
    use temper_engine_service::RestoreRangeKind as Kind;
    match kind {
        Kind::Deployment => "deployment",
        Kind::Settings => "settings",
        Kind::Accounts => "accounts",
        Kind::Tasks => "tasks",
        Kind::RunProofs => "run proofs",
        Kind::People => "people",
        Kind::Calls => "calls",
        Kind::TaskResult => "task result",
        Kind::RequesterWord => "requester word",
        Kind::Turns => "turns",
        Kind::Notes => "notes",
        Kind::Census => "census",
        Kind::Forge => "forge",
        Kind::History => "history",
    }
}
fn numbered_setting(setting: jig_domain_core::settings::Setting) -> (&'static str, u32) {
    use jig_domain_core::settings::Setting;
    match setting {
        Setting::Charter(n) => ("charter", n),
        Setting::Template(n) => ("template", n),
        Setting::Policy(n) => ("policy", n),
        Setting::Model(n) => ("model", n),
        Setting::Worker(_)
        | Setting::Provider(_)
        | Setting::Rules
        | Setting::Limits
        | Setting::Application { .. }
        | Setting::ProjectPolicy { .. } => unreachable!("only ranged kinds in a numbers refusal or taken refusal"),
    }
}

fn start_failure(failure: &jig_domain_core::start::Failure) -> String {
    use jig_domain_core::start::Failure;
    match failure {
        Failure::Numbers { settings } => {
            use jig_domain_core::settings::{NUMBERS, STOCK_NUMBERS};
            use std::fmt::Write as _;
            let own = STOCK_NUMBERS.checked_add(1).expect("own numbers follow stock");
            let mut line = format!(
                "settings out of their numbers (the stock's 1 to {STOCK_NUMBERS}, a deployment's own {own} to {NUMBERS}): "
            );
            for (at, numbered) in settings.iter().enumerate() {
                if at != 0 {
                    line.push_str(", ");
                }
                let source = if numbered.changed { "changed" } else { "default" };
                let (kind, number) = numbered_setting(numbered.setting);
                write!(line, "{source} {kind} {number}").expect("format numbered setting");
            }
            line
        }
        Failure::Taken { settings } => {
            use std::fmt::Write as _;
            let mut line = String::from(
                "the stock's new settings take numbers a deployment's own hold (the stock's 1 to 32, a deployment's own 33 to 64): ",
            );
            for (at, setting) in settings.iter().enumerate() {
                if at != 0 {
                    line.push_str(", ");
                }
                let (kind, number) = numbered_setting(*setting);
                write!(line, "{kind} {number}").expect("format taken setting");
            }
            line.push_str("; move each to 33 to 64 on the build that runs now, then start this one");
            line
        }
        Failure::Settings { failure, changed } => {
            let source = if *changed { "changed" } else { "default" };
            format!("{source} setting {:?}: {:?}", failure.setting, failure.fault)
        }
        Failure::ProjectPolicy { project } => format!("project {project}: policy refused"),
        Failure::Application { kind, scope, number, why } => {
            format!("application setting {kind}/{scope}/{number}: {why:?}")
        }
        Failure::HeldPastLimit { which, held, limit } => format!("{which:?}, held {held}, limit {limit}"),
        Failure::NeedsPastLimit { held, limit } => {
            format!(
                "stored settings' agent needs hold {held} bytes, past the {limit} byte bound; raise fleet.needs_bytes"
            )
        }
        Failure::AccountsPastLimit { failure } => {
            format!("accounts {:?}, key {:?}, limit {}", failure.set, failure.key, failure.limit)
        }
        Failure::LiveSet { set, key, held, limit } => {
            format!("{set:?}, key {key:?}, held {held}, limit {limit}")
        }
        Failure::Accounting { key } => format!("accounting refused, key {key:?}"),
        Failure::LlmChoice { key } => format!("LLM choice refused, key {key:?}"),
        Failure::Link { record, linked } => {
            use jig_domain_core::start::Linked;
            let target = match linked {
                Linked::Record(key) => format!("{key:?}"),
                Linked::Charter { charter, revision } => format!("charter {charter} at revision {revision}"),
                Linked::Project(project) => format!("project {project}"),
                Linked::Connector(connector) => format!("connector {connector}"),
                Linked::SettingsProposal { proposer, proposal } => format!("settings proposal {proposer:?}/{proposal}"),
                Linked::Holder(resource) => format!("hold owner for {resource:?}"),
            };
            format!("{record:?} links missing {target}")
        }
    }
}
fn page_failure(failure: Option<&jig_protocol_store::restore::Failure>) -> String {
    use std::fmt::Write as _;
    let Some(failure) = failure else { return "store load failed".into() };
    let mut line = if let jig_protocol_store::restore::Cause::History { task, failure } = failure.cause {
        format!("task {task} history fetch: {}", history_failure(failure))
    } else {
        format!("{:?}", failure.cause)
    };
    if let Some(row) = &failure.row {
        if let Some(family) = row.family {
            write!(line, ", family {family:?}").expect("format restore family");
        }
        write!(line, ", key {:02x?}", row.key).expect("format restore key");
        if row.truncated {
            write!(line, " (truncated, {} bytes total)", row.key_bytes).expect("format restore key size");
        }
    }
    line
}

fn history_failure(failure: jig_protocol_store::restore::HistoryFailure) -> &'static str {
    use jig_protocol_store::restore::HistoryFailure;
    match failure {
        HistoryFailure::LogId => "task has no history log id",
        HistoryFailure::ExtentMissing => "history extent missing",
        HistoryFailure::ExtentMalformed => "history extent malformed or unreadable",
        HistoryFailure::PlaceMissing => "history place missing",
        HistoryFailure::PlaceMalformed => "history place malformed or unreadable",
        HistoryFailure::Unread => "generation cannot give the history frame",
        HistoryFailure::Empty => "history fetch returned no frame",
        HistoryFailure::FramePosition => "history frame position invalid",
        HistoryFailure::FramePastExtent => "history frame past the extent",
        HistoryFailure::FramePastPage => "history frame past the fetch page",
        HistoryFailure::Record => "history record does not parse",
        HistoryFailure::RecordTask => "history record names another task",
        HistoryFailure::Incomplete => "history fetch ended before its extent",
        HistoryFailure::Continuation => "history fetch continuation disagrees with its frame",
        HistoryFailure::Arithmetic => "history fetch accounting overflow",
    }
}

// A landing's waits, temper/docs/design/domain/landing-waits.md, 4.
fn landing_read_line(binary: &str, note: temper_engine_domain_forge::LandingReadNote) -> String {
    use temper_engine_domain_forge::LandingReadNoteKind;
    let temper_engine_domain_forge::LandingReadNote { task, round, reading, kind } = note;
    let reading = String::from_utf8_lossy(&reading);
    match kind {
        LandingReadNoteKind::Started { why } => format!(
            "{binary} serve: landing {task} round {round} waits for a fresh read: {reading}: {}\n",
            String::from_utf8_lossy(&why),
        ),
        LandingReadNoteKind::Read { elapsed } => format!(
            "{binary} serve: landing {task} round {round} has its fresh read after {}: {reading}\n",
            landing_read_age(elapsed),
        ),
        LandingReadNoteKind::Held { elapsed, words } => format!(
            "{binary} serve: warning: landing {task} round {round} held after {}: {}\n",
            landing_read_age(elapsed),
            String::from_utf8_lossy(&words),
        ),
    }
}

fn landing_read_age(elapsed: skein_lib::Duration) -> String {
    let seconds = elapsed.as_nanos() / 1_000_000_000;
    for (unit, next, suffix, next_suffix) in [(86_400, 3600, 'd', 'h'), (3600, 60, 'h', 'm'), (60, 1, 'm', 's')] {
        if seconds >= unit {
            let count = seconds / unit;
            let rest = seconds % unit / next;
            return format!("{count}{suffix} {rest}{next_suffix}");
        }
    }
    format!("{seconds}s")
}

#[expect(clippy::too_many_lines, reason = "one exhaustive note match keeps the engine vocabulary beside its words")]
#[must_use]
pub fn note_line(binary: &str, note: temper_engine_service::Note) -> String {
    match note {
        temper_engine_service::Note::ClientStreamFailed { sign_in, why } => {
            format!("{binary} serve: client stream closed: sign_in={sign_in} reason={why:?}\n")
        }
        temper_engine_service::Note::StartWaiting { line } => start_wait_line(binary, &line),
        temper_engine_service::Note::CoreWaiting { why } => temper_service_shell::core_wait_line(binary, &why.words()),
        temper_engine_service::Note::CoreAnswers => temper_service_shell::core_answers_line(binary),
        temper_engine_service::Note::StartupWarning { line } => {
            format!("{binary} serve: warning: {}\n", String::from_utf8_lossy(&line))
        }
        temper_engine_service::Note::LandingRead(note) => landing_read_line(binary, note),
        temper_engine_service::Note::LinkDisagreed { change, named, landing } => {
            let where_now = match landing {
                Some(landing) => format!("landing {landing}"),
                None => "no landing".to_owned(),
            };
            format!(
                "{binary} serve: warning: change {change} named landing {named}; by the landings' rows it is in {where_now}\n"
            )
        }
        temper_engine_service::Note::ConversionAskDropped { change, landing, kind } => {
            format!(
                "{binary} serve: warning: change {change}: its {kind} of landing {landing} dropped as its asks were converted: its queue is full\n"
            )
        }
        temper_engine_service::Note::LandingUnmade {
            task,
            repository,
            branch,
            changes,
            readback,
            landed,
            unrecorded,
        } => {
            use std::fmt::Write as _;
            let numbers = |changes: &[u64]| changes.iter().map(u64::to_string).collect::<Vec<_>>().join(", ");
            let mut line = format!(
                "{binary} serve: warning: landing {task} into {} of repository {}/{} has no task: ended",
                String::from_utf8_lossy(&branch),
                repository.forge,
                repository.repository
            );
            if readback {
                line.push_str(" after reading back its merge");
                if !landed.is_empty() {
                    write!(line, "; changes {} landed", numbers(&landed)).expect("write to string");
                }
            }
            if !changes.is_empty() {
                write!(line, "; its changes {} wait again", numbers(&changes)).expect("write to string");
            }
            if !unrecorded.is_empty() {
                write!(line, "; the pull requests of changes {} are not recorded merged", numbers(&unrecorded))
                    .expect("write to string");
            }
            line.push('\n');
            line
        }
        temper_engine_service::Note::HoldDropped { name, task } => {
            let branch = match name.what {
                temper_engine_domain_forge::What::Branch(segments) => {
                    segments.iter().map(|segment| String::from_utf8_lossy(segment)).collect::<Vec<_>>().join("/")
                }
                temper_engine_domain_forge::What::Repository => "repository".to_owned(),
                temper_engine_domain_forge::What::Pull(number) => format!("pull {number}"),
                temper_engine_domain_forge::What::Issue(number) => format!("issue {number}"),
            };
            format!(
                "{binary} serve: warning: hold on {branch} of repository {}/{} by task {task}, which has no task: dropped\n",
                name.forge, name.repository
            )
        }
        temper_engine_service::Note::ForgeLink { forge, note } => {
            let words = match note {
                temper_engine_domain_forge::LinkNote::Gone => "their account is gone, the link kept",
                temper_engine_domain_forge::LinkNote::Writer => "refused, temper's writer",
                temper_engine_domain_forge::LinkNote::Replaced => "replaced",
                temper_engine_domain_forge::LinkNote::Cleared => "cleared",
            };
            format!("forge {forge}'s person: {words}\n")
        }
        temper_engine_service::Note::LandingRefused { task, repository, branch, why, changes } => {
            let why = String::from_utf8_lossy(&why);
            let members = landing_numbers(&changes);
            format!(
                "{binary} serve: warning: landing {task} into {} of repository {}/{} not formed: jig refused its task: {why}; its changes {members} wait\n",
                String::from_utf8_lossy(&branch),
                repository.forge,
                repository.repository
            )
        }
        temper_engine_service::Note::LandingTableRefused { repository, branch, limit, holders, changes } => {
            let holders = landing_numbers(&holders);
            let members = landing_numbers(&changes);
            format!(
                "{binary} serve: warning: landing into {} of repository {}/{} not formed: the forge's landings are full, {limit} of {limit}, held by landings {holders}; its changes {members} wait\n",
                String::from_utf8_lossy(&branch),
                repository.forge,
                repository.repository
            )
        }
        temper_engine_service::Note::RunFailed(diagnostic) => run_failure_line(
            binary,
            diagnostic.task,
            diagnostic.run,
            diagnostic.attempt,
            diagnostic.failure,
            &diagnostic.detail,
        ),
        temper_engine_service::Note::RunRefused(diagnostic) => run_refusal_line(binary, diagnostic),
        temper_engine_service::Note::JobAnswered(diagnostic) => job_line(binary, diagnostic),

        temper_engine_service::Note::HelloRefused { channel, text } => {
            format!("{binary} serve: hello refused channel={channel}: {}\n", String::from_utf8_lossy(&text))
        }
        temper_engine_service::Note::ReadRefused { task, attempt, tool, layer, reason, findings, error, job } => {
            let job = match job {
                Some(job) => format!(
                    " head={} run={} job={} job_attempt={}",
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&job.head)),
                    job.run,
                    job.job,
                    job.attempt
                ),
                None => String::new(),
            };
            format!(
                "{binary} serve: read refused task={task} attempt={attempt} tool={} layer={layer} reason={reason}{job} error={error:?} findings={findings:?}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_engine_service::Note::BriefReadRefused { task, section, stage, reason, heads, error } => {
            let (bound, error) = match error {
                Some(skein_forge::api::Error::TooLarge(excess)) => {
                    let bound = temper_engine_domain_forge::bound_words(excess);
                    let bound = String::from_utf8_lossy(&bound);
                    (format!(" {bound}"), format!(" error=TooLarge: past {bound}"))
                }
                Some(error) => (String::new(), format!(" error={error:?}")),
                None => (String::new(), String::new()),
            };
            let heads = match heads {
                Some((pinned, head)) => format!(
                    " pinned={} head={}",
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&pinned)),
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&head)),
                ),
                None => String::new(),
            };
            format!(
                "{binary} serve: brief read refused task={task} section={} stage={stage} reason={reason}{bound}{heads}{error}\n",
                String::from_utf8_lossy(section)
            )
        }
        temper_engine_service::Note::ForgeTable { words } => {
            let mut line = format!("{binary} serve: {}", String::from_utf8_lossy(&words));
            line = line.replace(['\r', '\n'], " ");
            let mut end = line.len().min(1023);
            while !line.is_char_boundary(end) {
                end = end.saturating_sub(1);
            }
            line.truncate(end);
            line.push('\n');
            line
        }
        temper_engine_service::Note::HeadRefused { task, words } => {
            let task = match task {
                Some(task) => format!(" task={task}"),
                None => String::new(),
            };
            format!("{binary} serve: branch head refused{task}: {}\n", String::from_utf8_lossy(&words))
        }
        temper_engine_service::Note::BriefFailed { task, counts, reason } => {
            format!(
                "{binary} serve: brief failed task={task} counts={counts} reason={}\n",
                String::from_utf8_lossy(&reason)
            )
        }
        temper_engine_service::Note::CallAdmissionRefused { task, attempt, tool, layer, reason } => {
            format!(
                "{binary} serve: call admission refused task={task} attempt={attempt} tool={} layer={layer} reason={reason}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_engine_service::Note::Ran { of, job, rows, commits, counts } => {
            let binary = if binary == "temper" { "temper-box" } else { binary };
            format!(
                "{binary} serve: ran {} {job}: {rows} rows in {commits} commits{}\n",
                crate::upgrade::owner(of),
                crate::upgrade::job_counts(&counts)
            )
        }
        temper_engine_service::Note::Listening { listener, addr } => {
            let name = match listener {
                temper_engine_service::Listening::Clients => "clients",
                temper_engine_service::Listening::Workers => "workers",
                temper_engine_service::Listening::Webhooks => "webhooks",
            };
            format!("listening {name} {addr}\n")
        }

        temper_engine_service::Note::WorkerFigures { limits, endpoints, revision } => {
            let (setting, revision) = match revision {
                temper_engine_service::FiguresRevision::Agents(revision) => ("agents setting", revision),
                temper_engine_service::FiguresRevision::Limits(revision) => ("limit settings", revision),
            };
            format!(
                "{binary} serve: figures for workers: {limits} limits, {endpoints} endpoints, {setting} revision {revision}\n"
            )
        }
        temper_engine_service::Note::WorkerFiguresTaken { worker } => {
            format!("{binary} serve: worker {} took its figures\n", String::from_utf8_lossy(&worker))
        }
        temper_engine_service::Note::WorkerUnpaused { worker, version: _, mode } => {
            let worker = String::from_utf8_lossy(&worker);
            let mode = match mode {
                jig_domain_link::PauseMode::Resume => "resume",
                jig_domain_link::PauseMode::Pause => "pause",
                jig_domain_link::PauseMode::Drain => "drain",
            };
            format!(
                "{binary} serve: worker {worker} not told to {mode}: its layout lacks the kind; the engine's placement follows the ask\n"
            )
        }
        temper_engine_service::Note::WorkerTold { worker, version: _, wait } => {
            let worker = String::from_utf8_lossy(&worker);
            match wait {
                Some(wait) => {
                    format!("{binary} serve: worker {worker} told the engine restarts: wait {} ms\n", wait.as_millis())
                }
                None => format!(
                    "{binary} serve: worker {worker} not told the engine restarts: its layout lacks the word; it keeps its grace\n"
                ),
            }
        }
        temper_engine_service::Note::WorkerOpened { worker, converted } => {
            let mut line = format!("{binary} serve: worker {} opened: ", String::from_utf8_lossy(&worker));
            if converted.is_empty() {
                line.push_str("at the engine's layouts");
            } else {
                line.push_str("converting ");
                let mut separator = "";
                for wire in &converted {
                    line.push_str(separator);
                    line.push_str(wire);
                    separator = ", ";
                }
            }
            line.push('\n');
            line
        }
        temper_engine_service::Note::WorkerOpeningRefused { worker, reason, text } => {
            let worker = match worker {
                Some(name) => format!("worker {}", String::from_utf8_lossy(&name)),
                None => "a worker".into(),
            };
            format!(
                "{binary} serve: {worker} refused at its opening: {}: {}\n",
                opening_reason(reason),
                String::from_utf8_lossy(&text)
            )
        }
        temper_engine_service::Note::WorkerLost { worker, channel, why } => {
            format!(
                "{binary} serve: worker lost worker={} channel={}: {}\n",
                worker_name(worker.as_deref()),
                channel.raw(),
                engine_worker_loss(why)
            )
        }
        temper_engine_service::Note::WorkerUnsent { worker, channel, unsent } => {
            use jig_protocol_engine_workers::Bound;
            let record = worker_record(unsent.kind);
            let bound = match unsent.bound {
                Bound::Translation(problem) => {
                    return format!(
                        "{binary} serve: cannot send {record} to worker {}: {}\n",
                        worker_name(Some(&worker)),
                        String::from_utf8_lossy(&problem.kept()),
                    );
                }
                Bound::Room(room) => format!("past the room of {room} bytes"),
                Bound::Channel(why) => {
                    let reason = match why {
                        skein_channel::Unsent::WrongDirection => "wrong-direction",
                        skein_channel::Unsent::PeerDoesNotTake => "not-taken",
                        skein_channel::Unsent::TooLarge => "too-large",
                        skein_channel::Unsent::Closed => "closed",
                        skein_channel::Unsent::Full => "full",
                    };
                    format!("refused: {reason}")
                }
            };
            format!(
                "{binary} serve: worker unsent worker={} channel={}: {record} of {} bytes {bound}\n",
                worker_name(Some(&worker)),
                channel.raw(),
                unsent.bytes
            )
        }
        temper_engine_service::Note::WorkerUnread { worker, kind, text } => run_detail_line(
            format!("{binary} serve: cannot read {} from worker {}: ", worker_record(kind), worker_name(Some(&worker))),
            &text,
        ),
        temper_engine_service::Note::WorkerOpeningEnded { worker, why } => {
            format!(
                "{binary} serve: worker opening ended worker={}: {}\n",
                worker_name(worker.as_deref()),
                engine_worker_loss(why)
            )
        }
        temper_engine_service::Note::WorkerListenerFailed { address, error } => {
            listener_failed_line(binary, temper_engine_service::Listening::Workers, address, error)
        }

        temper_engine_service::Note::ForgeLacking { number, names } => {
            format!("{binary} serve: forge {number}: App lacks {}\n", String::from_utf8_lossy(&names))
        }
        temper_engine_service::Note::SetUpRefused { number, why } => set_up_refused_line(binary, number, why),
        temper_engine_service::Note::Playground { line } | temper_engine_service::Note::BoxLink { line } => {
            format!("{}\n", String::from_utf8_lossy(&line))
        }
        temper_engine_service::Note::ForgeStopped { number, why } => stopped_line(binary, number, why),
        temper_engine_service::Note::ForgeSetupStopped { number, why } => setup_stopped_line(binary, number, why),
        temper_engine_service::Note::ForgeNotConnected { number, why } => {
            format!("{binary} serve: forge {number}: not connected: {why}\n")
        }

        temper_engine_service::Note::GitHubLogOriginUnresolved { host } => {
            format!(
                "{binary} serve: GitHub log origin https://{}:443: unresolved; skipped\n",
                String::from_utf8_lossy(&host)
            )
        }
        temper_engine_service::Note::GitHubLogOriginsUnresolved => {
            format!("{binary} serve: GitHub log origins: none resolved; no job log will be fetched\n")
        }
        temper_engine_service::Note::ForgeSignIn { number, client, why } => {
            use temper_engine_service::clients::forge_sign_in::Unserved;
            let why = match why {
                Unserved::NotKept => "no such forge is kept".into(),
                Unserved::NotForgejo => "it is not a Forgejo".into(),
                Unserved::Origin { origin } => format!(
                    "its browser origin {} is not client {}'s redirect_uri's",
                    String::from_utf8_lossy(&origin),
                    String::from_utf8_lossy(&client)
                ),
                Unserved::NoSource => "the temper sign-in source is not ready".into(),
                Unserved::NoRoom => "no room for its meta within clients.bundle_bytes".into(),
            };
            format!("{binary} serve: forge {number}: no sign-in through {binary}: {why}\n")
        }
        temper_engine_service::Note::ForgeOrigin { number, origin } => origin_line(binary, number, &origin),
        temper_engine_service::Note::ForgeUnkept { number, installation } => {
            format!("{binary} serve: forge {number}: installation {installation}: not kept\n")
        }
    }
}
// protocol/service-process.md, 1.3: names cannot introduce another operator line.
fn start_wait_line(binary: &str, words: &[u8]) -> String {
    let text = format!("{binary} serve: {}", String::from_utf8_lossy(words));
    let mut line = String::new();
    for ch in text.chars() {
        let part = if ch.is_control() { ch.escape_default().to_string() } else { ch.to_string() };
        if line.len().saturating_add(part.len()) > 1020 {
            line.push_str("...");
            break;
        }
        line.push_str(&part);
    }
    line.push('\n');
    line
}

// temper/docs/design/domain/worker/hosted-runs.md, 1.3: one bounded, escaped operator line.
fn run_refusal_line(binary: &str, diagnostic: temper_engine_service::RunRefusalDiagnostic) -> String {
    let temper_engine_service::RunRefusalDiagnostic { task, run, attempt, reason } = diagnostic;
    let (class, detail) = match &reason {
        jig_domain_host::Refusal::Version(mismatch) => {
            return run_detail_line(
                format!("{binary} serve: task {task} run {run} attempt {attempt} refused: "),
                &mismatch.reason(),
            );
        }
        jig_domain_host::Refusal::Untranslated(words) => {
            return format!(
                "{binary} serve: task {task} run {run} attempt {attempt} refused: {}\n",
                String::from_utf8_lossy(words),
            );
        }
        jig_domain_host::Refusal::Busy => (String::from("Busy"), b"".as_slice()),
        jig_domain_host::Refusal::Invalid(jig_domain_host::Invalid::Start { cause, why }) => {
            (format!("Invalid(Start({cause:?}))"), why.as_ref())
        }
        jig_domain_host::Refusal::Invalid(
            jig_domain_host::Invalid::Charter
            | jig_domain_host::Invalid::Transcript
            | jig_domain_host::Invalid::Grants
            | jig_domain_host::Invalid::Messages,
        ) => (format!("{reason:?}"), b"".as_slice()),
    };
    run_detail_line(format!("{binary} serve: task {task} run {run} attempt {attempt} refused {class}: "), detail)
}

fn landing_numbers(numbers: &[u64]) -> String {
    use std::fmt::Write as _;
    let mut line = String::new();
    for (index, number) in numbers.iter().enumerate() {
        if index != 0 {
            line.push_str(", ");
        }
        write!(line, "{number}").expect("format landing diagnostic numbers");
    }
    line
}

// temper/docs/design/domain/worker/checkouts-prepare.md, 12.4: keep the fields at the detail's head.
fn run_failure_line(
    binary: &str,
    task: u64,
    run: u64,
    attempt: u64,
    failure: jig_domain_host::Failure,
    detail: &[u8],
) -> String {
    if failure == jig_domain_host::Failure::Cancelled(jig_domain_host::Reason::Shutdown) {
        return format!("{binary} serve: task {task} run {run} attempt {attempt} suspended: its worker stopped\n");
    }
    run_detail_head_line(
        format!("{binary} serve: task {task} run {run} attempt {attempt} failed {failure:?}: "),
        detail,
    )
}

// temper/docs/design/domain/worker/jobs.md: the job's kind stays in the run line's detail.
fn job_line(binary: &str, diagnostic: temper_engine_service::JobDiagnostic) -> String {
    let temper_engine_service::JobDiagnostic { task, run, attempt, kind, ending } = diagnostic;
    let prefix = format!("{binary} serve: task {task} run {run} attempt {attempt} ");
    let (class, detail) = job_ending(&ending);
    run_detail_line(format!("{prefix}{class}: job {kind}: "), detail)
}

fn job_ending(ending: &temper_engine_service::JobDiagnosticEnding) -> (String, &[u8]) {
    use jig_domain_host::{JobInvalid, JobRefusal};
    use temper_engine_service::JobDiagnosticEnding;
    match ending {
        JobDiagnosticEnding::Failed { failure, detail } => (format!("failed {failure:?}"), detail),
        JobDiagnosticEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why })) => {
            (String::from("refused Invalid(Start)"), why)
        }
        JobDiagnosticEnding::Refused(JobRefusal::Untranslated(words)) => (String::from("refused"), words),
        JobDiagnosticEnding::Refused(
            reason @ (JobRefusal::Busy
            | JobRefusal::Invalid(
                JobInvalid::Kind | JobInvalid::Spec | JobInvalid::Deadline | JobInvalid::Grants,
            )),
        ) => (format!("refused {reason:?}"), b""),
    }
}

fn run_detail_head_line(line: String, detail: &[u8]) -> String {
    escaped_detail_line(line, detail.get(..4096).unwrap_or(detail))
}

fn run_detail_line(line: String, detail: &[u8]) -> String {
    let from = detail.len().saturating_sub(4096);
    escaped_detail_line(line, detail.get(from..).expect("bounded diagnostic tail"))
}

fn escaped_detail_line(mut line: String, detail: &[u8]) -> String {
    use std::fmt::Write as _;
    for byte in detail {
        for escaped in core::ascii::escape_default(*byte) {
            line.write_char(char::from(escaped)).expect("format escaped diagnostic");
        }
    }
    line.push('\n');
    line
}

fn origin_line(binary: &str, number: u16, origin: &skein_forge::github::redirect::Origin) -> String {
    let scheme = if origin.tls { "https" } else { "http" };
    format!(
        "{binary} serve: forge {number}: refused job-log origin {scheme}://{}:{}\n",
        String::from_utf8_lossy(&origin.host),
        origin.port
    )
}
fn set_up_refused_line(binary: &str, number: u16, why: temper_engine_domain_forge::SetUpRefusal) -> String {
    use temper_engine_domain_forge::SetUpRefusal;
    let why = match why {
        SetUpRefusal::NotUnderTheBox => "not under the box",
        SetUpRefusal::NotOffered => "not offered",
        SetUpRefusal::Busy => "busy",
        SetUpRefusal::Invalid => "invalid: the organization, repository or project name is empty or too long",
    };
    format!("{binary} serve: forge {number}: set up refused: {why}\n")
}

fn setup_stopped_line(binary: &str, number: u16, why: temper_engine_service::SetupStopped) -> String {
    use temper_engine_service::{SetupPlan, SetupStopped, SetupWhy};
    let why = match why {
        SetupStopped::NotPrepared => format!("prepare it with `{binary} forge prepare`"),
        SetupStopped::Missing => "setup host missing".into(),
        SetupStopped::Busy => "setup host busy".into(),
        SetupStopped::Removal { path, errno } => {
            format!("removing the handover: {}: errno {errno}", super::path(&path).display())
        }
        SetupStopped::Unreadable { path, why } => {
            format!("the handover is unreadable: {}: {why:?}", super::path(&path).display())
        }
        SetupStopped::Stopped { plan, step } => {
            let reason = match step.why {
                SetupWhy::Refused(status) => format!("refused {status}"),
                why @ (SetupWhy::Missing
                | SetupWhy::NotAdmin
                | SetupWhy::BotIsAdmin
                | SetupWhy::Unmarked
                | SetupWhy::OtherUser
                | SetupWhy::OtherApp
                | SetupWhy::KeyForm
                | SetupWhy::Unknown
                | SetupWhy::Busy
                | SetupWhy::Crowded
                | SetupWhy::Down
                | SetupWhy::Taken
                | SetupWhy::Empty
                | SetupWhy::TimedOut
                | SetupWhy::Unexpected) => format!("{why:?}"),
            };
            let action = match plan {
                SetupPlan::Reconcile => "reconciling",
                SetupPlan::TakeOver => "taking the handover",
                SetupPlan::Adopt => "adopting",
                SetupPlan::Create => "making",
            };
            format!("{action} stopped at {:?}: {reason}", step.stage)
        }
    };
    format!("{binary} serve: forge {number}: Not connected: {why}\n")
}

fn listener_failed_line(
    binary: &str,
    listener: temper_engine_service::Listening,
    address: Option<skein_io::Addr>,
    error: Option<skein_io::Error>,
) -> String {
    let name = match listener {
        temper_engine_service::Listening::Clients => "client",
        temper_engine_service::Listening::Workers => "worker",
        temper_engine_service::Listening::Webhooks => "webhooks",
    };
    let address = match address {
        Some(skein_io::Addr::Ip(socket)) => socket.to_string(),
        Some(skein_io::Addr::InProcess(name)) => String::from_utf8_lossy(name.as_bytes()).into_owned(),
        None => "unknown".into(),
    };
    let why = match error {
        Some(skein_io::Error::Refused) => "refused",
        Some(skein_io::Error::Unreachable) => "unreachable",
        Some(skein_io::Error::TimedOut) => "timed out",
        Some(skein_io::Error::Reset) => "reset",
        Some(skein_io::Error::Busy) => "busy",
        Some(skein_io::Error::Reach(_) | skein_io::Error::Other) => "other",
        None => "closed",
    };
    format!("{binary} serve: {name} listener {address} failed: {why}\n")
}

fn opening_reason(code: u16) -> String {
    match code {
        1 => "version".into(),
        2 => "limits".into(),
        3 => "framing".into(),
        code => refusal(temper_engine_service::Refusal::Peer(code)),
    }
}

fn loss_refusal(code: u16) -> String {
    refusal(match code {
        2 => temper_engine_service::Refusal::Terms,
        3 => temper_engine_service::Refusal::Malformed,
        code => temper_engine_service::Refusal::Peer(code),
    })
}

fn worker_record(kind: jig_protocol_worker_wire::Kind) -> &'static str {
    use jig_protocol_worker_wire::Kind;
    match kind {
        Kind::Hello => "hello",
        Kind::Turn => "turn",
        Kind::Part => "part",
        Kind::PartKept => "part-kept",
        Kind::PartBusy => "part-busy",
        Kind::Call => "call",
        Kind::Answer => "answer",
        Kind::Fact => "fact",
        Kind::Unauthorized => "unauthorized",
        Kind::Usage => "usage",
        Kind::UsageRead => "usage-read",
        Kind::ReadUsage => "read-usage",
        Kind::Limited => "limited",
        Kind::AccountRefused => "account-refused",
        Kind::Assignment => "assignment",
        Kind::Message => "message",
        Kind::Relayed => "relayed",
        Kind::TurnKept => "turn-kept",
        Kind::TurnBusy => "turn-busy",
        Kind::Acknowledge => "acknowledge",
        Kind::Cancel => "cancel",
        Kind::Grant => "grant",
        Kind::Sources => "sources",
        Kind::Check => "check",
        Kind::ReadList => "read-list",
        Kind::ListRead => "list-read",
        Kind::Draining => "draining",
        Kind::SignOut => "sign-out",
        Kind::Logins => "logins",
        Kind::SignedOut => "signed-out",
        Kind::Job => "job",
        Kind::JobAnswer => "job-answer",
        Kind::Figures => "figures",
        Kind::Ask => "ask",
        Kind::Pause => "pause",
        Kind::Paused => "paused",
        Kind::Restarting => "restarting",
        Kind::Asked => "asked",
    }
}

fn worker_name(worker: Option<&[u8]>) -> String {
    match worker {
        Some(name) => String::from_utf8_lossy(name).into_owned(),
        None => "-".into(),
    }
}

fn engine_worker_loss(why: temper_engine_service::EngineWorkerLoss) -> String {
    use temper_engine_service::EngineWorkerLoss as Loss;
    match why {
        Loss::Silence => "silence".into(),
        Loss::Stall => "stall".into(),
        Loss::Full => "full".into(),
        Loss::Deadline => "deadline".into(),
        Loss::Stream => "stream".into(),
        Loss::Framing => "framing".into(),
        Loss::Refused { reason } => format!("refused {}", loss_refusal(reason)),
        Loss::OutputRoom { engine, worker } => {
            format!(
                "engine io.output {engine} bytes differs from worker io.output {worker} bytes; change io.output to agree"
            )
        }
        Loss::RefusedPeer { reason } => format!("refused-by-peer {}", loss_refusal(reason)),
        Loss::NoRoom => "no-room".into(),
        Loss::Stopping => "stopping".into(),
    }
}

fn refusal(reason: temper_engine_service::Refusal) -> String {
    use temper_engine_service::WorkerRefusalReason as Reason;
    match reason {
        temper_engine_service::Refusal::Terms => "terms".into(),
        temper_engine_service::Refusal::Malformed => "malformed".into(),
        temper_engine_service::Refusal::Peer(code) => match Reason::from_code(code) {
            Some(Reason::Unauthorized) => "unauthorized".into(),
            Some(Reason::Wires) => "wires".into(),
            Some(Reason::Busy) => "busy".into(),
            Some(Reason::Replaced) => "replaced".into(),
            Some(Reason::NotAdmitted) => "not-admitted".into(),
            Some(Reason::Stopping) => "stopping".into(),
            Some(Reason::Starting) => "starting".into(),
            None => format!("code-{code}"),
        },
    }
}
fn stopped_line(binary: &str, number: u16, why: skein_forge_connection::Stopped) -> String {
    use skein_forge_connection::{StartRead, Stopped, Value};
    match why {
        Stopped::Invalid { value } => {
            let member = match value {
                Value::Credential | Value::Token => "api_token",
                Value::User => "user",
                Value::Key => "app_key",
                Value::Secret => "webhook_secret",
            };
            format!("{binary} serve: forge {number}: {member}: invalid\n")
        }
        Stopped::KeyRefused => format!("{binary} serve: forge {number}: app_key: refused\n"),
        Stopped::TokenRefused { installation, status } => {
            format!(
                "{binary} serve: forge {number}: /app/installations/{installation}/access_tokens: status {status}\n"
            )
        }
        Stopped::OtherUser { user } => {
            format!("{binary} serve: forge {number}: start answered as user {user}\n")
        }
        Stopped::Refused { read, status } => {
            let path = match read {
                StartRead::Installations => "/app/installations",
                StartRead::Mint => "/app/installations/{id}/access_tokens",
                StartRead::Viewer => "/graphql viewer",
                StartRead::User => "/api/v1/user",
                StartRead::ApiSettings => "/api/v1/settings/api",
            };
            format!("{binary} serve: forge {number}: {path}: status {status}\n")
        }
    }
}
fn start_phase(phase: jig_domain_core::start::Phase) -> String {
    use jig_domain_core::start::Phase;
    match phase {
        Phase::LoadCore => "core restore".into(),
        Phase::LoadInbox => "inbox census".into(),
        Phase::RestoreConnector { connector } => format!("connector {connector} restore"),
        Phase::Link => "link".into(),
        Phase::Sweep => "sweep".into(),
        Phase::Wires => "wires".into(),
        Phase::Switch => "store switch".into(),
        Phase::AdoptRuns => "run adoption".into(),
        Phase::Open => "opening decisions".into(),
    }
}

fn problems(binary: &str, found: &temper_engine_service::startup::Problems) -> String {
    let lines = temper_engine_service::startup::lines(found);
    super::prefixed(&format!("{binary} serve"), &String::from_utf8_lossy(&lines))
}

#[cfg(test)]
mod tests;
