//! Engine startup problems around the core's vocabulary (protocol/services.md, 3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};
use temper_service_core::startup::problems::{put, render_past, render_path, render_problem};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EngineWhat {
    Room(temper_engine_domain::room::Short),
    Stock(crate::stock::Floor),
    EngineAlone,
    UnbuiltWorker,
    Loopback,
    Connectors,
    OldAgentCommand,
    NoWebhooks,
    TierChanges { count: u32, bound: u32, ceiling: u32 },
    StorePage { transcript: u32, frame: u32, page: u32, ceiling: u32 },
    StoreMap(jig_protocol_store::sizes::SizeRefusal),
    StoreTask { needs: u32, has: u32, batch_bytes: u32 },
    Agent,
    Admission { why: jig_domain_fleet::AgentShortfall, offered: u64, required: u64 },
    Adapter,
    Redirect(super::Listener),
    NoHub,
    Credentials,
    Escalation,
    Provider,
    BoxSignIn,
    Handover,
    MissingNull,
    InlineSeedPolicy,
    LandingStored,
    ProjectLandingStored,
    CharterStored,
}
pub type What = temper_service_core::startup::problems::What<EngineWhat>;
pub type Problem = temper_service_core::startup::problems::Problem<What>;
pub type Problems = temper_service_core::startup::problems::Problems<What>;
fn words(what: EngineWhat) -> &'static [u8] {
    match what {
        EngineWhat::EngineAlone => b"an engine alone takes no local worker, one plaintext loopback listener and workers with distinct digest paths",
        EngineWhat::UnbuiltWorker => b"this build serves an engine alone",
        EngineWhat::Loopback => b"must be a socket address on loopback",
        EngineWhat::Connectors => b"must be [1], the forge connector",
        EngineWhat::OldAgentCommand => b"\"agent\" is now \"internal\", \"agent\": write [\"internal\", \"agent\", PATH], PATH agent.configuration's",
        EngineWhat::NoWebhooks => b"needs application.webhooks",
        EngineWhat::Agent => b"not in the agent's configuration under the same name",
        EngineWhat::Adapter => b"program adapter not declared in the agent's programs",
        EngineWhat::Redirect(_) => b"fixed port conflicts with ",
        EngineWhat::NoHub => b"the root has no hub, so the fleet must place every run on a worker",
        EngineWhat::Credentials => b"must be from 1 to accounts.credentials, in engine.limits and in the worker's limits",
        EngineWhat::Escalation => b"an owner's project must name an escalation role from 0 to 3",
        EngineWhat::Provider => b"the deployment's provider must not be 0, the forge's sign-in provider",
        EngineWhat::BoxSignIn => b"a box's sign-in takes application.sign_in null",
        EngineWhat::Handover => b"admin_token must be exactly 40 lowercase hexadecimal digits",
        EngineWhat::MissingNull => b"missing: null for none",
        EngineWhat::InlineSeedPolicy => b"name a policy, policy: 1; change it with temper ctl settings apply",
        EngineWhat::LandingStored => b"landing rules live in the store; read them with temper ctl settings show landing-rule",
        EngineWhat::ProjectLandingStored => b"landing rules live in the store; read them with temper ctl settings show landing-rule --project",
        EngineWhat::CharterStored => b"the forge's charters live in the store; read them with temper ctl settings show forge-charter 1",
EngineWhat::Room(_) | EngineWhat::Stock(_) | EngineWhat::TierChanges { .. } | EngineWhat::StorePage { .. } | EngineWhat::StoreMap(_) | EngineWhat::StoreTask { .. } | EngineWhat::Admission { .. } => b"",
 }
}
fn redirect_listener(listener: super::Listener, writer: &mut Writer, length: &mut usize, measuring: bool) {
    let name = match listener {
        super::Listener::Demo => b"application.demo.llm.listen".as_slice(),
        super::Listener::Clients => b"application.clients.listener".as_slice(),
        super::Listener::Webhooks => b"application.webhooks.listener".as_slice(),
        super::Listener::Owned(_) => b"application.machine.owned.listen".as_slice(),
        super::Listener::Fake(_) => b"application.machine.fake.listen".as_slice(),
    };
    put(writer, length, measuring, name);
}
/// The admission paragraph shared by startup paths and hello refusals (protocol/cluster.md, 3).
pub(crate) fn admission_words(why: jig_domain_fleet::AgentShortfall, offered: u64, required: u64) -> Box<[u8]> {
    let mut length = 0;
    admission(why, offered, required, &mut Writer::new(0), &mut length, true);
    let mut writer = Writer::new(length);
    let mut written = 0;
    admission(why, offered, required, &mut writer, &mut written, false);
    assert_eq!(written, length, "admission passes measure identically");
    writer.finish()
}

fn admission(
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
        put(writer, length, measuring, words(EngineWhat::Agent));
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
fn store_page(frame: u32, ceiling: u32, writer: &mut Writer, length: &mut usize, measuring: bool) {
    put(writer, length, measuring, b"its turn's transcript frame, ");
    put(writer, length, measuring, Decimal::of(u64::from(frame)).as_bytes());
    put(writer, length, measuring, b" bytes, needs a store page past the profile's ceiling of ");
    put(writer, length, measuring, Decimal::of(u64::from(ceiling)).as_bytes());
    put(writer, length, measuring, b" bytes");
}

fn store_map(why: jig_protocol_store::sizes::SizeRefusal, writer: &mut Writer, length: &mut usize, measuring: bool) {
    use jig_protocol_store::sizes::SizeRefusal;
    if why == SizeRefusal::Ceiling {
        put(writer, length, measuring, b"the store's map sizing is past its codec or arithmetic bounds");
        return;
    }
    let (name, needs, has, unit): (&[u8], u64, u64, &[u8]) = match why {
        SizeRefusal::Key { needs, has } => (b"key", u64::from(needs), u64::from(has), b"bytes"),
        SizeRefusal::Value { needs, has } => (b"value", u64::from(needs), u64::from(has), b"bytes"),
        SizeRefusal::Ops { needs, has } => (b"commit", u64::from(needs), u64::from(has), b"operations"),
        SizeRefusal::Commit { needs, has } => (b"commit", u64::from(needs), u64::from(has), b"bytes"),
        SizeRefusal::Page { needs, has } => (b"page", u64::from(needs), u64::from(has), b"bytes"),
        SizeRefusal::Queue { needs, has } => (b"queue", u64::from(needs), u64::from(has), b"commits"),
        SizeRefusal::QueuedBytes { needs, has } => (b"queue", needs, has, b"bytes"),
        SizeRefusal::Whole { needs, has } => (b"whole-file read", u64::from(needs), u64::from(has), b"bytes"),
        SizeRefusal::Ceiling => unreachable!("arithmetic diagnostic above"),
    };
    put(writer, length, measuring, b"the store's map ");
    put(writer, length, measuring, name);
    put(writer, length, measuring, b", ");
    put(writer, length, measuring, Decimal::of(needs).as_bytes());
    put(writer, length, measuring, b" ");
    put(writer, length, measuring, unit);
    put(writer, length, measuring, b", past its bound of ");
    put(writer, length, measuring, Decimal::of(has).as_bytes());
    put(writer, length, measuring, b" ");
    put(writer, length, measuring, unit);
}
fn store_task(needs: u32, has: u32, batch_bytes: u32, writer: &mut Writer, length: &mut usize, measuring: bool) {
    store_map(jig_protocol_store::sizes::SizeRefusal::Value { needs, has }, writer, length, measuring);
    put(writer, length, measuring, b"; binding record tasks-v9.TaskRecord with tasks.batch_bytes ");
    put(writer, length, measuring, Decimal::of(u64::from(batch_bytes)).as_bytes());
    put(writer, length, measuring, b"; fixed profile ceiling engine.limits.store.value; reduce tasks.batch_bytes, tasks.inbox_messages or the task record's spec/message/result/authority capacities; this ceiling has no scalar override");
}

fn stock(floor: crate::stock::Floor, writer: &mut Writer, length: &mut usize, measuring: bool) {
    use jig_domain_core::settings::Setting;
    put(writer, length, measuring, floor.limit);
    put(writer, length, measuring, b" is ");
    put(writer, length, measuring, Decimal::of(floor.configured).as_bytes());
    let (offered, unit): (bool, &[u8]) = match floor.limit {
        b"tasks.charters" => (true, b" charters"),
        b"core.templates" => (true, b" templates"),
        b"authority.projects" if floor.needed == 1 => (true, b" policy"),
        b"authority.projects" => (true, b" policies"),
        b"core.settings.alternatives" => (false, b" alternatives"),
        b"core.settings.permissions" => (false, b" permissions"),
        b"authority.grants" => (false, b" grants"),
        b"authority.executors" => (false, b" executors"),
        b"authority.segments" => (false, b" segments"),
        b"authority.roles" => (false, b" roles"),
        b"tasks.grace_ms" => (false, b" ms of grace"),
        _ => (false, b" bytes"),
    };
    let unit = if floor.needed == 1 { unit.strip_suffix(b"s").unwrap_or(unit) } else { unit };
    if offered {
        put(writer, length, measuring, b"; the stock offers ");
    } else {
        put(writer, length, measuring, b"; the stock's ");
        let (kind, number): (&[u8], Option<u32>) = match floor.setting {
            Setting::ProjectPolicy { project } => (b"project-policy ", Some(project)),
            Setting::Charter(number) => (b"charter ", Some(number)),
            Setting::Template(number) => (b"template ", Some(number)),
            Setting::Policy(number) => (b"policy ", Some(number)),
            Setting::Model(number) => (b"Model ", Some(number)),
            Setting::Rules => (b"rules", None),
            Setting::Application { kind, scope, number } => {
                put(writer, length, measuring, b"application kind ");
                put(writer, length, measuring, Decimal::of(u64::from(kind)).as_bytes());
                put(writer, length, measuring, b", ");
                put(writer, length, measuring, Decimal::of(u64::from(scope)).as_bytes());
                put(writer, length, measuring, b", ");
                (b"", Some(number))
            }
        };
        put(writer, length, measuring, kind);
        if let Some(number) = number {
            put(writer, length, measuring, Decimal::of(u64::from(number)).as_bytes());
            put(writer, length, measuring, b" needs ");
        } else {
            put(writer, length, measuring, b" need ");
        }
    }
    put(writer, length, measuring, Decimal::of(floor.needed).as_bytes());
    put(writer, length, measuring, unit);
    put(writer, length, measuring, b"; raise it in engine.limits.overrides");
}
fn render(problems: &Problems, writer: &mut Writer, measuring: bool) -> usize {
    let mut length = 0_usize;
    for problem in &problems.problems {
        let what = match problem.what {
            What::Half(what) => what,
            What::Jig(_)
            | What::Standalone
            | What::WorkerAlone
            | What::Absolute
            | What::Profile
            | What::Override(_)
            | What::Against { .. }
            | What::Checkout { .. }
            | What::Past { .. }
            | What::Short { .. }
            | What::Within { .. }
            | What::CallInput { .. } => {
                length = length.checked_add(render_problem(problem, writer, measuring)).expect("bounded diagnostics");
                continue;
            }
        };
        render_path(&problem.path, writer, &mut length, measuring);
        put(writer, &mut length, measuring, b": ");
        if let Some(file) = &problem.file {
            put(writer, &mut length, measuring, file);
            put(writer, &mut length, measuring, b": ");
        }
        put(writer, &mut length, measuring, words(what));
        match what {
            EngineWhat::Room(short) => put(writer, &mut length, measuring, &short.words()),
            EngineWhat::Stock(floor) => stock(floor, writer, &mut length, measuring),
            EngineWhat::TierChanges { count, bound, ceiling } => {
                put(writer, &mut length, measuring, Decimal::of(u64::from(count)).as_bytes());
                put(writer, &mut length, measuring, b", past its bound of ");
                put(writer, &mut length, measuring, Decimal::of(u64::from(bound)).as_bytes());
                put(
                    writer,
                    &mut length,
                    measuring,
                    b", the most the store's tier rows hold (2 * forge.changes + 1 within ",
                );
                put(writer, &mut length, measuring, Decimal::of(u64::from(ceiling)).as_bytes());
                put(writer, &mut length, measuring, b")");
            }
            EngineWhat::StorePage { frame, ceiling, .. } => {
                store_page(frame, ceiling, writer, &mut length, measuring);
            }
            EngineWhat::StoreMap(why) => store_map(why, writer, &mut length, measuring),
            EngineWhat::StoreTask { needs, has, batch_bytes } => {
                store_task(needs, has, batch_bytes, writer, &mut length, measuring);
            }
            EngineWhat::Admission { why, offered, required } => {
                admission(why, offered, required, writer, &mut length, measuring);
            }
            EngineWhat::Redirect(listener) => redirect_listener(listener, writer, &mut length, measuring),
            EngineWhat::EngineAlone
            | EngineWhat::UnbuiltWorker
            | EngineWhat::Loopback
            | EngineWhat::Connectors
            | EngineWhat::OldAgentCommand
            | EngineWhat::NoWebhooks
            | EngineWhat::Agent
            | EngineWhat::Adapter
            | EngineWhat::NoHub
            | EngineWhat::Credentials
            | EngineWhat::Escalation
            | EngineWhat::Provider
            | EngineWhat::BoxSignIn
            | EngineWhat::Handover
            | EngineWhat::MissingNull
            | EngineWhat::InlineSeedPolicy
            | EngineWhat::LandingStored
            | EngineWhat::ProjectLandingStored
            | EngineWhat::CharterStored => {}
        }
        put(writer, &mut length, measuring, b"\n");
    }
    render_past(problems.past, writer, &mut length, measuring);
    length
}
#[must_use]
pub fn lines(problems: &Problems) -> Box<[u8]> {
    let length = render(problems, &mut Writer::new(0), true);
    let mut writer = Writer::new(length);
    assert_eq!(render(problems, &mut writer, false), length, "diagnostic passes measure identically");
    writer.finish()
}

#[cfg(feature = "worker")]
pub(super) fn from_worker(found: temper_service_core::startup::problems::SharedProblems) -> Problems {
    let mut result = Problems::empty();
    for problem in found.problems.into_boxed() {
        result.push_file(problem.path, problem.what.extend(), problem.file);
    }
    result.past = result.past.saturating_add(found.past);
    result
}
