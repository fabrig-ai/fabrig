//! Owned setup on the world's box pipe (journey-walks.md, 4.4 and 7).
use skein_fake_person::{Find, Target};
use skein_lib::Duration;
use skein_ui::{Act, Node, Role};
use temper_client_walk_world::{Native, box_peer::OWNED, facts, web::Page};
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
        ("Fetching Forgejo", 0),
        ("Fetching Forgejo · 25%", 0),
        ("Fetching Forgejo · 100%", 0),
        ("Fetching Forgejo · 50%", 0),
        ("Fetching Forgejo · 100%", 0),
        ("Preparing Forgejo", 1),
        ("Starting Forgejo", 2),
        ("Connecting as temper", 3),
    ];
    let mut began = None;
    for (index, (line, (words, current))) in OWNED.iter().zip(states).enumerate() {
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
        assert!(now >= start.saturating_add(line.after), "{words} drawn before its pipe line at {now:?}");
        if let Some(next) = OWNED.get(index + 1) {
            assert!(now < start.saturating_add(next.after), "{words} drawn after the next pipe line at {now:?}");
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
    assert!(page.now() >= began.expect("start").saturating_add(Duration::from_secs(14)));
    assert_eq!(
        page.world().processes().iter().filter(|p| matches!(p, Process::Other(_))).count(),
        1,
        "only the peer, no ctl"
    );
}
