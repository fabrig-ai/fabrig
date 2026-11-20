//! A separate tab follows redirects without carrying the page's ambient authority.
use skein_http::Header;
use skein_lib::{Duration, Time};
use skein_playground::tab::Shown;
use skein_playground_world::site::{Body, Route, tab::{self, Point, Script}};
const A: &[u8] = b"http://127.0.0.1:9/a";
fn fixed(path: &[u8], status: u16, body: &[u8]) -> Route {
    Route::new(path, status, Body::Fixed(body.into()))
}
fn redirect(path: &[u8], status: u16, to: &[u8]) -> Route {
    let mut route = fixed(path, status, b"redirect");
    route.headers.push(Header { name: b"Location".as_slice().into(), value: to.into() });
    // A tab must not save this cookie for the next request.
    route.headers.push(Header { name: b"Set-Cookie".as_slice().into(), value: b"session=issuer; Path=/".as_slice().into() });
    route
}
fn shown(address: &[u8], status: u16, body: &[u8], failed: bool) -> Shown {
    Shown { address: address.into(), status, body: body.into(), failed }
}
fn script(routes: Vec<Route>, expected: Shown) -> Script {
    Script {
        hosts: vec![("127.0.0.1:9".parse().expect("socket"), routes)],
        points: vec![Point { requests: 0, to: Some(A.into()) }],
        bound: 1024, expected: Some(expected), requests: 1, until: Time::ZERO,
    }
}
#[test]
fn a_get_keeps_its_answer_and_sends_only_its_own_authority() {
    let mut script = script(vec![
        redirect(b"/a", 302, b"/headers"), Route::new(b"/headers", 201, Body::Headers),
    ], shown(b"http://127.0.0.1:9/headers", 201, b"Host:127.0.0.1:9\nContent-Length:0\nConnection:close\n", false));
    script.requests = 2;
    tab::run(&script);
}
#[test]
fn every_redirect_status_follows_location_to_another_host() {
    for status in [301, 302, 303, 307, 308] {
        let mut script = script(vec![redirect(b"/a", status, b"http://127.0.0.1:10/b")],
            shown(b"http://127.0.0.1:10/b", 200, b"done", false));
        script.hosts.push(("127.0.0.1:10".parse().expect("socket"), vec![fixed(b"/b", 200, b"done")]));
        script.requests = 2;
        tab::run(&script);
    }
}
#[test]
fn relative_locations_resolve_against_the_answering_address() {
    let mut script = script(vec![
        redirect(b"/a", 302, b"//127.0.0.1:10/dir/page?old"),
    ], shown(b"http://127.0.0.1:10/done?new", 200, b"done", false));
    script.hosts.push(("127.0.0.1:10".parse().expect("socket"), vec![
        redirect(b"/dir/page?old", 303, b"?changed"),
        redirect(b"/dir/page?changed", 307, b".././done?new#fragment"),
        fixed(b"/done?new", 200, b"done"),
    ]));
    script.requests = 4;
    tab::run(&script);
}
#[test]
fn the_ninth_redirect_is_kept_failed_and_not_followed() {
    let routes = (0..9).map(|at| redirect(format!("/{at}").as_bytes(), 302, format!("/{}", at + 1).as_bytes())).collect();
    let mut script = script(routes, shown(b"http://127.0.0.1:9/8", 302, b"redirect", true));
    script.points[0].to = Some(b"http://127.0.0.1:9/0".as_slice().into());
    script.requests = 9;
    tab::run(&script);
}
#[test]
fn a_body_at_the_bound_is_kept_and_one_past_it_is_kept_failed() {
    for (body, failed) in [(b"abc".as_slice(), false), (b"abcd".as_slice(), true)] {
        let mut script = script(vec![fixed(b"/a", 200, body)], shown(A, 200, b"abc", failed));
        script.bound = 3;
        tab::run(&script);
    }
}
#[test]
fn a_refused_connection_is_kept_failed() {
    let mut script = script(vec![], shown(A, 0, b"", true));
    script.hosts.clear();
    script.requests = 0;
    tab::run(&script);
}
fn replacing(to: Option<Box<[u8]>>, expected: Option<Shown>) -> Script {
    let mut script = script(vec![
        Route::new(b"/a", 200, Body::Pieces { pieces: vec![(Duration::from_millis(20), b"old".as_slice().into())], finish: true }),
        fixed(b"/next", 200, b"next"),
    ], shown(A, 200, b"old", false));
    script.expected = expected;
    script.points.push(Point { requests: 1, to });
    // Even after the first answer was due, it cannot replace the newer visit.
    script.until = Time::ZERO.saturating_add(Duration::from_millis(50));
    script
}
#[test]
fn pointing_mid_visit_abandons_the_first_visit() {
    let mut script = replacing(Some(b"http://127.0.0.1:9/next".as_slice().into()), Some(shown(b"http://127.0.0.1:9/next", 200, b"next", false)));
    script.requests = 2;
    tab::run(&script);
}
#[test]
fn closing_mid_visit_abandons_it_without_a_shown_answer() {
    tab::run(&replacing(None, None));
}
