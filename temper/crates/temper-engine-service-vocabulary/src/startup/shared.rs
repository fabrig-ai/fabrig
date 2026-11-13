//! Shared startup diagnostics (protocol/services.md, 4.2).
use alloc::boxed::Box;
use jig_domain_link::Problem;
use jig_protocol_config::{self as config, Path, Segment};
use skein_lib::{Decimal, Writer};
use temper_service_core::startup::problems::put;

/// The missing agent name diagnostic shared by admission and startup problems.
pub const AGENT_WORDS: &[u8] = b"not in the agent's configuration under the same name";

/// The admission paragraph shared by startup paths and hello refusals (protocol/cluster.md, 3).
#[must_use]
pub fn admission_words(why: jig_domain_fleet::AgentShortfall, offered: u64, required: u64) -> Box<[u8]> {
    let mut length = 0;
    admission(why, offered, required, &mut Writer::new(0), &mut length, true);
    let mut writer = Writer::new(length);
    let mut written = 0;
    admission(why, offered, required, &mut writer, &mut written, false);
    assert_eq!(written, length, "admission passes measure identically");
    writer.finish()
}

/// Render an admission paragraph in the startup renderer's measuring or writing pass.
pub fn admission(
    why: jig_domain_fleet::AgentShortfall,
    offered: u64,
    required: u64,
    writer: &mut Writer,
    length: &mut usize,
    measuring: bool,
) {
    use jig_domain_fleet::AgentShortfall;
    // Missing names carry no numerical claim, even if constructed by a caller.
    if let AgentShortfall::Model { .. } = why {
        put(writer, length, measuring, AGENT_WORDS);
        return;
    }
    let (before, between, after): (&[u8], &[u8], &[u8]) = match why {
        AgentShortfall::Model { .. } => unreachable!("name diagnostic above"),
        AgentShortfall::Window { .. } => (b"needs a context window of ", b" tokens; the agent offers ", b" tokens"),
        AgentShortfall::Output { .. } => (b"needs ", b" output tokens; the agent offers ", b" tokens"),
        AgentShortfall::Turns { .. } => (b"needs ", b" turns; the agent's profile.policy.max_turns allows ", b""),
        AgentShortfall::Spend => {
            (b"needs a spend allowance of ", b"; the agent's profile.policy.max_spend allows ", b"")
        }
        AgentShortfall::Time { .. } => {
            (b"needs ", b" ms of run time; the agent's profile.policy.max_time allows ", b" ms")
        }
        AgentShortfall::Waiting { .. } => {
            (b"needs ", b" ms of waiting; the agent's profile.policy.max_waiting allows ", b" ms")
        }
        AgentShortfall::Checks { .. } => {
            (b"needs checks within ", b" ms; the agent's profile.policy.check_timeout is ", b" ms")
        }
        AgentShortfall::Message => {
            (b"needs ", b"-byte messages (tasks.message_bytes); the agent offers ", b" (tool_payload less 2)")
        }
        AgentShortfall::Inbox => {
            (b"needs an inbox of ", b" (tasks.run_inbox); the agent's profile.policy.inbox allows ", b"")
        }
        AgentShortfall::Unacknowledged => (
            b"needs an unacknowledged-turn allowance of ",
            b" (hub.turns); the agent's profile.policy.unacknowledged allows ",
            b"",
        ),
    };
    put(writer, length, measuring, before);
    put(writer, length, measuring, Decimal::of(required).as_bytes());
    put(writer, length, measuring, between);
    put(writer, length, measuring, Decimal::of(offered).as_bytes());
    put(writer, length, measuring, after);
    if let AgentShortfall::Checks { charter } = why {
        put(writer, length, measuring, b"; raise charter ");
        put(writer, length, measuring, Decimal::of(u64::from(charter)).as_bytes());
        put(writer, length, measuring, b"'s reserve_time (temper help settings) or lower the timeout");
    }
}
/// Translate the kit's configured membership refusal into its startup path.
#[must_use]
pub fn membership(problem: Problem) -> (Path, config::What) {
    let hosts = Path::root().member(b"engine").member(b"hosts").member(b"workers");
    match problem {
        Problem::Workers { .. } => (hosts, config::What::Bound),
        Problem::WorkerName { worker, .. } => {
            (hosts.child(Segment::Index(worker)).member(b"name"), config::What::Bound)
        }
    }
}
