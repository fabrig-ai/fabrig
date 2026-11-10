//! The connector's making rows run on the setup host (temper/docs/design/protocol/setup/owned.md, 9).
use super::lifecycle::{self, Kind, Run};
use alloc::boxed::Box;
use skein_forge_setup::{host, reconcile};
use skein_lib::{Map, Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;

fn connected(engine: &crate::engine::Engine, forge: u16) -> bool {
    match engine.domain.forge_connection(forge) {
        domain::Connection::Connected { .. } => true,
        domain::Connection::NotConnected
        | domain::Connection::NotSetUp
        | domain::Connection::SettingUp { .. }
        | domain::Connection::SetUpFailed { .. }
        | domain::Connection::Connecting
        | domain::Connection::Stopped { .. } => false,
    }
}
fn available(engine: &crate::engine::Engine, forge: u16) -> bool {
    !engine.domain.forge_ended(forge)
        && engine.setup.served.contains_key(&forge)
        && engine.forge.unreachable(forge).is_none()
        && engine.setup.runtime.is_some()
        && connected(engine, forge)
}
pub(super) fn ready(engine: &crate::engine::Engine) -> bool {
    for forge in engine.workers.forges.iter() {
        if available(engine, forge.number)
            && (waiting(engine, forge.number).is_some()
                || !engine.setup.busy(forge.number)
                    && (engine.domain.making_due(forge.number).is_some()
                        || engine.setup.runs.len() < engine.setup.env.limits.runs
                            && engine.domain.next_making(forge.number).is_some()))
        {
            return true;
        }
    }
    false
}
// A Busy stop keeps the run waiting in the layer's bounded staged plans.
fn waiting(engine: &crate::engine::Engine, forge: u16) -> Option<Token> {
    for (token, run) in &engine.setup.runs {
        if run.forge == forge && run.kind == Kind::Making && engine.setup.staged.contains_key(token) {
            return Some(*token);
        }
    }
    None
}
pub(super) fn retry(engine: &mut crate::engine::Engine, forge: u16) -> bool {
    if !available(engine, forge) {
        return false;
    }
    let Some(token) = waiting(engine, forge) else { return false };
    let plan = engine.setup.staged.remove(&token).expect("waiting making plan");
    super::run_plan(engine, token, plan);
    true
}
pub(super) fn running(runs: &Map<Token, Run>, forge: u16) -> bool {
    for (_, run) in runs {
        if run.forge == forge && run.kind == Kind::Making {
            return true;
        }
    }
    false
}
pub(super) fn pull(engine: &mut crate::engine::Engine, forge: u16) -> bool {
    if !available(engine, forge) {
        return false;
    }
    if let Some(key) = engine.domain.making_due(forge) {
        event(engine, domain::Event::MakingAdopt { key });
        return true;
    }
    if engine.setup.runs.len() == engine.setup.env.limits.runs {
        return false;
    }
    let Some(row) = engine.domain.next_making(forge).cloned() else { return false };
    let plan = plan(engine, &row);
    let token = lifecycle::next_run(&mut engine.setup.lifecycle);
    engine
        .setup
        .runs
        .insert(
            token,
            Run {
                making: Some(row.key.clone()),
                words: None,
                first: None,
                previous: engine.domain.forge_connection(forge),
                forge,
                kind: Kind::Making,
                internal: true,
            },
        )
        .expect("making pull checks run room");
    event(engine, domain::Event::MakingStarted { key: row.key });
    super::run_plan(engine, token, plan);
    true
}
fn plan(engine: &crate::engine::Engine, row: &domain::RepositoryMaking) -> host::Plan {
    let mut record = None;
    for row_record in engine.domain.forges().rows {
        if row_record.number == row.key.forge {
            record = Some(row_record);
            break;
        }
    }
    let record = record.expect("making belongs to a kept forge");
    let label = match &record.first {
        Some(first) => first.label.as_ref(),
        None => match record.mode {
            domain::Mode::Owned | domain::Mode::Fake => b"temper-host".as_slice(),
            domain::Mode::Managed | domain::Mode::Connected => b"docker".as_slice(),
        },
    };
    let files = Box::new([host::File {
        path: temper_forge_setup::STARTER_PATH.into(),
        content: temper_forge_setup::starter_workflow(label).expect("admission checks the runner label"),
    }]);
    let limits = engine.setup.runtime.as_ref().expect("making runtime").making_limits();
    match &row.from {
        domain::MakeFrom::Empty => {
            let plan = host::Plan::Create {
                forge: row.key.forge,
                org: row.key.org.clone(),
                repository: row.key.name.clone(),
                files,
            };
            assert!(
                skein_forge_setup::given::check_create(&limits, engine.setup.env.limits.string_bytes, &plan).is_ok(),
                "making admission and starter profile prove Create inputs"
            );
            plan
        }
        domain::MakeFrom::Copy { address } => {
            let plan = host::Plan::Copy {
                forge: row.key.forge,
                org: row.key.org.clone(),
                repository: row.key.name.clone(),
                from: address.clone(),
                files,
            };
            assert!(
                skein_forge_setup::given::check_copy(&limits, engine.setup.env.limits.string_bytes, &plan).is_ok(),
                "making admission and starter profile prove Copy inputs"
            );
            plan
        }
    }
}
pub(super) fn read(engine: &crate::engine::Engine, token: Token) -> Option<Secret> {
    let key = engine.setup.runs.get(&token)?.making.as_ref()?;
    if engine.domain.making(key)?.copying { Some(Secret::new(Box::from([1]))) } else { None }
}
pub(super) fn copying(engine: &mut crate::engine::Engine, token: Token) {
    let key = engine.setup.runs.get(&token).expect("copying run").making.clone().expect("copying belongs to a row");
    event(engine, domain::Event::MakingCopying { key });
}
pub(crate) fn copied(engine: &mut crate::engine::Engine, key: &domain::MakingKey) {
    for (token, run) in &engine.setup.runs {
        if run.making.as_ref() == Some(key) {
            let token = *token;
            engine.setup.down(host::Request::Answer { run: token, answer: host::Answer::Written });
            return;
        }
    }
}
pub(super) fn said(engine: &mut crate::engine::Engine, token: Token, words: Box<[u8]>) {
    if let Some(run) = engine.setup.runs.get_mut(&token)
        && run.kind == Kind::Making
    {
        run.words = Some(words);
    }
}
pub(super) fn done(engine: &mut crate::engine::Engine, token: Token, outcome: reconcile::Outcome) {
    let run = engine.setup.runs.remove(&token).expect("making run ended");
    let (reconcile::Outcome::Created { id, branch } | reconcile::Outcome::Copied { id, branch }) = outcome else {
        unreachable!("making returns the repository and default branch")
    };
    event(engine, domain::Event::MakingMade { key: run.making.expect("making key"), id: id.get(), branch });
}
pub(super) fn stopped(engine: &mut crate::engine::Engine, token: Token, failure: reconcile::Failure) {
    if failure.why == reconcile::Why::Busy {
        let key = engine.setup.runs.get(&token).expect("busy making run").making.as_ref().expect("making key");
        let row = engine.domain.making(key).expect("busy making row");
        let plan = plan(engine, row);
        engine.setup.staged.insert(token, plan).expect("one waiting plan per admitted run");
        return;
    }
    let run = engine.setup.runs.remove(&token).expect("making run stopped");
    let step = match failure.step {
        reconcile::Stage::Admin => domain::MakeStep::Admin,
        reconcile::Stage::Org => domain::MakeStep::Org,
        reconcile::Stage::File => domain::MakeStep::File,
        reconcile::Stage::Repository
        | reconcile::Stage::Bot
        | reconcile::Stage::Token
        | reconcile::Stage::App
        | reconcile::Stage::Installation
        | reconcile::Stage::Person
        | reconcile::Stage::State => domain::MakeStep::Repository,
    };
    let reason = match failure.why {
        reconcile::Why::Taken => domain::MakeFailure::Taken,
        reconcile::Why::Empty => domain::MakeFailure::Empty,
        reconcile::Why::TimedOut => domain::MakeFailure::TimedOut,
        reconcile::Why::Refused(status) => domain::MakeFailure::Refused { status },
        reconcile::Why::Missing
        | reconcile::Why::NotAdmin
        | reconcile::Why::BotIsAdmin
        | reconcile::Why::Unmarked
        | reconcile::Why::OtherUser
        | reconcile::Why::OtherApp
        | reconcile::Why::Unknown
        | reconcile::Why::Busy
        | reconcile::Why::Crowded
        | reconcile::Why::Down
        | reconcile::Why::Unexpected
        | reconcile::Why::KeyForm => domain::MakeFailure::Other,
    };
    let words = match run.words {
        Some(words) => words,
        None => super::failure_text(&failure),
    };
    event(
        engine,
        domain::Event::MakingStopped {
            key: run.making.expect("making key"),
            failed: domain::MakingFailed { step, reason, words },
        },
    );
}
fn event(engine: &mut crate::engine::Engine, event: domain::Event) {
    engine.events.push(root::Event::Forge(Box::new(event)));
}
