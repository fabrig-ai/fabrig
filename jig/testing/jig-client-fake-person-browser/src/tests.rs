use crate::{States, stories};
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::Step;
use skein_lib::Duration;

const PAGE_WAIT: Duration = Duration::from_secs(5);

#[test]
fn the_framing_page_frames_the_application_s_page() {
    let target =
        crate::Target { prefix: b"/mapping", page: b"/inbox", request: b"forged", done: Find::text(b"Forged") };
    let page = crate::pages::framing(&target);
    assert_eq!(page.path.as_ref(), b"/framing");
    assert_eq!(page.kind.as_ref(), b"text/html");
    assert_eq!(page.headers.len(), 0);
    assert_eq!(
        page.body.as_ref(),
        br#"<!doctype html><title>Elsewhere</title><iframe src="http://{address}/inbox"></iframe>"#
    );
}

#[test]
fn the_forging_page_posts_to_the_mapping_s_request_route() {
    let target = crate::Target { prefix: b"/mapping", page: b"/", request: b"forged", done: Find::text(b"Forged") };
    let page = crate::pages::forging(&target);
    assert_eq!(page.path.as_ref(), b"/forging");
    assert_eq!(page.kind.as_ref(), b"text/html");
    assert_eq!(page.headers.len(), 0);
    assert_eq!(page.body.as_ref(), br#"<!doctype html><title>Elsewhere</title><h1>Forging requests</h1><script>
const body = Uint8Array.from(atob("Zm9yZ2Vk"), c => c.charCodeAt(0));
Promise.allSettled([
fetch("http://{address}/mapping/v1/request", {method: "POST", mode: "no-cors", credentials: "include", headers: {"Content-Type": "text/plain"}, body}),
fetch("http://{address}/mapping/v1/request", {method: "POST", credentials: "include", headers: {"Content-Type": "application/octet-stream", "Jig-Client": "browser"}, body})
]).then(() => { document.querySelector("h1").textContent = "Forged requests completed"; });
</script>
"#);
}

#[test]
fn the_spent_link_story_reopens_after_seeing_live() {
    let states =
        States { live: Find::text(b"Ready"), offline: Find::text(b"Unreachable"), signed_out: Find::text(b"Enter") };
    let steps = stories::spent_link_steps(&states);
    match steps.as_ref() {
        [
            Step::Person(PersonStep::See { find: live, within: first }),
            Step::Reopen,
            Step::Person(PersonStep::See { find: signed_out, within: last }),
        ] => {
            assert_eq!(live, &Find::text(b"Ready"));
            assert_eq!(signed_out, &Find::text(b"Enter"));
            assert_eq!((*first, *last), (PAGE_WAIT, PAGE_WAIT));
        }
        _ => panic!("see live, reopen the printed URL, then see signed out"),
    }
}
