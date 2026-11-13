//! Engine and named workers on one ring (protocol/cluster.md, 6).
use crate::{calls, fixtures, forge, llm, service::ServiceHost, world};
use skein_io::kernel::{Complete, Done, Op};
use std::{fmt::Write as _, net::SocketAddr, os::unix::fs::DirBuilderExt, path::Path};

/// Script hold released once every started worker has received an assignment.
pub const PLACED: &[u8] = b"cluster-placed";

pub struct Cluster {
    ran: crate::Ran,
    _scratch: Vec<crate::Scratch>,
    _git: Option<temper_fake_git_hosted::Typed>,
}
impl std::ops::Deref for Cluster {
    type Target = crate::Ran;
    fn deref(&self) -> &Self::Target {
        &self.ran
    }
}
impl Cluster {
    /// Engine starts refused before a service could listen.
    #[must_use]
    pub fn failed_starts(&self) -> &[String] {
        self.ran.service().failed_starts()
    }
    /// The engine's peak tries per class, retained before a successful park clears them.
    #[must_use]
    pub fn tries(&self, number: u64) -> jig_domain_tasks::Tries {
        *self.ran.service().tries.get(&number).expect("observed task")
    }
    #[must_use]
    pub fn worker(&self, name: &str) -> &ServiceHost {
        self.ran
            .outcome
            .procs
            .iter()
            .filter_map(|proc| if let world::Kind::Service(host) = &proc.kind { Some(host.as_ref()) } else { None })
            .find(|host| matches!(&host.cluster, Some(Shape::Worker { name: worker, .. }) if *worker == name))
            .expect("worker root")
    }
}
#[must_use]
pub fn run_cluster(
    workers: &[&'static str],
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<calls::Step>,
) -> Cluster {
    run_cluster_with_grace(workers, skein_lib::Duration::from_millis(10), recipe, steps)
}

/// Run the cluster with a worker's engine-away bound; link deadlines stay above patience.
#[must_use]
pub fn run_cluster_with_grace(
    workers: &[&'static str],
    grace: skein_lib::Duration,
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<calls::Step>,
) -> Cluster {
    let git = temper_worker_checkout_world::hosted::scenario().transport;
    crate::git::share(Some(git.clone()));
    run_with(workers, Some(git), |scratch| {
        crate::Outcome::Checked(world::run_cluster(workers, grace, scratch, recipe, steps))
    })
}

/// The cluster's services and hosted git share one forge, as in `run_forge`.
#[must_use]
pub fn run_cluster_forge(
    workers: &[&'static str],
    late: &[&str],
    fixture: forge::Fixture,
    scripts: fn(SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]>,
    rules: fn() -> Box<[Box<[skein_fake_llm_domain::api::Rule]>]>,
    steps: impl FnOnce() -> Vec<calls::Step>,
) -> Cluster {
    assert!(late.iter().all(|name| workers.contains(name)), "configured late workers");
    run_with(workers, None, |scratch| {
        crate::Outcome::Forge(world::run_cluster_forge(workers, late, scratch, fixture, scripts, rules, steps))
    })
}

fn run_with(
    workers: &[&'static str],
    git: Option<temper_fake_git_hosted::Typed>,
    run: impl FnOnce(&[crate::Scratch]) -> crate::Outcome,
) -> Cluster {
    assert!(!workers.is_empty(), "cluster workers");
    for (position, name) in workers.iter().enumerate() {
        assert!(!workers[..position].contains(name), "distinct worker names");
    }
    let scratch: Vec<_> = std::iter::once("engine").chain(workers.iter().copied()).map(crate::Scratch::new).collect();
    let outcome = run(&scratch);
    crate::git::share(None);
    let mut scratch = scratch.into_iter();
    let ran = crate::checked(crate::Ran { outcome, scratch: scratch.next().expect("engine scratch") });
    let cluster = Cluster { ran, _scratch: scratch.collect(), _git: git };
    for name in workers {
        assert_eq!(
            cluster.worker(name).end(),
            Some(&crate::End::Worker(temper_service::worker_alone::End::Drained)),
            "{name} drained"
        );
    }
    cluster
}

pub(crate) enum Shape {
    Engine { workers: Vec<&'static str>, grace: skein_lib::Duration },
    Worker { name: &'static str, engine: Option<SocketAddr>, started: bool, grace: skein_lib::Duration },
}
impl Shape {
    pub(crate) fn ready(&self) -> bool {
        !matches!(self, Self::Worker { engine: None, .. } | Self::Worker { started: false, .. })
    }
    pub(crate) fn write(&self, path: &Path, standing: bool) {
        use serde_json::json;
        let config = path.join("config");
        let data = path.join("data");
        if standing {
            fixtures::standing_chats(&data).expect("standing cluster chats");
        }
        for name in ["config", "cache"] {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .recursive(true)
                .create(path.join(name))
                .expect("cluster directories");
        }
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(data.join("temper.json")).expect("base document"))
                .expect("base JSON");
        // Link deadlines exceed PATIENCE, so load cannot lose a link or run in this story.
        // A restart closes the socket explicitly rather than waiting on a link timer.
        // The stop bound stays below fleet grace (protocol/cluster.md, 3).
        let ping_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000) + 1;
        let silence_ms = 3 * ping_ms;
        let (Self::Engine { grace, .. } | Self::Worker { grace, .. }) = self;
        let grace_ms = grace.as_millis();
        let mut agent: serde_json::Value =
            serde_json::from_slice(&std::fs::read(data.join("agent.json")).expect("agent document"))
                .expect("agent JSON");
        // Preserve the story's supervision policy; its stop bound is derived below.
        // Cancellation must leave the held peer time to release its response.
        let [delivery_ms, save_ms] = [5_u64, 5];
        let graces = jig_protocol_config::Graces {
            grace: *grace,
            // A held model call is cancelled promptly even in a long engine-away story.
            drain: skein_lib::Duration::from_millis(10),
            agent_stop: agent_stop_bound(&agent),
            delivery: skein_lib::Duration::from_millis(delivery_ms),
            save: skein_lib::Duration::from_millis(save_ms),
        };
        let agent_stop_ms = graces.agent_stop.as_millis();
        let stop_bound_ms = jig_protocol_config::stop_bound(
            &graces,
            skein_lib::Duration::from_millis(ping_ms),
            skein_lib::Duration::from_millis(silence_ms),
        )
        .expect("cluster stop bound fits a duration")
        .as_millis();
        match self {
            Self::Engine { workers, .. } => {
                document["workers"] = json!([]);
                document["application"]["workers"] = json!([]);
                document["engine"]["hosts"]["listener"] = json!([{"socket":"127.0.0.1:0"}]);
                document["engine"]["hosts"]["workers"] =
                    workers.iter().map(|name| json!({"name":name,"digest":format!("secrets/{name}.digest")})).collect();
                // Every configured position admitted: the small profile admits two workers.
                document["engine"]["limits"]["overrides"]["fleet.workers"] = workers.len().into();
                // One attempt per one-slot worker: the fleet counts attempts across the whole fleet.
                document["engine"]["limits"]["overrides"]["fleet.attempts"] = workers.len().into();
                document["engine"]["limits"]["overrides"]["fleet.grace_ms"] = (stop_bound_ms + 1).into();
                document["engine"]["limits"]["overrides"]["hub.turns"] = 1.into();
                document["engine"]["limits"]["overrides"]["link.ping_ms"] = ping_ms.into();
                document["engine"]["limits"]["overrides"]["link.silence_ms"] = silence_ms.into();
                for name in workers {
                    let secret = format!("{name}-fixture-secret");
                    let mut digest = String::new();
                    for word in skein_io::digest::digest(secret.as_bytes()).0 {
                        write!(digest, "{word:016x}").expect("fixture digest hex");
                    }
                    fixtures::private(&data.join(format!("secrets/{name}.digest")), digest.as_bytes())
                        .expect("fixture digest");
                }
            }
            Self::Worker { name, engine, .. } => {
                let worker = document["workers"].as_array_mut().expect("workers").first_mut().expect("one worker");
                // One retained turn, one held event, and no program-backed LLM source.
                for (limit, size) in [("hub.turns", 1), ("hub.held", 1), ("hub.program_sources", 0)] {
                    worker["limits"]["overrides"][limit] = size.into();
                }
                agent["profile"]["policy"]["unacknowledged"] = 1.into();
                fixtures::private(&data.join("agent.json"), &serde_json::to_vec(&agent).expect("agent JSON"))
                    .expect("cluster agent");
                worker["name"] = (*name).into();
                worker["secret"] = format!("secrets/{name}.secret").into();
                worker["engine"] = json!({"socket": engine.unwrap_or(([127,0,0,1],0).into()).to_string()});
                worker["graces"] = json!({"grace_ms":grace_ms,"drain_ms":graces.drain.as_millis(),"agent_stop_ms":agent_stop_ms,"delivery_ms":delivery_ms,"save_ms":save_ms});
                worker["limits"]["overrides"]["link.ping_ms"] = ping_ms.into();
                worker["limits"]["overrides"]["link.silence_ms"] = silence_ms.into();
                if grace_ms >= 1_000 {
                    // Redial promptly after the grace too; waiting out backoff is not this story's rule.
                    worker["limits"]["overrides"]["link.ceiling_ms"] = 1_000.into();
                }
                document = json!({"workers":document["workers"],"application":{"workers":document["application"]["workers"],"limits":document["application"]["limits"]}});
                fixtures::private(
                    &data.join(format!("secrets/{name}.secret")),
                    format!("{name}-fixture-secret").as_bytes(),
                )
                .expect("fixture secret");
            }
        }
        fixtures::private(&config.join("temper.json"), &serde_json::to_vec(&document).expect("cluster document"))
            .expect("private cluster document");
    }
}

/// The worker promises the configured host's cancel, terminate and exit tail.
fn agent_stop_bound(agent: &serde_json::Value) -> skein_lib::Duration {
    let configured = smith_agent_shell::config::from_value(agent.clone()).expect("cluster agent configuration");
    let host = smith_agent_profile::host_limits(&configured.profile, &configured.service.limits)
        .expect("cluster host supervision");
    host.cancel_grace
        .checked_add(host.kill_after)
        .and_then(|bound| bound.checked_add(host.exit_grace))
        .expect("cluster agent stop fits a duration")
}

// Observe the worker's own received assignments and successful agent spawns.
const WIRE_BYTES: usize = 65_536;
// skein's net/channel.md, 3: the eight-byte frame header.
const HEADER_BYTES: usize = 8;
#[derive(Default)]
pub(crate) struct Runs {
    fd: Option<skein_io::kernel::Fd>,
    bytes: Vec<u8>,
    pending: Option<u64>,
    pub(crate) tasks: Vec<u64>,
    pub(crate) assignments: Vec<jig_protocol_worker_wire::Assignment>,
    pub(crate) assigned_at: Option<skein_lib::Time>,
}
impl Runs {
    pub(crate) fn assigned(&self) -> bool {
        self.pending.is_some() || !self.tasks.is_empty()
    }
    pub(crate) fn complete(&mut self, complete: &Complete, engine: SocketAddr, now: skein_lib::Time) {
        match (&complete.kind, &complete.result) {
            (Op::Recv { fd, buf }, Ok(Done::Count(count))) if Some(*fd) == self.fd => {
                self.bytes.extend_from_slice(&buf[..*count as usize]);
                assert!(self.bytes.len() <= WIRE_BYTES, "bounded worker wire");
                while self.bytes.len() >= HEADER_BYTES {
                    let header =
                        skein_channel::parse_header(&self.bytes[..HEADER_BYTES]).expect("channel frame header");
                    let end = HEADER_BYTES + usize::try_from(header.body_len).expect("bounded frame body");
                    if self.bytes.len() < end {
                        break;
                    }
                    if header.kind == jig_protocol_worker_wire::Kind::Assignment.number() {
                        let assignment = jig_protocol_worker_wire::Assignment::decode(
                            &jig_protocol_worker_wire::CEILINGS,
                            &mut skein_lib::Reader::new(&self.bytes[HEADER_BYTES..end]),
                        )
                        .expect("assignment");
                        self.pending = Some(assignment.task());
                        self.assigned_at.get_or_insert(now);
                        self.assignments.push(assignment);
                    }
                    self.bytes.drain(..end);
                }
            }
            (Op::Spawn { spawn }, Ok(Done::Spawned { .. }))
                if spawn.program.ends_with(fixtures::PINNED_AGENT_SUFFIX) =>
            {
                self.tasks.push(self.pending.take().expect("agent follows assignment"));
            }
            (Op::Connect { fd, addr }, Ok(Done::Nothing)) if *addr == engine => {
                self.fd = Some(*fd);
                self.bytes.clear();
                self.pending = None;
            }
            _ => {}
        }
    }
}

/// Resolve the three independently owned scratch roots.
#[must_use]
pub fn directories_in(roots: [&Path; 3]) -> temper_box_shell::directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let options = ["--config", "--data", "--cache"].into_iter().zip(roots);
    let args: Vec<Box<[u8]>> = options
        .into_iter()
        .flat_map(|(option, path)| [option.as_bytes().into(), path.as_os_str().as_bytes().into()])
        .collect();
    temper_box_shell::directories::resolve(&args, &[], b"/").expect("fixture directories").0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_preparation_preserves_the_agents_supervision_policy() {
        let scratch = crate::Scratch::new("cluster-policy");
        let data = scratch.path().join("data");
        let address = ([127, 0, 0, 1], 0).into();
        fixtures::write(&data, address, address).expect("fixture documents");
        let path = data.join("agent.json");
        let mut expected: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("agent document")).expect("agent JSON");
        expected["profile"]["policy"]["unacknowledged"] = 1.into();

        Shape::Worker { name: "w1", engine: Some(address), started: true, grace: skein_lib::Duration::from_millis(10) }
            .write(scratch.path(), false);

        let prepared: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("prepared agent")).expect("prepared JSON");
        assert_eq!(prepared, expected);
    }
}
