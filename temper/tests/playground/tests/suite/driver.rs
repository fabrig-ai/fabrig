//! Journey driver behaviour (temper/docs/design/client/journey-walks.md, 3.2).
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_lib::Duration;
use skein_ui::{Act, Role, State};
use skein_ui_web::log;
use temper_client_walk_world::{
    facts,
    web::{Failed, Page},
};

const WITHIN: Duration = Duration::from_secs(60);

fn start() -> Page {
    Page::start(facts::start(None).expect("fixed start"), b"a").expect("native page starts")
}

fn welcome(page: &mut Page) -> skein_ui::NodeId {
    page.wait(&Find::exactly_named(Role::Button, b"Try the playground"), Some(&Act::Press), WITHIN)
        .expect("welcome appears")
}

#[test]
fn fixed_facts_set_the_listener_and_reject_bad_scripts() {
    let start = facts::start(None).expect("fixed start");
    assert_eq!(start.seed, 7);
    assert_eq!(start.wall.as_secs(), 1_767_690_000);
    assert_eq!(start.deployment.document["application"]["clients"]["listener"]["socket"], "127.0.0.1:10001");
    let mut world = temper_client_walk_world::world(facts::start(None).expect("owned start"));
    let mut printed = Vec::new();
    for _ in 0..10_000 {
        temper_playground::story::step(&mut world);
        printed.extend(world.printed());
        if !printed.is_empty() {
            break;
        }
    }
    assert!(temper_playground_world::printed_link(&printed).starts_with("http://127.0.0.1:10001/"));
    let page = Page::start(start, b"a").expect("fixed page");
    let bytes = page.log();
    let log = log::read(&bytes).expect("page log");
    assert_eq!(log.seed, facts::SEED);
    assert_eq!(log.start, facts::WALL);
    assert!(facts::start(Some(b"not json")).is_err());
}

#[test]
fn late_welcome_draws_html_and_press_reaches_the_new_chat_composer() {
    let mut page = start();
    assert_eq!(
        TreeFace::new(page.tree()).find(&Find::exactly_named(Role::Button, b"Try the playground")),
        Found::Absent
    );
    let node = welcome(&mut page);
    let html = page.html();
    assert!(html.contains("Try the playground"));
    assert!(html.contains(&format!("data-node=\"{}\"", node.0)));
    page.wait(&Find::text(b"Fully offline"), None, WITHIN).expect("welcome text");
    assert_eq!(
        page.wait(
            &Find::exactly_named(Role::Button, b"Try the playground"),
            Some(&Act::Input { value: Box::new([]) }),
            Duration::ZERO,
        ),
        Err(Failed::Bound),
        "a button that cannot take input does not count",
    );
    page.act(node, Act::Press).expect("press welcome");
    page.wait(&Find::exactly_named(Role::Button, b"Add CSV export to widget"), Some(&Act::Press), WITHIN)
        .expect("first New chat suggestion");
    let name = b"What do you want done?";
    assert_eq!(TreeFace::new(page.tree()).find(&Find::exactly_named(Role::TextField, name)), Found::Absent);
    let composer = page
        .wait(&Find::text_box(name), Some(&Act::Input { value: b"A request".as_slice().into() }), WITHIN)
        .expect("text area is a text box");
    assert_eq!(page.tree().find(composer).expect("composer node").role, Role::TextArea);
}

#[test]
fn absent_control_spends_the_bound_in_world_time() {
    let mut page = start();
    let began = page.world().now();
    assert_eq!(
        page.wait(&Find::exactly_named(Role::Button, b"Never drawn"), Some(&Act::Press), WITHIN),
        Err(Failed::Bound)
    );
    assert!(page.world().now() >= began.saturating_add(WITHIN));
}

#[test]
fn reload_keeps_the_night_look_and_draws_main() {
    let mut page = start();
    welcome(&mut page);
    let initial = page.host().night();
    let night =
        page.wait(&Find::exactly_named(Role::Button, b"Night mode"), Some(&Act::Press), WITHIN).expect("night switch");
    page.act(night, Act::Press).expect("toggle night");
    assert_eq!(page.host().night(), !initial);
    page.reload(b"/welcome", WITHIN).expect("reload at own address");
    assert_eq!(page.host().night(), !initial);
    assert!(page.tree().nodes().iter().any(|node| node.role == Role::Main));
    page.wait(&Find::exactly_named(Role::Button, b"Try the playground"), Some(&Act::Press), WITHIN)
        .expect("signed-in welcome after reload");
    let night = page
        .wait(&Find::exactly_named(Role::Button, b"Night mode"), Some(&Act::Press), WITHIN)
        .expect("reloaded switch");
    assert_eq!(
        page.tree().find(night).expect("drawn night switch").states.as_slice().contains(&State::Pressed),
        !initial
    );
}

#[test]
fn the_same_start_and_wait_draw_identical_html() {
    let mut first = start();
    let mut second = start();
    welcome(&mut first);
    welcome(&mut second);
    assert_eq!(first.html(), second.html());
}

#[test]
fn a_start_that_keeps_its_link_fails_as_no_link() {
    let mut start = facts::start(None).expect("fixed start");
    start.links = temper_playground::story::Links::Keep;
    assert!(matches!(Page::start(start, b"a"), Err(Failed::NoLink)));
}
