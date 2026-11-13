//! A usable account's trial through the joined client and engine
//! (temper/docs/design/domain/trial.md, 6; temper/docs/design/testing/checks.md, 3).
use jig_client_kit::harness::{App, Front, FrontFault};
use jig_domain_host::{AgentActivity, LinkUp};
use jig_domain_link::Phase;
use skein_lib::{Duration, Queue, Token};
use skein_ui::{Act, Role, State};
use temper_client_domain::{Address, ModelWay, Step, WayKind, forge::trials};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{Joint, account_acts as h},
};

const WORKER: Token = Token::new(7);
// The forge's task handle (temper-client-domain/src/forge/me.rs).
const TRIAL_TASK: u64 = 6;

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
    assert_eq!(root.person().forge().trials(), []);
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
    act(front, root, role, name, Act::Press);
}
fn act(front: &mut Joint, root: &mut Root, role: Role, name: &[u8], act: Act) {
    let mut view = drawn(root);
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("offered {}\n{}", String::from_utf8_lossy(name), shown(root)));
    assert!(!node.states.as_slice().contains(&State::Disabled), "offered control enabled");
    let id = node.id;
    let event = Temper::act(&mut view, id, act).expect("bound control");
    h::event(front, root, event);
}
fn checking() -> (Joint, Root) {
    let (mut front, mut root) = h::trial_fixture(fixtures::SEED);
    begin_check(&mut front, &mut root, h::PROVIDER);
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    (front, root)
}
fn begin_check(front: &mut Joint, root: &mut Root, provider: u32) {
    h::reopen(
        front,
        root,
        Address::Welcome {
            step: Step::Model { way: Some(ModelWay { provider, kind: WayKind::Program, source: None }) },
        },
    );
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::from_millis(1) });
    press(front, root, Role::Button, "I’ve signed in".as_bytes());
    running(root, 0, b"Trying: Signed in to your subscription");
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
    press(&mut front, &mut root, Role::Button, b"Confirm");
    assert_eq!(root.person().forge().trials(), []);
    assert!(front.engine.task(task).is_none(), "account removal cancels its trial");
    assert_eq!(front.assignments.len(), 1);
}

#[test]
fn a_restart_spends_no_second_turn() {
    let (mut front, mut root) = trying();
    let task = row(&root).task.expect("trial task");
    h::restart_trial(&mut front, &mut root);
    assert!(front.engine.task(task).is_none(), "restart cancels the restored attempt");
    assert!(root.person().forge().trials().is_empty(), "no trial fact survives");
    assert!(!has(&root, Role::Disclosure, b"Each step"));
    assert!(!shown(&root).contains("didn’t answer"));
    h::reopen(&mut front, &mut root, Address::Welcome { step: Step::Ready });
    h::elapse(&mut front, &mut root, Duration::from_millis(100));
    ready(&root, true);
    assert!(root.person().forge().trials().is_empty(), "a reload on the usable account asks nothing");
    assert_eq!(root.person().forge().trial_asked(), None);
    assert_eq!(front.assignments.len(), 1, "no restored or reloaded trial is claimed");
}

#[test]
fn a_later_account_is_the_one_tried() {
    let (mut front, mut root) = h::trial_selection_fixture(fixtures::SEED);
    h::event(
        &mut front,
        &mut root,
        Temper::went(Address::ConnectModel {
            way: Some(ModelWay { provider: h::PROVIDER, kind: WayKind::Key, source: None }),
        }),
    );
    act(
        &mut front,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: b"test-private-key".as_slice().into() },
    );
    press(&mut front, &mut root, Role::Button, b"Add key");
    h::kept(&mut front);
    h::settle(&mut front, &mut root);
    let first = row(&root).account;
    h::answer_trial(&mut front, &mut root);
    front.fault(
        front.engine.env.now.saturating_add(Duration::from_millis(1)),
        FrontFault::Slow { span: Duration::ZERO },
    );
    h::reopen(
        &mut front,
        &mut root,
        Address::ConnectModel {
            way: Some(ModelWay { provider: h::PROVIDER + 1, kind: WayKind::Program, source: None }),
        },
    );
    press(&mut front, &mut root, Role::Button, "I’ve signed in".as_bytes());
    login(&mut front, &mut root, true);
    let trial = row(&root);
    assert_ne!(trial.account, first, "a second model's usable account caused this trial");
    let later = trial.account;
    assert_eq!(trial.took, Some(later), "the trial takes the later account");
    let assignment = front.assignments.last().expect("second trial assigned");
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"second-scripted");
    assert_eq!(assignment.run.accounts.first().expect("assigned account").account, later);
    let task = trial.task.expect("trial task");
    // Read the person's real Task document already watched by the trial.
    let watched = root
        .person()
        .client()
        .document(jig_client::Handle { asker: temper_client_domain::owner::Owner::Forge.asker(), number: TRIAL_TASK })
        .expect("trial task watch");
    let Some(jig_client::Latest::Jig(jig_client::contract::Delivery::Task(document))) = watched.latest else {
        panic!("person task document");
    };
    assert_eq!(document.chip.task, task);
    let choice = document.llm.as_ref().expect("next_llm chose the task's model");
    assert_eq!(choice.main.model.bytes(), b"second-scripted");
    assert_eq!(choice.account, Some(later), "creator holds the later account");
    fail(&mut front, &mut root);
    assert_eq!(row(&root).state, trials::State::Failed { why: trials::Why::Ended { class: trials::Class::Run } });
    h::event(&mut front, &mut root, Temper::went(Address::Accounts));
    let mut view = drawn(&root);
    let later_nodes = account_nodes(&view, b"Second");
    assert!(later_nodes.iter().any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"stopped")));
    assert!(later_nodes.iter().any(|node| {
        node.text
            .iter()
            .any(|run| String::from_utf8_lossy(&run.text).contains("The run stopped before Second answered."))
    }));
    let retry = later_nodes
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Try again"))
        .expect("the later account offers its own retry")
        .id;
    let first_nodes = account_nodes(&view, b"Anthropic");
    assert!(first_nodes.iter().any(|node| {
        node.name.as_deref().is_some_and(|name| String::from_utf8_lossy(name).contains("answered in 1 s"))
    }));
    assert!(!first_nodes.iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Try again")));
    let event = Temper::act(&mut view, retry, Act::Press).expect("bound account retry");
    assert!(matches!(event.root(), Some(temper_client_domain::Event::Act {
        action: temper_client_domain::Action::Forge(temper_client_domain::forge::Act::TryAccount { account, again: true })
    }) if *account == later));
    let before = front.assignments.len();
    h::event(&mut front, &mut root, event);
    assert_eq!(front.assignments.len(), before + 1);
    assert_eq!(row(&root).account, later);
    assert_eq!(row(&root).took, Some(later));
    let retry = front.assignments.last().expect("later account retried");
    assert_eq!(retry.run.policy.model.name.as_ref(), b"second-scripted");
    assert_eq!(retry.run.accounts.first().expect("retried account").account, later);
    h::answer_trial(&mut front, &mut root);
}

fn account_nodes<'a>(view: &'a View, name: &[u8]) -> &'a [skein_ui::Node] {
    let nodes = Temper::tree(view).nodes();
    let at = nodes
        .iter()
        .position(|node| {
            node.role == Role::Disclosure && node.name.as_deref().is_some_and(|title| title.starts_with(name))
        })
        .expect("the account's drawn row");
    &nodes[at..at + usize::try_from(nodes[at].size).expect("row size")]
}

fn login(front: &mut Joint, root: &mut Root, usable: bool) {
    front.engine.send(temper_engine_domain_world::protocol::up(
        WORKER,
        LinkUp::Logins {
            logins: Box::new([jig_domain_link::Login {
                name: jig_domain_link::SourceName::new(b"claude-person").expect("fixture login"),
                state: if usable {
                    jig_domain_link::LoginState::Usable
                } else {
                    jig_domain_link::LoginState::NotSignedIn
                },
                email: None,
            }]),
        },
        temper_engine_domain::Parts::none(),
    ));
    h::settle(front, root);
}

#[test]
fn a_wait_says_so_at_once() {
    use jig_client::contract::{TaskRunWait, TaskWaitWhy};
    let (mut front, mut root) = checking();
    let account = temper_client_domain::pages::accounts::document(root.person().client())
        .expect("declared account")
        .accounts
        .as_slice()
        .first()
        .expect("one account")
        .number;
    let blocker = h::account_chat(&mut front, account);
    h::program_checked(&mut front, &mut root, true);
    let began = front.engine.env.now;
    login(&mut front, &mut root, false);
    let attempt = front
        .assignments
        .iter()
        .find(|assignment| assignment.task == blocker)
        .expect("older chat occupies the worker")
        .attempt;
    front.engine.answer(
        WORKER,
        blocker,
        attempt,
        0,
        jig_domain_tasks::End::Parked,
        temper_engine_domain::Parts::none(),
    );
    h::settle(&mut front, &mut root);
    assert_eq!(front.engine.env.now, began, "the next delivery, with no clock advanced");
    assert!(
        matches!(root.person().forge().trial_wait(), Some(TaskRunWait::Account { why: TaskWaitWhy::Host, .. })),
        "a first-usable-model wait: {:?}",
        root.person().forge().trial_wait()
    );
    assert_eq!(row(&root).state, trials::State::Running);
    stopped(&root, b"Claude Code starts on this machine");
    assert!(shown(&root).contains("login isn't signed in on a host that can run this."), "{}", shown(&root));
    assert!(has(&root, Role::Link, b"Sign in again"));
    assert!(!has(&root, Role::Button, b"Try again"), "a running trial offers no second turn");
    assert!(!shown(&root).contains("temper program-login"), "no tool command in the wait");
    h::elapse(&mut front, &mut root, Duration::from_secs(60));
    assert_eq!(row(&root).state, trials::State::Failed { why: trials::Why::Waited });
    assert!(shown(&root).contains("login isn't signed in on a host that can run this."));
    assert!(has(&root, Role::Button, b"Try again"));
}

fn without_models_line(root: &Root) -> Vec<skein_ui::Node> {
    let view = drawn(root);
    let nodes = Temper::tree(&view).nodes();
    let pane = nodes
        .iter()
        .position(|node| {
            node.role == Role::Complementary
                && node.styles.as_slice().contains(&temper_client_view::look::WELCOME_STATUS)
        })
        .expect("welcome's side pane");
    let end = pane + usize::try_from(nodes[pane].size).expect("pane size");
    let start = nodes
        .iter()
        .enumerate()
        .skip(pane)
        .take(end - pane)
        .find_map(|(at, node)| {
            (node.role == Role::Paragraph
                && node.name.as_deref().is_some_and(|name| name.starts_with(b"Claude subscription")))
            .then_some(at)
        })
        .expect("Model's line after Forge's line");
    let removed = u32::try_from(end - start).expect("Model's nodes");
    nodes
        .iter()
        .enumerate()
        .filter_map(|(at, node)| {
            if (start..end).contains(&at) {
                return None;
            }
            let mut node = node.clone();
            // Allocation ids and write revisions vary with deliveries; compare
            // every node's structure, content and controls.
            node.id = skein_ui::NodeId(0);
            if let Some(value) = &mut node.value {
                value.written = 0;
            }
            if at < start && at + usize::try_from(node.size).expect("node size") >= end {
                node.size -= removed;
            }
            Some(node)
        })
        .collect()
}

fn same_layout(root: &Root, at_press: &[skein_ui::Node], phase: &str) {
    let nodes = without_models_line(root);
    assert_eq!(nodes.len(), at_press.len(), "{phase}: same nodes");
    for (at, (node, expected)) in nodes.iter().zip(at_press).enumerate() {
        assert_eq!(node, expected, "{phase}: node {at}");
    }
}

#[test]
fn the_welcomes_layout_doesnt_change_for_the_check() {
    let (mut front, mut root) = h::trial_fixture(fixtures::SEED);
    begin_check(&mut front, &mut root, h::PROVIDER);
    let at_press = without_models_line(&root);
    ready(&root, true);
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::ZERO });
    h::settle(&mut front, &mut root);
    same_layout(&root, &at_press, "while the host's check is out");
    front.reply_in_pieces(&[b"Yes"]);
    h::program_checked(&mut front, &mut root, true);
    same_layout(&root, &at_press, "at the host's answer");
    active(&mut front, &mut root);
    same_layout(&root, &at_press, "while Claude answers");
    // Keep the try out across the request's 305 ms flash, on the world's clock.
    h::elapse(&mut front, &mut root, Duration::from_millis(305));
    same_layout(&root, &at_press, "305 ms into the try");
    assert!(front.release_reply(), "the worker commits its answer");
    h::settle(&mut front, &mut root);
    same_layout(&root, &at_press, "once the try answers");
    ready(&root, true);
    assert!(shown(&root).contains("answered in 1 s"));
}
