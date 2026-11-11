//! A person's landed draft crosses admission, forge reads and the commit barrier.
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people::{self as people, ApplicationScope};
use skein_forge::api;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};
const PATH: &[u8] = b"docs/design/draft/catch-up.md";
const WATCHER: Token = Token::new(700);
fn config() -> root::Config {
    let mut config = walking::config(71);
    let other = jig_domain_core::SeedProject { number: 2, name: "Other".into(), policy: config.core.seed_projects[0].policy.clone() };
    config.core.seed_projects = config.core.seed_projects.into_vec().into_iter().chain([other]).collect();
    config
}
fn application(driver: &mut Driver, sign_in: u64, key: u8, request: root::Application) {
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope: ApplicationScope::Project(2), reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in, key: [key; 16], digest: [key; 32], carried: None, request,
    }));
}
fn request(path: &[u8], change: forge::DraftChange) -> root::Application {
    root::Application::Forge(forge::Ask::SetDraft(forge::SetDraft { project: 1, repository: None, path: path.into(), change }))
}
fn answers(driver: &Driver) -> Vec<Option<root::ApplicationAnswer>> {
    driver.delivered.iter().filter_map(|delivery| match delivery { root::Delivery::Application { outcome, .. } => Some(outcome.clone()), _ => None }).collect()
}
fn setup() -> (Driver, u64, u64) {
    use skein_fake_forge_domain::api as raw;
    let mut limits = fixtures::forge_limits();
    limits.people.projects = 3;
    limits.people.requests = 16;
    limits.authority.projects = 2;
    let mut driver = Driver::configured(Store::new(), [config(), config()], &limits);
    driver.forge = Some(ForgeStand::with_setup(Box::new([raw::Setup {
        id: 2, name: b"org/repo".as_slice().into(), default: b"main".as_slice().into(),
        tree: Box::new([raw::File { path: PATH.into(), content: b"# Catch-up barrier\n\nA landed draft.\n".as_slice().into() }]),
        workflows: Box::new([]), labels: Box::new([]), protection: None, hooked: false,
        checks: raw::Checks { cues: Box::new([]), contexts: Box::new([]), latency_min: skein_lib::Duration::ZERO, latency_max: skein_lib::Duration::ZERO, silent: 0, passes: 1000, reruns: 0, cue: None },
    }])));
    driver.connect_forges(); driver.settle(); driver.sign_in(); driver.settle();
    let (party, sign_in) = driver.delivered.iter().find_map(|delivery| match delivery { root::Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. } => Some((*person, *sign_in)), _ => None }).expect("signed-in writer");
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope: ApplicationScope::Project(1), reply_to: ReplyTo::new(Token::new(95)), sign_in,
        key: [95; 16], digest: [95; 32], carried: None,
        request: root::Application::Forge(forge::Ask::Adopt(fixtures::adoption(1))),
    }));
    driver.settle(); driver.delivered.clear();
    (driver, party, sign_in)
}
fn watch(driver: &mut Driver, sign_in: u64, project: u32, watcher: Token, subject_project: u32) {
    driver.send(root::Event::Party(root::PartyInput::Watch {
        watcher, sign_in, subject: Watched::Application { scope: ApplicationScope::Project(project), connector: 0, subject: Token::new(forge::drafts_subject(subject_project)) }, window: Window { rows: 1, after: None },
    })); driver.settle();
}
fn document(driver: &Driver) -> forge::DraftsView {
    driver.delivered.iter().rev().find_map(|delivery| match delivery {
        root::Delivery::ForgeDocument { watcher, document } if *watcher == WATCHER => match document.as_ref() { forge::Document::Drafts(view) => Some(view.clone()), _ => None }, _ => None,
    }).expect("drafts subject delivered")
}
fn kept(driver: &Driver) -> forge::DraftView {
    let answers = answers(driver);
    let [Some(root::ApplicationAnswer::SetDraft(forge::SetDraftAnswer::Kept(view)))] = answers.as_slice() else { panic!("drafted: {answers:?}") };
    view.clone()
}
fn restart(driver: &mut Driver) -> Driver {
    let peer = driver.forge.take(); let limits = driver.env.limits; let wall = driver.env.wall; let store = driver.take_store();
    let mut driver = Driver::from_store_at(store, config(), &limits, wall);
    driver.forge = peer; driver.connect_forges(); driver.settle(); driver
}
#[test]
fn ready_drafts_wait_for_the_commit_and_survive_restart_whole() {
    let (mut driver, party, sign_in) = setup();
    let api::Answer::Commit(head) = driver.forge.as_mut().expect("forge").read(api::Repository { forge: 1, repository: 2 }, api::Read::Branch { branch: b"main".as_slice().into() }) else { panic!("main head") };
    application(&mut driver, sign_in, 120, request(PATH, forge::DraftChange::Ready { chat: Some(7) }));
    for _ in 0..16 { driver.advance(false); }
    assert!(answers(&driver).is_empty(), "no answer before commit"); assert!(!driver.store.pending.is_empty());
    driver.settle(); let expected = kept(&driver);
    assert_eq!(expected.row.path.as_ref(), PATH); assert_eq!(expected.row.branch.as_ref(), b"main"); assert_eq!(expected.row.commit, head);
    assert_eq!(expected.row.title.as_ref(), b"Catch-up barrier"); assert_eq!(expected.row.chat, Some(7)); assert_eq!(expected.row.goal, None); assert!(expected.row.ready); assert_eq!(expected.row.writer, party);
    watch(&mut driver, sign_in, 1, WATCHER, 1);
    assert_eq!(document(&driver).rows.as_ref(), std::slice::from_ref(&expected));
    for (key, change) in [(121, forge::DraftChange::NotReady), (122, forge::DraftChange::Started { goal: 40 }), (123, forge::DraftChange::Ready { chat: None })] {
        driver.delivered.clear(); application(&mut driver, sign_in, key, request(PATH, change)); driver.settle(); let row = kept(&driver).row;
        match change {
            forge::DraftChange::NotReady => { assert!(!row.ready); assert_eq!(row.goal, None); }
            forge::DraftChange::Started { goal } => { assert!(!row.ready); assert_eq!(row.goal, Some(goal)); }
            forge::DraftChange::Ready { .. } => { assert!(row.ready); assert_eq!(row.goal, None); assert_eq!(row.chat, Some(7)); }
        }
    }
    let mut driver = restart(&mut driver);
    watch(&mut driver, sign_in, 1, WATCHER, 1); assert_eq!(document(&driver).rows.as_ref(), [expected]);
}
#[test]
fn bad_paths_do_not_read_and_missing_unknown_and_unadopted_are_refused() {
    let (mut driver, _, sign_in) = setup(); let long = format!("{}.md", "a".repeat(1022));
    for (key, path, change, why) in [
        (120, b"/abs.md".as_slice(), forge::DraftChange::Ready { chat: None }, forge::DraftRefusal::Path),
        (121, b"a/../b.md".as_slice(), forge::DraftChange::Ready { chat: None }, forge::DraftRefusal::Path),
        (122, b"notes.txt".as_slice(), forge::DraftChange::Ready { chat: None }, forge::DraftRefusal::Path),
        (123, long.as_bytes(), forge::DraftChange::Ready { chat: None }, forge::DraftRefusal::Path),
        (124, b"docs/design/draft/nothing.md".as_slice(), forge::DraftChange::Ready { chat: None }, forge::DraftRefusal::Missing),
        (125, b"docs/design/draft/unknown.md".as_slice(), forge::DraftChange::NotReady, forge::DraftRefusal::Unknown),
    ] {
        driver.delivered.clear(); let calls = driver.forge.as_ref().expect("forge").calls;
        application(&mut driver, sign_in, key, request(path, change)); driver.settle();
        assert_eq!(answers(&driver), [Some(root::ApplicationAnswer::SetDraft(forge::SetDraftAnswer::Refused(why)))]);
        if why == forge::DraftRefusal::Path || why == forge::DraftRefusal::Unknown { assert_eq!(driver.forge.as_ref().expect("forge").calls, calls); }
    }
    driver.delivered.clear();
    application(&mut driver, sign_in, 126, root::Application::Forge(forge::Ask::SetDraft(forge::SetDraft { project: 1, repository: Some(forge::DraftRepository { owner: b"org".as_slice().into(), name: b"not-adopted".as_slice().into() }), path: PATH.into(), change: forge::DraftChange::Ready { chat: None } })));
    driver.settle(); assert_eq!(answers(&driver), [Some(root::ApplicationAnswer::SetDraft(forge::SetDraftAnswer::Refused(forge::DraftRefusal::NoRepository)))]);
}
#[test]
fn draft_requests_require_a_role_and_documents_do_not_cross_projects() {
    let (mut driver, _, sign_in) = setup();
    application(&mut driver, sign_in, 120, request(PATH, forge::DraftChange::Ready { chat: None })); driver.settle(); driver.delivered.clear();
    application(&mut driver, sign_in, 121, root::Application::Forge(forge::Ask::SetDraft(forge::SetDraft { project: 2, repository: None, path: PATH.into(), change: forge::DraftChange::NotReady })));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })), .. })));
    assert!(answers(&driver).is_empty());
    driver.delivered.clear();
    driver.send(root::Event::Party(root::PartyInput::SignedIn { held: None, reply_to: ReplyTo::new(Token::new(300)), identity: people::Identity { key: forge::user_key(1, 9), login: b"other".as_slice().into(), name: b"Other".as_slice().into() } })); driver.settle();
    let (other, other_sign_in) = driver.delivered.iter().find_map(|delivery| match delivery { root::Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. } => Some((*person, *sign_in)), _ => None }).expect("second project's watcher");
    driver.store.seed(Box::new([root::Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 2, holdings: Box::new([people::Holding { person: other, role: people::Role::Observer }]) }))]), Box::new([]));
    let mut driver = restart(&mut driver);
    watch(&mut driver, other_sign_in, 2, WATCHER, 2); assert!(document(&driver).rows.is_empty()); driver.delivered.clear();
    watch(&mut driver, other_sign_in, 2, Token::new(701), 1);
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::ForgeDocument { .. })));
}
