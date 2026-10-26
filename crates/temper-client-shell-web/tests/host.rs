//! The host's world: the platform's reports given by the test, through the seam
//! (client/journeys.md, 2 and 6).
use jig_client_shell_web::adapter::Asked;
use skein_lib::{Time, Wall};
use skein_ui_web::mapping::Primary;
use temper_client_shell_web::host::{self, Duties, Host, Report};

fn started(path: &[u8]) -> Host {
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
    host.start(path, Vec::new(), None);
    host
}

#[test]
fn a_start_opens_the_event_stream() {
    let mut host = started(b"/accounts/connect");
    let Duties { asked, .. } = host.report(Report::Start);
    assert_eq!(asked.len(), 1);
    assert!(matches!(asked[0], Asked::OpenStream { .. }));
}

#[test]
fn a_start_at_a_stale_filter_shows_its_printed_address() {
    let mut host = started(b"/?kind=reply&project=3");
    let duties = host.report(Report::Start);
    assert_eq!(duties.show, Some((Box::from(b"/?project=3".as_slice()), true)));
}

#[test]
fn a_drawn_page_carries_the_views_hrefs() {
    let mut host = started(b"/accounts/connect");
    host.report(Report::Start);
    let mut page = host::page(&host.bounds.tree, Primary::Control);
    host.draw(&mut page, &mut Vec::new());
    let html = page.html();
    assert!(
        html.split("</a>").any(|anchor| anchor.contains("href=\"/chats\"") && anchor.contains(">temper<")),
        "{html}"
    );
}
