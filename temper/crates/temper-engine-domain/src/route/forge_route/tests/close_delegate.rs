//! A close offer survives procedure admission (temper/docs/design/domain/forge/change/repairs-releases.md, 1).
use super::super::{Work, forge, forge_change as change, tasks};
use super::{SHA1, SHA256, change_start_fixture, restore_forge, task_change_row, turn_project};
use alloc::boxed::Box;
use jig_domain_core::{Ask, Event};
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Queue};

const CHILD: u64 = 90;
const TIPS: change::CloseAt = change::CloseAt { branch: SHA1, landing: SHA256, asked: true, refused: None };
// A job bound with room for the encoded command and request below and a refusal result.
const CLOSE_BOUND: JobBound =
    JobBound { kind: JobKind(forge::CLOSE_KIND), spec: 16_384, result: 512, deadline: Duration::from_secs(600) };

#[test]
fn the_root_keeps_the_close_tips_through_admission() {
    let (mut domain, env, context) = change_start_fixture();
    let task = context.task;
    let repository = super::adopted(1).provider;
    let mut row = task_change_row(task, None);
    row.repository = repository;
    row.close = Some(br#"{"version":1,"goal":7}"#.as_slice().into());
    row.change.state = change::State::Producing { requested: true };
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    turn_project(&mut domain);
    domain.forge.project_close_job(context.project, b".temper/close".as_slice().into());
    domain.config.close_kind = Some(CLOSE_BOUND);

    let mut out = Queue::uncapped();
    out.push(forge::Request::ChangeDecision {
        task,
        decision: change::Decision::Delegate(change::Delegate::Close),
        entry: None,
        evidence: None,
        close_at: Some(TIPS),
    });
    let mut journal = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::outputs(&mut domain, &env, &mut journal, &mut out);
    let Some(Work::Core(Event::ProcedureStep { connector, task: parent, step, decision, .. })) = domain.work.pop()
    else {
        panic!("close job offered in a procedure step");
    };
    let [job] = decision.batch.as_ref() else { panic!("one close job offered") };
    super::super::super::asks::ask(
        &mut domain,
        &env,
        &mut journal,
        connector,
        Ask::StepMade { task: parent, step, first: Some(CHILD), revisions: Box::new([]) },
    );
    let stored = domain.forge.change(task).expect("admitted change");
    let checkout = domain
        .forge
        .workspace_item(forge::items::Seed {
            project: context.project,
            modify: false,
            task: CHILD,
            parent: Some(task),
            repository,
            run_branch: b"run".as_slice().into(),
            saved_branch: None,
        })
        .expect("close checkout");
    assert_eq!(
        (job.executor, stored.close_at, checkout.start),
        (tasks::Executor::Job { kind: forge::CLOSE_KIND }, Some(TIPS), forge::items::Start::Commit(SHA1))
    );
}
