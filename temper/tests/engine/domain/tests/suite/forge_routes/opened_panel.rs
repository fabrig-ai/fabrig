//! A person's pushed branch reaches its pull request through the engine's world
//! (temper/docs/design/domain/forge/change/opened.md, 2–4 and 6).
use super::*;
use skein_forge::api::Write;
use temper_engine_domain_forge_change as change;

const STEPS: u32 = 1000;
const WATCH: Token = Token::new(501);
const SOURCE: &[u8] = b"docs/x";

fn watched(world: &mut World, step: &forge_top::PanelStep, kept: Option<[u8; 32]>) {
    for _ in 0..STEPS {
        if world.panels.last().is_some_and(|(watcher, panel)| *watcher == WATCH && panel.step == *step) {
            break;
        }
        if !world.panels.is_empty() {
            world.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: WATCH }));
            world.panels.clear();
        }
        world.tick();
    }
    let (watcher, panel) = world.panels.last().expect("watched panel");
    assert_eq!(*watcher, WATCH);
    assert_eq!(&panel.step, step);
    assert_eq!(panel.source, Some(forge_top::PanelSource { branch: SOURCE.into(), at: None, kept }));
    world.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: WATCH }));
    world.panels.clear();
}

#[test]
#[expect(clippy::too_many_lines, reason = "one story follows admission, selection, make and pull opening")]
fn a_persons_pushed_branch_is_taken_and_shown_through_its_pull_request() {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { create_branches: true, ..StoryLimits::default() },
    );
    world.adopt();
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: SOURCE.into(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", SOURCE, b"docs", b"Fixed docs", 2)
        .expect("person pushed");
    let h = translate::commit(head);
    world.writes.clear();
    world.forge_reads = Some(Vec::new());
    world.hold_reads_at_any_step = true;
    world.held_reads = Some(Vec::new());
    world.hold_write_answers = Some(HoldWrite::All);
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("person signed in"),
        key: [92; 16],
        digest: [92; 32],
        carried: None,
        request: engine::Application::OpenChange(forge_top::OpenChange {
            project: 1,
            repository: None,
            branch: SOURCE.into(),
            at: None,
            into: None,
            title: b"Fix the docs".as_slice().into(),
            body: None,
            budget: forge_top::OpenBudget::Amount(100),
        }),
    }));
    for _ in 0..STEPS {
        if world.adopted.iter().any(|answer| matches!(answer, forge_top::ApplicationOutcome::Opened { .. })) {
            break;
        }
        world.tick();
    }
    let task = world
        .adopted
        .iter()
        .find_map(|answer| match answer {
            forge_top::ApplicationOutcome::Opened { task } => Some(*task),
            _ => None,
        })
        .unwrap_or_else(|| panic!("opened: {:?}", world.adopted));
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: WATCH,
        sign_in: world.signed_in.expect("person signed in"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(task),
        },
        window: jig_domain_core::documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    watched(&mut world, &forge_top::PanelStep::Taking, None);
    let reads = world.held_reads.take().expect("taking reads paused");
    for request in reads {
        world.out.push(request);
    }
    watched(&mut world, &forge_top::PanelStep::Taking, Some(h));
    for _ in 0..STEPS {
        if !world.held_write_answers.is_empty() {
            break;
        }
        world.tick();
    }
    assert_eq!(world.held_write_answers.len(), 1, "make answer held");
    let row = light_rules::change_row(&world, task);
    assert_eq!(row.kept, Some(h));
    assert!(matches!(row.change.state, change::State::Taking { .. }));
    let own = format!("temper/{task}/c{task}").into_bytes().into_boxed_slice();
    assert_eq!(row.branch, own);
    assert_eq!(world.fake.branch(b"org/repo", &own), Some(head));
    world.hold_write_answers = None;
    world.events.extend(world.held_write_answers.drain(..));
    watched(&mut world, &forge_top::PanelStep::Checking, Some(h));
    let row = light_rules::change_row(&world, task);
    let number = row.pull.expect("opened pull request");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.head, own);
    assert_eq!(pull.base.as_ref(), b"main");
    assert_eq!(pull.commit, head);
    let [Write::CreateBranch { branch, commit }, Write::OpenPull { head: from, base, title, body, .. }] =
        world.writes.as_slice()
    else {
        panic!("only make and open: {:?}", world.writes)
    };
    assert_eq!(branch, &own);
    assert_eq!(*commit, h);
    assert_eq!(from, &own);
    assert_eq!(base.as_ref(), b"main");
    assert_eq!(title.as_ref(), b"Fix the docs");
    assert_eq!(body.as_ref(), b"Opened by person 1 from docs/x at its head into main.");
    assert_eq!(resource_admission::branch_reads(&world, SOURCE), 1);
    assert_eq!(world.fake.branch(b"org/repo", SOURCE), Some(head));
    assert!(world.assigned.is_empty(), "taking has no producer");
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: WATCH }));
    world.close();
}
