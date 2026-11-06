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
    front::account_acts as h,
};

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
