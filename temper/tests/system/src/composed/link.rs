//! The protocol layers between the engine root and the worker root, as the
//! composed world plays them (testing-strategy.md, 4). jig's link records
//! cross as they are, both roots speaking the hub's vocabulary; what differs
//! is translated here by small total functions: an assignment, which the
//! engine gives typed and the hub takes with smith's charter encoded and
//! temper's workspace items beside it; an answer's parts, which the worker
//! gives in its own detail and the engine takes typed (jig's
//! `protocol/channel/README.md`, 5.2); and the engine's hand-outs of host calls and
//! results, decoded and rendered with smith's codecs and temper's charter
//! translation.

use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Reader, Token};
use smith_domain_run as run;
use temper_engine_domain as engine;
use temper_protocol_worker_wire::v1;
use temper_service::{settings, translate};
use temper_worker_domain::wire;
use temper_worker_domain_world::protocol as worker_protocol;

pub const GIT_BASE: &[u8] = b"http://forge.test";

/// The hub's assignment beside the service's workspace and per-repository saves.
#[must_use]
pub fn assignment(assignment: &engine::Assignment) -> (host::Assignment, wire::Workspace) {
    let forges = forges();
    let encoding = temper_service::assignment::Encoding {
        limits: v1::CEILINGS,
        endpoints: worker_protocol::endpoints(),
        deployment: [0; 16],
        charter_bytes: 65_536,
    };
    let event = temper_service::assignment::down(
        assignment.clone(),
        &forges,
        &encoding,
        &temper_service::values::Values::new(0, 0),
        Token::new(7),
        skein_lib::Time::ZERO,
    )
    .expect("production assignment fits");
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment: hub }, parts, .. } =
        event
    else {
        panic!("assignment down")
    };
    let assigned = temper_service::worker_route::items(&hub, &parts, &v1::CEILINGS).expect("workspace parts decode");
    (hub, assigned)
}

/// A job crosses the same production translation and receiving bounds as a worker link.
pub fn job(
    job: host::JobAssignment,
    workspace: Option<&engine::ForgeWorkspace>,
    limits: &v1::Limits,
) -> temper_worker_domain::Event {
    let encoding = temper_service::assignment::Encoding {
        limits: *limits,
        endpoints: worker_protocol::endpoints(),
        deployment: [0; 16],
        charter_bytes: 65_536,
    };
    let event = temper_service::assignment::job_down(
        job,
        workspace,
        &forges(),
        &encoding,
        &temper_service::values::Values::new(0, 0),
        Token::new(7),
        skein_lib::Time::ZERO,
    )
    .expect("production job assignment fits");
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::AssignJob { job }, parts, .. } = event else {
        panic!("job assignment down")
    };
    let items = temper_service::worker_route::job_items(&job, &parts, limits);
    let spec = temper_service::worker_route::job_spec(&job.spec, limits);
    temper_worker_domain::Event::AssignJob { job, items, spec }
}

/// The worker's parts through the service's channel records, without bytes.
#[must_use]
pub fn parts(work: &wire::Work, preparation: Option<wire::Preparation>) -> engine::Parts {
    let (left, saved) = translate::work(work);
    let preparation = preparation.as_ref().and_then(translate::preparation);
    translate::parts(Some(left), saved, preparation)
}

/// A relayed call decoded into the engine's tool vocabulary: smith's call
/// name by smith's channel codec, its input by temper's charter translation.
#[must_use]
pub fn decode(call: &fleet::Call) -> engine::Call {
    let mut reader = Reader::new(&call.name);
    let name = smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader).expect("smith's call name");
    assert!(reader.is_empty(), "one whole call name");
    let name =
        run::CallName { activation: name.activation(), completion: name.completion(), position: name.position() };
    let input = run::HostInput::attested(call.input.clone()).expect("bounded host input");
    let temper_engine_smith::Decoded::Call(body) = temper_engine_smith::call(name, &call.tool, &input)
        .unwrap_or_else(|error| panic!("tool {:?}: {error:?}, input {:?}", call.tool, call.input))
    else {
        panic!("script tool answered at decode")
    };
    body
}

/// A decided call's answer rendered as smith takes it.
#[must_use]
#[expect(clippy::too_many_arguments, reason = "the world forwards the root's render-only HOW beside read context")]
pub fn render(
    answer: &engine::CallAnswer,
    tool: &[u8],
    maximum: u32,
    task: u64,
    receiving: u32,
    job: Option<skein_forge::api::JobAttempt>,
    page: Option<(u32, u32)>,
    ask: engine::AskHow,
) -> core::SettledAnswer {
    let event = temper_service::render_for_task_page(
        skein_lib::ReplyTo::new(Token::new(1)),
        answer,
        tool,
        maximum.saturating_sub(u32::try_from(tool.len()).expect("tool length")),
        task,
        receiving,
        temper_engine_smith::ForgeReadBounds::STOCK,
        job,
        page,
        ask,
    );
    let engine::Event::Worker(engine::WorkerInput::RenderedHostCall { answer, .. }) = event else {
        panic!("rendered host call")
    };
    answer
}

/// An ended answer's outcome read back as the engine's task ending: smith's
/// result codec, then temper's translation of an accepted result.
#[must_use]
pub fn end(outcome: &[u8]) -> tasks::End {
    end_with(outcome, None)
}

/// Translate a terminal with the resource established by the engine's forge route.
pub fn end_with(outcome: &[u8], change: Option<temper_engine_smith::ChangeResource>) -> tasks::End {
    let declared = smith_protocol_channel::decode_declared(outcome, &smith_charter::CEILINGS).expect("smith's result");
    let spent = run::Spend {
        turns: 0,
        input: None,
        output: None,
        cache_read: None,
        cache_write: None,
        reasoning: None,
        units: 0,
    };
    let answer = run::Answer::Accepted { outcome: declared, spent, turns: 0 };
    temper_engine_smith::result(answer, change).expect("a result the engine takes").end
}

fn forges() -> temper_service::forges::Forges {
    let mut table = temper_service::forges::Forges::with_capacity(1);
    assert!(table.insert(settings::Forge {
        log_origins: None,
        mode: temper_engine_domain_forge::Mode::Connected,
        number: temper_engine_forge_world::REPO.forge,
        kind: settings::Kind::Forgejo,
        api: jig_protocol_config::Address::Host { name: b"forge.test".as_slice().into(), port: 80 },
        transport: settings::Transport::Plaintext,
        git: GIT_BASE.into(),
        git_program: None,
        web: None,
        app_read_only: false,
        user: 1,
        identity: 1,
    }));
    table
}
