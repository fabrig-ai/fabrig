//! The deployment is opened before its engine is sized (configuration/limit-settings.md, 3).
use crate::path;
use jig_domain_core::settings::Override;
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Queue, Time};
use skein_shell::{Clock, Kernel, Now, Wait};
use std::fmt::Write;
use temper_engine_service::{
    self as service,
    limit_settings::{LimitInputs, LimitSettings},
    store::Store,
};

#[derive(Debug)]
pub struct Held {
    pub jig: Box<[Override]>,
    pub temper: Box<[Override]>,
    lines: Vec<(Box<[u8]>, usize)>,
    pub line: String,
}
/// Presence suppresses both setting kinds, including an empty file.
pub fn limits_hold(data: &std::path::Path) -> Result<Option<Held>, String> {
    let path = data.join("limits-held");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(format!("{}: {why}", path.display())),
    };
    parse_held(&path, &text).map(Some)
}

/// Parse the held file when it is present, even if it is empty.
pub fn parse_held(path: &std::path::Path, text: &str) -> Result<Held, String> {
    let mut jig = Vec::new();
    let mut temper = Vec::new();
    let mut lines: Vec<(Box<[u8]>, usize)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let words: Vec<_> = line.split_whitespace().collect();
        let [name, value] = words.as_slice() else {
            return Err(format!("limits-held line {number}: expected NAME VALUE"));
        };
        let value = value
            .parse::<u64>()
            .map_err(|_| format!("limits-held line {number}: {name}: expected an unsigned figure"))?;
        if lines.iter().any(|(old, _)| old.as_ref() == name.as_bytes()) {
            return Err(format!("limits-held line {number}: {name}: named twice"));
        }
        let item =
            Override { name: name.as_bytes().into(), value, reason: b"operator's limits-held".as_slice().into() };
        if jig_protocol_config::names::find(name.as_bytes()).is_some() {
            jig.push(item);
        } else if service::limits::find(name.as_bytes()).is_some() {
            temper.push(item);
        } else {
            return Err(format!("limits-held line {number}: {name} is no limit of this release"));
        }
        lines.push((name.as_bytes().into(), number));
    }
    jig.sort_by(|left, right| left.name.cmp(&right.name));
    temper.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(Held {
        jig: jig.into_boxed_slice(),
        temper: temper.into_boxed_slice(),
        lines,
        line: format!(
            "limits held: {}; the limit settings are not taken; remove it and restart to take them",
            path.display()
        ),
    })
}
pub(crate) fn held_failure(why: &str, held: &Held) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        let name = line.split_once(' ').map_or(line, |(name, _)| name);
        let number = held
            .lines
            .iter()
            .find(|(found, _)| found.as_ref() == name.as_bytes())
            .map_or_else(|| held.lines.first().map_or(1, |(_, n)| *n), |(_, n)| *n);
        let line = line.strip_suffix("; put limits-held in the data directory to start at the stock").unwrap_or(line);
        writeln!(lines, "limits-held line {number}: {line}").expect("writing to a string");
    }
    lines
}

/// This is the production store-only loop. No root, children, listeners or
/// worker exist until it returns the resolved settings and the opened store.
pub fn open_limits(
    inputs: LimitInputs,
    models: &[jig_domain_core::Model],
    data: &std::path::Path,
    deployment: &[u8],
    user: u32,
    build_name: &'static str,
    owned: &mut Vec<Fd>,
) -> Result<(Store, LimitSettings), String> {
    let held = limits_hold(data)?;
    let mut store = Store::for_limits(&inputs, models).map_err(|why| text(&why))?;
    let root = skein_shell::make_root(path(deployment), 0o700)
        .map_err(|errno| format!("engine.deployment.directory: errno {errno}"))?;
    owned.push(root);
    store.build_name(build_name);
    store.adopt(root, user).map_err(|_| "store refused its deployment root")?;
    let mut kernel =
        skein_shell::Kernel::open(skein_shell::Config { operations: service::store::KERNEL_HANDOFF_ENTRIES })
            .map_err(|why| why.to_string())?;
    let clock = skein_shell::Clock::new();
    let mut io = Ring { kernel: &mut kernel, clock: &clock };
    let result = opened(&mut store, &mut io, Some(deployment));
    if let Err(why) = result {
        closed(&mut store, &mut io);
        return Err(why);
    }
    let state = resolved(&mut store, &mut io, inputs, held.as_ref(), &mut std::io::stderr())?;
    if let Some(held) = held {
        eprintln!("{}", held.line);
    }
    Ok((store, state))
}

/// The check's copy, opened by the store-only loop before its engine is sized.
#[derive(Debug)]
pub struct Opening {
    pub store: Store,
    /// The limits a start takes from the copy; none when the copy didn't open,
    /// its terminal kept in the store for the engine's report.
    pub settings: Option<LimitSettings>,
    /// Before the copy was made, for the rehearsal's time.
    pub began: skein_lib::Time,
}
/// The check opens its copy as a start opens its store, at the stock's figures, and
/// resolves the copy's two kinds, or `held`'s figures, as a start does
/// (configuration/limit-settings.md, 3). `adopt` gives the store the copy's roots.
/// The rehearsal's progress and terminals stay queued for its engine's report.
pub fn rehearse_limits(
    inputs: LimitInputs,
    models: &[jig_domain_core::Model],
    held: Option<&Held>,
    build_name: &'static str,
    adopt: impl FnOnce(&mut Store) -> Result<(), String>,
) -> Result<Opening, String> {
    let mut store = Store::for_limits(&inputs, models).map_err(|why| text(&why))?;
    store.build_name(build_name);
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    adopt(&mut store)?;
    let mut kernel =
        skein_shell::Kernel::open(skein_shell::Config { operations: service::store::KERNEL_HANDOFF_ENTRIES })
            .map_err(|why| why.to_string())?;
    rehearse_opening(
        store,
        &mut Ring { kernel: &mut kernel, clock: &clock },
        began,
        inputs,
        held,
        &mut std::io::stderr(),
    )
}

/// The kernel and clock beneath the store-only opening loop.
#[doc(hidden)]
pub trait LimitsIo {
    fn now(&self) -> Now;
    fn reap(&mut self, completions: &mut Queue<Complete>);
    fn in_flight(&self) -> u32;
    fn submit(&mut self, submissions: &mut Queue<Submit>, wait: Wait);
}

struct Ring<'a> {
    kernel: &'a mut Kernel,
    clock: &'a Clock,
}

impl LimitsIo for Ring<'_> {
    fn now(&self) -> Now {
        self.clock.now()
    }
    fn reap(&mut self, completions: &mut Queue<Complete>) {
        self.kernel.reap(completions);
    }
    fn in_flight(&self) -> u32 {
        self.kernel.in_flight()
    }
    fn submit(&mut self, submissions: &mut Queue<Submit>, wait: Wait) {
        self.kernel.submit(submissions, wait);
    }
}

/// Open a rehearsal store that has adopted its source and scratch, then resolve the copy's limits.
#[doc(hidden)]
pub fn rehearse_opening(
    mut store: Store,
    io: &mut impl LimitsIo,
    began: Time,
    inputs: LimitInputs,
    held: Option<&Held>,
    diagnostics: &mut dyn std::io::Write,
) -> Result<Opening, String> {
    let settings = match opened(&mut store, io, None) {
        Ok(true) => Some(resolved(&mut store, io, inputs, held, diagnostics)?),
        Ok(false) => None,
        Err(why) => {
            closed(&mut store, io);
            return Err(why);
        }
    };
    Ok(Opening { store, settings, began })
}

/// Read, resolve and install the opened store's limit settings; a held pair replaces both kinds.
fn resolved(
    store: &mut Store,
    io: &mut impl LimitsIo,
    inputs: LimitInputs,
    held: Option<&Held>,
    diagnostics: &mut dyn std::io::Write,
) -> Result<LimitSettings, String> {
    let over = held.map(|held| (held.jig.as_ref(), held.temper.as_ref()));
    let state = match store.resolve_settings(inputs, over) {
        Ok(state) => state,
        Err(why) => {
            closed(store, io);
            return Err(limit_failure(&why, held));
        }
    };
    for left in &state.in_force.left_out {
        writeln!(diagnostics, "{}", left_out_line(left)).expect("printing to stderr failed");
    }
    Ok(state)
}
fn limit_failure(why: &service::startup::Problems, held: Option<&Held>) -> String {
    let why = text(why);
    match held {
        Some(held) => held_failure(&why, held),
        None => why,
    }
}
fn left_out_line(left: &service::startup::LeftOut) -> String {
    format!(
        "limits: {} names {}, which is no limit of this release; it is left out",
        String::from_utf8_lossy(left.kind.name()),
        String::from_utf8_lossy(&left.name)
    )
}
fn text(why: &service::startup::Problems) -> String {
    String::from_utf8_lossy(&service::startup::lines(why)).into_owned()
}
/// Until the store opens, or, for a rehearsal, ends at a terminal. A start, naming its
/// `deployment`, refuses in its own words; a rehearsal has none and keeps its terminals and
/// its jobs' progress for its engine's report. Whether the store opened.
fn opened(store: &mut Store, io: &mut impl LimitsIo, deployment: Option<&[u8]>) -> Result<bool, String> {
    let mut completions = Queue::uncapped();
    let mut ended = false;
    loop {
        io.reap(&mut completions);
        while let Some(complete) = completions.pop() {
            store.complete(complete);
        }
        let Now { now, wall } = io.now();
        store.clock(now, wall);
        store.iterate();
        let mut keep = Queue::uncapped();
        while let Some(event) = store.events().pop() {
            match event {
                jig_protocol_store::Event::Opened { installation, pending, .. } => {
                    store.opened_limits(installation, pending);
                }
                jig_protocol_store::Event::OpenRefused { owner, why } => {
                    if let Some(deployment) = deployment {
                        let (line, _) = service::refused_start(&why, deployment);
                        return Err(String::from_utf8_lossy(&line).into_owned());
                    }
                    ended = true;
                    keep.push(jig_protocol_store::Event::OpenRefused { owner, why });
                }
                event @ jig_protocol_store::Event::Job { .. } => {
                    if !store.opening_job(event) {
                        return Err("store opening job failed".into());
                    }
                }
                jig_protocol_store::Event::Corrupt { owner } => {
                    if deployment.is_some() {
                        return Err("store opening failed".into());
                    }
                    ended = true;
                    keep.push(jig_protocol_store::Event::Corrupt { owner });
                }
                event @ jig_protocol_store::Event::Undecodable { .. } => {
                    ended |= deployment.is_none();
                    keep.push(event);
                }
                event @ jig_protocol_store::Event::Ran { .. } => {
                    if deployment.is_none() {
                        keep.push(event);
                    }
                }
                event @ (jig_protocol_store::Event::Restored { .. }
                | jig_protocol_store::Event::RestoreRefused { .. }
                | jig_protocol_store::Event::Checkpointed { .. }
                | jig_protocol_store::Event::CheckpointHeld { .. }
                | jig_protocol_store::Event::CheckpointRemoved { .. }
                | jig_protocol_store::Event::CheckpointRefused { .. }
                | jig_protocol_store::Event::Rehearsing { .. }
                | jig_protocol_store::Event::Decoded { .. }
                | jig_protocol_store::Event::Back { .. }
                | jig_protocol_store::Event::BackRefused { .. }
                | jig_protocol_store::Event::Closed { .. }
                | jig_protocol_store::Event::Switched { .. }
                | jig_protocol_store::Event::Committed { .. }
                | jig_protocol_store::Event::Census { .. }
                | jig_protocol_store::Event::Failed { .. }
                | jig_protocol_store::Event::FrameTooLarge { .. }
                | jig_protocol_store::Event::Page { .. }
                | jig_protocol_store::Event::PageRefused { .. }
                | jig_protocol_store::Event::Loaded { .. }
                | jig_protocol_store::Event::Read { .. }
                | jig_protocol_store::Event::Got { .. }
                | jig_protocol_store::Event::Turns { .. }
                | jig_protocol_store::Event::Unread { .. }
                | jig_protocol_store::Event::SecretsNotReady { .. }
                | jig_protocol_store::Event::Kept { .. }
                | jig_protocol_store::Event::Carried { .. }
                | jig_protocol_store::Event::Stale { .. }
                | jig_protocol_store::Event::Erased { .. }
                | jig_protocol_store::Event::Secret { .. }
                | jig_protocol_store::Event::SecretsLoaded { .. }) => keep.push(event),
            }
        }
        while let Some(event) = keep.pop() {
            store.events().push(event);
        }
        if (store.opened() || ended) && io.in_flight() == 0 && store.submissions().is_empty() {
            return Ok(store.opened());
        }
        let wait = if store.work_pending(now) {
            skein_shell::Wait::No
        } else {
            store.next_deadline().map_or(skein_shell::Wait::Forever, skein_shell::Wait::Until)
        };
        io.submit(store.submissions(), wait);
    }
}
pub fn close_limits_store(store: &mut Store) -> Result<(), String> {
    let mut kernel =
        skein_shell::Kernel::open(skein_shell::Config { operations: service::store::KERNEL_HANDOFF_ENTRIES })
            .map_err(|why| why.to_string())?;
    closed(store, &mut Ring { kernel: &mut kernel, clock: &Clock::new() });
    Ok(())
}
fn closed(store: &mut Store, io: &mut impl LimitsIo) {
    store.close();
    let mut completions = Queue::uncapped();
    loop {
        io.reap(&mut completions);
        while let Some(complete) = completions.pop() {
            store.complete(complete);
        }
        let Now { now, wall } = io.now();
        store.clock(now, wall);
        store.iterate();
        while store.events().pop().is_some() {}
        if store.closed() && io.in_flight() == 0 {
            break;
        }
        let wait = if store.work_pending(now) {
            skein_shell::Wait::No
        } else {
            store.next_deadline().map_or(skein_shell::Wait::Forever, skein_shell::Wait::Until)
        };
        io.submit(store.submissions(), wait);
    }
}

#[cfg(test)]
mod tests {
    mod startup;
    use super::*;
    #[test]
    fn limits_held_is_presence_based_and_remembers_line_numbers() {
        let scratch = skein_scratch::Scratch::new("limits-held");
        assert!(limits_hold(scratch.path()).expect("held fixture").is_none());
        let path = scratch.path().join("limits-held");
        std::fs::write(&path, "").expect("held fixture");
        let held = limits_hold(scratch.path()).expect("held fixture").expect("held fixture");
        assert!(held.jig.is_empty() && held.temper.is_empty());
        assert_eq!(
            held.line,
            format!(
                "limits held: {}; the limit settings are not taken; remove it and restart to take them",
                path.display()
            )
        );
        std::fs::write(&path, "live_goals 20\ntasks.tasks 1024\n").expect("held fixture");
        let held = limits_hold(scratch.path()).expect("held fixture").expect("held fixture");
        assert_eq!(held.jig[0].value, 1024);
        assert_eq!(held.temper[0].value, 20);
        assert_eq!(
            held_failure(
                "tasks.tasks (setting limits): outside its bound; put limits-held in the data directory to start at the stock\n",
                &held
            ),
            "limits-held line 2: tasks.tasks (setting limits): outside its bound\n"
        );
        std::fs::write(&path, "tasks.tasks 1024\nlive_goals 101\n").expect("held fixture");
        let held = limits_hold(scratch.path()).expect("held fixture").expect("held fixture");
        assert_eq!(
            held_failure(
                "live_goals (setting temper-limits): 101, past its bound of 100, the most live goals the store is sized for; put limits-held in the data directory to start at the stock\n",
                &held,
            ),
            "limits-held line 2: live_goals (setting temper-limits): 101, past its bound of 100, the most live goals the store is sized for\n"
        );
        std::fs::write(&path, "live_goals 20\nforge.gone 16\n").expect("held fixture");
        assert_eq!(
            limits_hold(scratch.path()).expect_err("refused held figure"),
            "limits-held line 2: forge.gone is no limit of this release"
        );
    }
    #[test]
    fn a_removed_name_has_the_release_warning_words() {
        assert_eq!(
            left_out_line(&service::startup::LeftOut {
                kind: service::startup::LimitKind::Temper,
                name: b"forge.gone".as_slice().into()
            }),
            "limits: temper-limits names forge.gone, which is no limit of this release; it is left out"
        );
    }
}
