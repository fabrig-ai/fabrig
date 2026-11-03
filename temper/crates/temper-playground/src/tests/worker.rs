//! The worker's portable world construction (temper/docs/design/playground.md, 3–4).
use crate::{documents, programs, story};
use skein_lib::{Time, Wall};
use skein_ui_web::bridge::{self, ToPage};

fn start(name: &[u8]) -> bridge::Start {
    bridge::Start { story: name.into(), seed: 23, start: Wall::from_nanos(1_700_000_000_000_000_000) }
}

#[test]
fn start_message_makes_slices_that_post_the_printed_link() {
    let start = start(b"a");
    let browser = Time::from_nanos(3_000_000_000);
    let mut slices = story::start_slices(&start, browser).expect("known story");
    assert_eq!(slices.world().seed(), start.seed);
    assert_eq!(slices.world().wall(), start.start);
    let mut posts = Vec::new();
    // Startup has at most 32 slices of the composition's step budget.
    const STARTUP_SLICES: usize = 32;
    programs::with(story::parameters(slices.world()), || {
        for _ in 0..STARTUP_SLICES {
            slices.slice(browser, &mut posts);
            if posts.iter().any(|post| matches!(post, ToPage::Printed(_))) {
                break;
            }
        }
    });
    let lines: Vec<_> = posts.iter().filter_map(|post| {
        if let ToPage::Printed(line) = post { Some(line) } else { None }
    }).collect();
    let [line] = lines.as_slice() else { panic!("one printed sign-in link: {posts:?}") };
    assert_eq!(line.process.as_ref(), b"temper serve");
    assert_eq!(line.stream, bridge::Stream::Output);
    let url = std::str::from_utf8(&line.line).expect("sign-in URL");
    let prefix = format!("http://{}/jig/v1/sign-in/link?secret=", documents::CLIENTS);
    let secret = url.strip_prefix(&prefix).expect("service's sign-in URL");
    assert!(!secret.is_empty() && !secret.contains(['\r', '\n']));
    assert!(matches!(posts.last(), Some(ToPage::Clock(_))), "the clock closes the slice");
}

#[test]
fn unknown_story_has_no_slices() {
    assert!(story::start_slices(&start(b"unknown-story"), Time::ZERO).is_none());
}
