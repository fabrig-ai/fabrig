//! Public host at the fetch boundary (temper/docs/design/client/web/public.md, 4).
use super::*;
use temper_client_view_kit::public_records as records;

const TASK_PATH: &[u8] = b"/public/projects/3f2a9c0176543210/tasks/9abcdef076543210";
fn target(owner: u16, layout: [u8; 32], kind_and_ids: &str) -> String {
    use std::fmt::Write as _;
    let mut path = format!("/jig/v1/public/{owner}/");
    for byte in layout {
        write!(path, "{byte:02x}").expect("writes to a string");
    }
    write!(path, "/{kind_and_ids}").expect("writes to a string");
    path
}

fn fixture(path: &[u8]) -> (PublicHost, Duties) {
    let mut host = PublicHost::new(Time::ZERO, Wall::EPOCH, Ways::parse("", b"https://temper.example"));
    host.start(path);
    let duties = host.report(Report::Start);
    (host, duties)
}
fn exchange(duties: Duties) -> u32 {
    let mut id = None;
    for asked in duties.asked {
        match asked {
            Asked::Exchange { id: exchange, .. } => {
                id = Some(exchange);
            }
            Asked::OpenStream { .. } | Asked::CloseStream | Asked::Save { .. } | Asked::Erase { .. } => {
                panic!("only public exchanges")
            }
        }
    }
    id.expect("public exchange")
}
fn answered(host: &mut PublicHost, id: u32) -> Duties {
    answer(host, id, 200, records::projects_with(false))
}
fn answer(host: &mut PublicHost, id: u32, status: u16, body: Box<[u8]>) -> Duties {
    host.report(Report::Heard(Heard::Answered {
        exchange: id,
        status,
        body,
        headers: vec![
            ("cache-control".into(), "public, max-age=8, must-revalidate".into()),
            ("age".into(), "3".into()),
        ],
    }))
}
#[test]
fn the_public_mode_sends_no_credential_and_opens_no_stream() {
    for path in [
        b"/public".as_slice(),
        b"/public/projects/0123456789abcdef",
        b"/public/projects/0123456789abcdef/tasks/000000000000000a",
        b"/public/projects/0123456789abcdef/landings",
        b"/public/projects/0123456789abcdef/landings/000000000000000a",
    ] {
        let (_, duties) = fixture(path);
        assert_eq!(duties.writes, 0);
        assert!(!duties.asked.is_empty());
        for asked in duties.asked {
            match asked {
                Asked::Exchange { call, cache, keep, .. } => {
                    assert_eq!(call.method.as_bytes(), b"GET");
                    assert!(call.path.starts_with(b"/jig/v1/public/"));
                    assert!(call.body.is_empty());
                    assert!(call.headers.iter().all(|header| !header.name.eq_ignore_ascii_case(b"authorization")));
                    assert_eq!(cache, Cache::NoStore);
                    assert_eq!(keep, http::PUBLIC_KEEP);
                }
                Asked::OpenStream { .. } | Asked::CloseStream | Asked::Save { .. } | Asked::Erase { .. } => {
                    panic!("only public GETs")
                }
            }
        }
    }
}
#[test]
fn the_freshness_comes_from_the_kept_headers() {
    let (mut host, duties) = fixture(b"/public");
    let id = exchange(duties);
    answered(&mut host, id);
    assert_eq!(host.deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(5))));
    host.clocks(Time::ZERO.saturating_add(Duration::from_secs(4)), Wall::EPOCH);
    assert!(host.report(Report::Due).asked.is_empty());
    host.clocks(Time::ZERO.saturating_add(Duration::from_secs(5)), Wall::EPOCH);
    assert_eq!(host.report(Report::Due).asked.len(), 1);
}
#[test]
fn the_timer_follows_the_reader() {
    let (mut host, duties) = fixture(b"/public");
    let id = exchange(duties);
    answered(&mut host, id);
    let mut page = crate::host::page(&host.bounds, skein_ui_web::mapping::Primary::Control);
    host.draw(&mut page, &mut Vec::new());
    assert_eq!(host.deadline(), host.reader.deadline());
    host.report(Report::Visible { visible: false });
    assert_eq!(
        host.deadline(),
        Some(Time::ZERO.saturating_add(Duration::from_secs(5))),
        "fresh content still expires hidden"
    );
    host.clocks(Time::ZERO.saturating_add(Duration::from_secs(5)), Wall::EPOCH);
    assert!(host.report(Report::Due).asked.is_empty());
    assert_eq!(host.reader.shown(), temper_client_domain::public::Shown::Unavailable);
    assert_eq!(host.deadline(), None);
    assert_eq!(host.report(Report::Visible { visible: true }).asked.len(), 1);
    assert_eq!(host.deadline(), host.reader.deadline());
}
#[test]
fn malformed_paths_are_replaced_and_old_answers_are_ignored() {
    for path in [b"/public/projects/1".as_slice(), b"/public/projects/ABCDEF0123456789"] {
        let (mut host, duties) = fixture(path);
        assert_eq!(
            duties.show.as_ref().map(|(path, replace)| (path.as_ref(), *replace)),
            Some((b"/public".as_slice(), true)),
            "replace malformed path {path:?}"
        );
        let old = exchange(duties);
        let duties = host.report(Report::Place { path: b"/public/projects/0123456789abcdef".as_slice().into() });
        assert_eq!(duties.asked.len(), 2);
        answered(&mut host, old);
        assert!(matches!(host.reader.shown(), temper_client_domain::public::Shown::Reading));
    }
}

#[test]
fn a_task_made_private_reads_unknown_at_its_next_read() {
    let (mut host, duties) = fixture(TASK_PATH);
    let mut page = crate::host::page(&host.bounds, skein_ui_web::mapping::Primary::Control);
    assert!(duties.show.is_none());
    assert_eq!(duties.asked.len(), 2);
    let card_path = target(0, jig_client_protocol_wire::PUBLIC_LAYOUT, "3/3f2a9c0176543210/9abcdef076543210");
    let change_path = target(2, temper_client_protocol_wire::PUBLIC_LAYOUT, "1/3f2a9c0176543210/9abcdef076543210");
    let card = asked_for(&duties, card_path.as_bytes());
    let change = asked_for(&duties, change_path.as_bytes());
    answer(&mut host, card, 200, records::card());
    answer(&mut host, change, 200, records::change());
    host.draw(&mut page, &mut Vec::new());
    assert!(words(&host).contains(&b"A public goal title".as_slice()));
    assert!(words(&host).contains(&b"Approved".as_slice()), "the change was shown too");

    host.clocks(Time::ZERO.saturating_add(Duration::from_secs(5)), Wall::EPOCH);
    let due = host.report(Report::Due);
    assert!(due.show.is_none());
    assert_eq!(due.asked.len(), 2);
    let card = asked_for(&due, card_path.as_bytes());
    let change = asked_for(&due, change_path.as_bytes());
    let unknown = answer(&mut host, card, 404, Box::default());
    assert!(unknown.show.is_none(), "Unknown leaves the task address alone");
    let mut ops = Vec::new();
    host.draw(&mut page, &mut ops);
    let unknown_words: &[&[u8]] = &[b"temper", b"Sign in", b"Not public, or not here.", b"Read public projects"];
    assert_eq!(words(&host), unknown_words, "Unknown replaces the card and change");

    let late = answer(&mut host, change, 200, records::change());
    assert!(late.show.is_none());
    host.draw(&mut page, &mut Vec::new());
    assert_eq!(words(&host), unknown_words, "a late change leaves Unknown in place");

    let node = host
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Link && node.name.as_deref() == Some(b"Read public projects"))
        .expect("Unknown's link")
        .id;
    assert!(
        ops.iter().any(|op| matches!(op, Op::Href { node: id, path } if *id == node && path.as_ref() == b"/public"))
    );
    let projects = host.report(Report::Act { node, act: Act::Press });
    assert_eq!(projects.show, Some((b"/public".as_slice().into(), false)));
    assert_eq!(projects.asked.len(), 1);
    answered(&mut host, asked_for(&projects, target(0, jig_client_protocol_wire::PUBLIC_LAYOUT, "1").as_bytes()));
    host.draw(&mut page, &mut Vec::new());
    let empty_words: &[&[u8]] = &[b"temper", b"Sign in", b"Public projects", b"No public projects yet."];
    assert_eq!(words(&host), empty_words);
}

fn asked_for(duties: &Duties, path: &[u8]) -> u32 {
    duties
        .asked
        .iter()
        .find_map(|asked| match asked {
            Asked::Exchange { id, call, .. } if call.path.as_ref() == path => Some(*id),
            Asked::Exchange { .. } => None,
            Asked::OpenStream { .. } | Asked::CloseStream | Asked::Save { .. } | Asked::Erase { .. } => {
                panic!("only public exchanges")
            }
        })
        .unwrap_or_else(|| panic!("no exchange for {path:?}"))
}

fn words(host: &PublicHost) -> Vec<&[u8]> {
    host.tree().nodes().iter().flat_map(|node| node.text.iter().map(|run| run.text.as_ref())).collect()
}
