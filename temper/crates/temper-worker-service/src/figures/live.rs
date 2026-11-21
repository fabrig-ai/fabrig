//! Plan a whole live take over the running worker (temper/docs/design/protocol/configuration/figures.md, 3.2).
use super::{figure_path, problems, read};
use crate::startup::{Problems, What, lines, worker};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use jig_protocol_worker_link::Timers;
use jig_protocol_worker_wire::{Figures, Limit};
use skein_lib::{List, Map, bytes};
use temper_service_core::limits;
use temper_worker_domain as root;
use temper_worker_protocol_git as git;

pub(crate) struct Plan {
    pub(crate) waiting: List<Limit>,
    pub(crate) live: Result<Option<Take>, Box<[u8]>>,
}
pub(crate) struct Take {
    pub(crate) figures: Figures,
    pub(crate) names: List<Limit>,
    pub(crate) root: root::Limits,
    pub(crate) git: git::Limits,
    pub(crate) timers: Option<Timers>,
}

struct Changes {
    root: root::Limits,
    git: git::Limits,
    retime: bool,
    live: List<Limit>,
    waiting: List<Limit>,
    refused: Problems,
}
impl Changes {
    /// Select the holders that read their figures at each use (limit-settings.md, 5.2).
    fn moved(&mut self, name: &[u8], value: u64, read: Result<(), What>, jig: &config::Limits, own: &limits::Shared) {
        match name {
            b"checkout.preparations" => self.root.checkout.preparations = own.checkout.preparations,
            b"checkout.message_bytes" => self.root.checkout.message_bytes = own.checkout.message_bytes,
            b"checkout.remote_timeout_ms" => self.root.checkout.remote_timeout = own.checkout.remote_timeout,
            b"checkout.local_timeout_ms" => self.root.checkout.local_timeout = own.checkout.local_timeout,
            b"checkout.reds_read_ms" => self.root.checkout.reds_read_ms = own.checkout.reds_read_ms,
            b"checkout.rerun_processes" => self.root.checkout.rerun_processes = own.checkout.rerun_processes,
            b"hub.event_bytes" => self.root.host.event_bytes = jig.hub.event_bytes,
            b"hub.accounts" => self.root.host.accounts = jig.hub.accounts,
            b"hub.turn_queue_bytes" => self.root.host.turn_queue_bytes = jig.hub.turn_queue_bytes,
            b"git_component.stdout_bytes" => self.git.stdout_bytes = own.git_component.stdout_bytes,
            b"git_component.stderr_bytes" => self.git.stderr_bytes = own.git_component.stderr_bytes,
            b"git_component.read_bytes" => self.git.read_bytes = own.git_component.read_bytes,
            b"git_component.sweep_entries" => self.git.sweep_entries = own.git_component.sweep_entries,
            b"git_component.sweep_bytes" => self.git.sweep_bytes = own.git_component.sweep_bytes,
            b"link.hello_ms" => {} // The engine alone reads this live figure.
            remaining => {
                if is_timer(remaining) {
                    self.retime = true;
                } else {
                    self.wait(remaining, value);
                    return;
                }
            }
        }
        if let Err(what) = read {
            self.refused.push(figure_path(name, value), what);
        }
        self.live.push(Limit { name: bytes::copy_of(name), value }).expect("uncapped live figures");
    }
    fn wait(&mut self, name: &[u8], value: u64) {
        self.waiting.push(Limit { name: bytes::copy_of(name), value }).expect("uncapped waiting figures");
    }
    fn agents(&mut self, in_force: &Figures, heard: &Figures) {
        let before = named(&in_force.agents.profile);
        let after = named(&heard.agents.profile);
        for (name, value) in &after {
            if before.get(name) != Some(value) {
                self.wait(&bytes::concat(&[b"agents.profile.", name]), *value);
            }
        }
        for (name, _) in &before {
            if after.get(name).is_none() {
                self.wait(b"agents.profile", u64::from(heard.agents.profile.len()));
                break;
            }
        }
        if in_force.agents.endpoints != heard.agents.endpoints {
            self.wait(b"agents.endpoints", u64::from(heard.agents.endpoints.len()));
        }
    }
}

fn is_timer(name: &[u8]) -> bool {
    for timer in [
        b"link.handshake_ms".as_slice(),
        b"link.stall_ms",
        b"link.backoff_ms",
        b"link.ceiling_ms",
        b"link.proved_ms",
        b"link.busy_ms",
    ] {
        if timer == name {
            return true;
        }
    }
    false
}

fn named(limits: &List<Limit>) -> Map<&[u8], u64> {
    let mut names = Map::with_capacity(limits.len());
    for limit in limits {
        // As at startup, a later entry for a name replaces an earlier one.
        let _previous = names.insert(limit.name.as_ref(), limit.value).expect("counted figures");
    }
    names
}

pub(crate) fn plan(in_force: &Figures, heard: &Figures, root: &root::Limits, git: &git::Limits) -> Plan {
    let before = named(&in_force.limits);
    let after = named(&heard.limits);
    let mut changes = Changes {
        root: root.clone(),
        git: *git,
        retime: false,
        live: List::uncapped(),
        waiting: List::uncapped(),
        refused: Problems::empty(),
    };
    let stock = config::stock();
    let mut jig = stock;
    let mut own = limits::stock();
    for row in &config::names::NAMES {
        let name = row.name.as_bytes();
        let default = (row.value)(&stock);
        let next = after.get(&name).copied().unwrap_or(default);
        let moved = next != before.get(&name).copied().unwrap_or(default);
        if moved || is_timer(name) {
            let read = read(&mut jig, &mut own, name, next);
            if moved {
                changes.moved(name, next, read, &jig, &own);
            }
        }
    }
    for &(name, default) in &limits::list(&own) {
        let name = name.as_bytes();
        let next = after.get(&name).copied().unwrap_or(default);
        if next != before.get(&name).copied().unwrap_or(default) {
            let read = read(&mut jig, &mut own, name, next);
            changes.moved(name, next, read, &jig, &own);
        }
    }
    // The shared parser also recognizes temper's other names, without giving
    // them a worker-side value. Keep unknown names quiet as at startup.
    for (name, value) in &after {
        if limits::known(name) && !limits::shared_name(name) && before.get(name) != Some(value) {
            changes.wait(name, *value);
        }
    }
    if in_force.places != heard.places {
        changes.wait(b"places", u64::from(heard.places));
    }
    changes.agents(in_force, heard);
    let timers = if changes.retime {
        let link = jig.link;
        Some(Timers {
            handshake: link.handshake,
            stall: link.stall,
            backoff: link.backoff,
            ceiling: link.ceiling,
            proved: link.proved,
            busy: link.busy,
        })
    } else {
        None
    };
    let Changes { root, git, live, waiting, refused, .. } = changes;
    Plan { waiting, live: checked(in_force, live, refused, root, git, timers) }
}

fn checked(
    in_force: &Figures,
    live: List<Limit>,
    mut refused: Problems,
    root: root::Limits,
    git: git::Limits,
    timers: Option<Timers>,
) -> Result<Option<Take>, Box<[u8]>> {
    if live.is_empty() {
        return Ok(None);
    }
    if !refused.is_empty() {
        return Err(first_line(&refused));
    }
    let mut figures = Figures { places: in_force.places, agents: in_force.agents.clone(), limits: List::uncapped() };
    let moved = named(&live);
    for limit in &in_force.limits {
        if moved.get(&limit.name.as_ref()).is_none() {
            figures.limits.push(limit.clone()).expect("uncapped figures in force");
        }
    }
    for limit in &live {
        figures.limits.push(limit.clone()).expect("uncapped figures in force");
    }
    if git::check(&git, &root.checkout).is_err() {
        if let Some((needs, of)) = worker::checkout::stdout(&root.checkout, git.stdout_bytes) {
            refused.push(
                Path::root().member(b"git_component.stdout_bytes"),
                What::Short { bytes: u64::from(git.stdout_bytes), of, needs: u64::from(needs) },
            );
        } else {
            worker::git_rules(git, &root.checkout, &mut refused);
        }
    } else if !root::valid(&root) {
        // The running root contains the bounds of the pinned agent, not the
        // profile the engine heard for the next start.
        worker::worker(&root, &mut refused);
    } else {
        return Ok(Some(Take { figures, names: live, root, git, timers }));
    }
    if refused.is_empty() {
        return Err(bound_line());
    }
    Err(first_line(&problems(refused, &figures)))
}

/// A refusal for the generic construction check, including a refused link retime.
pub(crate) fn bound_line() -> Box<[u8]> {
    let mut refused = Problems::empty();
    refused.push(Path::root().member(b"the engine's figures"), What::Jig(config::What::Bound));
    first_line(&refused)
}
fn first_line(refused: &Problems) -> Box<[u8]> {
    let text = lines(refused);
    let end = bytes::find(&text, b"\n").expect("a startup problem ends with a line break");
    bytes::copy_of(text.get(..end).expect("position in the problem line"))
}
