//! A usable account's trial through the joined client and engine
//! (temper/docs/design/domain/trial.md, 6; temper/docs/design/domain/testing.md, 6.3).
use jig_client_kit::harness::{App, Front, FrontFault};
use jig_domain_host::{AgentActivity, LinkUp, Phase};
use skein_lib::{Duration, Queue, Token};
use skein_ui::{Act, Role, State};
use temper_client_domain::{Address, ModelWay, Step, WayKind, forge::trials};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{Joint, account_acts as h},
};

const WORKER: Token = Token::new(7);

fn drawn(root: &Root) -> View {
    let limits = fixtures::domain_limits();
    let words = temper_client_view::words::Words::english(
        fixtures::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit");
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = View::Person(Box::new(temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE)));
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    view
}
fn running(root: &Root, step: usize, words: &[u8]) {
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Ready });
    let view = drawn(root);
    let tree = Temper::tree(&view);
    assert!(
        tree.nodes().iter().any(|node| node.role == Role::Progress && node.name.as_deref() == Some(words)),
        "expected {:?}; wait {:?}; nodes {:?}",
        String::from_utf8_lossy(words),
        root.person().forge().trial_wait(),
        tree.nodes().iter().filter_map(|node| node.name.as_deref()).map(String::from_utf8_lossy).collect::<Vec<_>>()
    );
    let at = tree
        .nodes()
        .iter()
        .position(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(b"Each step"))
        .expect("trial steps");
    let disclosure = tree.nodes().get(at).expect("disclosure position");
    let steps: Vec<_> = tree
        .nodes()
        .iter()
        .skip(at)
        .take(usize::try_from(disclosure.size).expect("bounded trial steps"))
        .filter(|node| node.role == Role::Item)
        .collect();
    assert_eq!(steps.len(), 3, "the subscription's three steps");
    for (index, node) in steps.iter().enumerate() {
        assert_eq!(node.states.as_slice().contains(&State::Current), index == step);
        assert_eq!(node.styles.as_slice().contains(&temper_client_view::look::FLOW_DONE), index < step);
    }
}

#[test]
fn the_success_story() {
    let (mut front, mut root) = h::empty_program_fixture(fixtures::SEED);
    front.deployment = (true, true);
    h::reopen(
        &mut front,
        &mut root,
        Address::Welcome {
            step: Step::Model { way: Some(ModelWay { provider: h::PROVIDER, kind: WayKind::Program, source: None }) },
        },
        false,
    );
    let mut view = drawn(&root);
    let button = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some("I’ve signed in".as_bytes()))
        .expect("host check offered")
        .id;
    let event = Temper::act(&mut view, button, Act::Press).expect("host check bound");
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::from_millis(1) });
    h::event(&mut front, &mut root, event);
    running(&root, 0, b"Trying: Signed in to your subscription");
    assert!(root.person().forge().trials().is_empty());
    front.fault(now, FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    front.reply_in_pieces(&[b"Yes"]);
    h::program_checked(&mut front, &mut root, true);
    let row = *root.person().forge().trials().first().expect("usable account starts a trial");
    assert_eq!(row.state, trials::State::Running);
    assert_eq!(row.step, trials::Step::Starts);
    assert_eq!(root.person().forge().trial_asked(), None, "the row replaces the pending ask");
    running(&root, 1, b"Trying: Claude Code starts on this machine");
    let task = row.task.expect("trial task made");
    let assignment = front.assignments.last().expect("trial assigned to the worker");
    assert_eq!(assignment.task, task);
    assert_eq!(assignment.charter, 10);
    let attempt = assignment.attempt;
    front.engine.send(temper_engine_domain_world::protocol::up(
        Token::new(7),
        LinkUp::Activity {
            run: Token::new(task),
            attempt: Token::new(attempt),
            activity: AgentActivity::Phase(Phase::Active),
        },
        temper_engine_domain::Parts::none(),
    ));
    front.engine.settle();
    h::settle(&mut front, &mut root);
    assert_eq!(root.person().forge().trials().first().expect("running row").step, trials::Step::Answers);
    running(&root, 2, b"Trying: Scripted answers");
    assert!(front.release_reply(), "commit the scripted reply");
    h::settle(&mut front, &mut root);
    let row = root.person().forge().trials().first().expect("answered row");
    assert_eq!(row.state, trials::State::Answered { seconds: 1 });
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Ready });
    let view = drawn(&root);
    assert!(!Temper::tree(&view).nodes().iter().any(|node| node.role == Role::Progress));
    assert!(Temper::tree(&view).nodes().iter().any(|node| {
        node.name.as_deref().is_some_and(|name| String::from_utf8_lossy(name).contains("answered in 1 s"))
    }));
    assert_eq!(front.runs_answered, 1);
}

fn shown(root: &Root) -> String {
    Temper::tree(&drawn(root))
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .map(|run| String::from_utf8_lossy(&run.text).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}
fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    Temper::tree(&drawn(root)).nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn press(front: &mut Joint, root: &mut Root, role: Role, name: &[u8]) {
    let mut view = drawn(root);
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("offered {}\n{}", String::from_utf8_lossy(name), shown(root)));
    assert!(!node.states.as_slice().contains(&State::Disabled), "offered control enabled");
    let id = node.id;
    let event = Temper::act(&mut view, id, Act::Press).expect("bound control");
    h::event(front, root, event);
}
fn checking() -> (Joint, Root) {
    let (mut front, mut root) = h::trial_fixture(fixtures::SEED);
    h::reopen(
        &mut front,
        &mut root,
        Address::Welcome {
            step: Step::Model { way: Some(ModelWay { provider: h::PROVIDER, kind: WayKind::Program, source: None }) },
        },
        false,
    );
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::from_millis(1) });
    press(&mut front, &mut root, Role::Button, "I’ve signed in".as_bytes());
    running(&root, 0, b"Trying: Signed in to your subscription");
    front.fault(now, FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    (front, root)
}
fn trying() -> (Joint, Root) {
    let (mut front, mut root) = checking();
    h::program_checked(&mut front, &mut root, true);
    running(&root, 1, b"Trying: Claude Code starts on this machine");
    (front, root)
}
fn row(root: &Root) -> trials::Row {
    *root.person().forge().trials().first().expect("one trial row")
}
fn active(front: &mut Joint, root: &mut Root) {
    let assignment = front.assignments.last().expect("trial assigned");
    front.engine.send(temper_engine_domain_world::protocol::up(
        WORKER,
        LinkUp::Activity {
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            activity: AgentActivity::Phase(Phase::Active),
        },
        temper_engine_domain::Parts::none(),
    ));
    h::settle(front, root);
    running(root, 2, b"Trying: Claude answers");
}
fn fail(front: &mut Joint, root: &mut Root) {
    let assignment = front.assignments.last().expect("trial assigned");
    front.engine.answer(
        WORKER,
        assignment.task,
        assignment.attempt,
        0,
        jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Run),
        temper_engine_domain::Parts::none(),
    );
    h::settle(front, root);
}
fn stopped(root: &Root, step: &[u8]) {
    let view = drawn(root);
    let nodes = Temper::tree(&view).nodes();
    let at = nodes
        .iter()
        .enumerate()
        .find_map(|(at, node)| {
            (node.role == Role::Item
                && nodes
                    .iter()
                    .skip(at)
                    .take(usize::try_from(node.size).expect("step size"))
                    .any(|child| child.text.iter().any(|run| run.text.as_ref() == step)))
            .then_some(at)
        })
        .unwrap_or_else(|| panic!("named trial step {}\n{}", String::from_utf8_lossy(step), shown(root)));
    let item = &nodes[at];
    assert!(
        item.styles.as_slice().contains(&temper_client_view::look::FLOW_FAILED),
        "stopped at {}",
        String::from_utf8_lossy(step)
    );
    assert!(
        nodes
            .iter()
            .skip(at)
            .take(usize::try_from(item.size).expect("step size"))
            .any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"stopped")),
        "step says Stopped"
    );
    assert!(!nodes.iter().any(|node| node.role == Role::Progress), "a stopped step isn't spinning");
}
fn ready(root: &Root, enabled: bool) {
    let view = drawn(root);
    let nodes = Temper::tree(&view).nodes();
    let title: &[u8] = if enabled { b"Ready" } else { b"Almost ready" };
    assert!(
        nodes.iter().any(|node| node.role == Role::Heading { level: 1 } && node.name.as_deref() == Some(title)),
        "{}",
        shown(root)
    );
    let create = nodes
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Create your project"))
        .expect("Create");
    assert_eq!(!create.states.as_slice().contains(&State::Disabled), enabled);
}

#[test]
fn the_first_step_stops_on_a_login_not_signed_in() {
    let (mut front, mut root) = checking();
    h::program_checked(&mut front, &mut root, false);
    ready(&root, false);
    stopped(&root, b"Signed in to your subscription");
    assert!(has(&root, Role::Button, b"Try again"));
    assert!(has(&root, Role::Link, b"Sign in again"));
    assert!(front.assignments.is_empty(), "a failed host check spends no turn");
}
#[test]
fn the_second_step_stops_on_a_run_that_cant_start() {
    let (mut front, mut root) = trying();
    fail(&mut front, &mut root);
    ready(&root, true);
    stopped(&root, b"Claude Code starts on this machine");
    assert!(has(&root, Role::Button, b"Try again"));
    press(&mut front, &mut root, Role::Button, b"Create your project");
    assert!(matches!(root.person().pages().address(), Address::Home { .. }), "Create works after a failed turn");
}
#[test]
fn the_third_step_stops_on_a_run_that_ends_without_answering() {
    let (mut front, mut root) = trying();
    active(&mut front, &mut root);
    fail(&mut front, &mut root);
    stopped(&root, b"Claude answers");
    ready(&root, true);
}
#[test]
fn a_minute_stops_it() {
    let (mut front, mut root) = trying();
    let began = front.engine.env.now;
    let task = row(&root).task.expect("trial task");
    front.fault(began.saturating_add(Duration::from_secs(59)), FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    assert_eq!(row(&root).state, trials::State::Running, "not early");
    front.fault(began.saturating_add(Duration::from_secs(60)), FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    assert_eq!(row(&root).state, trials::State::Failed { why: trials::Why::Waited });
    stopped(&root, b"Claude Code starts on this machine");
    assert!(shown(&root).contains("Claude didn’t answer within a minute."));
    assert!(front.engine.task(task).is_none(), "timed out task cancelled");
    assert_eq!(front.assignments.len(), 1);
}
#[test]
fn a_person_whos_left_sees_it_on_home() {
    let (mut front, mut root) = trying();
    press(&mut front, &mut root, Role::Button, b"Create your project");
    assert!(matches!(root.person().pages().address(), Address::Home { .. }));
    fail(&mut front, &mut root);
    assert!(shown(&root).contains("Claude didn’t answer when tried"), "{}", shown(&root));
    front.answer_runs = true;
    front.reply_runs = true;
    press(&mut front, &mut root, Role::Button, b"Try again");
    assert!(matches!(row(&root).state, trials::State::Answered { .. }));
    assert!(!shown(&root).contains("didn’t answer when tried"));
    assert_eq!(front.assignments.len(), 2, "Home's retry runs once");
}
#[test]
fn removing_the_account_cancels_it() {
    let (mut front, mut root) = trying();
    let task = row(&root).task.expect("trial task");
    h::event(&mut front, &mut root, Temper::went(Address::Accounts));
    press(&mut front, &mut root, Role::Button, b"Remove account");
    press(&mut front, &mut root, Role::Button, b"Remove");
    assert!(root.person().forge().trials().is_empty());
    assert!(front.engine.task(task).is_none(), "account removal cancels its trial");
    assert_eq!(front.assignments.len(), 1);
}
