use super::*;

const WITHIN: Duration = Duration::from_secs(60);

fn press(page: &mut Page, role: Role, name: &str) {
    let node = page.wait(&Find::exactly_named(role, name.as_bytes()), Some(&Act::Press), WITHIN).expect(name);
    page.act(node, Act::Press).expect("press");
}

#[test]
fn an_owned_forges_repository_opens_through_tempers_sign_in() {
    let mut page = Page::start(facts::start(None).expect("walk facts"), b"forge sign-in").expect("page");
    press(&mut page, Role::Link, "Set up temper");
    press(&mut page, Role::Link, "Run on this machine");
    press(&mut page, Role::Button, "Run on this machine");
    press(&mut page, Role::Button, "Continue with ChatGPT");
    press(&mut page, Role::Button, "Create your project");
    page.wait(&Find::text(b"work/hello is in Hello"), None, WITHIN).expect("project created");
    page.reload(b"/projects/1", WITHIN).expect("reload project");
    let served = String::from_utf8_lossy(&page.served);
    assert!(served.contains("<meta name=\"temper-forge-sign-in\" content=\"1 forgejo http://127.0.0.1:10002 owned\">"), "{served}");
    page.wait(&Find::text("work/hello ↗".as_bytes()), None, WITHIN)
        .unwrap_or_else(|why| panic!("repository link: {why:?}\n{}", page.html()));
    let html = page.html();
    let href = "href=\"http://127.0.0.1:10002/user/oauth2/temper?redirect_to=%2Fwork%2Fhello\"";
    assert!(html.split("<a ").skip(1).any(|anchor| anchor.split_once('>').is_some_and(|(head, _)| head.contains(href))), "{html}");
}
