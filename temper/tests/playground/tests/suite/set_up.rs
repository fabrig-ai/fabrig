//! Owned setup on the world's box pipe (client/journey-walks-world.md, 1.4, and client/journey-walks.md, 7).
use skein_fake_person::{Find, Target};
use skein_lib::Duration;
use skein_ui::{Act, Node, Role};
use temper_client_walk_world::{Native, facts, web::Page};
use temper_playground::programs::Process;

const WITHIN: Duration = Duration::from_secs(60);

fn press(page: &mut Page, role: Role, name: &str) {
    let node = page.wait(&Find::exactly_named(role, name.as_bytes()), Some(&Act::Press), WITHIN).expect(name);
    page.act(node, Act::Press).expect("press");
}
fn steps(page: &Page) -> Vec<(String, Option<String>)> {
    let nodes = page.tree().nodes();
    let list = nodes
        .iter()
        .position(|n| n.role == Role::List && n.name.as_deref() == Some(b"Each step"))
        .expect("Each step list");
    let end = list + usize::try_from(nodes[list].size).expect("subtree size");
    nodes[list + 1..end]
        .iter()
        .enumerate()
        .filter(|(_, n)| n.role == Role::Item)
        .map(|(offset, n)| {
            let at = list + 1 + offset;
            let end = at + usize::try_from(n.size).expect("item size");
            let status = nodes[at + 1..end]
                .iter()
                .find(|n| n.role == Role::Status)
                .and_then(|n| n.name.as_deref())
                .map(|name| String::from_utf8_lossy(name).into_owned());
            (nodes[at + 1..end].iter().take_while(|n| n.role != Role::Status).map(text).collect(), status)
        })
        .collect()
}
fn text(node: &Node) -> String {
    node.text.iter().map(|run| String::from_utf8_lossy(&run.text)).collect()
}

#[test]
fn a_sign_in_tab_returns_to_the_listener_and_the_welcome_reaches_ready() {
    let mut page = Page::start(facts::start(None).expect("fixed facts"), b"sign-in").expect("page");
    press(&mut page, Role::Link, "Set up temper");
    press(&mut page, Role::Link, "Run on this machine");
    press(&mut page, Role::Button, "Run on this machine");
    press(&mut page, Role::Button, "Continue with ChatGPT");
    page.wait(&Find::exactly_named(Role::Button, b"Create your project"), Some(&Act::Press), WITHIN)
        .unwrap_or_else(|why| panic!("Ready: {why:?}\n{}", page.html()));
    let tab = page.tab().expect("sign-in opened and pointed a tab");
    assert_eq!(tab.status, 200);
    assert!(!tab.failed);
    assert_eq!(&*tab.body, b"Sign-in received. You can close this tab.\n");
    assert!(tab.address.starts_with(b"http://127.0.0.1:1455/auth/callback?"));
    assert!(page.html().contains("ChatGPT plan · sam@example.com"), "{}", page.html());
    // A fresh accounts read, not a locally optimistic connected state, sees the
    // account retained by the service after the callback's code exchange.
    page.reload(b"/welcome?at=ready", WITHIN).expect("fresh welcome read");
    page.wait(&Find::exactly_named(Role::Paragraph, "ChatGPT plan · sam@example.com".as_bytes()), None, WITHIN)
        .expect("service holds the signed-in account");
}

#[test]
fn owned_setup_connects_and_makes_its_sample_at_the_walks_forge() {
    let start = facts::start(None).expect("fixed facts");
    let shared = start.fakes.forge.forge.clone();
    let mut page = Page::start(start, b"owned forge").expect("page");
    let forge = page
        .world()
        .processes()
        .iter()
        .find_map(|process| if let Process::Forge(peer) = process { peer.address() } else { None });
    assert_eq!(forge, Some(facts::FORGE.parse().expect("fixed forge address")));
    press(&mut page, Role::Link, "Set up temper");
    press(&mut page, Role::Link, "Run on this machine");
    press(&mut page, Role::Button, "Run on this machine");
    page.wait(
        &Find::exactly_named(
            Role::Paragraph,
            "Forgejo on this machine · connected as temper · work/hello made".as_bytes(),
        ),
        None,
        WITHIN,
    )
    .unwrap_or_else(|why| panic!("done: {why:?}\n{}", page.html()));
    let service = page
        .world()
        .processes()
        .iter()
        .find_map(|process| if let Process::Service(service) = process { Some(service) } else { None })
        .expect("service");
    let engine = service.service.engine_domain().expect("engine");
    let forges = engine.forges();
    let [row] = forges.rows.as_ref() else { panic!("one owned forge") };
    assert_eq!(row.host.as_ref(), facts::FORGE.as_bytes());
    let record = engine.forge_record(row.number).expect("kept owned forge");
    assert_eq!(record.place.api.host.as_ref(), b"127.0.0.1");
    assert_eq!(record.place.api.port, 10002);
    assert_eq!(record.place.git.as_ref(), b"http://127.0.0.1:10002");
    if let Some(web) = &record.place.web {
        assert_eq!(web.as_ref(), b"http://127.0.0.1:10002");
    }
    assert!(shared.borrow().domain.metadata(b"work/hello").is_ok(), "the start's shared forge made the sample");

    let directories = jig_protocol_config::Directories::given(b"/config", b"/setup", b"/cache").expect("directories");
    let path = directories
        .join(
            jig_protocol_config::Directory::Data,
            &temper_forge_setup::setup_file::path(row.number, temper_forge_setup::machine::BoxKind::Owned),
        )
        .expect("setup path");
    let read = temper_service::HandoverRead { forge: row.number, run: skein_lib::Token::new(0), path, max: 65536 };
    let bytes = temper_playground::service::read_handover(&mut service.parameters.disk.borrow_mut(), &read)
        .expect("setup read")
        .expect("setup file");
    let setup = temper_forge_setup::setup_file::decode(bytes.expose(), &directories).expect("owned setup file");
    assert_eq!(setup.listen(), facts::FORGE.parse().expect("fixed forge address"));
    let temper_forge_setup::setup_file::SetupFile::Owned(owned) = setup else { panic!("owned setup") };
    assert_eq!(owned.remote.as_ref(), b"http://127.0.0.1:10002");
}

#[test]
fn owned_setup_draws_every_pipe_step_before_the_next_line() {
    let mut page = Page::start(facts::start(None).expect("fixed facts"), b"owned").expect("page");
    press(&mut page, Role::Link, "Set up temper");
    page.wait(&Find::text(b"Forgejo"), None, WITHIN).expect("Forgejo card");
    press(&mut page, Role::Link, "Run on this machine");
    let sample =
        page.wait(&Find::exactly_named(Role::Choice, b"A sample project"), None, WITHIN).expect("sheet defaults");
    assert!(page.tree().find(sample).expect("sample choice").states.as_slice().contains(&skein_ui::State::Selected));
    assert!(!page.tree().nodes().iter().any(|node| node.role == Role::TextField));
    for value in [b"work".as_slice(), b"hello"] {
        assert!(
            !page.tree().nodes().iter().any(|node| node.value.as_ref().is_some_and(|v| v.text.as_ref() == value)),
            "sample name stays hidden: {}",
            String::from_utf8_lossy(value)
        );
    }
    press(&mut page, Role::Button, "Run on this machine");
    let states = [
        (0, "Fetching Forgejo", 0),
        (2, "Fetching Forgejo · 25%", 0),
        (4, "Fetching Forgejo · 100%", 0),
        (6, "Fetching Forgejo · 50%", 0),
        (8, "Fetching Forgejo · 100%", 0),
        (10, "Preparing Forgejo", 1),
        (12, "Starting Forgejo", 2),
        (14, "Connecting sign-in through temper", 2),
        (18, "Connecting as temper", 3),
    ];
    let mut began = None;
    for (index, &(after, words, current)) in states.iter().enumerate() {
        let find = Find::exactly_named(Role::Status, b"under way").within(Box::new([
            Target::ExactRole { role: Role::Complementary, name: b"Forge and model".as_slice().into() },
            Target::ExactRole { role: Role::List, name: b"Each step".as_slice().into() },
            Target::Role { role: Role::Item, name: words.as_bytes().into() },
        ]));
        page.wait(&find, None, WITHIN).unwrap_or_else(|why| panic!("{words}: {why:?} at {:?}", page.now()));
        let start = *began.get_or_insert_with(|| {
            page.world()
                .processes()
                .iter()
                .find_map(|p| if let Process::Other(Native::Box(peer)) = p { peer.began() } else { None })
                .expect("owned start read")
        });
        let now = page.now();
        assert!(
            now >= start.saturating_add(Duration::from_secs(after)),
            "{words} drawn before its pipe line at {now:?}"
        );
        // `source added` is a line at 16 s, but does not change the visible step.
        let next = if after == 14 { Some(16) } else { states.get(index + 1).map(|state| state.0) };
        if let Some(next) = next {
            assert!(
                now < start.saturating_add(Duration::from_secs(next)),
                "{words} drawn after the next pipe line at {now:?}"
            );
        }
        let drawn = steps(&page);
        assert_eq!(drawn.len(), 5);
        assert_eq!(drawn[4].0, "Making work/hello");
        assert_eq!(drawn[current].0, words);
        for (at, (_, status)) in drawn.iter().enumerate() {
            let wanted = match at.cmp(&current) {
                std::cmp::Ordering::Less => Some("done"),
                std::cmp::Ordering::Equal => Some("under way"),
                std::cmp::Ordering::Greater => None,
            };
            assert_eq!(status.as_deref(), wanted, "{words}: {drawn:?}");
        }
    }
    page.wait(
        &Find::exactly_named(
            Role::Paragraph,
            "Forgejo on this machine · connected as temper · work/hello made".as_bytes(),
        ),
        None,
        WITHIN,
    )
    .unwrap_or_else(|why| panic!("done: {why:?}\n{}", page.html()));
    assert!(page.now() >= began.expect("start").saturating_add(Duration::from_secs(18)));
    assert_eq!(
        page.world().processes().iter().filter(|p| matches!(p, Process::Other(_))).count(),
        1,
        "only the peer, no ctl"
    );
}
