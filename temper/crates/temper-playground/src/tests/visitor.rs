//! The visitor and printed link through skein's slices (temper/docs/design/playground.md, 3–4).
use crate::{
    machine::Machine,
    programs::{self, Process},
    stories, story,
};
use skein_lib::{Time, Wall};
use skein_playground::slices::{Slices, Wake};
use skein_ui_web::{bridge, fetch};

const SEED: u64 = 7;
// Startup and one exchange must finish within 128 slices of SLICE_STEPS each.
const MAX_SLICES: u32 = 128;
// temper-client-shell-web's limits::BODY_BYTES; this composition does not link the shell.
const BODY_BYTES: u32 = 33_554_432;
const EXCHANGE: u32 = 1;
const PREFIX: &[u8] = b"http://127.0.0.1:7102/jig/v1/sign-in/link?secret=";

fn start() -> Slices<Process<()>, Machine> {
    story::slices(stories::A.start(SEED, Wall::EPOCH), story::SLICE_STEPS, Time::ZERO)
}

fn until(
    slices: &mut Slices<Process<()>, Machine>,
    browser: &mut Time,
    ready: impl Fn(&[bridge::ToPage]) -> bool,
) -> (Vec<bridge::ToPage>, u32, Wake) {
    let mut posts = Vec::new();
    for count in 1..=MAX_SLICES {
        let wake = slices.slice(*browser, &mut posts);
        if let Wake::At(at) = wake {
            *browser = at;
        }
        if ready(&posts) {
            return (posts, count, wake);
        }
    }
    panic!(
        "seed {SEED}: expected posts not reached within {MAX_SLICES} slices; step {}, now {:?}, due {:?}",
        slices.world().steps(),
        slices.world().now(),
        slices.world().next_due(),
    );
}

fn idle(slices: &mut Slices<Process<()>, Machine>, browser: &mut Time) -> (Vec<bridge::ToPage>, u32, Wake) {
    until(slices, browser, |posts| {
        posts.iter().any(|post| matches!(post, bridge::ToPage::Printed(_)))
            && matches!(posts.last(), Some(bridge::ToPage::Clock(clock)) if clock.idle)
    })
}

fn printed(posts: &[bridge::ToPage]) -> &bridge::Printed {
    let lines: Vec<_> = posts
        .iter()
        .filter_map(|post| if let bridge::ToPage::Printed(line) = post { Some(line) } else { None })
        .collect();
    let [line] = lines.as_slice() else { panic!("one printed link, got {} lines", lines.len()) };
    line
}

#[test]
fn story_a_posts_its_printed_link() {
    let mut slices = start();
    programs::with(story::parameters(slices.world()), || {
        let mut browser = Time::ZERO;
        let (posts, count, wake) = idle(&mut slices, &mut browser);
        let line = printed(&posts);
        assert_eq!(line.process.as_ref(), b"temper serve");
        assert_eq!(line.stream, bridge::Stream::Output);
        let secret = line.line.strip_prefix(PREFIX).expect("the clients listener's sign-in link");
        assert!(!secret.is_empty() && !secret.contains(&b'\r') && !secret.contains(&b'\n'));
        assert!(matches!(posts.last(), Some(bridge::ToPage::Clock(clock)) if clock.idle));
        assert!(matches!(wake, Wake::At(_) | Wake::Message), "idle wake: {wake:?}");
        eprintln!("seed {SEED}: printed link and idle in {count} slices, {} steps", slices.world().steps());
    });
}

#[test]
fn the_visitor_spends_the_printed_link() {
    let mut slices = start();
    programs::with(story::parameters(slices.world()), || {
        let mut browser = Time::ZERO;
        let (posts, _, _) = idle(&mut slices, &mut browser);
        let path = printed(&posts).line.strip_prefix(b"http://127.0.0.1:7102").expect("link authority");
        slices.input(bridge::Input::Exchange(fetch::Exchange {
            id: EXCHANGE,
            method: "GET".into(),
            path: path.into(),
            headers: Vec::new(),
            body: Box::new([]),
            bound: BODY_BYTES,
        }));
        let (posts, _, _) = until(&mut slices, &mut browser, |posts| {
            posts.iter().any(|post| matches!(post, bridge::ToPage::Report(_)))
        });
        let reports: Vec<_> = posts
            .iter()
            .filter_map(|post| if let bridge::ToPage::Report(report) = post { Some(report) } else { None })
            .collect();
        assert!(
            matches!(reports.as_slice(), [fetch::Report::Answered { exchange: EXCHANGE, status, .. }] if *status < 400),
            "one successful exchange: {reports:?}"
        );
    });
}

#[test]
fn named_finds_each_story() {
    let slices = start();
    programs::with(story::parameters(slices.world()), || {
        let story = stories::named(b"a").expect("story a");
        assert_eq!(story.name, stories::A.name);
        assert_eq!(story.description, stories::A.description);
        assert_eq!(story.deployment, stories::A.deployment);
        assert_eq!(story.llm, stories::A.llm);
        assert_eq!(story.forge, stories::A.forge);
        assert!(stories::named(b"unknown").is_none());
    });
}
