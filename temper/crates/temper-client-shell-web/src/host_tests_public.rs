//! Public host at the fetch boundary (temper/docs/design/client/web/public.md, 4).
use super::*;
fn fixture(path: &[u8]) -> (PublicHost, Duties) {
    let mut host = PublicHost::new(Time::ZERO, Wall::EPOCH, Ways::NONE);
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
    host.report(Report::Heard(Heard::Answered {
        exchange: id,
        status: 200,
        body: projects(),
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
    let (mut host, duties) = fixture(b"/public/projects/1");
    assert_eq!(
        duties.show.as_ref().map(|(path, replace)| (path.as_ref(), *replace)),
        Some((b"/public".as_slice(), true))
    );
    let old = exchange(duties);
    let duties = host.report(Report::Place { path: b"/public/projects/0123456789abcdef".as_slice().into() });
    assert_eq!(duties.asked.len(), 2);
    answered(&mut host, old);
    assert!(matches!(host.reader.shown(), temper_client_domain::public::Shown::Reading));
}

fn projects() -> Box<[u8]> {
    use jig_client_protocol_wire::public_v1 as j;
    let projects =
        j::PublicProjects::new(&j::CEILINGS, j::PublicProjectsParts { projects: List::with_capacity(0), max_age: 8 })
            .expect("projects");
    let mut writer = skein_lib::Writer::new(usize::try_from(projects.measure()).expect("small projects"));
    projects.encode(&mut writer).expect("projects record");
    writer.finish()
}
