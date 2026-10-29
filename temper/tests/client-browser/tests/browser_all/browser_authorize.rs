//! Strict cookies survive a navigation from another site because the Authorize
//! page sends a same-origin exchange (client/browser.md, 4.5).
use skein_fake_person::{Find, Next, Step as PersonStep};
use skein_fake_person_browser::harness::{Origin, Page, ServerAction};
use skein_fake_person_browser::{BrowserFace, Harness, Outcome, Play, Script, Step};
use skein_lib::Time;
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN};

struct Authorization<'a> {
    tier: &'a Tier,
    before: Script,
    after: Script,
    reading: bool,
    configured: bool,
}
impl Play for Authorization<'_> {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        if self.configured {
            return self.after.next(face, now);
        }
        if !self.reading {
            let next = self.before.next(face, now);
            if next != Next::Done {
                return next;
            }
            face.address();
            self.reading = true;
            return Next::Wait { until: None };
        }
        if face.pending() {
            return Next::Wait { until: None };
        }
        let url = face.take_address().expect("foreign page address read");
        let url = std::str::from_utf8(&url).expect("test URL is UTF-8");
        let origin = url.strip_suffix('/').expect("foreign root page");
        assert!(origin.starts_with("http://localhost:"), "the forge is on the other loopback site");
        self.tier.authorize_callback(&format!("{origin}/callback"));
        self.configured = true;
        self.after.next(face, now)
    }
    fn server_action(&mut self) -> Option<ServerAction> {
        if self.configured { self.after.server_action() } else { self.before.server_action() }
    }
    fn expected(&self) -> Vec<Box<[u8]>> {
        if self.configured { self.after.expected() } else { self.before.expected() }
    }
    fn at(&self) -> u32 {
        if self.configured { self.after.at() } else { self.before.at() }
    }
}

#[test]
fn a_cross_site_authorization_returns_a_code_and_the_links_state() {
    let (tier, temper) = Tier::authorize("browser-authorize");
    let mut setup = tier.setup();
    // The live page can reconnect while its scratch server restarts.
    setup.expected = vec![
        Box::from(b"net::ERR_CONNECTION_REFUSED".as_slice()),
        Box::from(b"net::ERR_INCOMPLETE_CHUNKED_ENCODING".as_slice()),
    ];
    let start = format!(
        r##"<!doctype html><title>Test forge</title><h1>Test forge</h1><a id="authorize" href="#">Sign in with temper</a><script>
const url = new URL('/oauth/authorize', '{temper}');
url.search = new URLSearchParams({{client_id:'forgejo',redirect_uri:location.origin+'/callback',response_type:'code',scope:'openid profile',state:'browser-state',code_challenge:'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',code_challenge_method:'S256'}});
document.querySelector('#authorize').href = url;
</script>"##
    );
    setup.origins.push(Origin { host: b"localhost".as_slice().into(), pages: vec![
        Page { path: b"/".as_slice().into(), kind: b"text/html".as_slice().into(), body: start.into_bytes().into(), headers: vec![] },
        Page { path: b"/callback".as_slice().into(), kind: b"text/html".as_slice().into(), body: br"<!doctype html><title>Callback</title><h1>Checking authorization</h1><script>const q=new URLSearchParams(location.search);if(q.get('code') && q.get('state')==='browser-state' && !q.has('error'))document.querySelector('h1').textContent='Authorization received';</script>".as_slice().into(), headers: vec![] },
    ] });
    let see = |name: &[u8]| {
        Step::Person(PersonStep::See { find: Find::exactly_named(Role::Heading { level: 1 }, name), within: WITHIN })
    };
    // The heading precedes the startup facts. The welcome decision's link
    // appears only after those facts arrive, including after Back reloads it.
    let welcome =
        || Step::Person(PersonStep::See { find: Find::exactly_named(Role::Link, b"Set up temper"), within: WITHIN });
    let mut person = Authorization {
        tier: &tier,
        before: Script::new(Box::new([
            welcome(),
            Step::Visit { origin: 0, path: b"/".as_slice().into() },
            see(b"Test forge"),
        ])),
        after: Script::new(Box::new([
            // The callback's exact dynamically allocated port is now registered.
            // Show temper's page again before stopping it. A temper page left in
            // the tab's history keeps its connection open, and the stop then waits
            // out jig's 30 s close deadline (jig's profile/small.rs): without this
            // step the story takes 31 s rather than 1.5 s.
            Step::Back,
            welcome(),
            Step::StopServer,
            Step::StartServer,
            // Exercise the strict cookie from the other site after the restart.
            Step::Visit { origin: 0, path: b"/".as_slice().into() },
            see(b"Test forge"),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Sign in with temper") }),
            see(b"Authorization received"),
        ])),
        reading: false,
        configured: false,
    };
    assert_eq!(Harness::new(setup).run(&mut [&mut person]), Outcome::Done);
}
