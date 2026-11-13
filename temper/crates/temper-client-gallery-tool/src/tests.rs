use super::*;
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_scratch::Scratch;

#[test]
fn names_are_the_promised_ones() {
    let samples = samples::samples();
    let mut names = Vec::new();
    for sample in &samples {
        assert!(!names.contains(&sample.name), "duplicate {}", sample.name);
        names.push(sample.name);
    }
    if temper_test_env::rewrite() {
        fs::write(skein_scratch::paths::package().join("names.txt"), format!("{}\n", names.join("\n"))).unwrap();
    } else {
        let promised: Vec<&str> = include_str!("../names.txt").lines().collect();
        assert_eq!(names, promised);
    }
}

#[test]
fn sheet_entry_links_reach_the_adoption_address() {
    for (name, link) in [("home-add-repository", "Adopt another…"), ("chats-no-repository", "Adopt a repository")] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sheet entry sample");
        let mut view = view(&sample);
        let node = find(view.tree(), Role::Link, link, name);
        assert_eq!(
            temper_client_view::href(&mut view, node).as_deref(),
            Some(temper_client_domain::address::print(&temper_client_domain::Address::Adopt { project: 1 }).as_ref()),
            "{name}: sheet destination"
        );
    }
}

#[test]
fn a_later_settings_card_replaces_a_withdrawn_one_without_call_objects() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "ask-chat-replaced").unwrap();
    let view = view(&sample);
    assert!(view.tree().nodes().iter().any(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(b"Replaced by the proposal below")));
}

#[test]
fn retired_chat_style_slots_stay_reserved() {
    let names = &temper_client_view::look::NAMES;
    for (token, expected) in [
        (62, b"temper-reserved-62".as_slice()),
        (295, b"temper-reserved-295"),
        (296, b"temper-reserved-296"),
        (297, b"temper-reserved-297"),
        (298, b"temper-reserved-298"),
        (299, b"temper-reserved-299"),
        (597, b"temper-reserved-597"),
    ] {
        assert_eq!(names.style(skein_ui::Style { owner: names.owner, token }), Some(expected));
    }
}

fn navigation_links_have_addresses_and_install_action_links_keep_their_binding(
    sample: &Sample,
    view: &mut temper_client_view::View,
) {
    let mut links = Vec::new();
    for node in view.tree().nodes() {
        if node.role == Role::Link {
            assert!(node.link.is_some() || node.external.is_some(), "{}: link {:?}", sample.name, node.name);
            links.push(node.id);
        }
    }
    for node in links {
        if let Some(address) = temper_client_view::href(view, node) {
            assert!(!address.is_empty(), "{}: empty href for {node:?}", sample.name);
        } else if let Some(temper_client_domain::Action::Forge(temper_client_domain::forge::Act::Install {
            forge,
            ..
        })) = temper_client_view::act(view, node, skein_ui::Act::Press)
        {
            // The existing picker link starts an asynchronous install request;
            // it has no static destination until the forge answers.
            assert_eq!(sample.name, "github-picker", "{}: install link state", sample.name);
            assert_eq!(forge, 2, "{}: install link forge", sample.name);
        } else if let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::ConnectModel(
            temper_client_domain::pages::connect_model::Act::Holder { .. },
        ))) = temper_client_view::act(view, node, skein_ui::Act::Press)
        {
            assert!(
                sample.name.starts_with("connect-model-")
                    || matches!(sample.name, "accounts-added" | "accounts-renewed"),
                "{}: model connection link state",
                sample.name
            );
        } else if let Some(temper_client_domain::Action::Forge(temper_client_domain::forge::Act::EndPlayground)) =
            temper_client_view::act(view, node, skein_ui::Act::Press)
        {
            let reset = view.tree().nodes().iter().find(|shown| shown.id == node).expect("bound reset link");
            assert!(
                matches!(reset.name.as_deref(), Some(b"set up temper" | b"Set up temper")),
                "{}: reset link label",
                sample.name
            );
        } else if let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Needs(
            temper_client_domain::pages::needs::Act::Open { object },
        ))) = temper_client_view::act(view, node, skein_ui::Act::Press)
        {
            assert!(
                matches!(sample.address, temper_client_domain::Address::Home { .. })
                    || matches!(sample.name, "rail-away-on" | "rail-away-off"),
                "{}: needs link address",
                sample.name
            );
            assert!(
                samples::domain(sample)
                    .pages()
                    .inbox()
                    .unwrap_or_else(|| panic!("{}: needs link inbox", sample.name))
                    .cards
                    .iter()
                    .any(|card| card.object == object),
                "{}: needs link object {object:?}",
                sample.name
            );
        } else {
            panic!("{}: link {node:?} has neither a destination nor its action", sample.name);
        }
    }
}

// Expected words are the shipped view's; available counts are the worlds'
// live/closed windows, combined in the named Your chats list.
const SOLO_LIVE: u32 = 2;
const SOLO_CLOSED: u32 = 1;
const REFERENCE_LIVE: u32 = 1;
const REFERENCE_CLOSED: u32 = 2;
const SOLO_PLAN_LIVE: u32 = 3;
struct Holds {
    name: &'static str,
    heading: &'static str,
    parts: &'static [Part],
}
enum Part {
    WelcomeFrame { box_note: bool },
    Projects { rows: &'static [(&'static str, &'static str)] },
    StyleCount { style: skein_ui::Style, count: usize },
    HomeCards { primary: &'static [&'static str], projects: &'static [&'static str] },
    Destination { name: &'static str, href: &'static str },
    Current { name: &'static str },
    FirstButton { disclosure: &'static str, name: &'static str },
    CardChanges { title: &'static str, changes: &'static [&'static str] },
    Cards { section: &'static str, from: &'static [&'static str] },
    StyledText { style: skein_ui::Style, text: &'static [&'static str] },
    GroupLines { name: &'static str, lines: &'static [&'static str] },
    Tabs { names: &'static [&'static str], selected: usize },
    Path { stops: &'static [&'static str] },
    Drawing { style: skein_ui::Style, count: usize },
    Expanded { name: &'static str },
    Folded { name: &'static str },
    Configured { value: bool },
    Reader { role: jig_client::contract::HeldRole },
    After { first: skein_ui::Style, then: skein_ui::Style },
    Buttons { style: skein_ui::Style, names: &'static [&'static str] },
    Links { style: skein_ui::Style, names: &'static [&'static str] },
    NoStyle { style: skein_ui::Style },
    Selected { name: &'static str },
    NotPressed { name: &'static str },
    Pressed { name: &'static str },
    Text { text: &'static str },
    Named { role: Role, name: &'static str },
    Ordered { role: Role, name: &'static str },
    Described { role: Role, name: &'static str, description: &'static str },
    Absent { role: Role, name: &'static str },
    NoRole { role: Role },
    Value { role: Role, name: &'static str, value: &'static str },
    Chats { live: u32, closed: u32 },
    Goals { rows: &'static [&'static str] },
    Plan { live: u32, title: &'static str },
}
const ASK_KEEP: Part =
    Part::Buttons { style: temper_client_view::look::WAYS_ACTS, names: &["How does temper work?", "Keep it"] };
const ASK_USE: Part =
    Part::Buttons { style: temper_client_view::look::WAYS_ACTS, names: &["How does temper work?", "Use this"] };
const ASK_ALONE: Part = Part::Buttons { style: temper_client_view::look::WAYS_ACTS, names: &["How does temper work?"] };
const REMEMBERED_THREE: &[&str] = &[
    "Keep answers concise and start with the result.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Use Rust examples when explaining code.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Ask before changing a public interface.",
    "Changed by you · Today · Thursday 22 October",
];
const REMEMBERED_FULL: &[&str] = &[
    "Keep answers concise and start with the result.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Use Rust examples when explaining code.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Ask before changing a public interface.",
    "Changed by you · Today · Thursday 22 October",
    "Show the command that checks a change.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Explain unfamiliar abbreviations the first time.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Use metric units in examples.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Keep a summary of decisions at the end of a long chat.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Prefer plain text examples I can copy into a terminal.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Say which assumptions a calculation depends on.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Use UTC when comparing times from different places.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Give one next step when a check fails.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "When comparing approaches, explain the cost of each and which evidence would make you choose a different one next time.",
    "Ask temper chat 14 · Today · Thursday 22 October",
];
const REMEMBERED_REFUSED: &[&str] = &[
    "Keep answers concise and start with the result.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "Use Rust examples when explaining code.",
    "Ask temper chat 14 · Today · Thursday 22 October",
    "This line changed meanwhile; it shows as it is now.",
    "Ask before changing a public interface.",
    "Changed by you · Today · Thursday 22 October",
];
const HOW_WAIT: Part = Part::GroupLines {
    name: "Always waits for you",
    lines: &[
        "money past a budget",
        "what can’t be undone: rewriting history, a force-push, deleting unlanded work",
        "changes to this page",
        "anything an agent can’t place",
    ],
};
const HOW_REACHES_DECIDE: Part = Part::GroupLines {
    name: "Agents decide, then tell you",
    lines: &[
        "wording and names inside a spec",
        "retrying or splitting a task",
        "implementation choices a spec leaves open",
        "test-only changes",
    ],
};
const HOW_REACHES_ASK: Part = Part::GroupLines {
    name: "Asks you",
    lines: &["a change to vsr’s protocol core", "a new dependency", "a departure from a landed design"],
};
const HOW_REACHES_NOTE: Part = Part::Named {
    role: Role::Paragraph,
    name: "While you’re away, all but changes to vsr’s protocol core are decided with the recommended choice and listed.",
};
const HOW_DECIDE_NOTE: Part =
    Part::Named { role: Role::Paragraph, name: "Listed in Decided for you for 24 hours, to overturn." };
const HOW_WAIT_NOTE: Part = Part::Named { role: Role::Paragraph, name: "Away or not." };
const HOW_RULES_TABLE: Part = Part::StyledText {
    style: temper_client_view::look::HOW_TABLE,
    text: &[
        "Paths",
        "Sign-off",
        "Lands",
        "include/vsr.h · src/protocol.* · src/transition.* · docs/protocol.md",
        "you",
        "once you sign off",
    ],
};
const HOW_TEAM_TABLE: Part = Part::StyledText {
    style: temper_client_view::look::HOW_TABLE,
    text: &[
        "Paths",
        "Review",
        "Sign-off",
        "Lands",
        "payments/",
        "an agent",
        "two of you",
        "once two have signed this head",
        "everything else",
        "an agent",
        "one of you",
        "once signed",
    ],
};
const HOW_RULES_NOTE: Part = Part::Named {
    role: Role::Paragraph,
    name: "A sign-off is for one head. A change pushed after it needs signing again; taking main cleanly keeps it.",
};
static HOLDS: [Holds; samples::N] = [
    Holds {
        name: "tell-area",
        heading: "Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Tell temper" },
            Part::Named { role: Role::TextArea, name: "Something off?" },
            Part::Named { role: Role::Paragraph, name: "About shop · the page you were on, Prover" },
            Part::Named { role: Role::Paragraph, name: "Goes to Prover, tests's owner" },
            Part::Named { role: Role::Button, name: "File it" },
            Part::Named { role: Role::Button, name: "Ask temper instead" },
        ],
    },
    Holds {
        name: "tell-page",
        heading: "Something off?",
        parts: &[
            Part::Destination { name: "‹ Back", href: "/projects/1/ask" },
            Part::Named { role: Role::TextArea, name: "What you saw" },
            Part::Named { role: Role::Paragraph, name: "About shop · the page you were on, Team" },
            Part::Named {
                role: Role::Paragraph,
                name: "Goes to Prover, shop's triager, who hands it to its area's owner",
            },
            Part::Named { role: Role::Button, name: "File it" },
            Part::Named { role: Role::Button, name: "It's a question: ask temper instead" },
        ],
    },
    Holds {
        name: "told-not-set-up",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "Not set up" },
            Part::Absent { role: Role::Button, name: "Turn on" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Your devices" },
        ],
    },
    Holds {
        name: "told-off",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "Off" },
            Part::Named { role: Role::Button, name: "Turn on" },
            Part::Text { text: "Firefox on Linux" },
        ],
    },
    Holds {
        name: "told-on",
        heading: "Telling you",
        parts: &[Part::Named { role: Role::Button, name: "Turn off" }, Part::Text { text: "Safari on iPhone" }],
    },
    Holds {
        name: "told-cannot",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "This browser can't be pushed to." },
            Part::Text { text: "Add temper to the Home Screen, then open it from there and turn this on." },
            Part::Absent { role: Role::Button, name: "Turn on" },
        ],
    },
    Holds {
        name: "told-refused",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "You have 8 devices: remove one below first." },
            Part::Named { role: Role::Button, name: "Turn on" },
        ],
    },
    Holds {
        name: "night-told-not-set-up",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "Not set up" },
            Part::Absent { role: Role::Button, name: "Turn on" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Your devices" },
        ],
    },
    Holds {
        name: "night-told-off",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "Off" },
            Part::Named { role: Role::Button, name: "Turn on" },
            Part::Text { text: "Firefox on Linux" },
        ],
    },
    Holds {
        name: "night-told-on",
        heading: "Telling you",
        parts: &[Part::Named { role: Role::Button, name: "Turn off" }, Part::Text { text: "Safari on iPhone" }],
    },
    Holds {
        name: "night-told-cannot",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "This browser can't be pushed to." },
            Part::Text { text: "Add temper to the Home Screen, then open it from there and turn this on." },
            Part::Absent { role: Role::Button, name: "Turn on" },
        ],
    },
    Holds {
        name: "night-told-refused",
        heading: "Telling you",
        parts: &[
            Part::Text { text: "You have 8 devices: remove one below first." },
            Part::Named { role: Role::Button, name: "Turn on" },
        ],
    },
    Holds {
        name: "remembered-none",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::SETTINGS_TAG, text: &["Nothing kept yet"] },
            Part::Named {
                role: Role::Paragraph,
                name: "Ask temper keeps a line here when you ask it to remember something, such as how you like its answers.",
            },
            Part::NoStyle { style: temper_client_view::look::REMEMBERED_ROWS },
            Part::Absent { role: Role::TextField, name: "Line" },
        ],
    },
    Holds {
        name: "remembered-three",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_THREE },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Absent { role: Role::TextField, name: "Line" },
        ],
    },
    Holds {
        name: "remembered-full",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_FULL },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::StyledText {
                style: temper_client_view::look::SETTINGS_FACTS,
                text: &["12 lines, the most Ask temper keeps: it asks you which to change before it keeps another."],
            },
            Part::Absent { role: Role::TextField, name: "Line" },
        ],
    },
    Holds {
        name: "remembered-changing",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_THREE },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Value { role: Role::TextField, name: "Line", value: "Use short Rust examples with their output." },
            Part::Named { role: Role::Button, name: "Save" },
            Part::Named { role: Role::Button, name: "Cancel" },
        ],
    },
    Holds {
        name: "remembered-refused",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_REFUSED },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Absent { role: Role::TextField, name: "Line" },
            Part::StyledText {
                style: temper_client_view::look::SETTINGS_REFUSAL,
                text: &["This line changed meanwhile; it shows as it is now."],
            },
        ],
    },
    Holds {
        name: "night-remembered-none",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::SETTINGS_TAG, text: &["Nothing kept yet"] },
            Part::Named {
                role: Role::Paragraph,
                name: "Ask temper keeps a line here when you ask it to remember something, such as how you like its answers.",
            },
            Part::NoStyle { style: temper_client_view::look::REMEMBERED_ROWS },
            Part::Absent { role: Role::TextField, name: "Line" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "night-remembered-three",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_THREE },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Absent { role: Role::TextField, name: "Line" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "night-remembered-full",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_FULL },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::StyledText {
                style: temper_client_view::look::SETTINGS_FACTS,
                text: &["12 lines, the most Ask temper keeps: it asks you which to change before it keeps another."],
            },
            Part::Absent { role: Role::TextField, name: "Line" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "night-remembered-changing",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_THREE },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Value { role: Role::TextField, name: "Line", value: "Use short Rust examples with their output." },
            Part::Named { role: Role::Button, name: "Save" },
            Part::Named { role: Role::Button, name: "Cancel" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "night-remembered-refused",
        heading: "What temper remembers about you",
        parts: &[
            Part::StyledText { style: temper_client_view::look::REMEMBERED_ROWS, text: REMEMBERED_REFUSED },
            Part::Absent { role: Role::Text, name: "Nothing kept yet" },
            Part::Absent { role: Role::TextField, name: "Line" },
            Part::StyledText {
                style: temper_client_view::look::SETTINGS_REFUSAL,
                text: &["This line changed meanwhile; it shows as it is now."],
            },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "team",
        heading: "Who owns what on shop",
        parts: &[
            Part::Links {
                style: temper_client_view::look::RAIL_NAV,
                names: &["Home", "Needs you 3", "Work", "Team", "How work runs"],
            },
            Part::Named { role: Role::Paragraph, name: "Two of you." },
            Part::Named { role: Role::Text, name: "You" },
            Part::Named { role: Role::Paragraph, name: "Owner" },
            Part::Named { role: Role::Text, name: "owners of shop" },
            Part::Named { role: Role::Paragraph, name: "Three things wait on you." },
            Part::Named { role: Role::Text, name: "Sam" },
            Part::Named { role: Role::Paragraph, name: "Owner" },
            Part::Named { role: Role::Paragraph, name: "One question waits on Sam." },
            Part::Absent { role: Role::Text, name: "Service" },
        ],
    },
    Holds {
        name: "team-alone",
        heading: "Who owns what on shop",
        parts: &[
            Part::Absent { role: Role::Link, name: "Team" },
            Part::Named { role: Role::Paragraph, name: "Just you." },
            Part::Named { role: Role::Paragraph, name: "Owner" },
            Part::Absent { role: Role::Text, name: "owners of shop" },
            Part::Named { role: Role::Text, name: "You" },
            Part::Named { role: Role::Paragraph, name: "Nothing waits on you." },
            Part::Absent { role: Role::Text, name: "Sam" },
        ],
    },
    Holds {
        name: "team-hand",
        heading: "Who owns what on shop",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Hand an area to a role" },
            Part::Named { role: Role::Choice, name: "core" },
            Part::Named { role: Role::Choice, name: "tests" },
            Part::Named { role: Role::Choice, name: "A new area" },
            Part::Named { role: Role::Choice, name: "Prover" },
            Part::Named { role: Role::Choice, name: "Prover, from the stock" },
            Part::Named {
                role: Role::Paragraph,
                name: "Owns its area's tests: triages their reds and flakes and makes the repairs.",
            },
            Part::Named { role: Role::Choice, name: "Porter, from the stock" },
            Part::Named { role: Role::Paragraph, name: "Keeps its area's port in step with its source." },
            Part::Named { role: Role::Choice, name: "Write one with Ask temper" },
            Part::Named { role: Role::Button, name: "Not now" },
            Part::Named { role: Role::Button, name: "Hand it over" },
        ],
    },
    Holds {
        name: "team-role",
        heading: "Who owns what on shop",
        parts: &[
            Part::Named { role: Role::Link, name: "Team" },
            Part::Named {
                role: Role::Paragraph,
                name: "Just you. Each area has one owner. Prover hands each finding and red to its area's owner. An owner that can't decide asks you, as How work runs says.",
            },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Standing roles" },
            Part::Named { role: Role::Link, name: "Prover" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Areas" },
            Part::Named { role: Role::Cell, name: "core" },
            Part::Named { role: Role::Cell, name: "tests" },
            Part::Named { role: Role::Cell, name: "Anything else" },
            Part::Named { role: Role::Paragraph, name: "in Prover's" },
            Part::Named { role: Role::Paragraph, name: "Owner · core" },
            Part::Named { role: Role::Link, name: "Change by proposal" },
            Part::Named { role: Role::Link, name: "Talk to it" },
            Part::Named { role: Role::Cell, name: "21 of 40" },
            Part::Named {
                role: Role::Paragraph,
                name: "A standing role · owns tests, and Anything else, since 2 November · started by you",
            },
            Part::Named { role: Role::Button, name: "+ Hand an area to a role" },
            Part::Named { role: Role::Button, name: "Pause" },
            Part::Named { role: Role::Button, name: "Give the area back" },
            Part::Named { role: Role::Paragraph, name: "Its Now · written at 02:41" },
            Part::Named { role: Role::Paragraph, name: "Follows" },
            Part::Named {
                role: Role::Paragraph,
                name: "Follows acme/upstream · 2 behind · check passes at 9c04d7e1a2b3",
            },
            Part::Named { role: Role::Paragraph, name: "9c04d7e1a2b3 · held by its owner: waits for the handoff fix" },
            Part::Named { role: Role::Paragraph, name: "abababababab · waits for change 812" },
            Part::Named { role: Role::Paragraph, name: "Follows acme/tools · caught up · no check yet" },
            Part::Named { role: Role::Paragraph, name: "Sessions this week" },
            Part::Named { role: Role::Paragraph, name: "Sun 01:52 · running" },
            Part::Named { role: Role::Paragraph, name: "Reproduced finding 91; asked you about its fix." },
            Part::Named { role: Role::Paragraph, name: "Its weekly report · written Mon 06:00" },
            Part::Named { role: Role::Paragraph, name: "Week of 26 October." },
            Part::Named {
                role: Role::Item,
                name: "Triaged three findings: two routed to you, one repaired and landed.",
            },
            Part::Named { role: Role::Item, name: "Waiting on you: finding 91's protocol fix." },
            Part::Named { role: Role::Paragraph, name: "Spent 21 of 40 this period." },
            Part::Named { role: Role::Paragraph, name: "Sat 16:20 · 22 min" },
            Part::Named { role: Role::Paragraph, name: "Triaged three findings; routed two to you." },
            Part::Named { role: Role::Paragraph, name: "Fri 23:05 · 31 min" },
            Part::Named { role: Role::Paragraph, name: "0" },
            Part::Named { role: Role::Paragraph, name: "17" },
            Part::Named { role: Role::Paragraph, name: "4" },
            Part::Named {
                role: Role::Paragraph,
                name: "Each session starts fresh from its rules, its Now and what woke it, and ends by writing its Now.",
            },
            Part::Named { role: Role::Item, name: "Next: the fuzz tier's next run, then my weekly report on Monday." },
            Part::Named { role: Role::Paragraph, name: "Its rules · change by proposal" },
            Part::Named { role: Role::Link, name: "change by proposal" },
            Part::Named {
                role: Role::Paragraph,
                name: "Reproduce before routing: a finding gets a seed or a test first.",
            },
            Part::Named { role: Role::Paragraph, name: "Wakes on" },
            Part::Named { role: Role::Text, name: "News from the forge, on 2 topics" },
            Part::Named { role: Role::Text, name: "every 30 minutes" },
            Part::Named { role: Role::Text, name: "A timer" },
            Part::Named { role: Role::Text, name: "Mondays 06:00" },
            Part::Named { role: Role::Text, name: "Task 61's result" },
            Part::Named { role: Role::Text, name: "at once" },
        ],
    },
    Holds {
        name: "night-team-role",
        heading: "Who owns what on shop",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Areas" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Standing roles" },
            Part::Named { role: Role::Link, name: "Talk to it" },
            Part::Named { role: Role::Paragraph, name: "Owner · core" },
            Part::Named { role: Role::Cell, name: "21 of 40" },
            Part::Named { role: Role::Button, name: "+ Hand an area to a role" },
            Part::Named { role: Role::Button, name: "Pause" },
            Part::Named { role: Role::Button, name: "Give the area back" },
            Part::Named { role: Role::Paragraph, name: "Its Now · written at 02:41" },
            Part::Named { role: Role::Paragraph, name: "Follows" },
            Part::Named {
                role: Role::Paragraph,
                name: "Follows acme/upstream · 2 behind · check passes at 9c04d7e1a2b3",
            },
            Part::Named { role: Role::Paragraph, name: "9c04d7e1a2b3 · held by its owner: waits for the handoff fix" },
            Part::Named { role: Role::Paragraph, name: "abababababab · waits for change 812" },
            Part::Named { role: Role::Paragraph, name: "Follows acme/tools · caught up · no check yet" },
            Part::Named { role: Role::Paragraph, name: "Sessions this week" },
            Part::Named { role: Role::Paragraph, name: "Sun 01:52 · running" },
            Part::Named { role: Role::Paragraph, name: "Reproduced finding 91; asked you about its fix." },
            Part::Named { role: Role::Paragraph, name: "Its weekly report · written Mon 06:00" },
            Part::Named { role: Role::Paragraph, name: "Week of 26 October." },
            Part::Named {
                role: Role::Item,
                name: "Triaged three findings: two routed to you, one repaired and landed.",
            },
            Part::Named { role: Role::Item, name: "Waiting on you: finding 91's protocol fix." },
            Part::Named { role: Role::Paragraph, name: "Spent 21 of 40 this period." },
            Part::Named { role: Role::Paragraph, name: "Sat 16:20 · 22 min" },
            Part::Named { role: Role::Paragraph, name: "Triaged three findings; routed two to you." },
            Part::Named { role: Role::Paragraph, name: "Fri 23:05 · 31 min" },
            Part::Named { role: Role::Paragraph, name: "0" },
            Part::Named { role: Role::Paragraph, name: "17" },
            Part::Named { role: Role::Paragraph, name: "4" },
            Part::Named {
                role: Role::Paragraph,
                name: "Each session starts fresh from its rules, its Now and what woke it, and ends by writing its Now.",
            },
            Part::Named { role: Role::Paragraph, name: "Its rules · change by proposal" },
            Part::Named {
                role: Role::Paragraph,
                name: "Reproduce before routing: a finding gets a seed or a test first.",
            },
            Part::Named { role: Role::Paragraph, name: "Wakes on" },
            Part::Named { role: Role::Text, name: "News from the forge, on 2 topics" },
            Part::Named { role: Role::Text, name: "every 30 minutes" },
            Part::Named { role: Role::Text, name: "A timer" },
            Part::Named { role: Role::Text, name: "Mondays 06:00" },
            Part::Named { role: Role::Text, name: "Task 61's result" },
            Part::Named { role: Role::Text, name: "at once" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds { name: "night-team", heading: "Who owns what on shop", parts: &[Part::Pressed { name: "Night mode" }] },
    Holds {
        name: "rail-new-change",
        heading: "Four in flight, two landed this week.",
        parts: &[
            Part::Named { role: Role::Link, name: "New change" },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "rail-all-projects",
        heading: "Nothing in flight yet.",
        parts: &[
            Part::FirstButton { disclosure: "vsr", name: "All projects" },
            Part::Named { role: Role::Text, name: "2 projects" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "rail-across",
        heading: "Two things need you.",
        parts: &[
            Part::Pressed { name: "All projects" },
            Part::Named { role: Role::Disclosure, name: "All projects" },
            Part::Destination { name: "temper home", href: "/home" },
            Part::Destination { name: "Home", href: "/home" },
            Part::Destination { name: "Needs you 2", href: "/" },
            Part::Destination { name: "Needs 2", href: "/" },
            Part::Current { name: "Needs you 2" },
            Part::Current { name: "Needs 2" },
            Part::Destination { name: "Work", href: "/projects/1/work" },
            Part::Destination { name: "How work runs", href: "/projects/1/settings" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs 2", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work",
        heading: "Four in flight, two drafts, three landed this week.",
        parts: &[
            Part::Cards {
                section: "In flight · 4",
                from: &[
                    "goalmembership-handoff · Draft first · 31 of 50 spent",
                    "goalfuzz-corpus · Draft first · 6 of 15 spent",
                    "changeWitness rejoin test · queued 3rd",
                    "chatwith you · “the comment on vsr_step_many is stale”",
                ],
            },
            Part::Cards {
                section: "Drafts · 2",
                from: &[
                    "draftread-barriers.md · a chat with you · 07:21",
                    "draft · witness-catch-up.md · landed in docs/design/draft/ · ready since Fri 14:10",
                ],
            },
            Part::Named { role: Role::Button, name: "Start a goal: Draft first" },
            Part::Named { role: Role::Button, name: "Another way" },
            Part::Text { text: "vsr’s way" },
            Part::Named { role: Role::Region, name: "Landed this week · 3" },
            Part::CardChanges {
                title: "membership-handoff",
                changes: &[
                    "Witness rejoin fix",
                    "Witness retention across a handoff",
                    "Handoff states and their tests",
                ],
            },
            Part::CardChanges {
                title: "with you · “the comment on vsr_step_many is stale”",
                changes: &["Comment rewritten from the code", "io_uring: retry a short write once"],
            },
            Part::Named { role: Role::Progress, name: "6 / 9" },
            Part::Named { role: Role::Paragraph, name: "Written by the goal at 07:31" },
            Part::Named { role: Role::Paragraph, name: "Written by the goal at 06:55" },
            Part::Links {
                style: temper_client_view::look::WORK_FILTERS,
                names: &["All", "Drafts · 2", "Goals · 2", "Changes · 7", "Chats · 1"],
            },
            Part::Named {
                role: Role::Paragraph,
                name: "Six of nine tasks landed on the goal's branch. The witness rejoin fix waits on your answer; two tasks wait on it.",
            },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work-pair",
        heading: "Four in flight, two landed this week.",
        parts: &[
            Part::Cards {
                section: "In flight · 4",
                from: &[
                    "chatwith you · “Monthly totals by category”",
                    "chatwith you · “Budget alerts”",
                    "chatwith you · “Dark mode for the dashboard”",
                    "chatwith you · “Export a year as CSV”",
                ],
            },
            Part::Named { role: Role::Link, name: "New change" },
            Part::Links {
                style: temper_client_view::look::WORK_FILTERS, names: &["All", "Changes · 6", "Chats · 6"]
            },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work-drafts",
        heading: "Four in flight, two drafts, three landed this week.",
        parts: &[
            Part::Cards {
                section: "Drafts · 2",
                from: &[
                    "draftread-barriers.md · a chat with you · 07:21",
                    "draft · witness-catch-up.md · landed in docs/design/draft/ · ready since Fri 14:10",
                ],
            },
            Part::Named { role: Role::Button, name: "Start a goal: Draft first" },
            Part::Named { role: Role::Button, name: "Another way" },
            Part::Text { text: "vsr’s way" },
            Part::Absent { role: Role::Region, name: "In flight · 4" },
            Part::Absent { role: Role::Region, name: "Landed this week · 3" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work-goals",
        heading: "2 goals, 2 under way.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Four in flight, two drafts, three landed this week.",
            },
            Part::Named { role: Role::Navigation, name: "Work's filters" },
            Part::Goals { rows: &["membership-handoff", "fuzz-corpus"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work-changes",
        heading: "Three changes are on the way to main.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Four in flight, two drafts, three landed this week.",
            },
            Part::Named { role: Role::Navigation, name: "Work's filters" },
            Part::Named { role: Role::Link, name: "Witness rejoin test" },
            Part::Named { role: Role::Text, name: "checking" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "work-empty",
        heading: "Nothing in flight yet.",
        parts: &[
            Part::Links { style: temper_client_view::look::WORK_FILTERS, names: &["All"] },
            Part::Absent { role: Role::Region, name: "In flight · 0" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "members",
        heading: "Who may do what.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "The deployment" },
            Part::Named { role: Role::Paragraph, name: "You are its owner." },
            Part::Named { role: Role::Heading { level: 2 }, name: "vsr · 2" },
            Part::Named { role: Role::Heading { level: 2 }, name: "cowsql · 2" },
            Part::Named { role: Role::Text, name: "Owner" },
            Part::Named { role: Role::Text, name: "Prover" },
            Part::Named { role: Role::Text, name: "owner" },
            Part::Named { role: Role::Text, name: "maintainer" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "members-refused",
        heading: "Who may do what.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "vsr · 2" },
            Part::Named { role: Role::Heading { level: 2 }, name: "cowsql" },
            Part::Named { role: Role::Paragraph, name: "Read by the project’s members." },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "limits",
        heading: "What this deployment holds.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Capacities" },
            Part::Named { role: Role::Heading { level: 2 }, name: "By project" },
            Part::Named { role: Role::Text, name: "12 of 100" },
            Part::Named { role: Role::Text, name: "3 of 8" },
            Part::Named { role: Role::Text, name: "4 of 16 · 2 waiting" },
            Part::Named { role: Role::Text, name: "1 of 32" },
            Part::Named { role: Role::Text, name: "4.1 MB of 64 MB" },
            Part::Named { role: Role::Text, name: "9 of 50 tasks" },
            Part::Named { role: Role::Text, name: "3 of 50 tasks" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "limits-not-owner",
        heading: "What this deployment holds.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Limits are seen by the deployment’s owners." },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Capacities" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "night-work",
        heading: "Four in flight, two drafts, three landed this week.",
        parts: &[
            Part::Cards {
                section: "In flight · 4",
                from: &[
                    "goalmembership-handoff · Draft first · 31 of 50 spent",
                    "goalfuzz-corpus · Draft first · 6 of 15 spent",
                    "changeWitness rejoin test · queued 3rd",
                    "chatwith you · “the comment on vsr_step_many is stale”",
                ],
            },
            Part::Pressed { name: "Night mode" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "night-limits",
        heading: "What this deployment holds.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Capacities" },
            Part::Named { role: Role::Heading { level: 2 }, name: "By project" },
            Part::Named { role: Role::Text, name: "12 of 100" },
            Part::Named { role: Role::Text, name: "3 of 8" },
            Part::Named { role: Role::Text, name: "4 of 16 · 2 waiting" },
            Part::Named { role: Role::Text, name: "1 of 32" },
            Part::Named { role: Role::Text, name: "4.1 MB of 64 MB" },
            Part::Named { role: Role::Text, name: "9 of 50 tasks" },
            Part::Named { role: Role::Text, name: "3 of 50 tasks" },
            Part::Pressed { name: "Night mode" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
        ],
    },
    Holds {
        name: "how",
        heading: "Agents review every change, and code lands it",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What reaches you" },
            Part::Named { role: Role::Text, name: "on Hello" },
            Part::GroupLines {
                name: "Agents decide, then tell you",
                lines: &["wording and names inside a spec", "retrying or splitting a task", "test-only changes"],
            },
            Part::GroupLines {
                name: "Asks you",
                lines: &[
                    "implementation choices a spec leaves open",
                    "a new dependency",
                    "a departure from a landed design",
                ],
            },
            HOW_WAIT,
            HOW_DECIDE_NOTE,
            Part::Named {
                role: Role::Paragraph,
                name: "While you’re away, they are decided with the recommended choice and listed.",
            },
            HOW_WAIT_NOTE,
            Part::Named { role: Role::Select, name: "Away until" },
            Part::Text { text: "the way: Agents review" },
            Part::Text { text: "An agent makes each change, another reviews it, and code lands it." },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 1 },
            Part::Named { role: Role::Item, name: "review, an agent" },
            Part::Named { role: Role::Item, name: "verify, code" },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 1 },
            Part::Named {
                role: Role::Paragraph,
                name: "An agent, an agent when needed, a person, code; no merge tool above the dotted line",
            },
            Part::NoRole { role: Role::Table },
            Part::Configured { value: false },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Absent { role: Role::Button, name: "Use this" },
            Part::Named { role: Role::Heading { level: 2 }, name: "What the project shows" },
            Part::Named { role: Role::Check, name: "Goals" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Budgets" },
        ],
    },
    Holds {
        name: "how-approve",
        heading: "You approve every change",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What reaches you" },
            Part::Named { role: Role::Text, name: "on Hello" },
            Part::Absent { role: Role::Group, name: "Agents decide, then tell you" },
            Part::GroupLines {
                name: "Asks you",
                lines: &[
                    "wording and names inside a spec",
                    "retrying or splitting a task",
                    "implementation choices a spec leaves open",
                    "test-only changes",
                    "a new dependency",
                    "a departure from a landed design",
                ],
            },
            HOW_WAIT,
            Part::Named { role: Role::Paragraph, name: "They ask even while you’re away." },
            HOW_WAIT_NOTE,
            Part::Absent { role: Role::Select, name: "Away until" },
            Part::Text { text: "the way: You approve" },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Tabs { names: &["You approvenow", "Agents review", "Draft first"], selected: 0 },
            Part::Named { role: Role::Item, name: "you approve, a person" },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 1 },
            Part::Named {
                role: Role::Paragraph,
                name: "An agent, an agent when needed, a person, code; no merge tool above the dotted line",
            },
            Part::NoRole { role: Role::Table },
            Part::Absent { role: Role::Item, name: "check, code" },
            Part::Absent { role: Role::Item, name: "verify, code" },
        ],
    },
    Holds {
        name: "how-draft",
        heading: "Draft first, then agents build it on its own branch",
        parts: &[
            Part::Text { text: "the way: Draft first" },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Tabs { names: &["You approve", "Agents review", "Draft firstnow"], selected: 2 },
            Part::Named { role: Role::Item, name: "draft, a person" },
            Part::Named { role: Role::Item, name: "goal branch, a branch" },
            Part::Named { role: Role::Item, name: "verify, code" },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 2 },
            Part::Named {
                role: Role::Paragraph,
                name: "An agent, an agent when needed, a person, code; no merge tool above the dotted line",
            },
            Part::NoRole { role: Role::Table },
            Part::Named { role: Role::Heading { level: 2 }, name: "Roles" },
            Part::Described {
                role: Role::Disclosure,
                name: "Chat",
                description: "For each chat: talks with you and asks for the changes",
            },
            Part::Described {
                role: Role::Disclosure,
                name: "Ask temper",
                description: "When you ask: explains how work runs here, and proposes changes to it",
            },
            Part::Described {
                role: Role::Disclosure,
                name: "Drafter",
                description: "When you start a draft: writes it with you, in the terminal",
            },
            Part::Described {
                role: Role::Disclosure,
                name: "Coordinator",
                description: "For each goal: designs it, cuts it into changes and lands it",
            },
            Part::Described {
                role: Role::Disclosure,
                name: "Coding agent",
                description: "For each change: makes it, repairs it and resolves its conflicts",
            },
            Part::Described {
                role: Role::Disclosure,
                name: "Status agent",
                description: "When you look at Home after landings: writes the lines at its top",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Budgets" },
            Part::StyledText {
                style: temper_client_view::look::HOW_ROW,
                text: &["Goals", "each given its own when it starts", "80"],
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Ways of working" },
        ],
    },
    Holds {
        name: "how-yours",
        heading: "Agents review everything, then one of you signs off",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "shop runs a way of its own: agents review every change, and a rule of yours adds a sign-off. Ask temper to change any of it, in words; an owner accepts its proposal, and new work follows it.",
            },
            Part::Text { text: "the way: its own" },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Tabs { names: &["You approve", "Agents review", "Draft first", "Yoursnow"], selected: 3 },
            Part::Named { role: Role::Item, name: "produce, an agent" },
            Part::Named { role: Role::Item, name: "main, a branch" },
            Part::NoRole { role: Role::Table },
            Part::Absent { role: Role::Button, name: "Use this" },
            Part::Absent { role: Role::Button, name: "Keep it" },
        ],
    },
    Holds {
        name: "how-card",
        heading: "Agents review every change, and code lands it",
        parts: &[
            Part::Text { text: "the way: Agents review" },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Named { role: Role::Item, name: "review, an agent" },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 1 },
            Part::Named {
                role: Role::Paragraph,
                name: "An agent, an agent when needed, a person, code; no merge tool above the dotted line",
            },
            Part::NoRole { role: Role::Table },
            Part::Expanded { name: "review, an agent" },
            Part::Named { role: Role::Region, name: "Review" },
            Part::Named { role: Role::Link, name: "Open the reviewer ›" },
            Part::Named { role: Role::Term, name: "Model" },
            Part::Named { role: Role::Description, name: "Claude Opus 5.5 · high" },
            Part::Named { role: Role::Term, name: "Can" },
            Part::Named { role: Role::Description, name: "read the change and its checks; never edit, push or merge" },
            Part::Named { role: Role::Term, name: "Asked to" },
            Part::Named {
                role: Role::Description,
                name: "“Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.”",
            },
            Part::Drawing { style: temper_client_view::look::WAYS_CARET, count: 1 },
        ],
    },
    Holds {
        name: "night-how",
        heading: "Agents review",
        parts: &[
            Part::Text { text: "the way: Agents review" },
            Part::Named { role: Role::Link, name: "Ask temper to change this" },
            Part::Named { role: Role::Item, name: "review, an agent" },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 1 },
            Part::Named {
                role: Role::Paragraph,
                name: "An agent, an agent when needed, a person, code; no merge tool above the dotted line",
            },
            Part::NoRole { role: Role::Table },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "how-follows",
        heading: "Draft first, then agents build it on its own branch",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What follows a landing" },
            Part::Named { role: Role::Text, name: "code triggers, agents judge" },
            Part::Named { role: Role::Heading { level: 3 }, name: "cowsql/vsr-rs follows cowsql/vsr" },
            Part::Text {
                text: "Each landing on cowsql/vsr's main that touches src or include makes a change in cowsql/vsr-rs's main, owned by Pat. Each waits for Pat's word. Its check: .forgejo/workflows/vsr-tests.yml.",
            },
            Part::Text { text: "2 behind · check passes at abababababab" },
            Part::Named { role: Role::Text, name: "code hears the landing" },
            Part::Named { role: Role::Text, name: "an agent ports" },
            Part::Named { role: Role::Text, name: "code checks" },
            Part::Named { role: Role::Heading { level: 3 }, name: "Tiers on main" },
            Part::Text {
                text: "Code runs each after landings on main: fuzz (fuzzing for 6 hours). A red reaches the goals whose changes landed in it, and their agents repair it.",
            },
            Part::Named { role: Role::Text, name: "code runs" },
            Part::Named { role: Role::Text, name: "an agent repairs" },
            Part::Named { role: Role::Text, name: "a week · renews Tuesday" },
            Part::Named { role: Role::Text, name: "Owns the tests" },
            Part::Named { role: Role::Text, name: "a standing role · 28 left" },
            Part::Named { role: Role::Text, name: "40" },
            Part::Named { role: Role::Text, name: "Goals" },
            Part::Named { role: Role::Text, name: "80" },
        ],
    },
    Holds {
        name: "night-how-follows",
        heading: "Draft first, then agents build it on its own branch",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What follows a landing" },
            Part::Named { role: Role::Text, name: "code triggers, agents judge" },
            Part::Named { role: Role::Heading { level: 3 }, name: "cowsql/vsr-rs follows cowsql/vsr" },
            Part::Text {
                text: "Each landing on cowsql/vsr's main that touches src or include makes a change in cowsql/vsr-rs's main, owned by Pat. Each waits for Pat's word. Its check: .forgejo/workflows/vsr-tests.yml.",
            },
            Part::Text { text: "2 behind · check passes at abababababab" },
            Part::Named { role: Role::Text, name: "code hears the landing" },
            Part::Named { role: Role::Text, name: "an agent ports" },
            Part::Named { role: Role::Text, name: "code checks" },
            Part::Named { role: Role::Heading { level: 3 }, name: "Tiers on main" },
            Part::Text {
                text: "Code runs each after landings on main: fuzz (fuzzing for 6 hours). A red reaches the goals whose changes landed in it, and their agents repair it.",
            },
            Part::Named { role: Role::Text, name: "code runs" },
            Part::Named { role: Role::Text, name: "an agent repairs" },
            Part::Named { role: Role::Text, name: "a week · renews Tuesday" },
            Part::Named { role: Role::Text, name: "Owns the tests" },
            Part::Named { role: Role::Text, name: "a standing role · 28 left" },
            Part::Named { role: Role::Text, name: "40" },
            Part::Named { role: Role::Text, name: "Goals" },
            Part::Named { role: Role::Text, name: "80" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "how-reaches",
        heading: "Draft first, then agents build it on its own branch",
        parts: &[
            Part::Tabs { names: &["You approve", "Agents review", "Draft firstnow"], selected: 2 },
            Part::Named { role: Role::Heading { level: 2 }, name: "What reaches you" },
            Part::Named { role: Role::Text, name: "on factory" },
            HOW_REACHES_DECIDE,
            HOW_REACHES_ASK,
            HOW_WAIT,
            HOW_DECIDE_NOTE,
            HOW_REACHES_NOTE,
            HOW_WAIT_NOTE,
            Part::Named { role: Role::Select, name: "Questions a goal can answer from its plan" },
            Part::Named { role: Role::Select, name: "Away until" },
        ],
    },
    Holds {
        name: "night-how-reaches",
        heading: "Draft first, then agents build it on its own branch",
        parts: &[
            Part::Tabs { names: &["You approve", "Agents review", "Draft firstnow"], selected: 2 },
            Part::Named { role: Role::Heading { level: 2 }, name: "What reaches you" },
            Part::Named { role: Role::Text, name: "on factory" },
            HOW_REACHES_DECIDE,
            HOW_REACHES_ASK,
            HOW_WAIT,
            HOW_DECIDE_NOTE,
            HOW_REACHES_NOTE,
            HOW_WAIT_NOTE,
            Part::Named { role: Role::Select, name: "Questions a goal can answer from its plan" },
            Part::Named { role: Role::Select, name: "Away until" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "how-rules",
        heading: "Draft first, with the protocol core asking you",
        parts: &[
            Part::Tabs { names: &["You approve", "Agents review", "Draft first", "Yoursnow"], selected: 3 },
            Part::Named { role: Role::Item, name: "you, a person (if protocol core)" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Rules by path" },
            Part::Destination { name: "+ A rule", href: "/projects/1/ask" },
            Part::Absent { role: Role::ColumnHeader, name: "Review" },
            HOW_RULES_TABLE,
            HOW_RULES_NOTE,
            Part::StyleCount { style: temper_client_view::look::HOW_WHO_PERSON, count: 3 },
        ],
    },
    Holds {
        name: "how-rules-card",
        heading: "Draft first, with the protocol core asking you",
        parts: &[
            HOW_RULES_TABLE,
            HOW_RULES_NOTE,
            Part::Expanded { name: "you, a person (if protocol core)" },
            Part::Named { role: Role::Term, name: "Only for" },
            Part::Named {
                role: Role::Description,
                name: "include/vsr.h · src/protocol.* · src/transition.* · docs/protocol.md",
            },
            Part::Named { role: Role::Term, name: "Role" },
            Part::Named { role: Role::Description, name: "owner" },
            Part::Named { role: Role::Term, name: "Asks" },
            Part::Named { role: Role::Description, name: "1 of them" },
            Part::Named { role: Role::Term, name: "Counts" },
            Part::Named {
                role: Role::Description,
                name: "code counts the sign-offs at this head; a clean update keeps them",
            },
            Part::Drawing { style: temper_client_view::look::WAYS_CARET, count: 1 },
        ],
    },
    Holds {
        name: "how-team",
        heading: "Agents review everything, then one of you signs off",
        parts: &[
            Part::Reader { role: jig_client::contract::HeldRole::Owner },
            Part::Tabs { names: &["You approve", "Agents review", "Draft first", "Yoursnow"], selected: 3 },
            Part::Named { role: Role::Item, name: "review, an agent" },
            Part::Named { role: Role::Item, name: "one of you, a person" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Rules by path" },
            Part::Destination { name: "+ A rule", href: "/projects/1/ask" },
            HOW_TEAM_TABLE,
            HOW_RULES_NOTE,
            Part::StyleCount { style: temper_client_view::look::HOW_WHO_PERSON, count: 4 },
            Part::Described {
                role: Role::Disclosure,
                name: "Reviewer",
                description: "For each change: reviews it, and holds it until it approves",
            },
            Part::Absent { role: Role::Disclosure, name: "Foreman" },
        ],
    },
    Holds {
        name: "night-how-team",
        heading: "Agents review everything, then one of you signs off",
        parts: &[
            HOW_TEAM_TABLE,
            HOW_RULES_NOTE,
            Part::StyleCount { style: temper_client_view::look::HOW_WHO_PERSON, count: 4 },
            Part::Described {
                role: Role::Disclosure,
                name: "Reviewer",
                description: "For each change: reviews it, and holds it until it approves",
            },
            Part::Absent { role: Role::Disclosure, name: "Foreman" },
            Part::Pressed { name: "Night mode" },
        ],
    },
    Holds {
        name: "chat-view-way",
        heading: "T10 · How changes land",
        parts: &[
            Part::Named { role: Role::Group, name: "Agents review" },
            Part::Text { text: "Agents review" },
            Part::Text { text: " · now" },
            Part::Named { role: Role::List, name: "The way a change goes" },
            Part::Text { text: "produce, an agent" },
            Part::Text { text: "This is the way tally uses now." },
        ],
    },
    Holds {
        name: "chat-view-proposal",
        heading: "T10 · How changes land",
        parts: &[
            Part::Named { role: Role::Group, name: "Before" },
            Part::Named { role: Role::Group, name: "After" },
            Part::Text { text: "you approve, a person" },
            Part::Text { text: "review, an agent" },
        ],
    },
    Holds {
        name: "chat-view-charter",
        heading: "T10 · How changes land",
        parts: &[
            Part::Named { role: Role::Group, name: "The coding agent" },
            Part::Named { role: Role::Heading { level: 3 }, name: "The coding agent" },
            Part::Text { text: "an agent · makes the change" },
            Part::Named { role: Role::Term, name: "Model" },
            Part::Named { role: Role::Description, name: "Opus · high" },
            Part::Named { role: Role::Term, name: "Can" },
            Part::Named { role: Role::Term, name: "Asked to" },
        ],
    },
    Holds {
        name: "night-home",
        heading: "Hello is ready.",
        parts: &[
            Part::Pressed { name: "Night mode" },
            Part::Named { role: Role::Paragraph, name: "One step left. Skip it if you like; it stays in Ask temper." },
            Part::Text { text: "temper can change its code now." },
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Named { role: Role::Link, name: "Ask temper" },
            Part::Text { text: "work/hello is in Hello" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "night-ask",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Pressed { name: "Night mode" },
            Part::Named { role: Role::Region, name: "Ways of working" },
            Part::Value { role: Role::TextArea, name: "Ask temper, or tell it how you work", value: "" },
        ],
    },
    Holds {
        name: "night-chat-split",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Pressed { name: "Night mode" },
            Part::Pressed { name: "Split" },
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Named { role: Role::Button, name: "Show" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Paragraph, name: "Ctrl+\\ switches the draft" },
            Part::Named { role: Role::Region, name: "Away mode" },
        ],
    },
    Holds {
        name: "approve-typing",
        heading: "Sign in a terminal?",
        parts: &[
            Part::Value { role: Role::TextField, name: "Code", value: "wdjb mjht" },
            Part::Named { role: Role::Button, name: "Check" },
        ],
    },
    Holds {
        name: "approve-unread",
        heading: "Sign in a terminal?",
        parts: &[Part::Named {
            role: Role::Alert,
            name: "That isn’t a terminal’s code. Type the eight letters your terminal shows.",
        }],
    },
    Holds {
        name: "approve-asking",
        heading: "Sign in a terminal?",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "WDJB-MJHT" },
            Part::Text { text: "Asked 2 min ago" },
            Part::Named { role: Role::Button, name: "Approve" },
            Part::Named { role: Role::Button, name: "Refuse" },
        ],
    },
    Holds {
        name: "approve-approved",
        heading: "Approved",
        parts: &[Part::Named { role: Role::Paragraph, name: "WDJB-MJHT" }],
    },
    Holds {
        name: "approve-signed-in",
        heading: "Terminal signed in",
        parts: &[Part::Text { text: "Signed in. You can close this tab." }],
    },
    Holds {
        name: "approve-refused",
        heading: "Sign-in refused",
        parts: &[Part::Text { text: "This terminal’s sign-in was refused. Nothing was signed in." }],
    },
    Holds {
        name: "approve-expired",
        heading: "Code expired",
        parts: &[Part::Text { text: "This code has expired. Ask again in your terminal." }],
    },
    Holds {
        name: "approve-used",
        heading: "Code already used",
        parts: &[Part::Text {
            text: "This code was already approved or refused. If your terminal isn’t signed in, ask again there.",
        }],
    },
    Holds {
        name: "approve-unknown",
        heading: "Unknown code",
        parts: &[Part::Named { role: Role::Button, name: "Type a code" }],
    },
    Holds {
        name: "approve-signed-out",
        heading: "Sign in first",
        parts: &[Part::Text {
            text: "This browser isn’t signed in to temper. Open the sign-in link temper printed first, then open this page’s address again: your terminal shows it.",
        }],
    },
    Holds {
        name: "approve-signed-out-google",
        heading: "Sign in first",
        parts: &[
            Part::Text {
                text: "This browser isn’t signed in to temper. Sign in, and this page comes back with your terminal’s code.",
            },
            Part::Links { style: temper_client_view::look::OUTSIDE_PRIMARY, names: &["Continue with Google"] },
        ],
    },
    Holds {
        name: "approve-busy",
        heading: "temper couldn’t answer just now",
        parts: &[Part::Named { role: Role::Button, name: "Try again" }],
    },
    Holds {
        name: "ask",
        heading: "Ask temper",
        parts: &[
            Part::Named { role: Role::Form, name: "Ask temper" },
            Part::Value { role: Role::TextArea, name: "Ask temper, or tell it how you work", value: "" },
        ],
    },
    Holds {
        name: "chat-terminal-question",
        heading: "T1 · Make tally sum stream its input",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Menu, name: "Keep the trailing comma?" },
            Part::Named { role: Role::MenuItem, name: "Keep it" },
            Part::Named { role: Role::MenuItem, name: "Drop it" },
            Part::Named { role: Role::MenuItem, name: "Later" },
        ],
    },
    Holds {
        name: "chat-terminal-proposal",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Text, name: "Proposal" },
            Part::Named { role: Role::Disclosure, name: "Details" },
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "chat-terminal-draft",
        heading: "T1 · Away mode, draft",
        parts: &[Part::Named { role: Role::Button, name: "Exit" }, Part::Pressed { name: "Draft" }],
    },
    Holds {
        name: "chat-terminal-commands",
        heading: "T1 · Away mode, draft",
        parts: &[Part::Named { role: Role::Button, name: "Exit" }, Part::Named { role: Role::Menu, name: "Commands" }],
    },
    Holds {
        name: "chat-terminal-model",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Menu, name: "Model. A change applies from the next turn." },
        ],
    },
    Holds {
        name: "chat-terminal-light",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Paragraph, name: "Font: IBM Plex Mono." },
        ],
    },
    Holds {
        name: "chat-terminal-closed",
        heading: "T6",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::TextArea, name: "Message" },
        ],
    },
    Holds {
        name: "chat-terminal",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Paragraph, name: "/ for commands" },
        ],
    },
    Holds {
        name: "chat-terminal-working",
        heading: "T6",
        parts: &[
            Part::Named { role: Role::Button, name: "Exit" },
            Part::Named { role: Role::Paragraph, name: "/ for commands" },
            Part::Named { role: Role::Status, name: "Working" },
        ],
    },
    Holds {
        name: "role-talk",
        heading: "T52 · Prover",
        parts: &[
            Part::Named { role: Role::TextArea, name: "Talk" },
            Part::Named { role: Role::Button, name: "Leave" },
            Part::Text { text: "Add it to my rules?" },
        ],
    },
    Holds {
        name: "agent-work",
        heading: "T1 · please describe how temper works",
        parts: &[
            Part::Described {
                role: Role::Disclosure,
                name: "Checked settings 2×, read 1 doc",
                description: "checking template 2",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Run 1 · live now" },
            Part::Named { role: Role::Paragraph, name: "Agent" },
            Part::Named { role: Role::Button, name: "Latest" },
        ],
    },
    Holds {
        name: "chat-work-done",
        heading: "please describe how temper works",
        parts: &[
            Part::Described { role: Role::Disclosure, name: "Checked settings 3×, read 7 docs", description: "29 s" },
            Part::Named { role: Role::Paragraph, name: "settings charter 1: default" },
            Part::Named { role: Role::Paragraph, name: "read_docs settings, from 1482: Settings, on" },
            Part::Text {
                text: "Settings changes are proposed to an owner for acceptance; I haven’t changed anything.",
            },
        ],
    },
    Holds {
        name: "chat-work-open",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Disclosure, name: "Checked settings 3×, read 7 docs" }],
    },
    Holds {
        name: "chat-work-running",
        heading: "please describe how temper works",
        parts: &[
            Part::Described {
                role: Role::Disclosure,
                name: "Checked settings 2×, read 1 doc",
                description: "checking template 2",
            },
            Part::Named { role: Role::Paragraph, name: "settings template 2" },
            Part::Absent { role: Role::Paragraph, name: "settings template 2: default" },
        ],
    },
    Holds {
        name: "chat-work-writing",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Disclosure, name: "Checked settings 3×, read 7 docs" }],
    },
    Holds {
        name: "chat-work-failed",
        heading: "please describe how temper works",
        parts: &[
            Part::Described {
                role: Role::Disclosure,
                name: "Checked settings 3×, read 7 docs",
                description: "1 failed · 29 s",
            },
            Part::Named { role: Role::Paragraph, name: "settings charter 1: failed: not allowed" },
        ],
    },
    Holds {
        name: "chat-work-one",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Disclosure, name: "Read the page hold" }],
    },
    Holds {
        name: "chat-work-unknown",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Disclosure, name: "read_file ×2" }],
    },
    Holds {
        name: "chat-work-separated",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Disclosure, name: "Read the page hold" }],
    },
    Holds {
        name: "chat-work-empty",
        heading: "please describe how temper works",
        parts: &[Part::Named { role: Role::Slot, name: "Writing" }],
    },
    Holds {
        name: "chat-work-reading",
        heading: "please describe how temper works",
        parts: &[
            Part::Named { role: Role::Status, name: "Loading earlier messages" },
            Part::Named { role: Role::Button, name: "Latest" },
            Part::Absent { role: Role::Button, name: "Older" },
        ],
    },
    Holds {
        name: "chat-closing-below",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this chat?" },
            Part::Text { text: "1 task below it is cancelled." },
        ],
    },
    Holds {
        name: "chat-closing-proposal",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this chat?" },
            Part::Text { text: "1 proposal is withdrawn." },
        ],
    },
    Holds {
        name: "task-releasing-uncertain",
        heading: "T3 · CSV import",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Release this task?" },
            Part::Named { role: Role::Button, name: "Repeat the effect" },
            Part::Named { role: Role::Button, name: "Do not repeat the effect" },
        ],
    },
    Holds {
        name: "chat-closing-work",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this chat?" },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Named { role: Role::Button, name: "Close chat" },
        ],
    },
    Holds {
        name: "chat-closing-failed",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this chat?" },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Named { role: Role::Button, name: "Close chat" },
        ],
    },
    Holds {
        name: "chat-closing-start-over",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start over?" },
            Part::Named { role: Role::Button, name: "Keep going" },
            Part::Named { role: Role::Button, name: "Start over" },
        ],
    },
    Holds {
        name: "task-closing",
        heading: "T3 · CSV import",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this task?" },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Named { role: Role::Button, name: "Close task" },
        ],
    },
    Holds {
        name: "task-paused",
        heading: "T3 · CSV import",
        parts: &[
            Part::Named { role: Role::Button, name: "Resume" },
            Part::Named { role: Role::Button, name: "Close" },
            Part::Text { text: "Paused" },
        ],
    },
    Holds {
        name: "proposal-accepting",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Accept this proposal?" },
            Part::Named { role: Role::Button, name: "Not yet" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "proposal-rejecting",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Reject this proposal?" },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Named { role: Role::Button, name: "Reject" },
        ],
    },
    Holds {
        name: "chat-paused",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Button, name: "Resume" },
            Part::Named { role: Role::Button, name: "Close" },
            Part::Named { role: Role::Status, name: "Paused" },
            Part::Text { text: "Sending resumes this chat. · Ctrl+\\ switches the draft" },
            Part::Absent { role: Role::Button, name: "Pause" },
            Part::Absent { role: Role::Status, name: "Applied" },
        ],
    },
    Holds {
        name: "chat-closed",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Closed at 21:05" },
            Part::NoStyle { style: temper_client_view::look::STATUS_LINE },
            Part::Absent { role: Role::TextArea, name: "Message" },
        ],
    },
    Holds {
        name: "rail-person-menu",
        heading: "tally",
        parts: &[
            Part::Named { role: Role::Menu, name: "Robin" },
            Part::Named { role: Role::MenuItem, name: "Accounts" },
            Part::Named { role: Role::MenuItem, name: "Sign out" },
        ],
    },
    Holds {
        name: "rail-new-menu",
        heading: "factory",
        parts: &[
            Part::Named { role: Role::Menu, name: "New draft or change" },
            Part::Named { role: Role::MenuItem, name: "Ask temper" },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Needs", "Work", "Ask temper"] },
            Part::Described {
                role: Role::MenuItem,
                name: "Draft",
                description: "A design session that ends in a goal, best had in the terminal.",
            },
            Part::Described { role: Role::MenuItem, name: "Change", description: "Ask for one change." },
        ],
    },
    Holds {
        name: "home-try-stopped",
        heading: "Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "ChatGPT didn’t answer when tried" },
            Part::Named { role: Role::Button, name: "Try again" },
        ],
    },
    Holds {
        name: "home-decided",
        heading: "57 changes landed. One thing needs you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 2" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 1" },
            Part::Named { role: Role::Text, name: "Allowed: wording choices are decided and listed." },
        ],
    },
    Holds {
        name: "needs-decided",
        heading: "The repository has no license. Name one in the README?",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 2" },
            Part::Named { role: Role::Text, name: "for 24 hours, then in each task's history" },
            Part::Absent { role: Role::Button, name: "Limits" },
            Part::Named { role: Role::Text, name: "fabrig · welcome-stall · 01:05" },
            Part::Named { role: Role::Text, name: "Widget · forge-cards · 06:15" },
        ],
    },
    Holds {
        name: "needs-decided-project",
        heading: "The repository has no license. Name one in the README?",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 2" },
            Part::Named { role: Role::Button, name: "Limits" },
            Part::Named { role: Role::Text, name: "welcome-stall · 01:05" },
            Part::Absent { role: Role::Text, name: "for 24 hours, then in each task's history" },
        ],
    },
    Holds {
        name: "needs-only-you",
        heading: "One thing needs you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Region, name: "Only you · 2" },
            Part::Text { text: "Your ChatGPT sign-in is signed out. Sign in again?" },
            Part::Text { text: "Your Claude Code login claude-pat is expiring. Sign in again?" },
            Part::Destination { name: "Sign in again", href: "/accounts" },
        ],
    },
    Holds {
        name: "needs-only-you-done",
        heading: "Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Only you · 1" },
            Part::Named { role: Role::Link, name: "Home" },
        ],
    },
    Holds {
        name: "night-needs-only-you",
        heading: "One thing needs you.",
        parts: &[Part::Named { role: Role::Region, name: "Only you · 2" }],
    },
    Holds {
        name: "home-away",
        heading: "57 changes landed. Four things were decided for you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 4" },
            Part::Named { role: Role::Button, name: "Limits" },
        ],
    },
    Holds {
        name: "home-away-quiet",
        heading: "57 changes landed. One thing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "While you were away · Mon 23:10 to now" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Still needs you · 1" },
            Part::NoStyle { style: temper_client_view::look::DECIDED },
        ],
    },
    Holds {
        name: "home-away-overturned",
        heading: "57 changes landed. Four things were decided for you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 4" },
            Part::Named { role: Role::Button, name: "Overturn" },
        ],
    },
    Holds {
        name: "home-away-one",
        heading: "57 changes landed. One thing was decided for you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 1" },
            Part::Named { role: Role::Button, name: "Limits" },
            Part::Named { role: Role::Button, name: "Away: off" },
        ],
    },
    Holds {
        name: "rail-away-off",
        heading: "41 changes landed. Three things need you.",
        parts: &[
            Part::Named { role: Role::Button, name: "Away: off" },
            Part::Text { text: "Away is off: questions wait for you again." },
        ],
    },
    Holds {
        name: "rail-away-on",
        heading: "41 changes landed. Three things need you.",
        parts: &[
            Part::Named { role: Role::Button, name: "Away until 08:00" },
            Part::Text { text: "Away until 08:00: agents decide within Hello’s limits." },
            Part::Named { role: Role::Button, name: "Limits" },
        ],
    },
    Holds {
        name: "start-goal",
        heading: "6 goals, 4 under way.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start a goal" },
            Part::Value { role: Role::TextField, name: "Draft", value: "" },
            Part::Value { role: Role::TextField, name: "Name", value: "" },
        ],
    },
    Holds {
        name: "start-goal-draft",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start a goal from away-mode.md" },
            Part::Value { role: Role::TextField, name: "Name", value: "away-mode" },
            Part::Named { role: Role::ChoiceGroup, name: "How it runs" },
            Part::Named { role: Role::ChoiceGroup, name: "When it needs you" },
            Part::Named { role: Role::Choice, name: "As the project says" },
            Part::Text {
                text: "Decides what the project’s settings let agents decide, Away included, and tells you; asks you the rest.",
            },
            Part::Named { role: Role::Choice, name: "Always ask" },
            Part::Named { role: Role::Button, name: "Start goal" },
        ],
    },
    Holds {
        name: "start-goal-refused",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start a goal from away-mode.md" },
            Part::Text { text: "This act is not offered" },
        ],
    },
    Holds {
        name: "goal-sending",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[
            Part::Text { text: "Keep the Ready page in this goal." },
            Part::Named { role: Role::Status, name: "Being sent" },
            Part::Named { role: Role::TextArea, name: "Tell first-run-projects something" },
        ],
    },
    Holds {
        name: "goal-amending",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[
            Part::Named { role: Role::Region, name: "Amend task 108" },
            Part::Named { role: Role::TextArea, name: "Specification" },
        ],
    },
    Holds {
        name: "goal-closing",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Close this goal?" },
            Part::Text { text: "What landed on its branch stays there." },
        ],
    },
    Holds {
        name: "goal-toast",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[Part::Text { text: "Paused: running tasks stop, nothing new starts." }],
    },
    Holds {
        name: "goal-running",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[
            Part::Text { text: "Plan, tasks, land whole · asks as the project says" },
            Part::Text { text: "31 of 40 spent" },
            Part::Text { text: "since Mon 19:10" },
            Part::Text { text: "waits on 2 tasks" },
            Part::Text { text: "Left: " },
            Part::Text { text: "Risk: " },
            Part::Named { role: Role::Disclosure, name: "6 landed" },
            Part::Named { role: Role::Figure, name: "6 landed of 9" },
        ],
    },
    Holds {
        name: "goal-needs-funds",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[
            Part::Text { text: "needs funds" },
            Part::Text { text: "31 of 40 used." },
            Part::Named { role: Role::Button, name: "Release" },
            Part::Named { role: Role::Disclosure, name: "Leave held" },
        ],
    },
    Holds {
        name: "goal-needs-question",
        heading: "Reproduced in a box test; the fault is in jig’s store. Waiting on where to fix it.",
        parts: &[
            Part::Text { text: "needs you" },
            Part::Named { role: Role::Button, name: "In jig’s store" },
            Part::Named { role: Role::Button, name: "In temper serve" },
            Part::Named { role: Role::Link, name: "Chat T2" },
            Part::Text { text: "waits on you" },
            Part::Text { text: "waits on The fix" },
        ],
    },
    Holds {
        name: "goal-planning",
        heading: "Planning from forge-cards.md: reading it and the designs it cites.",
        parts: &[
            Part::Text { text: "Proposing tasks from the draft…" },
            Part::Named { role: Role::Figure, name: "Planning" },
        ],
    },
    Holds {
        name: "goal-paused",
        heading: "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
        parts: &[Part::Text { text: "paused by you" }],
    },
    Holds {
        name: "goal-landed",
        heading: "Landed whole at 01:12: two tasks and one journey print.",
        parts: &[Part::Text { text: "Landed at 01:12." }, Part::Named { role: Role::Disclosure, name: "2 landed" }],
    },
    Holds {
        name: "goal-landed-open",
        heading: "Landed whole at 01:12: two tasks and one journey print.",
        parts: &[
            Part::Named { role: Role::Link, name: "Journey print" },
            Part::Named { role: Role::Link, name: "Pane chat line" },
        ],
    },
    Holds {
        name: "goal-closed",
        heading: "Closed by you. What landed on the goal branch stays there.",
        parts: &[Part::Text { text: "Closed before it landed." }],
    },
    Holds {
        name: "goal-failed",
        heading: "Failed: The owned set up could not create work/hello.",
        parts: &[Part::Text { text: "Failed before it landed." }],
    },
    Holds {
        name: "goal-no-report",
        heading: "6 of 9 tasks done.",
        parts: &[Part::Text { text: "running" }, Part::Named { role: Role::Figure, name: "6 landed of 9" }],
    },
    Holds {
        name: "chat-draft-empty",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Region, name: "Draft" },
            Part::Named { role: Role::Paragraph, name: "The draft appears here as you write it together." },
            Part::Absent { role: Role::Button, name: "+ Draft" },
            Part::Named { role: Role::Button, name: "Write it up from this chat" },
        ],
    },
    Holds {
        name: "ask-chat-draft-empty",
        heading: "T10 · Ask temper",
        parts: &[
            Part::Named { role: Role::Region, name: "Draft" },
            Part::Text { text: "T10 · no draft yet" },
            Part::Named { role: Role::Button, name: "Write it up from this chat" },
            Part::Named { role: Role::Button, name: "Start over" },
            Part::Named { role: Role::Group, name: "The draft" },
        ],
    },
    Holds {
        name: "chat-new-draft",
        heading: "New chat",
        parts: &[
            Part::Named { role: Role::Region, name: "untitled.md" },
            Part::Named { role: Role::TextArea, name: "What should this draft hold?" },
        ],
    },
    Holds {
        name: "models",
        heading: "What the picker offers.",
        parts: &[
            Part::Text { text: "Your Claude subscription. These come with temper." },
            Part::Text { text: "Your ChatGPT plan. These come with temper." },
            Part::Text { text: "claude-opus-5-5 · high · the default" },
            Part::Text { text: "claude-sonnet-5-5 · high" },
            Part::Text { text: "claude-fable-5-1 · high" },
            Part::Text { text: "gpt-6.1-sol · high" },
            Part::Text { text: "gpt-6-astra · high" },
            Part::Text { text: "gpt-6.1-terra · medium" },
            Part::Text { text: "gpt-6-luna · medium" },
            Part::Text { text: "Off" },
            Part::Named { role: Role::Link, name: "ask temper." },
        ],
    },
    Holds {
        name: "models-read",
        heading: "What the picker offers.",
        parts: &[
            Part::Text { text: "Your ChatGPT plan. Read from OpenAI 2 hours ago." },
            Part::Named { role: Role::Button, name: "Read again" },
        ],
    },
    Holds {
        name: "models-read-failed",
        heading: "What the picker offers.",
        parts: &[
            Part::Text { text: "Your ChatGPT plan. Couldn't read OpenAI's list · 2 hours ago." },
            Part::Named { role: Role::Button, name: "Read again" },
        ],
    },
    Holds {
        name: "models-new",
        heading: "What the picker offers.",
        parts: &[Part::Text { text: "gpt-6-luna · medium · new" }],
    },
    Holds {
        name: "chat-model-new",
        heading: "New chat",
        parts: &[Part::Text { text: "new" }, Part::Named { role: Role::Button, name: "Haiku 5.5" }],
    },
    Holds {
        name: "models-add",
        heading: "What the picker offers.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Add a model by name" },
            Part::Named { role: Role::Select, name: "Provider" },
            Part::Value { role: Role::TextField, name: "The model’s name", value: "claude-haiku-5-5" },
            Part::Named { role: Role::Button, name: "Add" },
        ],
    },
    Holds {
        name: "models-off",
        heading: "What the picker offers.",
        parts: &[Part::Named { role: Role::Check, name: "Sonnet 5.5 in the picker" }, Part::Text { text: "Off" }],
    },
    Holds {
        name: "models-added",
        heading: "What the picker offers.",
        parts: &[Part::Text { text: "claude-sonnet-5-5 · high · added by you · estimated prices" }],
    },
    Holds {
        name: "models-member",
        heading: "What the picker offers.",
        parts: &[Part::NoRole { role: Role::Check }, Part::Text { text: "On" }, Part::Text { text: "Off" }],
    },
    Holds {
        name: "models-one-account",
        heading: "What the picker offers.",
        parts: &[
            Part::Text { text: "Your Claude subscription. These come with temper." },
            Part::Text { text: "claude-opus-5-5 · high · the default" },
            Part::Named { role: Role::Link, name: "Connect a plan or key ›" },
            Part::Absent { role: Role::Text, name: "GPT-6.1 Sol" },
            Part::Absent { role: Role::Text, name: "GPT-6 Astra" },
            Part::Absent { role: Role::Text, name: "GPT-6.1 Terra" },
            Part::Absent { role: Role::Text, name: "GPT-6 Luna" },
        ],
    },
    Holds {
        name: "builders",
        heading: "Where agents work.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Workers · 3" },
            Part::Text { text: "Workshop" },
            Part::Text { text: "Finishing" },
            Part::Text { text: "Away" },
            Part::Text { text: "2 slots · 1 in use" },
            Part::Text { text: "Connected" },
            Part::Text { text: "Draining" },
            Part::Text { text: "Not connected" },
        ],
    },
    Holds {
        name: "builders-none",
        heading: "Where agents work.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Workers · 0" },
            Part::Text { text: "No worker is connected." },
        ],
    },
    Holds {
        name: "builders-not-owner",
        heading: "Where agents work.",
        parts: &[Part::Text { text: "Workers are seen by the deployment’s owners." }],
    },
    Holds {
        name: "forges-empty",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Text, name: "No forge yet" },
            Part::Named { role: Role::Link, name: "Connect a forge" },
            Part::Text { text: "No forge of yours yet." },
        ],
    },
    Holds {
        name: "forges-added",
        heading: "Where your code lives.",
        parts: &[Part::Text { text: "Connected Forgejo on this machine." }],
    },
    Holds {
        name: "forges-setting-up",
        heading: "Where your code lives.",
        parts: &[
            Part::Described {
                role: Role::Disclosure,
                name: "Forgejo on this machine",
                description: "Forge 3 · Forgejo · Setting up: preparing · work/hello",
            },
            Part::Text { text: "Preparing: its database, temper’s admin, your account" },
        ],
    },
    Holds {
        name: "forges-fetching",
        heading: "Where your code lives.",
        parts: &[Part::Described {
            role: Role::Disclosure,
            name: "Forgejo on this machine",
            description: "Forge 3 · Forgejo · Setting up: fetching Forgejo · 42% · work/hello",
        }],
    },
    Holds {
        name: "forges-fetching-bytes",
        heading: "Where your code lives.",
        parts: &[Part::Described {
            role: Role::Disclosure,
            name: "Forgejo on this machine",
            description: "Forge 3 · Forgejo · Setting up: fetching Forgejo · 41 MB · work/hello",
        }],
    },
    Holds {
        name: "forges-failed",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Button, name: "Try again" },
            Part::Described {
                role: Role::Disclosure,
                name: "Forgejo on this machine",
                description: "Forge 3 · Forgejo · Set up failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
            },
        ],
    },
    Holds {
        name: "forges-replacing",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Private key" },
            Part::Named { role: Role::Button, name: "Replace the key" },
        ],
    },
    Holds {
        name: "forges-removing",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Remove this forge?" },
            Part::Named { role: Role::Button, name: "Remove" },
        ],
    },
    Holds {
        name: "needs-list",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Text { text: "oldest first" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Decided for you · 3" },
            Part::StyleCount { style: temper_client_view::look::DECISION, count: 3 },
            Part::After { first: temper_client_view::look::NEED_ROW, then: temper_client_view::look::DECIDED },
            Part::Text { text: "Ported it as checked_add." },
            Part::Text { text: "Split it in two." },
            Part::Text { text: "Accepted it." },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::TextArea, name: "Or tell it something else" },
            Part::Absent { role: Role::Button, name: "Send" },
        ],
    },
    Holds {
        name: "needs-list-second",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "31 of 40 used." },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Text { text: "oldest first" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Button, name: "Release" },
        ],
    },
    Holds {
        name: "needs-tell",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Text { text: "oldest first" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Button, name: "Send" },
        ],
    },
    Holds {
        name: "needs-approval-first",
        heading: "Three things need you.",
        parts: &[
            Part::StyledText { style: temper_client_view::look::CHIPS, text: &["+142 -18", " in 4 files"] },
            Part::Named { role: Role::Heading { level: 1 }, name: "Approve A change on GitHub" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Text { text: "oldest first" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Link, name: "Review on GitHub ↗" },
        ],
    },
    Holds {
        name: "night-needs",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Text { text: "oldest first" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::TextArea, name: "Or tell it something else" },
        ],
    },
    Holds {
        name: "needs-one",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
        ],
    },
    Holds {
        name: "needs-saw-open",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
        ],
    },
    Holds {
        name: "needs-reply-open",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::TextArea, name: "Or tell it something else" },
        ],
    },
    Holds {
        name: "needs-second-answered",
        heading: "Two things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Button, name: "Undo" },
            Part::Text { text: "Answered: In jig’s store" },
        ],
    },
    Holds {
        name: "needs-later",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
            // The browser keeps this state name for opening the next closed row.
            Part::Named { role: Role::TextArea, name: "Or tell it something else" },
        ],
    },
    Holds {
        name: "needs-last",
        heading: "One thing needs you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "The repository has no license. Name one in the README?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Text { text: "3 of 3" },
        ],
    },
    Holds {
        name: "needs-done",
        heading: "Nothing needs you.",
        parts: &[Part::Named { role: Role::Link, name: "Home" }],
    },
    Holds {
        name: "needs-approval",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "Approve A change on Forgejo" },
            Part::Text { text: "This project's landing rule 1 asks for one approval by an owner." },
            Part::Named { role: Role::Link, name: "Review on Forgejo ↗" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Absent { role: Role::Button, name: "Later" },
        ],
    },
    Holds {
        name: "needs-sign-off-part",
        heading: "One thing needs you.",
        parts: &[
            Part::Text { text: "This project's landing rule 1 asks for 2 approvals by members." },
            Part::Text { text: "Signs off changes to payments." },
            Part::Text { text: "1 of 2 signed: Sam; you or Kit can give the second." },
            Part::Named { role: Role::Link, name: "Sign off on Forgejo ↗" },
        ],
    },
    Holds {
        name: "needs-approval-path",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "Approve A change on Forgejo" },
            Part::Text { text: "Signs off changes to vsr's protocol core." },
            Part::Named { role: Role::Link, name: "Sign off on Forgejo ↗" },
            Part::Named { role: Role::Button, name: "Open" },
        ],
    },
    Holds {
        name: "needs-approval-no-account",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "Approve A change on GitHub" },
            Part::Text { text: "The deployment's landing rule 2 asks for 2 approvals by maintainers." },
            Part::Text { text: "temper knows no GitHub account of yours, so your review wouldn't count." },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Absent { role: Role::Button, name: "Later" },
        ],
    },
    Holds {
        name: "needs-finding",
        heading: "The phone card is clipped",
        parts: &[Part::Named { role: Role::Button, name: "Hand on" }],
    },
    Holds {
        name: "needs-finding-hand",
        heading: "The phone card is clipped",
        parts: &[Part::Named { role: Role::TextField, name: "Why hand it on" }],
    },
    Holds {
        name: "needs-finding-route",
        heading: "The phone card is clipped",
        parts: &[Part::Named { role: Role::Disclosure, name: "Goal" }],
    },
    Holds {
        name: "needs-finding-drop",
        heading: "The phone card is clipped",
        parts: &[Part::Named { role: Role::TextField, name: "Reason to drop" }],
    },
    Holds {
        name: "needs-finding-refused",
        heading: "The phone card is clipped",
        parts: &[Part::Named { role: Role::Paragraph, name: "not a live goal of the project" }],
    },
    Holds {
        name: "needs-proposal",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "Accept how temper will run your work." },
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Disclosure, name: "Reject" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Text { text: "Funding after: 34" },
        ],
    },
    Holds {
        name: "needs-stop-asking",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[
            Part::Named { role: Role::Button, name: "Keep asking" },
            Part::Expanded { name: "Stop asking: let agents decide implementation choices" },
            Part::Reader { role: jig_client::contract::HeldRole::Owner },
            Part::Text { text: "Let agents decide implementation choices, then tell you." },
            Part::Named { role: Role::Button, name: "Apply" },
            Part::Absent { role: Role::Button, name: "Propose" },
        ],
    },
    Holds {
        name: "needs-stop-asking-set",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[
            Part::Named { role: Role::Button, name: "Keep asking" },
            Part::Expanded { name: "Stop asking: let agents decide some changes to protocol core" },
            Part::Reader { role: jig_client::contract::HeldRole::Member },
            Part::Text { text: "Let agents decide changes to protocol core that are:" },
            Part::Named { role: Role::TextField, name: "Which changes" },
            Part::Text { text: "Let agents decide changes to src/wire/**, apart from the rest of protocol core." },
            Part::Named { role: Role::Button, name: "Propose" },
            Part::Absent { role: Role::Button, name: "Apply" },
        ],
    },
    Holds {
        name: "needs-why",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[
            Part::Named { role: Role::Button, name: "Keep asking" },
            Part::Text {
                text: "Reached you because it is an implementation choice, which asks you unless you’re away.",
            },
            Part::Named { role: Role::Button, name: "Stop asking: let agents decide implementation choices" },
        ],
    },
    Holds {
        name: "needs-held",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "first-run-projects" },
            Part::Named { role: Role::Button, name: "Release" },
            Part::Named { role: Role::Disclosure, name: "Leave held" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Text { text: "Both halves of the store task run. The Ready page waits for this decision." },
        ],
    },
    Holds {
        name: "needs-funds",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "31 of 40 used." },
            Part::Named { role: Role::Button, name: "Release" },
            Part::Named { role: Role::Disclosure, name: "Leave held" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Absent { role: Role::Button, name: "Add $10" },
            Part::Text { text: "first-run-projects" },
        ],
    },
    Holds {
        name: "needs-plain-words",
        heading: "Three things need you.",
        parts: &[
            Part::Named {
                role: Role::Heading { level: 1 },
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Absent { role: Role::Button, name: "In jig’s store" },
        ],
    },
    Holds {
        name: "changes",
        heading: "Three changes are on the way to main.",
        parts: &[
            Part::Text { text: "3 queued" },
            Part::Text { text: "last landed 6 min ago" },
            Part::Text { text: "first-run-projects" },
            Part::Text { text: "queued 2nd" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Landed recently" },
        ],
    },
    Holds {
        name: "changes-empty",
        heading: "Nothing is on the way to main.",
        parts: &[Part::Text { text: "nothing has landed yet" }, Part::Text { text: "Empty." }],
    },
    Holds {
        name: "changes-one",
        heading: "One change is on the way to main.",
        parts: &[Part::Text { text: "1 queued" }, Part::Text { text: "checking" }],
    },
    Holds {
        name: "changes-landed-only",
        heading: "Nothing is on the way to main.",
        parts: &[
            Part::Text { text: "main last moved 1 h ago" },
            Part::Text { text: "nothing queued" },
            Part::Text { text: "Hello in Rust" },
            Part::Text { text: "07:02" },
        ],
    },
    Holds {
        name: "all-home",
        heading: "31 changes landed across three projects. Two things need you.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Sunday, 08:04 · you last looked 9 hours ago · away 23:10 to 08:00",
            },
            Part::StyledText {
                style: temper_client_view::look::HOME_LEDE,
                text: &[
                    "vsr: ",
                    "vsr landed 12 changes, and its membership-handoff goal is six of nine tasks in.",
                    " ",
                    "vsr-rs: ",
                    "vsr-rs landed 4 ports and is two landings behind vsr, every C test passing on Rust.",
                    " ",
                    "fabrig: ",
                    "fabrig landed 15, proof-6 whole among them.",
                ],
            },
            Part::Named {
                role: Role::Paragraph,
                name: "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52",
            },
            Part::Named { role: Role::Heading { level: 1 }, name: "31 changes landed. Two things need you." },
            Part::Text { text: "The status agent, 07:52" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 2" },
            Part::HomeCards { primary: &["Only once caught up", "Release"], projects: &["vsr", "vsr-rs"] },
            Part::Text {
                text: "Count a rejoining witness toward the view-change quorum only once it holds the log suffix?",
            },
            Part::Text { text: "49 of 60 used." },
            Part::Named { role: Role::Heading { level: 2 }, name: "Decided for you · 4" },
            Part::StyleCount { style: temper_client_view::look::DECISION, count: 4 },
            Part::Text { text: "vsr · Prover · 02:10" },
            Part::Text { text: "vsr-rs · Porter · 03:40" },
            Part::Text { text: "fabrig · a goal · 05:12" },
            Part::Text { text: "fabrig · UI · 06:30" },
            Part::Named { role: Role::Button, name: "Overturn" },
            Part::Text { text: "Fixed the harness." },
            Part::Text { text: "Ported it as checked_add." },
            Part::Text { text: "Split it in two." },
            Part::Text { text: "Accepted it." },
            Part::Named { role: Role::Link, name: "Decided for you · 4 ›" },
            Part::Projects { rows: &[("vsr", "main"), ("vsr-rs", "main"), ("cowsql", "main"), ("fabrig", "main red")] },
            Part::Text { text: "main red" },
            Part::After { first: temper_client_view::look::HOME_NEEDS, then: temper_client_view::look::DECIDED },
            Part::After { first: temper_client_view::look::DECIDED, then: temper_client_view::look::PROJECTS },
            Part::Named { role: Role::Region, name: "What follows what" },
            Part::Named { role: Role::Heading { level: 2 }, name: "What follows what" },
            Part::Named { role: Role::Text, name: "across projects" },
            Part::Named { role: Role::Group, name: "vsr-rs follows work/vsr" },
            Part::Named { role: Role::Text, name: "follows work/vsr" },
            Part::Named { role: Role::Heading { level: 3 }, name: "vsr-rs" },
            Part::Named { role: Role::Paragraph, name: "2 behind · check passes at 9c04d7e1a2b3" },
            Part::Named { role: Role::Group, name: "fabrig depends on vsr-rs" },
            Part::Named { role: Role::Text, name: "depends on vsr-rs" },
            Part::Named { role: Role::Heading { level: 3 }, name: "fabrig" },
            Part::Named { role: Role::Paragraph, name: "Nothing follows its landings." },
            Part::After { first: temper_client_view::look::PROJECTS, then: temper_client_view::look::HOW_FOLLOWS },
            Part::After { first: temper_client_view::look::HOW_FOLLOWS, then: temper_client_view::look::ASK_LINE },
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::NoStyle { style: temper_client_view::look::HOME_FACTS },
            Part::Absent { role: Role::Region, name: "Chats" },
        ],
    },
    Holds {
        name: "all-home-nothing-new",
        heading: "The last change landed 10 hours ago. Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Sunday, 08:04 · you last looked 9 hours ago" },
            Part::NoStyle { style: temper_client_view::look::HOME_LEDE },
            Part::NoStyle { style: temper_client_view::look::HOME_BYLINE },
            Part::Absent { role: Role::Region, name: "Needs you" },
            Part::Absent { role: Role::Region, name: "Decided for you" },
            Part::Projects { rows: &[("vsr", "main"), ("vsr-rs", "main"), ("cowsql", "main"), ("fabrig", "main red")] },
            Part::Named { role: Role::TextField, name: "Ask temper" },
        ],
    },
    Holds {
        name: "all-home-one-project",
        heading: "12 changes landed in vsr. One thing needs you.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52",
            },
            Part::HomeCards { primary: &["Only once caught up"], projects: &["vsr"] },
            Part::Absent { role: Role::Region, name: "Projects" },
            Part::StyleCount { style: temper_client_view::look::DECISION, count: 1 },
            Part::Named { role: Role::TextField, name: "Ask temper" },
        ],
    },
    Holds {
        name: "all-home-away",
        heading: "31 changes landed across three projects. Two things need you.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52",
            },
            Part::Named { role: Role::Button, name: "Away on 2 of 4" },
            Part::NotPressed { name: "Away on 2 of 4" },
        ],
    },
    Holds {
        name: "all-home-away-on",
        heading: "31 changes landed across three projects. Two things need you.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52",
            },
            Part::Pressed { name: "Away until 08:00" },
            Part::Text { text: "Away turned on for 2 projects: agents decide within their limits." },
        ],
    },
    Holds {
        name: "night-all-home",
        heading: "31 changes landed across three projects. Two things need you.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Each project's brief, its first line · the newest, vsr's, by the status agent at 07:52",
            },
            Part::Pressed { name: "Night mode" },
            Part::Text { text: "vsr · Prover · 02:10" },
            Part::Named { role: Role::Button, name: "Overturn" },
            Part::HomeCards { primary: &["Only once caught up", "Release"], projects: &["vsr", "vsr-rs"] },
            Part::StyleCount { style: temper_client_view::look::DECISION, count: 4 },
            Part::Projects { rows: &[("vsr", "main"), ("vsr-rs", "main"), ("cowsql", "main"), ("fabrig", "main red")] },
        ],
    },
    Holds {
        name: "home-step-pair",
        heading: "One change landed today. Three things need you.",
        parts: &[
            Part::StyledText { style: temper_client_view::look::CHIPS, text: &["+142 -18", " in 4 files"] },
            Part::Named { role: Role::Paragraph, name: "Written by the status agent at 19:12" },
            Part::Text {
                text: "Monthly totals by category and dark mode are both checked by CI; nothing reaches main until you approve. The budget alerts change asks whether alerts go by email. The CSV import fix you approved this afternoon landed at 16:20.",
            },
            Part::HomeCards {
                primary: &["Only in the app", "Review on GitHub ↗", "Review on GitHub ↗"],
                projects: &[],
            },
            Part::Text { text: "Approve Monthly totals by category on GitHub" },
            Part::Text { text: "Send budget alerts by email, or only show them in the app?" },
            Part::Text { text: "main last moved 2 h 54 min ago" },
            Part::Absent { role: Role::Region, name: "Decided for you" },
            // Home has no landed-changes list; the brief and Landing foot hold what landed.
        ],
    },
    Holds {
        name: "home-step-team",
        heading: "9 changes landed since Friday. One thing needs you.",
        parts: &[
            Part::HomeCards { primary: &["Keep them case-insensitive"], projects: &[] },
            Part::Named { role: Role::Heading { level: 2 }, name: "Decided for you · 1" },
            Part::Text { text: "A task ran out of turns on the coupon rules." },
            Part::Text { text: "Split it in two." },
            Part::Absent { role: Role::Link, name: "Sign off on Forgejo ↗" },
            // These approvals name no path set.
        ],
    },
    Holds {
        name: "home-step-stock",
        heading: "12 changes landed. Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Its membership-handoff goal is six of nine tasks in." },
            Part::Named { role: Role::Paragraph, name: "Written by the status agent at 07:52" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Decided for you · 1" },
            Part::Text { text: "Fixed the harness." },
            Part::Absent { role: Role::Region, name: "Needs you" },
        ],
    },
    Holds {
        name: "home-step-scale",
        heading: "12 changes landed. Two things need you.",
        parts: &[
            Part::HomeCards { primary: &["Only once caught up", "Release"], projects: &[] },
            Part::Text { text: "49 of 60 used." },
            Part::Named { role: Role::Heading { level: 2 }, name: "Decided for you · 1" },
            // Owners' Nows and standing-role weekly budgets have no client read.
        ],
    },
    Holds {
        name: "home-morning",
        heading: "41 changes landed overnight. Three things need you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Tuesday, 08:04 · you last looked 9 hours ago" },
            Part::Named {
                role: Role::Paragraph,
                name: "first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two.",
            },
            Part::Named { role: Role::Paragraph, name: "Written by the status agent at 07:42" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Chats" },
            Part::Text { text: "Landing: " },
            Part::Text { text: "3 queued" },
            Part::Text { text: "last landed 6 min ago" },
            Part::Named { role: Role::Link, name: "Changes" },
        ],
    },
    Holds {
        name: "home-last-landed",
        heading: "The last change landed 6 minutes ago. Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Thursday, 21:19 · you last looked 9 hours ago" },
            Part::Text { text: "main last moved 6 min ago" },
            Part::Named { role: Role::Link, name: "Changes" },
        ],
    },
    Holds {
        name: "home-computed",
        heading: "2 changes landed since you last looked. Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Thursday, 21:19 · you last looked 9 hours ago" },
            Part::Text { text: "3 queued" },
            Part::Text { text: "last landed 6 min ago" },
            Part::Named { role: Role::Link, name: "Changes" },
        ],
    },
    Holds {
        name: "home-last-looked-none",
        heading: "2 changes landed since you last looked. Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Thursday, 21:19" },
            Part::Text { text: "3 queued" },
            Part::Named { role: Role::Link, name: "Changes" },
        ],
    },
    Holds {
        name: "home-landing-none",
        heading: "Nothing needs you.",
        parts: &[Part::Text { text: "nothing has landed yet" }, Part::Named { role: Role::Link, name: "Changes" }],
    },
    Holds {
        name: "home-landing-red",
        heading: "2 changes landed since you last looked. Nothing needs you.",
        parts: &[
            Part::Text { text: "main last moved 1 h ago" },
            Part::Text { text: "nothing queued" },
            Part::Text { text: "main red" },
        ],
    },
    Holds {
        name: "home-chats-sentence",
        heading: "Nothing needs you.",
        parts: &[Part::Named {
            role: Role::Paragraph,
            name: "A chat suits one change. For bigger work, a chat can write a draft beside it and start a goal from it: a plan, its tasks, landing.",
        }],
    },
    Holds {
        name: "home-begin-both-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::StyleCount { style: temper_client_view::look::RAIL_SEP, count: 1 },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Named {
                role: Role::Paragraph,
                name: "Two steps left. Skip them if you like; they stay in How work runs and Ask temper.",
            },
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-begin-ask-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "One step left. Skip it if you like; it stays in Ask temper." },
            Part::Text { text: "temper can change its code now." },
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Named { role: Role::Link, name: "Ask temper" },
            Part::Text { text: "work/hello is in Hello" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-accounts-claude",
        heading: "Hello is ready.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Your agents run on your Claude subscription, and Forgejo on this machine.",
            },
            Part::Named { role: Role::Link, name: "Add ChatGPT or another plan" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-accounts-both",
        heading: "Hello is ready.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Your agents run on your ChatGPT plan, and Forgejo on this machine.",
            },
            Part::Named { role: Role::Link, name: "Add another account" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-accounts-key",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Your agents run on an API key, and Forgejo on this machine." },
            Part::Named { role: Role::Link, name: "Add another account" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-with-goals",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Link, name: "Work" },
            Part::Named { role: Role::Region, name: "Goals" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Goals" },
            Part::Named { role: Role::Link, name: "All 8" },
            Part::Goals {
                rows: &[
                    "CSV import · 1 / 3 · needs funds",
                    "JSON output for tally report · 0 / 2 · needs you",
                    "Make tally sum stream its input · 1 / 2 · running",
                    "CSV export · planning",
                    "Profile tally sum on a 2 GB file · 0 / 1 · paused by you",
                ],
            },
            Part::Absent { role: Role::Region, name: "Chats" },
            Part::Named { role: Role::Link, name: "Needs you 1" },
            Part::Named { role: Role::Button, name: "Tell it something else" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-at-work",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::Named { role: Role::Link, name: "Needs you 1" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 1" },
            Part::Named { role: Role::Paragraph, name: "The repository has no license. Name one in the README?" },
            Part::Named { role: Role::Button, name: "Tell it something else" },
            Part::Named { role: Role::Link, name: "All 3 chats" },
            Part::Named { role: Role::Link, name: "Add a README, waiting on you, 4 min" },
            Part::Named { role: Role::Link, name: "Hello in Rust, closed, 1 h" },
            Part::Named { role: Role::Link, name: "What is in this repository?, closed, 1 h" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-needs-several",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::Named { role: Role::Link, name: "Needs you 3" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Absent { role: Role::Link, name: "Answer in a row" },
            Part::Named { role: Role::Paragraph, name: "The repository has no license. Name one in the README?" },
            Part::Named { role: Role::Link, name: "All 3 chats" },
        ],
    },
    Holds {
        name: "home-nothing-needs",
        heading: "Nothing needs you.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::Absent { role: Role::Region, name: "Needs you" },
            Part::Absent {
                role: Role::Paragraph,
                name: "Work carries on, and this page tells you what changed when you come back.",
            },
            Part::Named { role: Role::Link, name: "All 1 chats" },
            Part::Named { role: Role::Link, name: "What is in this repository?, closed, 1 h" },
        ],
    },
    Holds {
        name: "home-ready",
        heading: "Nothing needs you.",
        parts: &[Part::Absent { role: Role::Heading { level: 2 }, name: "To begin" }],
    },
    Holds {
        name: "home-adopting",
        heading: "Hello is ready.",
        parts: &[
            Part::Text { text: "Adopting work/hello into Hello…" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-copying",
        heading: "Widget is ready.",
        parts: &[
            Part::Text { text: "Copying acme/widget from github.com…" },
            Part::Text { text: "Waiting to copy" },
            Part::Absent { role: Role::Button, name: "Copy again" },
        ],
    },
    Holds {
        name: "home-copy-failed",
        heading: "Widget is ready.",
        parts: &[
            Part::Text { text: "acme/widget is in Widget" },
            Part::Text { text: "acme/other failed at repository: name taken" },
            Part::Named { role: Role::Button, name: "Copy again" },
        ],
    },
    Holds {
        name: "home-copied",
        heading: "Widget is ready.",
        parts: &[
            Part::Text { text: "acme/widget, acme/other are in Widget" },
            Part::Text {
                text: "temper can change their code now, on your Forgejo; nothing goes back where they were copied from.",
            },
            Part::Absent { role: Role::Button, name: "Copy" },
        ],
    },
    Holds {
        name: "home-copy-one",
        heading: "Widget is ready.",
        parts: &[
            Part::Text { text: "acme/widget is in Widget" },
            Part::Text {
                text: "temper can change its code now, on your Forgejo; nothing goes back where it was copied from.",
            },
        ],
    },
    Holds {
        name: "home-copy-offer",
        heading: "Widget is ready.",
        parts: &[
            Part::Text { text: "Copy https://github.com/acme/widget" },
            Part::Text { text: "https://github.com/acme/other" },
            Part::Named { role: Role::Button, name: "Copy" },
        ],
    },
    Holds {
        name: "home-add-repository",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Button, name: "Make an empty one" },
            Part::Named { role: Role::Button, name: "Copy a public one" },
            Part::Named { role: Role::Link, name: "Adopt another…" },
            Part::NoRole { role: Role::Dialog },
        ],
    },
    Holds {
        name: "home-add-repository-sheet",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Add a repository to Hello" },
            Part::Named { role: Role::ChoiceGroup, name: "A repository" },
            Part::Value { role: Role::TextField, name: "Organization", value: "work" },
            Part::Value { role: Role::TextField, name: "Name", value: "hello" },
            Part::Named { role: Role::Button, name: "Cancel" },
            Part::Named { role: Role::Button, name: "Add it" },
        ],
    },
    Holds {
        name: "home-add-repository-copies",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Add a repository to Hello" },
            Part::Value { role: Role::TextArea, name: "Their addresses, one per line", value: "" },
            Part::Absent { role: Role::TextField, name: "Organization" },
            Part::Named { role: Role::Button, name: "Add it" },
        ],
    },
    Holds {
        name: "home-refused",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-no-repository",
        heading: "Hello is ready.",
        parts: &[
            Part::Text { text: "Your agents run on your ChatGPT plan." },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-after-skip",
        heading: "Nothing needs you.",
        parts: &[
            Part::Absent { role: Role::Heading { level: 2 }, name: "To begin" },
            Part::Absent { role: Role::Region, name: "Needs you" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Chats" },
            Part::Named { role: Role::Paragraph, name: "No chats yet." },
        ],
    },
    Holds {
        name: "home-skip-refused",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Alert, name: "temper couldn't keep that. Try Skip again." },
            Part::Named { role: Role::Button, name: "Skip" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-begin-with-needs",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
            Part::NoStyle { style: temper_client_view::look::HOME_NEEDS },
            Part::NoStyle { style: temper_client_view::look::HOME_CHATS },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Needs you · 1" },
            Part::Absent { role: Role::Link, name: "All 1 chats" },
        ],
    },
    Holds {
        name: "home-begin-settings-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::StyleCount { style: temper_client_view::look::RAIL_SEP, count: 1 },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Named {
                role: Role::Paragraph,
                name: "One step left. Skip it if you like; it stays in How work runs.",
            },
            Part::Absent { role: Role::Button, name: "Start" },
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Absent { role: Role::Link, name: "New chat" },
            Part::NoStyle { style: temper_client_view::look::HOME_CHATS },
        ],
    },
    Holds {
        name: "goals-solo",
        heading: "8 goals, 5 under way.",
        parts: &[Part::Goals {
            rows: &[
                "CSV import · 1 / 3 · needs funds",
                "JSON output for tally report · 0 / 2 · needs you",
                "Make tally sum stream its input · 1 / 2 · running",
                "CSV export · planning",
                "Profile tally sum on a 2 GB file · 0 / 1 · paused by you",
                "CSV import: the quoted-field parser · 2 / 2 · landed ",
                "The process documents, read aloud · 0 / 1 · closed",
                "CSV import: errors with line and column · 0 / 1 · failed",
            ],
        }],
    },
    Holds {
        name: "goals-reference",
        heading: "6 goals, 4 under way.",
        parts: &[
            Part::Goals {
                rows: &[
                    "first-run-projects · 6 / 9 · needs funds",
                    "welcome-stall · 1 / 3 · needs you",
                    "forge-cards · 0 / 4 · needs you",
                    "A hundred tasks · 96 / 112 · running",
                    "pane-chat-line · 2 / 2 · landed ",
                    "welcome-model-cards · 3 / 3 · landed Mon 18:40",
                ],
            },
            Part::Text { text: "Six of nine tasks landed; the store task is split in two after two size failures." },
            Part::Text { text: "96 of 112 tasks done." },
        ],
    },
    Holds {
        name: "goals-none",
        heading: "No goals yet.",
        parts: &[Part::Goals { rows: &[] }, Part::Text { text: "None." }],
    },
    Holds {
        name: "goals-needs",
        heading: "8 goals, 5 under way.",
        parts: &[Part::Goals {
            rows: &["CSV import · 1 / 3 · needs funds", "JSON output for tally report · 0 / 2 · needs you"],
        }],
    },
    Holds {
        name: "goals-under-way",
        heading: "8 goals, 5 under way.",
        parts: &[
            Part::Goals {
                rows: &[
                    "Make tally sum stream its input · 1 / 2 · running",
                    "CSV export · planning",
                    "Profile tally sum on a 2 GB file · 0 / 1 · paused by you",
                ],
            },
            Part::Text { text: "Planning from the draft." },
            Part::Text { text: "0 of 1 tasks done." },
        ],
    },
    Holds {
        name: "goals-ended",
        heading: "8 goals, 5 under way.",
        parts: &[
            Part::Goals {
                rows: &[
                    "CSV import: the quoted-field parser · 2 / 2 · landed ",
                    "The process documents, read aloud · 0 / 1 · closed",
                    "CSV import: errors with line and column · 0 / 1 · failed",
                ],
            },
            Part::Text { text: "Closed by you." },
            Part::Text { text: "Failed." },
        ],
    },
    Holds {
        name: "goals-filter-empty",
        heading: "3 goals, 0 under way.",
        parts: &[Part::Goals { rows: &[] }, Part::Text { text: "None." }],
    },
    Holds {
        name: "goals-more",
        heading: "More than 6 goals, 5 under way.",
        parts: &[
            Part::Goals {
                rows: &[
                    "CSV import · 1 / 3 · needs funds",
                    "JSON output for tally report · 0 / 2 · needs you",
                    "Make tally sum stream its input · 1 / 2 · running",
                    "CSV export · planning",
                    "Profile tally sum on a 2 GB file · 0 / 1 · paused by you",
                    "CSV import: the quoted-field parser · 2 / 2 · landed ",
                ],
            },
            Part::Named { role: Role::Button, name: "More goals" },
        ],
    },
    Holds {
        name: "home-github",
        heading: "Hello is ready.",
        parts: &[
            Part::Text { text: "temper’s App reaches its repositories on sam." },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-morning-card",
        heading: "Three things need you.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::Named {
                role: Role::Paragraph,
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Button, name: "In jig’s store" },
            Part::Absent { role: Role::Disclosure, name: "What the agent saw" },
            Part::Named { role: Role::Button, name: "Tell it something else" },
            Part::Absent { role: Role::Button, name: "Later" },
        ],
    },
    Holds {
        name: "home-answered",
        heading: "Two things need you.",
        parts: &[
            Part::Named { role: Role::Text, name: "In jig’s store" },
            Part::Named { role: Role::Button, name: "Undo" },
            Part::Named {
                role: Role::Paragraph,
                name: "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?",
            },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 2" },
        ],
    },
    Holds {
        name: "home-one-need",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Ask temper" },
            Part::Named { role: Role::Button, name: "MIT" },
            Part::Absent { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Button, name: "More" },
            Part::Absent { role: Role::Link, name: "Answer in a row" },
        ],
    },
    Holds {
        name: "welcome-model-claude-terminal",
        heading: "Connect a model",
        parts: &[
            Part::Ordered { role: Role::Heading { level: 2 }, name: "Claude" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Your Claude subscription, through a Claude Code temper keeps apart from your own.",
            },
            Part::Ordered { role: Role::Paragraph, name: "Signs in from the terminal where temper runs" },
            Part::Ordered { role: Role::Button, name: "Sign in to Claude Code" },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "ChatGPT" },
            Part::Ordered { role: Role::Paragraph, name: "Sign in, in a new tab" },
            Part::Ordered { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Absent { role: Role::Link, name: "Use your Claude subscription" },
        ],
    },
    Holds {
        name: "welcome-model-claude-signing-in",
        heading: "Connect a model",
        parts: &[
            Part::Ordered { role: Role::Status, name: "Claude Code is signing in, in the terminal where temper runs" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Approve it on Anthropic’s page, in the tab Claude Code opened. This page moves on by itself.",
            },
            Part::Ordered { role: Role::Button, name: "Cancel" },
            Part::Ordered { role: Role::Disclosure, name: "No tab opened, or a code to paste?" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Claude Code’s sign-in is in the terminal where temper runs. If no tab opened, press c there to copy its link. If Anthropic’s page shows a code, paste it there.",
            },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Folded { name: "No tab opened, or a code to paste?" },
            Part::StyleCount { style: temper_client_view::look::WELCOME_SPIN, count: 1 },
            Part::Absent { role: Role::Paragraph, name: "temper’s agents run on a plan you sign in to." },
            Part::Absent { role: Role::Region, name: "Claude" },
        ],
    },
    Holds {
        name: "welcome-model-claude-signed-in",
        heading: "Connect a model",
        parts: &[
            Part::Ordered { role: Role::Status, name: "Claude subscription · through Claude Code, signed in" },
            Part::Text { text: "✓" },
            Part::StyleCount { style: temper_client_view::look::WELCOME_OK, count: 1 },
            Part::Ordered { role: Role::Link, name: "Continue" },
            Part::Ordered { role: Role::Complementary, name: "Forge and model" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Ordered { role: Role::Paragraph, name: "Claude subscription · through Claude Code, signed in" },
            Part::Absent { role: Role::Region, name: "Claude" },
        ],
    },
    Holds {
        name: "welcome-model-claude-no-terminal",
        heading: "Connect a model",
        parts: &[
            Part::Ordered { role: Role::Heading { level: 2 }, name: "Claude" },
            Part::Ordered { role: Role::Paragraph, name: "One command in a terminal, on this machine" },
            Part::Ordered { role: Role::Link, name: "Use your Claude subscription" },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "ChatGPT" },
            Part::Ordered { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Absent { role: Role::Button, name: "Sign in to Claude Code" },
        ],
    },
    Holds {
        name: "welcome-model-claude-not-finished",
        heading: "Connect a model",
        parts: &[
            Part::Ordered { role: Role::Paragraph, name: "temper’s agents run on a plan you sign in to." },
            Part::Ordered { role: Role::Paragraph, name: "Claude Code’s sign-in didn’t finish." },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "Claude" },
            Part::Ordered { role: Role::Paragraph, name: "Signs in from the terminal where temper runs" },
            Part::Ordered { role: Role::Button, name: "Sign in to Claude Code" },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "ChatGPT" },
            Part::Ordered { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Ordered {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
        ],
    },
    Holds {
        name: "welcome-model-forge-under-way",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Progress, name: "Setting up: preparing" },
            Part::Named { role: Role::Complementary, name: "Forge and model" },
            Part::Named {
                role: Role::Paragraph, name: "Forgejo on this machine · work/hello · Setting up: preparing"
            },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
        ],
    },
    Holds {
        name: "welcome-model-forge-fetching",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Progress, name: "Fetching Forgejo" },
            Part::Named { role: Role::Complementary, name: "Forge and model" },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · work/hello · Setting up: fetching Forgejo · 42%",
            },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Text { text: "42%" },
            Part::Text { text: "Fetching Forgejo 17.0.0 and its runner 13.2.0 · 42%" },
        ],
    },
    Holds {
        name: "welcome-model-forge-fetching-bytes",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Progress, name: "Fetching Forgejo" },
            Part::Named { role: Role::Complementary, name: "Forge and model" },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · work/hello · Setting up: fetching Forgejo · 41 MB",
            },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Text { text: "41 MB" },
            Part::Text { text: "Fetching Forgejo 17.0.0 and its runner 13.2.0 · 41 MB" },
        ],
    },
    Holds {
        name: "welcome-model-forge-done",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::NoRole { role: Role::Progress },
            Part::Named { role: Role::Complementary, name: "Forge and model" },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Absent { role: Role::Disclosure, name: "Each step" },
        ],
    },
    Holds {
        name: "welcome-model-forge-failed",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::NoRole { role: Role::Progress },
            Part::Named { role: Role::Complementary, name: "Forge and model" },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · work/hello · Set up failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
            },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
        ],
    },
    Holds {
        name: "playground-resetting",
        heading: "Talk to an agent",
        parts: &[Part::Text { text: "Leaving the playground…" }],
    },
    Holds {
        name: "chats-first-run-5",
        heading: "Talk to an agent",
        parts: &[
            Part::Absent { role: Role::List, name: "Suggested requests" },
            Part::Named { role: Role::Region, name: "Playground" },
            Part::Absent { role: Role::Link, name: "Manage usage" },
        ],
    },
    Holds {
        name: "welcome-paste-expired",
        heading: "Signing in to ChatGPT",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "This sign-in has ended." },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Absent { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-model-declined",
        heading: "Signing in to ChatGPT",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "welcome-model-renewed",
        heading: "Connect a model",
        parts: &[Part::WelcomeFrame { box_note: false }, Part::Named { role: Role::Link, name: "Continue" }],
    },
    Holds {
        name: "welcome-model-refused",
        heading: "Signing in to ChatGPT",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "welcome-paste-waiting",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-written",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-too-long",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-sending",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-finishing",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "Finishing your sign-in…" },
        ],
    },
    Holds {
        name: "welcome-paste-address",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-state",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-too-many",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "welcome-paste-ended",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "welcome-paste-lost",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-paste-unsent",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "forges-github-install",
        heading: "Where your code lives.",
        parts: &[Part::Named { role: Role::Button, name: "Install on GitHub" }],
    },
    Holds {
        name: "forge-admin-details",
        heading: "Your Forgejo",
        parts: &[
            Part::Named { role: Role::TextField, name: "First repository" },
            Part::Named { role: Role::Disclosure, name: "Its runner label: docker" },
            Part::Named { role: Role::TextField, name: "Runner label" },
        ],
    },
    Holds {
        name: "forge-setup-failed",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Disclosure, name: "Forgejo on this machine" },
            Part::Named { role: Role::Button, name: "Try again" },
        ],
    },
    Holds {
        name: "welcome-setup-failed",
        heading: "Almost ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "failed" },
            Part::Named { role: Role::List, name: "Each step" },
            Part::Named { role: Role::Button, name: "Set up again" },
        ],
    },
    Holds {
        name: "welcome-model-claude",
        heading: "Your Claude subscription",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Text { text: "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat" },
            Part::Named { role: Role::Button, name: "I’ve signed in" },
            Part::Absent { role: Role::TextField, name: "Name" },
        ],
    },
    Holds {
        name: "welcome-ready-checking",
        heading: "Ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named {
                role: Role::Paragraph, name: "Claude subscription · trying: Signed in to your subscription"
            },
            Part::Absent { role: Role::Button, name: "I’ve signed in" },
        ],
    },
    Holds {
        name: "welcome-ready-not-yet",
        heading: "Almost ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Paragraph, name: "Claude subscription · Claude didn’t answer" },
            Part::Named {
                role: Role::Paragraph,
                name: "Claude Code isn’t signed in here yet. Sign in again, then try again.",
            },
            Part::Named { role: Role::Button, name: "Try again" },
            Part::Named { role: Role::Link, name: "Sign in again" },
            Part::Absent { role: Role::Button, name: "I’ve signed in" },
        ],
    },
    Holds {
        name: "welcome-model-key-details",
        heading: "An OpenAI API key",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::TextField, name: "Key, pasted once" },
        ],
    },
    Holds {
        name: "welcome-model-connected",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "ChatGPT plan · sam@example.com" },
            Part::Named { role: Role::Link, name: "Continue" },
        ],
    },
    Holds {
        name: "welcome-model-away",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "welcome-model-signing-in",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "welcome-model-signin-ready",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Link, name: "Continue" },
            Part::Absent { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "github-app-details",
        heading: "Connect a forge",
        parts: &[Part::Named { role: Role::Button, name: "GitHub" }],
    },
    Holds {
        name: "github-install",
        heading: "Where your code lives.",
        parts: &[Part::Named { role: Role::Button, name: "Install on GitHub" }],
    },
    Holds {
        name: "github-picker",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Connected GitHub." },
            Part::Named { role: Role::Link, name: "Adopt a repository" },
        ],
    },
    Holds {
        name: "github-adopted",
        heading: "Where your code lives.",
        parts: &[Part::Named { role: Role::Link, name: "Adopt a repository" }],
    },
    Holds {
        name: "welcome-forge-choose",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Region, name: "Forgejo" },
            Part::Named { role: Role::Link, name: "Run on this machine" },
            Part::Named { role: Role::Region, name: "GitHub" },
            Part::Named { role: Role::Button, name: "Connect GitHub" },
        ],
    },
    Holds {
        name: "welcome-forge-details",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Dialog, name: "Forgejo on this machine" },
            Part::Named { role: Role::Choice, name: "An empty repository" },
            Part::Named { role: Role::TextField, name: "Organization" },
            Part::Named { role: Role::TextField, name: "Name" },
            Part::Named { role: Role::Button, name: "Run on this machine" },
        ],
    },
    Holds {
        name: "welcome-sheet-copies",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Dialog, name: "Forgejo on this machine" },
            Part::Named { role: Role::Choice, name: "Copies of public repositories" },
            Part::Named { role: Role::TextArea, name: "Their addresses, one per line" },
            Part::Named { role: Role::Button, name: "Run on this machine" },
        ],
    },
    Holds {
        name: "welcome-sheet-copies-refused",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Dialog, name: "Forgejo on this machine" },
            Part::Named { role: Role::Choice, name: "Copies of public repositories" },
            Part::Named {
                role: Role::Alert,
                name: "Not an address: not an address. Write it as github.com/owner/name.",
            },
            Part::Named { role: Role::Button, name: "Run on this machine" },
        ],
    },
    Holds {
        name: "welcome-sheet-nothing",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Dialog, name: "Forgejo on this machine" },
            Part::Named { role: Role::Choice, name: "Nothing for now" },
            Part::Named { role: Role::Button, name: "Run on this machine" },
        ],
    },
    Holds {
        name: "welcome-forge-under-way",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Paragraph, name: "Forgejo on this machine · Setting up: preparing" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Named { role: Role::Link, name: "Continue" },
            Part::Absent { role: Role::Link, name: "View" },
        ],
    },
    Holds {
        name: "welcome-forge-connected",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Named { role: Role::Link, name: "Continue" },
            Part::Absent { role: Role::Disclosure, name: "Each step" },
            Part::Absent { role: Role::Link, name: "View" },
        ],
    },
    Holds {
        name: "welcome-forge-github",
        heading: "Connect a forge",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Paragraph, name: "GitHub · temper’s App on sam" },
            Part::Named { role: Role::Link, name: "Continue" },
            Part::Absent { role: Role::Disclosure, name: "Each step" },
            Part::Absent { role: Role::Link, name: "View" },
        ],
    },
    Holds {
        name: "welcome-model-choose",
        heading: "Connect a model",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "Claude" },
            Part::Ordered { role: Role::Paragraph, name: "One command in a terminal, on this machine" },
            Part::Ordered { role: Role::Link, name: "Use your Claude subscription" },
            Part::Ordered { role: Role::Heading { level: 2 }, name: "ChatGPT" },
            Part::Ordered { role: Role::Paragraph, name: "Sign in, in a new tab" },
            Part::Ordered { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Absent { role: Role::Button, name: "Sign in to Claude Code" },
        ],
    },
    Holds {
        name: "forge-details",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::TextField, name: "Organization" },
            Part::Named { role: Role::TextField, name: "Name" },
            Part::Absent { role: Role::TextField, name: "Into a new project, named" },
        ],
    },
    Holds {
        name: "forge-setting-up-middle",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Disclosure, name: "Forgejo on this machine" },
            Part::Text { text: "Preparing: its database, temper’s admin, your account" },
        ],
    },
    Holds {
        name: "welcome-ready-copies",
        heading: "Ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named {
                role: Role::Paragraph,
                name: "Everything is connected. Create your first project, with 2 repositories in it, and start working.",
            },
            Part::Named { role: Role::Paragraph, name: "Forgejo on this machine · connected as temper" },
            Part::Named { role: Role::TextField, name: "Your first project" },
            Part::Named { role: Role::Button, name: "Create your project" },
        ],
    },
    Holds {
        name: "welcome-ready-none",
        heading: "Almost ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Slot, name: "Forge none yet" },
            Part::Named { role: Role::Link, name: "Connect a forge" },
            Part::Named { role: Role::Slot, name: "Model none yet" },
            Part::Named { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::TextField, name: "Your first project" },
            Part::Named { role: Role::Button, name: "Create your project" },
            Part::Named { role: Role::Link, name: "‹ Back" },
            Part::Named { role: Role::Button, name: "Try the playground instead" },
        ],
    },
    Holds {
        name: "welcome-ready-signing",
        heading: "Almost ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named { role: Role::Slot, name: "Forge none yet" },
            Part::Named { role: Role::Paragraph, name: "Model ChatGPT plan · signing in…" },
            Part::Absent { role: Role::Link, name: "Connect a model" },
            Part::Named { role: Role::TextField, name: "Your first project" },
            Part::Named { role: Role::Link, name: "‹ Back" },
        ],
    },
    Holds {
        name: "welcome-ready-under-way",
        heading: "Almost ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named {
                role: Role::Paragraph, name: "Forgejo on this machine · work/hello · Setting up: preparing"
            },
            Part::Named { role: Role::Paragraph, name: "ChatGPT plan · sam@example.com" },
            Part::Named { role: Role::Paragraph, name: "Forgejo is still setting up; this page moves on by itself." },
            Part::Named { role: Role::Link, name: "‹ Back" },
            Part::Absent { role: Role::Button, name: "Try the playground instead" },
            Part::Named {
                role: Role::Paragraph,
                name: "Your first project can be created once your forge and model are ready.",
            },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Named { role: Role::TextField, name: "Your first project" },
            Part::Named { role: Role::Button, name: "Create your project" },
            Part::Named { role: Role::Paragraph, name: "Rename it whenever you like." },
        ],
    },
    Holds {
        name: "welcome-ready-adopted",
        heading: "Ready",
        parts: &[
            Part::WelcomeFrame { box_note: false },
            Part::Named {
                role: Role::Paragraph,
                name: "Everything is connected. Create your first project and start working.",
            },
            Part::Named {
                role: Role::Paragraph,
                name: "Forgejo on this machine · connected as temper · work/hello made",
            },
            Part::Named { role: Role::Paragraph, name: "ChatGPT plan · sam@example.com" },
            Part::Named { role: Role::TextField, name: "Your first project" },
            Part::Named { role: Role::Button, name: "Create your project" },
            Part::Absent { role: Role::Disclosure, name: "Each step" },
            Part::Named { role: Role::Link, name: "‹ Back" },
            Part::Named { role: Role::Link, name: "Open Forgejo ↗" },
            Part::Absent { role: Role::Button, name: "Try the playground instead" },
            Part::Absent { role: Role::Paragraph, name: "Forgejo is still setting up; this page moves on by itself." },
        ],
    },
    Holds {
        name: "connect-forge-first-run-6",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Disclosure, name: "Forgejo on this machine" },
            Part::Text { text: "Preparing: its database, temper’s admin, your account" },
        ],
    },
    Holds {
        name: "forges-first-run-7",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your forges · 1" },
            Part::Described {
                role: Role::Disclosure,
                name: "Forgejo on this machine",
                description: "Forge 3 · Forgejo · Connected as temper\nwork/hello",
            },
            Part::Absent { role: Role::Heading { level: 2 }, name: "The playground" },
        ],
    },
    Holds {
        name: "connect-forge-choose",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Region, name: "GitHub" },
            Part::Named { role: Role::Region, name: "Forgejo" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Or a server of your own" },
            Part::Named { role: Role::Link, name: "Connect your Forgejo" },
            Part::Absent { role: Role::List, name: "Steps" },
        ],
    },
    Holds {
        name: "connect-forge-connected",
        heading: "Connect a forge",
        parts: &[
            Part::Text { text: "✓ Connected" },
            Part::Named { role: Role::Link, name: "Open Forgejo ↗" },
            Part::Named { role: Role::Button, name: "Connect GitHub" },
            Part::Absent { role: Role::List, name: "Steps" },
        ],
    },
    Holds {
        name: "connect-forge-elsewhere",
        heading: "A Forgejo elsewhere",
        parts: &[
            Part::Named { role: Role::Slot, name: "A Forgejo elsewhere" },
            Part::Named { role: Role::Button, name: "Copy" },
            Part::Named { role: Role::Link, name: "Other forges" },
            Part::Absent { role: Role::List, name: "Steps" },
        ],
    },
    Holds {
        name: "connect-forge-yours-admin",
        heading: "Your Forgejo",
        parts: &[
            Part::Named { role: Role::Choice, name: "As its admin" },
            Part::Named { role: Role::TextField, name: "The site admin’s token" },
            Part::Named { role: Role::Disclosure, name: "Its runner label: docker" },
            Part::Absent { role: Role::TextField, name: "API token" },
            Part::Named { role: Role::Link, name: "Other forges" },
        ],
    },
    Holds {
        name: "connect-forge-yours-tokens",
        heading: "Your Forgejo",
        parts: &[
            Part::Named { role: Role::Choice, name: "With its bot’s tokens" },
            Part::Named { role: Role::TextField, name: "API token" },
            Part::Named { role: Role::Disclosure, name: "A webhook secret, if you made hooks to temper’s listener" },
            Part::Absent { role: Role::TextField, name: "The site admin’s token" },
            Part::Named { role: Role::Link, name: "Other forges" },
        ],
    },
    Holds {
        name: "welcome-go-on",
        heading: "Welcome to temper",
        parts: &[
            Part::WelcomeFrame { box_note: true },
            Part::Named { role: Role::List, name: "Steps" },
            Part::Named { role: Role::Region, name: "Set up temper" },
            Part::Named { role: Role::Link, name: "Go on setting up" },
            Part::Absent { role: Role::Region, name: "Try the playground" },
            Part::Absent {
                role: Role::Paragraph,
                name: "Tried the playground first? Set up whenever you like; the playground then steps aside.",
            },
        ],
    },
    Holds {
        name: "welcome-first-run-0",
        heading: "Welcome to temper",
        parts: &[
            Part::WelcomeFrame { box_note: true },
            Part::Named { role: Role::List, name: "Steps" },
            Part::Named { role: Role::Region, name: "Try the playground" },
            Part::Named { role: Role::Button, name: "Try the playground" },
            Part::Named { role: Role::Region, name: "Set up temper" },
            Part::Named { role: Role::Link, name: "Set up temper" },
        ],
    },
    Holds {
        name: "ask-first-run",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::StyleCount { style: temper_client_view::look::RAIL_SEP, count: 1 },
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Named { role: Role::Region, name: "Ways of working" },
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 1 },
            Part::Named { role: Role::Group, name: "Agents review" },
            Part::Named { role: Role::Button, name: "Keep it" },
            Part::Configured { value: false },
            Part::Reader { role: jig_client::contract::HeldRole::Owner },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Ways to begin" },
            Part::Path {
                stops: &[
                    "you ask, a person",
                    "produce, an agent",
                    "check, code",
                    "review, an agent",
                    "repair, an agent",
                    "check, code",
                    "conflict, code",
                    "resolve, an agent",
                    "merge, code",
                    "main, a branch",
                ],
            },
            Part::Value { role: Role::TextArea, name: "Ask temper, or tell it how you work", value: "" },
        ],
    },
    Holds {
        name: "ask-ways-approve",
        heading: "Ask temper",
        parts: &[
            ASK_USE,
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 0 },
            Part::Named { role: Role::Group, name: "You approve" },
            Part::Text { text: "An agent makes each change; nothing reaches main until you approve it." },
            Part::Path {
                stops: &[
                    "you ask, a person",
                    "produce, an agent",
                    "check, code",
                    "repair, an agent",
                    "check, code",
                    "you approve, a person",
                    "conflict, code",
                    "resolve, an agent",
                    "merge, code",
                    "main, a branch",
                ],
            },
            Part::Named { role: Role::Button, name: "Use this" },
            Part::Absent { role: Role::Button, name: "Keep it" },
        ],
    },
    Holds {
        name: "ask-ways-draft",
        heading: "Ask temper",
        parts: &[
            ASK_USE,
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 2 },
            Part::Named { role: Role::Group, name: "Draft first" },
            Part::Text {
                text: "You and an agent write a design draft; agents build it on its own branch, and it lands whole.",
            },
            Part::Path {
                stops: &[
                    "draft, a person",
                    "plan, an agent",
                    "produce ×n, an agent",
                    "merge, code",
                    "check, code",
                    "repair, an agent",
                    "goal branch, a branch",
                    "check, code",
                    "conflict, code",
                    "resolve, an agent",
                    "merge, code",
                    "verify, code",
                    "main, a branch",
                ],
            },
            Part::Drawing { style: temper_client_view::look::WAYS_MAIN, count: 2 },
            Part::Drawing { style: temper_client_view::look::WAYS_LANDING, count: 1 },
            Part::Named { role: Role::Button, name: "Use this" },
        ],
    },
    Holds {
        name: "ask-ways-card",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::Named { role: Role::Group, name: "Agents review" },
            Part::Named { role: Role::Region, name: "Review" },
            Part::Expanded { name: "review, an agent" },
            Part::Drawing { style: temper_client_view::look::WAYS_CARET, count: 1 },
            Part::Drawing { style: temper_client_view::look::WAYS_HALO, count: 1 },
            Part::Text { text: "an agent · holds a change until it approves" },
            Part::Named { role: Role::Link, name: "Open the reviewer ›" },
            Part::Named { role: Role::Term, name: "Model" },
            Part::Named { role: Role::Description, name: "Claude Opus 5.5 · high" },
            Part::Named { role: Role::Term, name: "Can" },
            Part::Named { role: Role::Description, name: "read the change and its checks; never edit, push or merge" },
            Part::Named { role: Role::Term, name: "Asked to" },
            Part::Named {
                role: Role::Description,
                name: "“Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.”",
            },
        ],
    },
    Holds {
        name: "ask-ways-card-person",
        heading: "Ask temper",
        parts: &[
            ASK_USE,
            Part::Named { role: Role::Group, name: "You approve" },
            Part::Named { role: Role::Region, name: "You approve" },
            Part::Expanded { name: "you approve, a person" },
            Part::Text { text: "a person · decides whether it lands" },
            Part::Named { role: Role::Term, name: "Role" },
            Part::Named { role: Role::Description, name: "owner" },
            Part::Named { role: Role::Term, name: "Asks" },
            Part::Named { role: Role::Description, name: "1 of them" },
        ],
    },
    Holds {
        name: "ask-ways-yours",
        heading: "Ask temper",
        parts: &[
            ASK_ALONE,
            Part::Tabs { names: &["You approve", "Agents review", "Draft first", "Yoursnow"], selected: 3 },
            Part::Named { role: Role::Group, name: "Yours" },
            Part::Path { stops: &["you ask, a person", "produce, an agent", "merge, code", "main, a branch"] },
            Part::Absent { role: Role::Button, name: "Use this" },
            Part::Absent { role: Role::Button, name: "Keep it" },
        ],
    },
    Holds {
        name: "ask-ways-refused",
        heading: "Ask temper",
        parts: &[
            ASK_USE,
            Part::Named { role: Role::Group, name: "You approve" },
            Part::Named { role: Role::Button, name: "Use this" },
            Part::Named { role: Role::Alert, name: "" },
            Part::Text {
                text: "refused by its owner: project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible",
            },
            Part::After { first: temper_client_view::look::WAYS_FOOT, then: temper_client_view::look::WAYS_REFUSED },
            Part::Configured { value: false },
        ],
    },
    Holds {
        name: "ask-ways-member",
        heading: "Ask temper",
        parts: &[
            ASK_ALONE,
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 1 },
            Part::Named { role: Role::Group, name: "Agents review" },
            Part::Named { role: Role::List, name: "The way a change goes" },
            Part::Reader { role: jig_client::contract::HeldRole::Member },
            Part::Absent { role: Role::Button, name: "Use this" },
            Part::Absent { role: Role::Button, name: "Keep it" },
        ],
    },
    Holds {
        name: "ask-ways-kept",
        heading: "Ask temper",
        parts: &[
            ASK_ALONE,
            Part::Text { text: "Agents review is now this project's own, as revision 1." },
            Part::Tabs { names: &["You approve", "Agents reviewnow", "Draft first"], selected: 1 },
            Part::Named { role: Role::Group, name: "Agents review" },
            Part::Absent { role: Role::Button, name: "Use this" },
            Part::Absent { role: Role::Button, name: "Keep it" },
            Part::Configured { value: true },
        ],
    },
    Holds {
        name: "ask-typing",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Named { role: Role::Region, name: "Ways of working" },
            Part::Value {
                role: Role::TextArea,
                name: "Ask temper, or tell it how you work",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "ask-sending",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Named { role: Role::Region, name: "Ways of working" },
            Part::Value {
                role: Role::TextArea,
                name: "Ask temper, or tell it how you work",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "ask-refused",
        heading: "Ask temper",
        parts: &[
            ASK_KEEP,
            Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper", "How work runs"] },
            Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Deployment"] },
            Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
            Part::Absent { role: Role::Menu, name: "New draft or change" },
            Part::Named { role: Role::Region, name: "Ways of working" },
            Part::Value {
                role: Role::TextArea,
                name: "Ask temper, or tell it how you work",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "ask-chat-document",
        heading: "T10 · Ask temper",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "T10" },
            Part::Named { role: Role::Button, name: "Start over" },
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Absent { role: Role::Region, name: "How Hello works" },
        ],
    },
    Holds {
        name: "ask-chat-document-split",
        heading: "T10 · Ask temper",
        parts: &[
            Part::Named { role: Role::Button, name: "Start over" },
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Named { role: Role::Region, name: "How Hello works" },
        ],
    },
    Holds {
        name: "chat-draft-chat",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Named { role: Role::Button, name: "Show" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Paragraph, name: "Ctrl+\\ switches the draft" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Away mode" },
        ],
    },
    Holds {
        name: "chat-start-goal",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Start goal…" },
            Part::Named { role: Role::Dialog, name: "Start a goal from away-mode.md" },
            Part::Value { role: Role::TextField, name: "Name", value: "away-mode" },
            Part::Named { role: Role::Button, name: "Start goal" },
        ],
    },
    Holds {
        name: "chat-start-goal-refused",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start a goal from away-mode.md" },
            Part::Text { text: "This act is not offered" },
            Part::Named { role: Role::Button, name: "Start goal" },
        ],
    },
    Holds {
        name: "chat-goal-linked",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Link, name: "Goal Away mode →" },
            Part::Named { role: Role::Paragraph, name: "Started goal" },
            Part::Named { role: Role::Link, name: "Open" },
            Part::Absent { role: Role::Button, name: "Start goal…" },
        ],
    },
    Holds {
        name: "chat-draft-split",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Named { role: Role::Button, name: "Show" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Paragraph, name: "Ctrl+\\ switches the draft" },
            Part::Named { role: Role::Region, name: "Away mode" },
        ],
    },
    Holds {
        name: "chat-draft-draft",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Button, name: "Chat" },
            Part::Named { role: Role::Button, name: "Split" },
            Part::Named { role: Role::Button, name: "Draft" },
            Part::Named { role: Role::Button, name: "Show" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Paragraph, name: "Ctrl+\\ switches the draft" },
            Part::Named { role: Role::Region, name: "Away mode" },
        ],
    },
    Holds {
        name: "ask-chat",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Navigation, name: "Where" },
            Part::Links { style: temper_client_view::look::CRUMBS, names: &["Ask temper", "Ask temper"] },
            Part::Links { style: temper_client_view::look::TRAIL, names: &["Ask temper"] },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Button, name: "Close" },
            Part::Absent { role: Role::Disclosure, name: "Model" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "ask-chat-proposal-deployment",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run your projects" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "ask-chat-proposal-two-projects",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally and 2" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "ask-chat-proposal",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Named { role: Role::Heading { level: 5 }, name: "temper enforces" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: asked of me too" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "ask-chat-accepted",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: asked of me too" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "✓ Accepted · new work follows it" },
            Part::Absent { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "ask-chat-rejected",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: asked of me too" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Rejected · nothing changes" },
            Part::Absent { role: Role::Paragraph, name: "Waiting for an owner" },
            Part::Absent { role: Role::Button, name: "Accept" },
            Part::Absent { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "ask-chat-withdrawn",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: asked of me too" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Withdrawn · nothing changes" },
            Part::Absent { role: Role::Paragraph, name: "Waiting for an owner" },
            Part::Absent { role: Role::Button, name: "Accept" },
            Part::Absent { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "ask-chat-replaced",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: asked of me too" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Replaced by the proposal below" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "ask-chats-tag",
        heading: "Talk to an agent",
        parts: &[Part::Text { text: "Ask temper" }, Part::Chats { live: SOLO_LIVE, closed: SOLO_CLOSED }],
    },
    Holds {
        name: "chat-new",
        heading: "New chat",
        parts: &[
            Part::Text { text: "Say what you want; the agent works in work/hello." },
            Part::Named { role: Role::TextArea, name: "What do you want done?" },
            Part::Named { role: Role::Button, name: "Send" },
            Part::Absent { role: Role::Button, name: "Start chat" },
        ],
    },
    Holds {
        name: "chat-new-no-repository",
        heading: "New chat",
        parts: &[
            Part::Text {
                text: "Say what you want. Hello has no repository yet, so the agent can talk and plan but not change code.",
            },
            Part::Named { role: Role::TextArea, name: "What do you want done?" },
        ],
    },
    Holds {
        name: "chats-no-repository",
        heading: "Talk to an agent.",
        parts: &[
            Part::Text { text: "No repository yet" },
            Part::Text { text: "Chats in Hello talk and plan, but change no code until a repository is in." },
            Part::Named { role: Role::Link, name: "Adopt a repository" },
        ],
    },
    Holds {
        name: "chats-drafts",
        heading: "Talk to an agent",
        parts: &[
            Part::Named { role: Role::Button, name: "Drafts" },
            Part::Text { text: "writes away-mode.md" },
            Part::Named { role: Role::Link, name: "Away mode" },
            Part::Chats { live: 1, closed: 1 },
        ],
    },
    Holds { name: "chats-live", heading: "Talk to an agent", parts: &[Part::Chats { live: SOLO_LIVE, closed: 0 }] },
    Holds { name: "chats-closed", heading: "Talk to an agent", parts: &[Part::Chats { live: 0, closed: SOLO_CLOSED }] },
    Holds {
        name: "chats-solo",
        heading: "Talk to an agent",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Why is tally sum slow on big files?" },
            Part::Chats { live: SOLO_LIVE, closed: SOLO_CLOSED },
        ],
    },
    Holds {
        name: "chat-solo-T10",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Heading { level: 2 }, name: "JSON output for tally report" },
            Part::Named { role: Role::Disclosure, name: "Read 2 files" },
        ],
    },
    Holds {
        name: "inbox-solo",
        heading: "It changes report’s output and its tests, and lands into main: more than a chat does alone.",
        parts: &[Part::Named { role: Role::Button, name: "Open" }, Part::Absent { role: Role::Button, name: "Answer" }],
    },
    Holds {
        name: "repositories",
        heading: "What tally works on.",
        parts: &[
            Part::Named { role: Role::Navigation, name: "How work runs" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Repositories · 1" },
            Part::Named { role: Role::Text, name: "robin/tally" },
            Part::Named { role: Role::Link, name: "Add repositories" },
        ],
    },
    Holds {
        name: "repositories-making",
        heading: "What Hello works on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Repositories · 0" },
            Part::Named { role: Role::Text, name: "acme/other" },
            Part::Named { role: Role::Text, name: "acme/widget" },
            Part::Named { role: Role::Text, name: "octo/vsr" },
            Part::Text { text: "Forgejo said: “You can not import from disallowed hosts.”" },
            Part::Named { role: Role::Button, name: "Copy again" },
            Part::Named { role: Role::Button, name: "Dismiss" },
        ],
    },
    Holds {
        name: "repositories-none",
        heading: "What gallery works on.",
        parts: &[
            Part::Named { role: Role::Text, name: "No repository yet" },
            Part::Text { text: "Chats in gallery talk and plan, but change no code until a repository is in." },
            Part::Named { role: Role::Link, name: "Add repositories" },
        ],
    },
    Holds {
        name: "forges-in-settings",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Navigation, name: "Deployment" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Your forges · 1" },
        ],
    },
    Holds {
        name: "accounts-in-settings",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 1" },
            Part::Named { role: Role::Navigation, name: "Deployment" },
            Part::Named { role: Role::Text, name: "Signed in as Robin" },
            Part::Named { role: Role::Button, name: "Sign out" },
        ],
    },
    Holds {
        name: "settings",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Ways of working" },
            Part::Absent { role: Role::Check, name: "Away switch" },
            Part::Links { style: temper_client_view::look::SETTINGS_TABS, names: &["How work runs", "Repositories"] },
            Part::Absent { role: Role::Link, name: "Start" },
            Part::Named { role: Role::Navigation, name: "How work runs" },
        ],
    },
    Holds {
        name: "settings-drafts-off",
        heading: "How work runs in tally.",
        parts: &[Part::Named { role: Role::Check, name: "Chats can write drafts" }],
    },
    Holds {
        name: "settings-goals-off",
        heading: "How work runs in tally.",
        parts: &[Part::Named { role: Role::Check, name: "Goals" }],
    },
    Holds {
        name: "settings-changes-off",
        heading: "How work runs in tally.",
        parts: &[Part::Named { role: Role::Check, name: "Changes" }],
    },
    Holds {
        name: "settings-all-hidden",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What the project shows" },
            Part::Named { role: Role::Check, name: "Chats can write drafts" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "Ways of working" },
        ],
    },
    Holds {
        name: "settings-goals",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What the project shows" },
            Part::Named { role: Role::Check, name: "Chats can write drafts" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Ways of working" },
            Part::Named { role: Role::Choice, name: "The default" },
        ],
    },
    Holds {
        name: "settings-chat-open",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What the project shows" },
            Part::Named { role: Role::Check, name: "Chats can write drafts" },
            Part::Named { role: Role::Link, name: "Ask temper" },
            Part::Absent { role: Role::Link, name: "Go on" },
            Part::Absent { role: Role::Link, name: "Start" },
        ],
    },
    Holds {
        name: "settings-refused",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "What the project shows" },
            Part::Named { role: Role::Check, name: "Chats can write drafts" },
            Part::Named { role: Role::Status, name: "This act is not offered" },
        ],
    },
    Holds {
        name: "task-solo-T3",
        heading: "0 of 2 tasks done.",
        parts: &[
            Part::Text { text: "CSV import" },
            Part::Text { text: "needs you" },
            Part::Text {
                text: "The old parser accepts a trailing comma as an empty last field. RFC 4180 does not. Keep it, or drop it in 0.5?",
            },
            Part::Named { role: Role::Figure, name: "0 landed of 2" },
        ],
    },
    Holds {
        name: "accounts-solo",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 1" },
            Part::Named { role: Role::Disclosure, name: "Claude subscription" },
            Part::Named { role: Role::Item, name: "Claude Code login claude-robin · Ready" },
            Part::Named { role: Role::Item, name: "API key · Ready" },
            Part::Named { role: Role::Paragraph, name: "laptop" },
        ],
    },
    Holds {
        name: "chats-first-run-1",
        heading: "Talk to an agent",
        parts: &[
            Part::Named { role: Role::Link, name: "Ask temper" },
            Part::Absent { role: Role::List, name: "Suggested requests" },
            Part::Named { role: Role::Paragraph, name: "None." },
            Part::Named { role: Role::Region, name: "Playground" },
        ],
    },
    Holds {
        name: "chat-new-first-run-1",
        heading: "New chat",
        parts: &[
            Part::Named { role: Role::List, name: "Suggested requests" },
            Part::Named { role: Role::Button, name: "Add CSV export to widget" },
            Part::Named { role: Role::Button, name: "Add a dark theme to widget" },
            Part::Named { role: Role::Button, name: "Fix widget’s empty search state" },
        ],
    },
    Holds {
        name: "sheet-adopt",
        heading: "What factory works on.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Add repositories to factory" },
            Part::Named { role: Role::Search, name: "Find by name" },
        ],
    },
    Holds {
        name: "sheet-searching",
        heading: "What factory works on.",
        parts: &[Part::Named { role: Role::Button, name: "Adopt 2 repositories" }],
    },
    Holds {
        name: "sheet-none",
        heading: "What factory works on.",
        parts: &[
            Part::Value { role: Role::TextField, name: "Owner", value: "sam" },
            Part::Named { role: Role::Disclosure, name: "Not listed" },
        ],
    },
    Holds {
        name: "sheet-few",
        heading: "What factory works on.",
        parts: &[
            Part::Absent { role: Role::Search, name: "Find by name" },
            Part::Named { role: Role::Check, name: "sam/widget" },
        ],
    },
    Holds { name: "sheet-empty", heading: "What factory works on.", parts: &[Part::Text { text: "None reachable." }] },
    Holds {
        name: "sheet-copy",
        heading: "What Hello works on.",
        parts: &[
            Part::Value {
                role: Role::TextArea,
                name: "Their addresses, one per line",
                value: "github.com/octo/vsr\ncodeberg.org/forgejo/runner",
            },
            Part::Named { role: Role::Button, name: "Copy 2 repositories" },
        ],
    },
    Holds {
        name: "sheet-copy-lines",
        heading: "What Hello works on.",
        parts: &[
            Part::Named { role: Role::Button, name: "Use it" },
            Part::Named { role: Role::Button, name: "Adopt it" },
            Part::Text { text: "github isn’t a host. Did you mean github.com/octo/vsr?" },
            Part::Text { text: "octo/raft is already on this forge." },
        ],
    },
    Holds {
        name: "sheet-empty-source",
        heading: "What Hello works on.",
        parts: &[
            Part::Value { role: Role::TextField, name: "Organization", value: "work" },
            Part::Named { role: Role::Button, name: "Make it" },
        ],
    },
    Holds {
        name: "sheet-from-home",
        heading: "What Hello works on.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Add repositories to Hello" },
            Part::Selected { name: "Adopt" },
            Part::Named { role: Role::Choice, name: "Make empty" },
            Part::Named { role: Role::Choice, name: "Copy public" },
        ],
    },
    Holds {
        name: "sheet-from-chats",
        heading: "What Hello works on.",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Add repositories to Hello" },
            Part::Value { role: Role::TextField, name: "Owner", value: "" },
            Part::Value { role: Role::TextField, name: "Name", value: "" },
            Part::Named { role: Role::Button, name: "Adopt" },
        ],
    },
    Holds {
        name: "chats-reference",
        heading: "Talk to an agent",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Director" },
            Part::Chats { live: REFERENCE_LIVE, closed: REFERENCE_CLOSED },
        ],
    },
    Holds {
        name: "chat-reference-T1",
        heading: "T1 · Director",
        parts: &[Part::Named { role: Role::Paragraph, name: "Check · two-hour timer" }],
    },
    Holds {
        name: "chat-solo-T1",
        heading: "T1 · Make tally sum stream its input",
        parts: &[
            Part::NoStyle { style: temper_client_view::look::STATUS_LINE },
            Part::Absent { role: Role::TextArea, name: "Message" },
            Part::Absent { role: Role::Button, name: "Send" },
            Part::Absent { role: Role::Disclosure, name: "Model" },
        ],
    },
    Holds {
        name: "chat-model-open",
        heading: "New chat",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-model-both",
        heading: "New chat",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-model-all",
        heading: "New chat",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-model-two-accounts",
        heading: "New chat",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-model-sign-in-again",
        heading: "New chat",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-model-fixed",
        heading: "T1",
        parts: &[Part::Named { role: Role::Paragraph, name: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-started-model-open",
        heading: "T1",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-started-model-changing",
        heading: "T1",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "GPT-6.1 Sol · high" }],
    },
    Holds {
        name: "chat-started-model-refused",
        heading: "T1",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "ask-model-open",
        heading: "Ask temper",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds {
        name: "chat-working",
        heading: "T6",
        parts: &[
            Part::Named { role: Role::Slot, name: "Writing" },
            Part::Named { role: Role::Status, name: "Working" },
        ],
    },
    Holds {
        name: "chat-solo-T6",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[Part::Named { role: Role::Paragraph, name: "T7 found it" }],
    },
    Holds {
        name: "chat-held-task",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "The CSV import task is held" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "chat-change-running",
        heading: "T1 · Add CSV export to widget",
        parts: &[
            Part::Named { role: Role::Link, name: "Task 2" },
            Part::Named { role: Role::Paragraph, name: "Change" },
            Part::Named { role: Role::Status, name: "Running" },
        ],
    },
    Holds {
        name: "chat-change-landed",
        heading: "T1 · Add CSV export to widget",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "CI passed. Landed" },
            Part::Named { role: Role::Status, name: "Landed" },
        ],
    },
    Holds {
        name: "chat-change-held",
        heading: "T1 · Add CSV export to widget",
        parts: &[Part::Named { role: Role::Status, name: "Held" }],
    },
    Holds {
        name: "chat-goal",
        heading: "T1 · Add CSV export to widget",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Started goal" },
            Part::Named { role: Role::Link, name: "Task 2" },
        ],
    },
    Holds {
        name: "chat-first-run-2-T1",
        heading: "T1 · Add CSV export to widget",
        parts: &[
            Part::Named { role: Role::Link, name: "Task 2" },
            Part::Named { role: Role::Disclosure, name: "wait" },
        ],
    },
    Holds {
        name: "chat-first-run-3-T1",
        heading: "T1 · Add CSV export to widget",
        parts: &[Part::Named { role: Role::Paragraph, name: "CSV export is available" }],
    },
    Holds {
        name: "chat-first-run-4-T4",
        heading: "T4 · Add a dark theme to widget",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "I would give widget" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Absent { role: Role::Disclosure, name: "note" },
        ],
    },
    Holds {
        name: "inbox-reference",
        heading: "Nothing needs you.",
        parts: &[Part::Named { role: Role::Link, name: "Home" }, Part::Absent { role: Role::Button, name: "Answer" }],
    },
    Holds {
        name: "accounts-states",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 11" },
            Part::Text { text: "Ready" },
            Part::Text { text: "Checking" },
            Part::Text { text: "Granted" },
            Part::Text { text: "Limited by " },
            Part::Text { text: "temper reached its usage limit on your ChatGPT plan. Tasks on it wait." },
            Part::Text { text: "Signed out of ChatGPT." },
            Part::Text { text: "OpenAI refused temper’s sign-in. Check the configuration’s registration." },
            Part::Text { text: "Cannot load " },
            Part::Text { text: "Cancelled" },
            Part::Text { text: "Closing" },
            Part::Text { text: "Moving to another deployment" },
        ],
    },
    Holds {
        name: "accounts-expiring",
        heading: "What agents run on.",
        parts: &[
            Part::Text { text: "Expiring: sign in again before it ends." },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "accounts-key",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Key (pasted once)" },
            Part::Named { role: Role::Button, name: "Add key" },
        ],
    },
    Holds {
        name: "accounts-removing",
        heading: "What agents run on.",
        parts: &[
            Part::Text { text: "Remove this account and all its sources?" },
            Part::Named { role: Role::Button, name: "Confirm" },
        ],
    },
    Holds {
        name: "accounts-signing-in",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::TextField, name: "Address of the page that couldn't be reached" },
            Part::Named { role: Role::Button, name: "Finish connecting" },
        ],
    },
    Holds {
        name: "accounts-program-needs-sign-in",
        heading: "What agents run on.",
        parts: &[
            Part::Text { text: "Claude Code isn't signed in for 'claude-robin' on " },
            Part::Named { role: Role::Button, name: "Check again" },
        ],
    },
    Holds {
        name: "accounts-empty",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 0" },
            Part::Text { text: "No model of yours yet." },
            Part::Named { role: Role::Link, name: "Connect a model" },
        ],
    },
    Holds {
        name: "accounts-reference",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 2" },
            Part::Named { role: Role::Disclosure, name: "Claude subscription" },
        ],
    },
    Holds {
        name: "accounts-first-run-1",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 0" },
            Part::Named { role: Role::Disclosure, name: "Scripted model · the playground’s" },
        ],
    },
    Holds {
        name: "google-sign-in",
        heading: "Sign in",
        parts: &[
            Part::Named { role: Role::Link, name: "Continue with Google" },
            Part::Links {
                style: temper_client_view::look::OUTSIDE,
                names: &["Continue with Google", "Read public projects"],
            },
            Part::Links { style: temper_client_view::look::OUTSIDE_PRIMARY, names: &["Continue with Google"] },
            Part::Links { style: temper_client_view::look::OUTSIDE_QUIET, names: &["Read public projects"] },
            Part::NoStyle { style: temper_client_view::look::CARD },
        ],
    },
    Holds {
        name: "no-project",
        heading: "No project yet",
        parts: &[
            Part::Named { role: Role::Form, name: "New project" },
            Part::Named { role: Role::TextField, name: "Project name" },
            Part::Named { role: Role::Button, name: "Create" },
            Part::Named { role: Role::Button, name: "Sign out" },
            Part::Named { role: Role::Status, name: "Pat" },
            Part::NoStyle { style: temper_client_view::look::RAIL },
            Part::NoStyle { style: temper_client_view::look::PROJECTS },
        ],
    },
    Holds {
        name: "signed-out",
        heading: "Signed out",
        parts: &[Part::Named { role: Role::Paragraph, name: "To sign in, open a sign-in link temper printed." }],
    },
    Holds {
        name: "out-of-date",
        heading: "Out of date",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "This page doesn't match temper's engine. Reload it to continue.",
            },
            Part::Named { role: Role::Button, name: "Reload" },
        ],
    },
    Holds {
        name: "chats-empty",
        heading: "Talk to an agent",
        parts: &[
            Part::Named { role: Role::Heading { level: 1 }, name: "Talk to an agent" },
            Part::Chats { live: 0, closed: 0 },
        ],
    },
    Holds {
        name: "inbox-empty",
        heading: "Nothing needs you.",
        parts: &[Part::Named { role: Role::Link, name: "Home" }, Part::Absent { role: Role::Button, name: "Answer" }],
    },
    Holds {
        name: "chat-closing",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[Part::Named { role: Role::Dialog, name: "Close this chat?" }],
    },
    Holds {
        name: "chat-rejecting",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Heading { level: 2 }, name: "JSON output for tally report" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "task-held",
        heading: "T3 · CSV import",
        parts: &[
            Part::Named { role: Role::Button, name: "Release" },
            Part::Plan { live: SOLO_PLAN_LIVE, title: "CSV import: the quoted-field parser" },
        ],
    },
    Holds {
        name: "task-releasing",
        heading: "T3 · CSV import",
        parts: &[Part::Named { role: Role::Dialog, name: "Release this task?" }],
    },
    Holds {
        name: "connect-model-choose",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Region, name: "ChatGPT" }],
    },
    Holds {
        name: "connect-model-connected",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Text, name: "✓ Connected" }],
    },
    Holds {
        name: "connect-model-key-details",
        heading: "An OpenAI API key",
        parts: &[Part::Named { role: Role::TextField, name: "Key (pasted once)" }],
    },
    Holds {
        name: "connect-model-program-details",
        heading: "Your Claude subscription",
        parts: &[Part::Named { role: Role::TextField, name: "Name" }],
    },
    Holds {
        name: "accounts-added",
        heading: "What agents run on.",
        parts: &[Part::Text { text: "Added your ChatGPT plan. temper’s agents now run on it." }],
    },
    Holds {
        name: "connect-model-declined",
        heading: "Your ChatGPT plan",
        parts: &[Part::Named { role: Role::Alert, name: "You signed in without plan usage." }],
    },
    Holds {
        name: "accounts-renewed",
        heading: "What agents run on.",
        parts: &[Part::Text { text: "Added your ChatGPT plan. temper’s agents now run on it." }],
    },
    Holds {
        name: "connect-model-refused",
        heading: "Your ChatGPT plan",
        parts: &[Part::Named {
            role: Role::Alert,
            name: "Another person here holds that ChatGPT account. To use one of yours, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
        }],
    },
    Holds {
        name: "connect-model-signing-in",
        heading: "Your ChatGPT plan",
        parts: &[
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "connect-model-key-lost",
        heading: "An OpenAI API key",
        parts: &[Part::Named { role: Role::Alert, name: "" }],
    },
    Holds {
        name: "task-reference-T28",
        heading: "Build path: one cache per lane",
        parts: &[
            Part::Named { role: Role::Item, name: "Queued · 2nd" },
            Part::Named { role: Role::Slot, name: "2nd in the queue for main, after Task 26." },
            Part::Named {
                role: Role::Text,
                name: "PR #216 · acme/kit · temper/28/build-path-one-cache-per-lane → main",
            },
            Part::Named { role: Role::Paragraph, name: "CI at its head" },
            Part::Named { role: Role::Paragraph, name: "Passed" },
            Part::Named { role: Role::Paragraph, name: "No other gate." },
            Part::Named { role: Role::Text, name: "Repairs 0 of 3 · resolutions 0 of 3 · updates 0 of 3" },
            Part::Named { role: Role::Link, name: "View on forge ↗" },
            Part::Named { role: Role::Tree, name: "Its tasks" },
            Part::Named { role: Role::Paragraph, name: "Produce T28's branch" },
        ],
    },
    Holds {
        name: "task-reference-T24",
        heading: "Fake server: replay fixtures",
        parts: &[
            Part::Named { role: Role::Item, name: "Landed" },
            Part::Named { role: Role::Slot, name: "Landed · merge commit 4e1a9c7" },
            Part::Named {
                role: Role::Text, name: "PR #212 · acme/kit · temper/24/fake-server-replay-fixtures → main"
            },
            Part::Named { role: Role::Tree, name: "Its tasks" },
            Part::Named { role: Role::Paragraph, name: "Produce T24's branch" },
            Part::Absent { role: Role::Paragraph, name: "More of its ended tasks are not shown." },
        ],
    },
    Holds {
        name: "task-reference-T12",
        heading: "3 of 9 tasks done.",
        parts: &[Part::Text { text: "Harness" }, Part::Named { role: Role::Disclosure, name: "1 landed" }],
    },
    Holds {
        name: "task-solo-T9",
        heading: "T9 · Produce T8's branch",
        parts: &[
            Part::Named { role: Role::Log, name: "Conversation" },
            Part::Named { role: Role::Disclosure, name: "Read 4 files, ran 1 command" },
            Part::Named { role: Role::Disclosure, name: "Edited 3 files, ran 2 commands" },
            Part::Named { role: Role::Paragraph, name: "Errors carry their line now" },
        ],
    },
];
const SPENDS: [(&str, Role, &str); 55] = [
    ("chat-view-way", Role::Paragraph, "0 spent · of 40"),
    ("chat-view-proposal", Role::Paragraph, "0 spent · of 40"),
    ("chat-view-charter", Role::Paragraph, "0 spent · of 40"),
    ("chat-terminal-proposal", Role::Paragraph, "0 spent · of 40"),
    ("chat-terminal", Role::Paragraph, "1 spent · of 40"),
    ("chat-terminal-draft", Role::Paragraph, "1 spent · of 40"),
    ("chat-terminal-light", Role::Paragraph, "1 spent · of 40"),
    ("chat-terminal-closed", Role::Paragraph, "1 spent · of 40"),
    ("chat-terminal-working", Role::Paragraph, "1 spent · of 40"),
    ("chat-model-fixed", Role::Paragraph, "70 spent · of 101472"),
    ("chat-started-model-open", Role::Paragraph, "70 spent · of 101472"),
    ("chat-started-model-changing", Role::Paragraph, "70 spent · of 101472"),
    ("chat-started-model-refused", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-done", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-open", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-running", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-writing", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-failed", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-one", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-unknown", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-separated", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-empty", Role::Paragraph, "70 spent · of 101472"),
    ("chat-work-reading", Role::Paragraph, "70 spent · of 101472"),
    ("chat-paused", Role::Paragraph, "1 spent · of 40"),
    ("chat-closed", Role::Paragraph, "1 spent · of 40"),
    ("chat-closing-below", Role::Paragraph, "1 spent · of 40"),
    ("chat-closing-proposal", Role::Paragraph, "1 spent · of 40"),
    ("chat-closing-work", Role::Paragraph, "1 spent · of 40"),
    ("chat-closing-failed", Role::Paragraph, "1 spent · of 40"),
    ("chat-closing-start-over", Role::Paragraph, "0 spent · of 40"),
    ("chat-first-run-7-T1", Role::Paragraph, "0 spent · of 20"),
    ("chat-solo-T10", Role::Paragraph, "0 spent · of 40"),
    ("chat-held-task", Role::Paragraph, "0 spent · of 40"),
    ("chat-reference-T1", Role::Paragraph, "6 spent · of 100"),
    ("chat-solo-T1", Role::Paragraph, "1 spent · of 40"),
    ("chat-draft-chat", Role::Paragraph, "1 spent · of 40"),
    ("chat-draft-empty", Role::Paragraph, "1 spent · of 40"),
    ("chat-draft-split", Role::Paragraph, "1 spent · of 40"),
    ("chat-draft-draft", Role::Paragraph, "1 spent · of 40"),
    ("chat-start-goal", Role::Paragraph, "1 spent · of 40"),
    ("chat-start-goal-refused", Role::Paragraph, "1 spent · of 40"),
    ("chat-goal-linked", Role::Paragraph, "1 spent · of 40"),
    ("chat-solo-T6", Role::Paragraph, "1 spent · of 40"),
    ("chat-working", Role::Paragraph, "1 spent · of 40"),
    ("chat-change-running", Role::Paragraph, "0 spent · of 20"),
    ("chat-change-landed", Role::Paragraph, "0 spent · of 20"),
    ("chat-change-held", Role::Paragraph, "0 spent · of 20"),
    ("chat-goal", Role::Paragraph, "0 spent · of 20"),
    ("chat-first-run-2-T1", Role::Paragraph, "0 spent · of 20"),
    ("chat-first-run-3-T1", Role::Paragraph, "0 spent · of 20"),
    ("chat-first-run-4-T4", Role::Paragraph, "0 spent · of 20"),
    ("chat-closing", Role::Paragraph, "1 spent · of 40"),
    ("chat-rejecting", Role::Paragraph, "0 spent · of 40"),
    ("chats-reference", Role::Paragraph, "6 spent"),
    ("chats-solo", Role::Paragraph, "1 spent"),
];
fn each_chat_reads_its_allotment(sample: &Sample, view: &temper_client_view::View) {
    if matches!(sample.name, "chat-terminal-commands" | "chat-terminal-model" | "chat-terminal-question") {
        assert!(
            !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&temper_client_view::look::TERMINAL_LINE)),
            "a list replaces the quiet line and spend"
        );
        return;
    }
    if (sample.name.starts_with("chat-") || matches!(sample.name, "chats-reference" | "chats-solo"))
        && !matches!(sample.address, temper_client_domain::Address::NewChat { .. })
    {
        let (_, role, spend) =
            SPENDS.iter().find(|row| row.0 == sample.name).unwrap_or_else(|| panic!("{}: allotment row", sample.name));
        find(view.tree(), *role, spend, sample.name);
    }
}
fn no_state_reads_of_zero(sample: &Sample, view: &temper_client_view::View) {
    for node in view.tree().nodes() {
        let mut text = Vec::new();
        for run in &node.text {
            text.extend_from_slice(&run.text);
        }
        for text in [node.name.as_deref(), Some(text.as_slice())].into_iter().flatten() {
            // A space or end after zero: times (of 08:00) and fractions (of 0.5) are not allotments.
            assert!(
                !text.windows(b"of 0 ".len()).any(|part| part == b"of 0 ") && !text.ends_with(b"of 0"),
                "{}: zero allotment in {:?}",
                sample.name,
                String::from_utf8_lossy(text)
            );
        }
    }
}
// A browser's button, tab and option show their name as their text (skein/docs/design/ui/web/elements.md), so text under one
// is more words; recommended/default/new tags, Work's project way and All models' count are intentional children (temper/docs/design/client/pages/README.md).
fn no_named_control_draws_a_text_child_in_any_state(sample: &Sample, view: &temper_client_view::View) {
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.name.is_none() || !matches!(node.role, Role::Button | Role::Tab | Role::Option) {
            continue;
        }
        let mut text = Vec::new();
        let mut tagged = Vec::new();
        let mut way = Vec::new();
        let mut way_end = 0;
        let mut tag_end = 0;
        for (offset, inner) in nodes.iter().enumerate().skip(index).take(usize::try_from(node.size).unwrap()) {
            if inner.styles.as_slice().contains(&temper_client_view::look::MODEL_TAG) {
                tag_end = offset + usize::try_from(inner.size).unwrap();
            }
            if inner.styles.as_slice().contains(&temper_client_view::look::WORK_WAY) {
                way_end = offset + usize::try_from(inner.size).unwrap();
            }
            for run in &inner.text {
                text.extend_from_slice(&run.text);
                if offset < way_end {
                    way.extend_from_slice(&run.text);
                }
                if offset < tag_end {
                    tagged.extend_from_slice(&run.text);
                }
            }
        }
        let model_tag = node.role == Role::Button
            && text == tagged
            && (tagged == b"default"
                || tagged == b"new"
                || tagged == b"defaultnew"
                || (node.styles.as_slice().contains(&temper_client_view::look::MODEL_ALL)
                    && tagged.iter().all(u8::is_ascii_digit)));
        let project_way = node.role == Role::Button
            && node.styles.as_slice().contains(&temper_client_view::look::WORK_CHOICE)
            && !way.is_empty()
            && text == way;
        let path_stop = node.role == Role::Button
            && node.name.as_deref() == Some(text.as_slice())
            && nodes[..index].iter().enumerate().any(|(start, ancestor)| {
                ancestor.drawing.as_ref().is_some_and(|drawing| drawing.kind == temper_client_view::paths::KIND)
                    && index < start + usize::try_from(ancestor.size).expect("drawing span")
            });
        assert!(
            text.is_empty() || text == b"recommended" || model_tag || project_way || path_stop,
            "{}: {:?} {:?} also draws the text {:?}",
            sample.name,
            node.role,
            node.name.as_deref().map(String::from_utf8_lossy),
            String::from_utf8_lossy(&text),
        );
    }
}
fn has_text(tree: &skein_ui::Tree, text: &str) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == text.as_bytes()))
}
fn in_main(role: Role, name: &str) -> Find {
    Find::named(role, name.as_bytes())
        .within(Box::new([skein_fake_person::Target::Role { role: Role::Main, name: Box::new([]) }]))
}
fn find(tree: &skein_ui::Tree, role: Role, name: &str, sample: &str) -> skein_ui::NodeId {
    let mut face = TreeFace::new(tree);
    match face.find(&Find::named(role, name.as_bytes())) {
        Found::Node(node) => node,
        Found::Absent | Found::Pending => panic!("{sample}: missing {role:?} {name:?}"),
    }
}
fn descendants<'a>(tree: &'a skein_ui::Tree, role: Role, name: &str, sample: &str) -> &'a [skein_ui::Node] {
    let id = find(tree, role, name, sample);
    let nodes = tree.nodes();
    let start = nodes.iter().position(|node| node.id == id).expect("subtree root");
    let end = start + usize::try_from(nodes[start].size).expect("subtree span");
    &nodes[start..end]
}
#[test]
fn team_hand_opens_the_sheet_with_the_first_area_and_role() {
    let sample = samples::samples().into_iter().find(|s| s.name == "team-hand").expect("hand-off sample");
    let root = samples::domain(&sample);
    let open = root.pages().hand_area().expect("hand-off sheet is open");
    assert_eq!(open.area.as_deref(), Some(b"core".as_slice()));
    assert_eq!(open.role, Some(52));
    assert!(open.apply);
    let (view, _, _) = built_root(&sample, ViewInputs::new(), &root);
    find(view.tree(), Role::Dialog, "Hand an area to a role", sample.name);
    let mut last = 0;
    for title in ["Prover", "Prover, from the stock", "Porter, from the stock", "Write one with Ask temper"] {
        let id = find(view.tree(), Role::Choice, title, sample.name);
        let position = view.tree().nodes().iter().position(|n| n.id == id).unwrap();
        assert!(position > last, "role choices follow the gallery's order");
        last = position;
    }
    for words in [
        "Owns its area's tests: triages their reds and flakes and makes the repairs.",
        "Keeps its area's port in step with its source.",
    ] {
        find(view.tree(), Role::Paragraph, words, sample.name);
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "the gallery table walk checks each kind of named part")]
fn each_state_holds_what_its_name_says() {
    assert_eq!(HOLDS.len(), samples::N, "every promised row has a sample");
    let inputs = ViewInputs::new();
    for sample in samples::samples() {
        let mut row = None;
        for holds in &HOLDS {
            if holds.name == sample.name {
                assert!(row.is_none(), "{}: duplicate holds row", sample.name);
                row = Some(holds);
            }
        }
        let row = row.unwrap_or_else(|| panic!("{}: every sample has a HOLDS row", sample.name));
        let root = samples::domain(&sample);
        let (mut view, bounds, _) = built_root(&sample, inputs.clone(), &root);
        let tree = view.tree();
        let mut main = TreeFace::new(tree);
        let level = if matches!(
            root.pages().address(),
            temper_client_domain::Address::Goals { .. } | temper_client_domain::Address::Changes { .. }
        ) {
            2
        } else {
            1
        };
        let h1 = in_main(Role::Heading { level }, row.heading);
        if row.heading.is_empty() {
            assert_eq!(main.find(&h1), Found::Absent, "{} has no main heading", sample.name);
        } else {
            assert!(matches!(main.find(&h1), Found::Node(_)), "{}: heading {}", sample.name, row.heading);
        }
        // Count-only names may repeat; described parts follow conversation order.
        let mut described = tree.nodes().iter();
        let mut destinations = Vec::new();
        for part in row.parts {
            match part {
                Part::WelcomeFrame { box_note } => {
                    let banner = descendants(tree, Role::Banner, "", sample.name);
                    let mut children = Vec::new();
                    let mut rest = &banner[1..];
                    while let Some(node) = rest.first() {
                        children.push(node);
                        rest = &rest[usize::try_from(node.size).expect("banner child span")..];
                    }
                    assert_eq!(children.len(), 3, "{}: brand, steps, end alone", sample.name);
                    assert_eq!(
                        (children[0].role, children[0].name.as_deref()),
                        (Role::Link, Some(b"temper".as_slice()))
                    );
                    assert_eq!(
                        (children[1].role, children[1].name.as_deref()),
                        (Role::List, Some(b"Steps".as_slice()))
                    );
                    assert!(children[2].styles.as_slice().contains(&temper_client_view::look::TOP_END));
                    let end = &banner[banner.len() - usize::try_from(children[2].size).expect("end span")..];
                    assert_eq!(
                        end[1].region,
                        Some(skein_ui::Region { owner: temper_client_view::look::NAMES.owner, number: 0 })
                    );
                    assert_eq!(
                        usize::try_from(end[1].size).expect("region span") + 1,
                        end.len(),
                        "{}: link alone",
                        sample.name
                    );
                    let notes: Vec<_> = tree
                        .nodes()
                        .iter()
                        .filter(|n| n.styles.as_slice().contains(&temper_client_view::look::WELCOME_BOX_NOTE))
                        .collect();
                    assert_eq!(notes.len(), usize::from(*box_note), "{}: box paragraph on Welcome alone", sample.name);
                    if *box_note {
                        assert_eq!(notes[0].role, Role::Paragraph);
                        assert!(descendants(tree, Role::Main, "", sample.name).iter().any(|n| n.id == notes[0].id));
                    }
                }
                Part::Projects { rows } => {
                    let region = descendants(tree, Role::Region, "Projects", sample.name);
                    let actual: Vec<_> = region
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.role == Role::Item)
                        .map(|(start, node)| {
                            let end = start + usize::try_from(node.size).expect("project span");
                            let row = &region[start..end];
                            let link = row.iter().find(|node| node.role == Role::Link).expect("project link");
                            let health = row
                                .iter()
                                .position(|node| node.styles.as_slice().contains(&temper_client_view::look::CHIP))
                                .expect("project health");
                            let end = health + usize::try_from(row[health].size).expect("health span");
                            let words: Vec<_> = row[health..end]
                                .iter()
                                .flat_map(|node| node.text.iter())
                                .flat_map(|run| run.text.iter().copied())
                                .collect();
                            (
                                String::from_utf8(link.name.as_deref().expect("project name").to_vec()).expect("UTF-8"),
                                String::from_utf8(words).expect("UTF-8"),
                            )
                        })
                        .collect();
                    let expected: Vec<_> =
                        rows.iter().map(|(name, health)| ((*name).to_owned(), (*health).to_owned())).collect();
                    assert_eq!(actual, expected, "{}: projects and their health", sample.name);
                }
                Part::StyleCount { style, count } => {
                    let actual = tree.nodes().iter().filter(|node| node.styles.as_slice().contains(style)).count();
                    assert_eq!(actual, *count, "{}: {style:?} rows", sample.name);
                }
                Part::HomeCards { primary, projects } => {
                    let nodes = tree.nodes();
                    let cards: Vec<_> = nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::NEED_CARD))
                        .collect();
                    assert_eq!(cards.len(), primary.len(), "{}: Home cards", sample.name);
                    for (index, ((start, card), primary)) in cards.iter().zip(*primary).enumerate() {
                        let end = start + usize::try_from(card.size).expect("card size");
                        let card = &nodes[*start..end];
                        if !projects.is_empty() {
                            assert!(
                                card.iter().any(|node| node
                                    .text
                                    .iter()
                                    .any(|run| run.text.as_ref() == projects[index].as_bytes())),
                                "{}: card {index} project",
                                sample.name
                            );
                        }
                        let acts = card
                            .iter()
                            .position(|node| node.styles.as_slice().contains(&temper_client_view::look::NEED_ACTS))
                            .expect("card acts");
                        let end = acts + usize::try_from(card[acts].size).expect("acts size");
                        let first = card[acts..end]
                            .iter()
                            .find(|node| matches!(node.role, Role::Button | Role::Link))
                            .expect("primary act");
                        assert_eq!(
                            first.name.as_deref(),
                            Some(primary.as_bytes()),
                            "{}: card {index} primary act",
                            sample.name
                        );
                    }
                }
                Part::Current { name } => {
                    let id = find(tree, Role::Link, name, sample.name);
                    assert!(
                        tree.find(id).expect("current link").states.as_slice().contains(&skein_ui::State::Current),
                        "{}: {name} current",
                        sample.name
                    );
                }
                Part::Destination { name, href } => {
                    let Found::Node(node) = main.find(&Find::exactly_named(Role::Link, name.as_bytes())) else {
                        panic!("{}: missing link {name}", sample.name)
                    };
                    destinations.push((node, href));
                }
                Part::FirstButton { disclosure, name } => {
                    let nodes = descendants(tree, Role::Disclosure, disclosure, sample.name);
                    let first = nodes.iter().find(|node| node.role == Role::Button).expect("switcher's first row");
                    assert_eq!(first.name.as_deref(), Some(name.as_bytes()), "{}: first project row", sample.name);
                }
                Part::CardChanges { title, changes } => {
                    let link = find(tree, Role::Link, title, sample.name);
                    let nodes = tree.nodes();
                    let at = nodes.iter().position(|node| node.id == link).expect("card's link");
                    let (start, card) = nodes
                        .iter()
                        .enumerate()
                        .take(at)
                        .rev()
                        .find(|(start, node)| {
                            node.styles.as_slice().contains(&temper_client_view::look::WORK_CARD)
                                && at < start + usize::try_from(node.size).expect("card size")
                        })
                        .expect("link's card");
                    let end = start + usize::try_from(card.size).expect("card size");
                    let links: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Link && node.id != link)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("change name")).expect("English"))
                        .collect();
                    assert_eq!(links, *changes, "{}: {title} changes", sample.name);
                }
                Part::Cards { section, from } => {
                    let nodes = descendants(tree, Role::Region, section, sample.name);
                    let cards: Vec<_> = nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::WORK_CARD))
                        .collect();
                    assert_eq!(cards.len(), from.len(), "{}: {section} card count", sample.name);
                    for ((start, card), expected) in cards.iter().zip(*from) {
                        let end = start + usize::try_from(card.size).expect("card subtree");
                        let card = &nodes[*start..end];
                        let start = card
                            .iter()
                            .position(|node| {
                                node.role == Role::Paragraph
                                    && node.styles.as_slice().contains(&temper_client_view::look::WORK_FROM)
                            })
                            .expect("card's from line");
                        let end = start + usize::try_from(card[start].size).expect("from subtree");
                        let text: Vec<_> = card[start..end]
                            .iter()
                            .flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied()))
                            .collect();
                        assert_eq!(String::from_utf8_lossy(&text), *expected, "{}: {section} from line", sample.name);
                    }
                }
                Part::GroupLines { name, lines } => {
                    let group = descendants(tree, Role::Group, name, sample.name);
                    let drawn: Vec<_> = group
                        .iter()
                        .filter(|node| node.role == Role::Item)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("line name")).expect("English"))
                        .collect();
                    assert_eq!(drawn, *lines, "{}: {name} lines in order", sample.name);
                }
                Part::StyledText { style, text } => {
                    let nodes = tree.nodes();
                    let start = nodes
                        .iter()
                        .position(|node| node.styles.as_slice().contains(style))
                        .unwrap_or_else(|| panic!("{}: missing row {style:?}", sample.name));
                    let end = start + usize::try_from(nodes[start].size).expect("row span");
                    let actual: Vec<_> = nodes[start..end]
                        .iter()
                        .flat_map(|node| node.text.iter())
                        .map(|run| std::str::from_utf8(&run.text).expect("row words"))
                        .collect();
                    assert_eq!(actual, *text, "{}: row text", sample.name);
                }
                Part::Tabs { names, selected } => {
                    let tabs = descendants(tree, Role::Tabs, "Ways of working", sample.name);
                    let actual: Vec<_> = tabs.iter().filter(|node| node.role == Role::Tab).collect();
                    assert_eq!(actual.len(), names.len(), "{}: tabs", sample.name);
                    for (index, (tab, expected)) in actual.iter().zip(*names).enumerate() {
                        let start = tabs.iter().position(|node| node.id == tab.id).expect("tab position");
                        let end = start + usize::try_from(tab.size).expect("tab span");
                        let text: Vec<_> = tabs[start..end]
                            .iter()
                            .flat_map(|node| node.text.iter())
                            .flat_map(|run| run.text.iter().copied())
                            .collect();
                        assert_eq!(text, expected.as_bytes(), "{}: tab {index}", sample.name);
                        assert_eq!(
                            tab.states.as_slice().contains(&skein_ui::State::Selected),
                            index == *selected,
                            "{}: selected tab",
                            sample.name
                        );
                        let now = tabs[start..end]
                            .iter()
                            .any(|node| node.styles.as_slice().contains(&temper_client_view::look::WAYS_NOW));
                        assert_eq!(now, expected.ends_with("now"), "{}: current tab marker", sample.name);
                    }
                }
                Part::Path { stops } => {
                    let path = descendants(tree, Role::List, "The way a change goes", sample.name);
                    let actual: Vec<_> = path
                        .iter()
                        .filter(|node| node.role == Role::Item)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("stop words")).expect("UTF-8 stop"))
                        .collect();
                    assert_eq!(actual, *stops, "{}: path stops and actors", sample.name);
                }
                Part::Drawing { style, count } => {
                    let drawing = tree
                        .nodes()
                        .iter()
                        .find_map(|node| {
                            node.drawing.as_ref().filter(|drawing| drawing.kind == temper_client_view::paths::KIND)
                        })
                        .expect("way drawing");
                    let mut scene = Scene::new(&bounds.tree);
                    assert!(draw(drawing.kind, &drawing.data, 1000, 0, &mut scene), "{}: way scene", sample.name);
                    assert!(!scene.refused(), "{}: scene fits", sample.name);
                    let actual = (0..u32::try_from(scene.marks().len()).expect("scene marks"))
                        .filter(|index| scene.styles(*index).contains(style))
                        .count();
                    assert_eq!(actual, *count, "{}: drawing {style:?}", sample.name);
                }
                Part::Expanded { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        tree.find(id).expect("stop").states.as_slice().contains(&skein_ui::State::Expanded),
                        "{}: open stop",
                        sample.name
                    );
                }
                Part::Folded { name } => {
                    let id = find(tree, Role::Disclosure, name, sample.name);
                    assert!(
                        !tree.find(id).expect("disclosure").states.as_slice().contains(&skein_ui::State::Expanded),
                        "{}: {name} folded",
                        sample.name
                    );
                }
                Part::Configured { value } => {
                    let facts = temper_client_domain::pages::home::facts(
                        &root,
                        1,
                        worlds::sample_now(sample.world, sample.name),
                    );
                    assert_eq!(facts.configured, *value, "{}: To begin's first step", sample.name);
                }
                Part::Reader { role } => {
                    let me = root.pages().me(root.client()).expect("signed-in reader");
                    let member = me.projects.iter().find(|member| member.project == 1).expect("ways project");
                    assert_eq!(&member.role, role, "{}: reader's role", sample.name);
                }
                Part::After { first, then } => {
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.role == Role::Main).expect("main");
                    let end = start + usize::try_from(nodes[start].size).expect("main span");
                    let at = |style| {
                        nodes[start..end]
                            .iter()
                            .position(|node| node.styles.as_slice().contains(style))
                            .expect("ordered part")
                    };
                    assert!(at(first) < at(then), "{}: part order", sample.name);
                }
                Part::Links { style, names } => {
                    let nodes = tree.nodes();
                    let start =
                        nodes.iter().position(|node| node.styles.as_slice().contains(style)).expect("navigation style");
                    let end = start + usize::try_from(nodes[start].size).expect("navigation size");
                    let links: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Link)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("link name")).expect("English"))
                        .collect();
                    assert_eq!(links, *names, "{}: navigation", sample.name);
                }
                Part::NoStyle { style } => assert!(
                    !tree.nodes().iter().any(|node| node.styles.as_slice().contains(style)),
                    "{}: absent style",
                    sample.name
                ),
                Part::Selected { name } => {
                    let id = find(tree, Role::Choice, name, sample.name);
                    assert!(
                        tree.find(id).expect("source choice").states.as_slice().contains(&skein_ui::State::Selected),
                        "{}: {name} selected",
                        sample.name
                    );
                }
                Part::NotPressed { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        !tree.find(id).expect("mode button").states.as_slice().contains(&skein_ui::State::Pressed),
                        "{}: {name} not pressed",
                        sample.name
                    );
                }
                Part::Pressed { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        tree.find(id).expect("mode button").states.as_slice().contains(&skein_ui::State::Pressed),
                        "{}: {name} pressed",
                        sample.name
                    );
                }
                Part::Text { text } => {
                    assert!(has_text(tree, text), "{}: missing text {text:?}", sample.name);
                }
                Part::Buttons { style, names } => {
                    let nodes = tree.nodes();
                    let start =
                        nodes.iter().position(|node| node.styles.as_slice().contains(style)).expect("button row");
                    let end = start + usize::try_from(nodes[start].size).expect("row span");
                    let actual: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Button)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("button name")).expect("English"))
                        .collect();
                    assert_eq!(actual, *names, "{}: buttons in order", sample.name);
                }
                Part::Named { role, name } => {
                    find(tree, *role, name, sample.name);
                }
                Part::Ordered { role, name } => {
                    assert!(
                        described.any(|node| node.role == *role && node.name.as_deref() == Some(name.as_bytes())),
                        "{}: missing or out of order {role:?} {name:?}",
                        sample.name
                    );
                }
                Part::Absent { role, name } => {
                    let query =
                        if *role == Role::Menu { Find::named(*role, name.as_bytes()) } else { in_main(*role, name) };
                    assert_eq!(main.find(&query), Found::Absent, "{}: absent {role:?} {name:?}", sample.name);
                }
                Part::NoRole { role } => {
                    assert!(!tree.nodes().iter().any(|node| node.role == *role), "{}: absent {role:?}", sample.name);
                }
                Part::Value { role, name, value } => {
                    let node = find(tree, *role, name, sample.name);
                    let value = value.as_bytes();
                    assert_eq!(
                        tree.find(node).expect("field").value.as_ref().expect("field value").text.as_ref(),
                        value,
                        "{}",
                        sample.name
                    );
                }
                Part::Described { role, name, description } => {
                    let node = described
                        .find(|node| node.role == *role && node.name.as_deref() == Some(name.as_bytes()))
                        .unwrap_or_else(|| panic!("{}: missing {role:?} {name:?}", sample.name));
                    assert_eq!(node.description.as_deref(), Some(description.as_bytes()), "{}", sample.name);
                }
                Part::Goals { rows } => {
                    let actual: Vec<_> = tree
                        .nodes()
                        .iter()
                        .filter(|node| node.styles.as_slice().contains(&temper_client_view::look::GOALS_ROW))
                        .map(|node| String::from_utf8_lossy(node.name.as_deref().expect("goal link name")))
                        .collect();
                    assert_eq!(actual.len(), rows.len(), "{}: goal rows", sample.name);
                    for (actual, expected) in actual.iter().zip(*rows) {
                        assert!(actual.starts_with(expected), "{}: expected {expected:?}, got {actual:?}", sample.name);
                    }
                }
                Part::Chats { live, closed } => {
                    let available = (*live).min(driver::limits().client.domain.rows)
                        + (*closed).min(driver::limits().client.domain.rows);
                    if available == 0 {
                        assert_eq!(main.find(&in_main(Role::List, "Your chats")), Found::Absent, "{}", sample.name);
                        continue;
                    }
                    let list = find(tree, Role::List, "Your chats", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == list).expect("chats list");
                    let end = start + usize::try_from(nodes[start].size).expect("chats subtree");
                    let mut count = 0;
                    for node in &nodes[start..end] {
                        if node.role == Role::Link
                            && let Some(name) = &node.name
                            && name
                                .strip_prefix(b"T")
                                .is_some_and(|number| !number.is_empty() && number.iter().all(u8::is_ascii_digit))
                        {
                            count += 1;
                        }
                    }
                    assert_eq!(count, available, "{}: windowed chat rows", sample.name);
                }
                Part::Plan { live, title } => {
                    let plan = find(tree, Role::Tree, "Plan", sample.name);
                    let row = Find::named(Role::Paragraph, title.as_bytes()).within(Box::new([
                        skein_fake_person::Target::Role { role: Role::Tree, name: Box::from(b"Plan".as_slice()) },
                    ]));
                    assert!(matches!(main.find(&row), Found::Node(_)), "{}: delegate title", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == plan).expect("plan node");
                    let end = start + usize::try_from(nodes[start].size).expect("plan subtree");
                    let count = nodes[start..end].iter().filter(|node| node.role == Role::TreeItem).count();
                    let window = driver::limits().client.domain.rows;
                    assert_eq!(
                        count,
                        usize::try_from((*live).min(window)).expect("row count"),
                        "{}: plan window",
                        sample.name
                    );
                }
            }
        }
        for (node, expected) in destinations {
            assert_eq!(
                temper_client_view::href(&mut view, node).as_deref(),
                Some(expected.as_bytes()),
                "{}: destination",
                sample.name
            );
        }
        each_chat_reads_its_allotment(&sample, &view);
        no_state_reads_of_zero(&sample, &view);
        no_named_control_draws_a_text_child_in_any_state(&sample, &view);
        navigation_links_have_addresses_and_install_action_links_keep_their_binding(&sample, &mut view);
    }
}
#[test]
fn goal_pages_hold_reports_counts_and_only_their_needs() {
    use temper_client_domain::pages::goals::State;
    for (name, number, done, total, state, report) in [
        ("goal-running", 108, 6, 9, State::Running, true),
        ("goal-needs-funds", 108, 6, 9, State::Needs { funds: true }, true),
        ("goal-needs-question", 118, 1, 3, State::Needs { funds: false }, true),
        ("goal-planning", 122, 0, 0, State::Planning, false),
        ("goal-paused", 108, 6, 9, State::Paused, true),
        ("goal-landed", 123, 2, 2, State::Landed, true),
        ("goal-landed-open", 123, 2, 2, State::Landed, true),
        ("goal-closed", 118, 1, 3, State::Closed, false),
        ("goal-failed", 122, 0, 0, State::Failed, false),
        ("goal-no-report", 108, 6, 9, State::Running, false),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("goal sample");
        let root = samples::domain(&sample);
        let page = root.pages().task().expect("task page");
        let task = page.document(root.client()).expect("goal document");
        let goal = page.goal.as_ref().expect("goal page reads");
        assert_eq!(task.chip.task, number, "{name}");
        assert_eq!(goal.line.as_ref().expect("own line read").chip.task, number, "{name}");
        assert_eq!(goal.counts(task, None), temper_client_domain::pages::goal::Counts { done, total }, "{name}");
        assert_eq!(goal.report.is_some(), report, "{name}");
        assert!(page.cards.is_empty(), "{name}: needs belong to the filtered inbox");
        let inbox = root.pages().inbox().expect("goal inbox");
        let open = root.pages().needs().open(inbox);
        assert_eq!(open.len(), usize::from(matches!(state, State::Needs { .. })), "{name}");
        assert_eq!(goal.state(task, None, !open.is_empty()), state, "{name}");
        for card in &open {
            assert!(card.page.is_some(), "{name}: need document delivered");
        }
        let view = view(&sample);
        let bylines = view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.styles.as_slice().contains(&temper_client_view::look::GOAL_BYLINE));
        assert_eq!(bylines, report, "{name}: only a report has a byline");
        if name == "goal-needs-question" {
            let choice = find(view.tree(), Role::Button, "In jig’s store", name);
            assert!(
                view.tree()
                    .find(choice)
                    .expect("recommended choice")
                    .styles
                    .as_slice()
                    .contains(&temper_client_view::look::GOAL_PRIMARY)
            );
        }
        if matches!(state, State::Landed | State::Closed | State::Failed) {
            assert!(
                !view.tree().nodes().iter().any(|node| node.role == Role::TextArea),
                "{name}: ended page has no composer"
            );
        }
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "the print walk checks every sample and its shared bundle")]
fn print_writes_every_page_and_the_bundle() {
    let scratch = Scratch::new("temper-client-gallery-print");
    let out = scratch.path().join("gallery");
    fs::create_dir_all(&out).expect("previous gallery");
    fs::write(out.join("index.html"), b"previous index").expect("previous index");
    let stale = out.join("stale.html");
    fs::write(&stale, b"obsolete").expect("stale page");
    print(&out).expect("replace gallery");
    assert!(!stale.exists(), "old output removed");
    let index = fs::read_to_string(out.join("index.html")).expect("index");
    let bundle = out.join("bundle");
    let mut digest = None;
    for entry in fs::read_dir(&bundle).expect("bundle") {
        let entry = entry.expect("bundle entry");
        if entry.file_type().expect("file type").is_dir() {
            assert!(digest.is_none(), "one digest");
            digest = Some(entry.file_name().into_string().expect("digest ASCII"));
        }
    }
    let digest = digest.expect("digest directory");
    for file in ["menu.svg", "send.svg"] {
        assert!(bundle.join(&digest).join("files").join(file).is_file(), "bundled decorative image {file}");
    }
    for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
        assert!(bundle.join(&digest).join(file).is_file(), "bundled {file}");
    }
    let mut position = 0;
    for sample in samples::samples() {
        let link = format!("href=\"{}.html\"", sample.name);
        assert_eq!(index.matches(&link).count(), 1, "one link for {}", sample.name);
        let at = index.find(&link).expect("index link");
        assert!(at >= position, "table order");
        position = at + link.len();
        let page = fs::read_to_string(out.join(format!("{}.html", sample.name))).expect("sample page");
        let view = crate::view(&sample);
        let paths = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.drawing.as_ref().is_some_and(|drawing| drawing.kind == temper_client_view::paths::KIND))
            .count();
        assert_eq!(page.matches("data-gallery-narrow=\"\"").count(), paths, "{}: narrow paths", sample.name);
        for svg in page.split("<svg").skip(1) {
            let (attributes, _) = svg.split_once('>').expect("SVG attributes");
            if attributes.contains("data-gallery-narrow=\"\"") {
                assert!(attributes.contains("viewBox=\"0 0 388 "), "{}: narrow viewBox", sample.name);
            }
        }
        assert_eq!(
            page.matches("@media (max-width: 760px)").count(),
            usize::from(paths > 0),
            "{}: narrow media rule",
            sample.name
        );
        let night = sample.name.starts_with("night-");
        assert_eq!(page.contains("<html lang=\"en\" data-night>"), night, "{}: night attribute", sample.name);
        assert_eq!(page.contains("<html lang=\"en\">"), !night, "{}: day root", sample.name);
        if matches!(sample.address, temper_client_domain::Address::Welcome { .. }) {
            assert!(page.contains("class=\"temper-top\""), "welcome has a top bar");
            assert!(!page.contains("temper-menu"), "welcome has no menu");
            assert!(page.contains("<title>Welcome · first run · temper</title>"), "{}: welcome title", sample.name);
        } else if sample.name.starts_with("chat-terminal") {
            let (_, main) = page.split_once("<main ").expect("terminal main element");
            let (main, _) = main.split_once('>').expect("terminal main attributes");
            assert!(main.contains("class=\"temper-terminal "), "{}: terminal main", sample.name);
            for class in ["temper-rail", "temper-tabbar", "temper-menu"] {
                assert!(!page.contains(&format!("class=\"{class}\"")), "{}: no {class}", sample.name);
            }
        } else if page.contains("class=\"temper-rail\"") {
            assert!(page.contains("class=\"temper-tabbar"), "{}: project sections", sample.name);
        } else {
            assert!(page.contains("class=\"temper-top\""), "{}: top bar", sample.name);
            assert!(!page.contains("temper-menu"), "{}: no menu", sample.name);
        }
        let mut position = 0;
        for file in ["skein-ui.css", "temper.css", "task-pages.css"] {
            let link = format!("href=\"bundle/{digest}/{file}\"");
            let at = page.find(&link).expect("stylesheet linked");
            assert!(at >= position, "stylesheet order");
            position = at + link.len();
        }
    }
    for name in ["goal-landed", "goal-landed-open"] {
        let page = fs::read_to_string(out.join(format!("{name}.html"))).expect("landed goal print");
        assert_eq!(page.contains("open class=\"temper-goal-landed-fold\""), name == "goal-landed-open");
    }
    for (name, parts) in [
        ("needs-list", ["class=\"temper-needs-bar\"", "1 of 3"]),
        ("ask-chat", ["class=\"temper-tabbar temper-tabbar-ask\"", ">Ask</span"]),
        ("rail-new-change", ["aria-label=\"Needs\"", ">Needs</span"]),
    ] {
        let page = fs::read_to_string(out.join(format!("{name}.html"))).expect("page print");
        for part in parts {
            assert!(page.contains(part), "{name}: {part}");
        }
    }
    let signin = fs::read_to_string(out.join("connect-model-signing-in.html")).expect("sign-in page");
    assert!(signin.contains("href=\"https://issuer.example/visit/gallery-sign-in\""), "external sign-in href");
    let closing = fs::read_to_string(out.join("chat-closing.html")).expect("confirmation page");
    assert!(closing.contains("<dialog open "), "static confirmation opens");
    let chats = fs::read_to_string(out.join("chats-solo.html")).expect("chats page");
    assert!(chats.contains("<title>Talk to an agent about robin/tally. · temper</title>"), "shell heading title");
    let signed_out = fs::read_to_string(out.join("signed-out.html")).expect("signed-out page");
    assert!(signed_out.contains("<title>Signed out · temper</title>"), "shell heading title");
    let mut expected = Vec::new();
    for path in STYLESHEETS {
        expected.push(Path::new(path).file_name().expect("stylesheet basename").to_str().expect("ASCII"));
    }
    let mut actual = Vec::new();
    for line in include_str!("../../../scripts/bundle.sh").lines() {
        let line = line.trim();
        if line == "else" {
            break;
        }
        if line.starts_with("--stylesheet ") {
            let (_, path) = line.split_once("$shell/").expect("shell stylesheet");
            actual.push(path.split('"').next().expect("basename"));
        }
    }
    assert_eq!(expected, actual);
}
#[test]
fn replacing_a_gallery_removes_the_whole_previous_print() {
    let scratch = Scratch::new("temper-client-gallery-replace");
    let out = scratch.path().join("gallery");
    replace_gallery(&out).expect("new gallery directory");
    assert!(out.is_dir(), "missing output created");
    let old_bundle = out.join("bundle").join("old-digest");
    fs::create_dir_all(&old_bundle).expect("previous bundle");
    fs::write(old_bundle.join("old.css"), b"old styles").expect("previous stylesheet");
    fs::write(out.join("index.html"), b"previous index").expect("previous index");
    fs::write(out.join("stale.html"), b"obsolete").expect("stale page");
    replace_gallery(&out).expect("replace owned gallery");
    assert!(out.is_dir(), "replacement output exists");
    assert_eq!(fs::read_dir(&out).expect("replacement directory").count(), 0, "whole previous print removed");
}
#[test]
fn print_refuses_a_directory_not_its_own() {
    let scratch = Scratch::new("temper-client-gallery-refusal");
    let file = scratch.path().join("precious");
    fs::write(&file, b"keep me").expect("foreign file");
    match print(scratch.path()) {
        Err(Refusal::NotGallery { out }) => assert_eq!(out, scratch.path()),
        other => panic!("expected NotGallery, got {other:?}"),
    }
    assert_eq!(fs::read(&file).expect("file untouched"), b"keep me");
    assert_eq!(fs::read_dir(scratch.path()).expect("directory untouched").count(), 1);
}

#[test]
fn times_read_against_each_mockup_worlds_fixed_now() {
    for (name, expected) in [
        ("chats-solo", "1 h 28 min"),
        ("chats-reference", "50 min"),
        ("inbox-solo", "a proposal"),
        ("inbox-solo", "12 min"),
        ("chat-solo-T10", "waiting 12 min"),
        ("chat-reference-T1", "Today · Thursday 22 October"),
        ("chat-solo-T10", "Today · Tuesday 27 October"),
        ("chat-first-run-4-T4", "21:05"),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("named sample");
        let view = view(&sample);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .any(|node| node.text.iter().any(|run| String::from_utf8_lossy(&run.text).contains(expected))),
            "{name}: {expected}"
        );
    }
}

#[test]
fn gallery_chat_clocks_are_separate_leaves_under_the_speakers() {
    for (name, clock) in [("chat-reference-T1", "06:56"), ("chat-solo-T10", "09:18")] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sample");
        let view = view(&sample);
        let tree = view.tree();
        let who = tree
            .nodes()
            .iter()
            .enumerate()
            .find(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::WHO))
            .expect("speaker")
            .0;
        assert_eq!(tree.nodes()[who + 1].name.as_deref(), Some(b"You".as_slice()));
        let clock_at = who + 1 + usize::try_from(tree.nodes()[who + 1].size).expect("speaker size");
        assert_eq!(tree.nodes()[clock_at].role, Role::Time);
        assert_eq!(tree.nodes()[clock_at].name.as_deref(), Some(clock.as_bytes()));
    }
}

#[test]
fn model_samples_link_the_mockup_state_that_draws_them() {
    let mut rows = [
        ("connect-model-choose", Some("connect-model.html?world=reference&bare")),
        ("connect-model-connected", Some("connect-model.html?world=reference&bare")),
        ("connect-model-key-details", Some("connect-model.html?world=reference&way=key&bare")),
        ("connect-model-program-details", Some("connect-model.html?world=reference&way=program&bare")),
        ("connect-model-signing-in", None),
        ("accounts-added", Some("accounts.html?world=reference&bare")),
        (
            "connect-model-declined",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-declined"),
        ),
        ("accounts-renewed", Some("accounts.html?world=reference&bare")),
        (
            "connect-model-refused",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-refused"),
        ),
        ("connect-model-key-lost", None),
        ("welcome-model-claude-terminal", Some("../web/claude-sign-in/01-model.html")),
        ("welcome-model-claude-signing-in", Some("../web/claude-sign-in/02-signing-in.html")),
        ("welcome-model-claude-signed-in", Some("../web/claude-sign-in/04-signed-in.html")),
        ("welcome-model-claude-no-terminal", Some("../web/claude-sign-in/05-no-terminal.html")),
        ("welcome-model-claude-not-finished", None),
    ];
    rows.sort_by_key(|row| row.0);
    let models: Vec<_> = samples::samples()
        .into_iter()
        .filter(|sample| {
            sample.name.starts_with("connect-model-")
                || sample.name.starts_with("welcome-model-claude-")
                || matches!(sample.name, "accounts-added" | "accounts-renewed")
        })
        .collect();
    assert_eq!(models.len(), rows.len(), "every model sample has a row");
    for (sample, (name, expected)) in models.iter().zip(rows) {
        assert_eq!(sample.name, name);
        assert_eq!(samples::mockup(sample).as_deref(), expected, "{name}");
    }
}

#[test]
fn connection_samples_assign_priority_and_progress_to_the_bundled_nodes() {
    use temper_client_view::look;
    for (name, expected) in [
        ("welcome-forge-choose", vec!["Run on this machine"]),
        ("welcome-forge-details", vec!["Run on this machine", "Run on this machine"]),
        ("welcome-forge-under-way", vec!["Continue"]),
        ("welcome-forge-connected", vec!["Continue"]),
        ("welcome-forge-github", vec!["Continue"]),
        ("welcome-model-choose", vec!["Use your Claude subscription"]),
        ("welcome-model-claude-terminal", vec!["Sign in to Claude Code"]),
        ("welcome-model-claude-signing-in", vec![]),
        ("welcome-model-claude-signed-in", vec!["Continue"]),
        ("welcome-model-claude-no-terminal", vec!["Use your Claude subscription"]),
        ("welcome-model-claude-not-finished", vec!["Sign in to Claude Code"]),
        ("github-app-details", vec!["GitHub"]),
        ("github-install", vec!["Install on GitHub"]),
        ("github-picker", vec![]),
        ("github-adopted", vec![]),
        ("forges-github-install", vec!["Install on GitHub"]),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sample");
        let view = view(&sample);
        let primary: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.styles.as_slice().contains(&look::FLOW_PRIMARY))
            .map(|node| std::str::from_utf8(node.name.as_deref().expect("primary name")).expect("UTF-8"))
            .collect();
        assert_eq!(primary, expected, "{name}");
        if name == "welcome-forge-details" {
            assert_eq!(
                view.tree()
                    .nodes()
                    .iter()
                    .filter(|node| node.styles.as_slice().contains(&look::WELCOME_OPTION_MORE))
                    .count(),
                1
            );
        }
    }
    let sample =
        samples::samples().into_iter().find(|sample| sample.name == "forge-setting-up-middle").expect("sample");
    let view = view(&sample);
    let progress = view
        .tree()
        .nodes()
        .iter()
        .position(|node| node.styles.as_slice().contains(&look::CONNECT_LIVE))
        .expect("progress");
    let nodes = view.tree().nodes();
    let end = progress + usize::try_from(nodes[progress].size).expect("subtree size");
    let items: Vec<_> = nodes[progress..end].iter().filter(|node| node.role == Role::Item).collect();
    assert_eq!(items.len(), 5);
    assert!(items[0].styles.is_empty());
    assert!(items[1].states.as_slice().contains(&skein_ui::State::Current));
    assert!(items[2].styles.as_slice().contains(&look::CONNECT_LATER));
    assert!(items[2].states.is_empty());
}

#[test]
fn asking_gallery_states_list_the_template_and_send_readiness() {
    for (name, sending, disabled, refusal) in [
        ("ask-first-run", false, true, false),
        ("night-ask", false, true, false),
        ("ask-typing", false, false, false),
        ("ask-sending", true, true, false),
        ("ask-refused", false, false, true),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).expect("sample");
        let root = samples::domain(&sample);
        let wall = worlds::now(sample.world);
        assert_eq!(root.pages().address(), &temper_client_domain::Address::Ask { project: 1 }, "{name}");
        let facts = temper_client_domain::pages::home::facts(&root, 1, wall);
        assert!(temper_client_domain::pages::home::beginning(&root, 1, wall), "{name}");
        assert_eq!(facts.left(), 1, "{name}: only the way of working is left");
        assert!(matches!(facts.repository, temper_client_domain::pages::home::Repository::In { .. }), "{name}");
        assert!(
            root.pages()
                .me(root.client())
                .expect("me")
                .projects
                .get(0)
                .expect("project")
                .templates
                .as_slice()
                .iter()
                .any(|t| t.number == 5)
        );
        let chats = root.pages().chats().expect("Ask temper");
        assert_eq!(chats.start.is_some(), sending, "{name}");
        assert_eq!(chats.outcome.is_some(), refusal, "{name}");
        let view = view(&sample);
        let send = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Send"))
            .expect("send");
        assert_eq!(send.states.as_slice().contains(&skein_ui::State::Disabled), disabled, "{name}");
        let how = find(view.tree(), Role::Button, "How does temper work?", name);
        let how = view.tree().find(how).expect("how button");
        assert_eq!(how.states.as_slice().contains(&skein_ui::State::Disabled), sending, "{name}");
        assert!(has_text(view.tree(), "Choose how work runs, or describe how you work below."), "{name}");
    }
}

#[test]
fn new_chat_focuses_the_composer_in_its_print() {
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-new").expect("new chat");
    let view = view(&sample);
    let entry = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"What do you want done?"))
        .expect("composer");
    assert_eq!(entry.focus, Some(0));
    let page = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(page.contains(&format!("data-node=\"{}\" autofocus", entry.id.0)));
}

#[test]
fn goal_and_chat_queries_keep_live_and_ended_rows_separate() {
    use samples::{Then, World};
    use temper_client_domain::{Address, ChatsFilter, GoalsFilter};
    for world in [World::Solo, World::Reference] {
        let (chats, _) = worlds::world(world, Address::Chats { filter: ChatsFilter::All }, Then::Shown);
        let page = chats.pages().chats().expect("chats page");
        let live = &page.live(chats.client()).expect("live chats").rows;
        let jig_client::contract::EndedGroup::Rows { rows: ended, .. } =
            &page.page.as_ref().expect("closed chats").ended
        else {
            panic!("ended chat rows")
        };
        assert!(!live.is_empty() && !ended.is_empty(), "both chat sources have rows");
        for row in live.iter().chain(ended) {
            assert!(row.flags.conversation && row.tracked.is_none(), "{world:?}: chats contain only conversations");
        }
        let (goals, _) = worlds::world(world, Address::Goals { project: 1, filter: GoalsFilter::All }, Then::Shown);
        let page = goals.pages().goals().expect("goals page");
        let live = &page.live(goals.client()).expect("live goals").rows;
        assert!(!live.is_empty() && !page.ended().is_empty(), "both goal sources have rows");
        for row in live.iter().chain(page.ended()) {
            assert!(!row.flags.conversation && row.tracked.is_some(), "{world:?}: goals contain only tracked tasks");
        }
    }
}

#[test]
fn reference_lede_names_the_worlds_templates_in_number_order() {
    use jig_client::contract as c;
    let sample =
        samples::samples().into_iter().find(|sample| sample.name == "goals-reference").expect("reference goals");
    let root = samples::domain(&sample);
    let me = root.pages().me(root.client()).expect("reference me");
    let templates = &me.projects.get(0).expect("reference project").templates;
    let goals: Vec<_> = templates
        .iter()
        .filter(|template| template.tracked.is_some())
        .map(|template| (template.number, template.name.bytes()))
        .collect();
    assert_eq!(
        goals,
        vec![
            (2, b"goal".as_slice()),
            (4, b"goal for a draft".as_slice()),
            (6, b"Plan, then wait for me".as_slice()),
            (7, b"One agent, one branch".as_slice()),
        ]
    );
    assert_eq!(
        templates.iter().find(|template| template.number == 5).expect("Ask temper template").executor,
        c::TemplateExecutor::Charter(11)
    );
    let view = view(&sample);
    let nodes = view.tree().nodes();
    let start = nodes
        .iter()
        .position(|node| node.styles.as_slice().contains(&temper_client_view::look::GOALS_LEDE))
        .expect("goals lede");
    let end = start + usize::try_from(nodes[start].size).expect("lede size");
    let text: Vec<_> =
        nodes[start..end].iter().flat_map(|node| node.text.iter()).flat_map(|run| run.text.iter().copied()).collect();
    assert_eq!(
        String::from_utf8(text).expect("lede UTF-8"),
        "A goal turns a draft into landed work, its own way: task by task, plan, tasks, land whole, plan, then wait for me, one agent, one branch."
    );
}

#[test]
fn each_settings_switch_off_keeps_its_value_and_changes_its_navigation() {
    for (name, off, changes) in [
        ("settings-drafts-off", "Chats can write drafts", true),
        ("settings-goals-off", "Goals", true),
        ("settings-changes-off", "Changes", false),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("switch-off sample");
        let view = view(&sample);
        let tree = view.tree();
        for label in ["Chats can write drafts", "Goals", "Changes"] {
            let check = tree
                .nodes()
                .iter()
                .find(|node| node.role == Role::Check && node.name.as_deref() == Some(label.as_bytes()))
                .expect("exact project switch name");
            assert_eq!(check.states.as_slice().contains(&skein_ui::State::Pressed), label != off, "{name}: {label}");
        }
        let root = samples::domain(&sample);
        let filters = temper_client_domain::pages::work::shown(
            &temper_client_domain::pages::work::Filters {
                goals: 1,
                changes: 1,
                ..temper_client_domain::pages::work::Filters::default()
            },
            temper_client_domain::pages::work::overrides(&root, 1),
        );
        for (label, present, expected) in
            [("Goals", filters.goals, off != "Goals"), ("Changes", filters.changes, changes)]
        {
            assert_eq!(present, expected, "{name}: {label} Work filter");
        }
        let ways = tree
            .nodes()
            .iter()
            .any(|node| node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"Ways of working"));
        assert_eq!(ways, off != "Goals", "{name}: ways");
    }
}

#[test]
fn goal_acts_samples_reach_the_editor_dialog_and_answered_toast() {
    for (name, role, label) in [
        ("goal-amending", Role::Region, "Amend task 108"),
        ("goal-closing", Role::Dialog, "Close this goal?"),
        ("goal-toast", Role::Status, ""),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).unwrap();
        let root = samples::domain(&sample);
        let view = view(&sample);
        if name == "goal-toast" {
            assert_eq!(root.pages().needs().toast(), Some(&temper_client_domain::pages::needs::Toasted::Paused));
            assert!(root.pages().task().unwrap().pending.is_empty());
            assert!(view.tree().nodes().iter().any(|node| node.role == role));
        } else {
            find(view.tree(), role, label, name);
        }
    }
}

#[test]
fn the_open_samples_open_only_their_named_disclosure() {
    for (name, disclosure) in [
        ("chat-model-open", "Model"),
        ("chat-work-open", "Checked settings 3×, read 7 docs"),
        ("rail-all-projects", "vsr"),
        ("rail-across", "All projects"),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).unwrap();
        let view = view(&sample);
        let node = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(disclosure.as_bytes()))
            .unwrap();
        let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
        assert!(html.contains(&format!("data-node=\"{}\" open", node.id.0)), "{name}");
        assert_eq!(html.matches(" open ").count(), 1);
    }
}

#[test]
fn the_rail_menu_sample_opens_the_menu() {
    let sample = samples::samples().into_iter().find(|s| s.name == "rail-new-menu").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("aria-expanded=\"true\""));
    let menu = html.split("role=\"menu\"").nth(1).unwrap().split('>').next().unwrap();
    assert!(!menu.contains("hidden"), "{menu}");
    assert!(html.contains("Ask for one change."));
    assert!(html.contains("Ask about this project, or change how its work runs."));
    assert!(html.contains("A design session that ends in a goal, best had in the terminal."));
}

#[test]
fn the_person_menu_sample_opens_above_its_row() {
    let sample = samples::samples().into_iter().find(|s| s.name == "rail-person-menu").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    let person = html.split("class=\"temper-rail-person\"").nth(1).unwrap();
    assert!(person.contains("aria-expanded=\"true\""));
    let menu = person.split("role=\"menu\"").nth(1).unwrap().split('>').next().unwrap();
    assert!(!menu.contains("hidden"), "{menu}");
    assert!(person.contains("role=\"menuitem\">Accounts</div>"));
    assert!(person.contains("role=\"menuitem\">What temper remembers about you</div>"));
    assert!(person.contains("role=\"menuitem\">Telling you</div>"));
    assert!(person.contains("role=\"menuitem\">Sign out</div>"));
    assert_eq!(person.matches("role=\"menuitem\"").count(), 4);
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let panel = css
        .lines()
        .find(|line| line.starts_with("#app > .temper-rail > .temper-rail-foot > .temper-rail-person > [role=menu] {"))
        .unwrap();
    assert!(panel.contains("bottom: calc(100% + 6px)"));
    assert!(!panel.contains("top:"));
}

#[test]
fn the_accounts_tabs_sign_out_hides_over_a_phones_width_alone() {
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut hiding_rules = 0;
    for (at, character) in css.char_indices() {
        match character {
            '{' => {
                selectors.push(css[start..at].trim());
                start = at + 1;
            }
            '}' => {
                let selector = selectors.pop().unwrap();
                let declarations = css[start..at].split_whitespace().collect::<String>();
                if selector.contains("temper-settings-sign-out") && declarations.contains("display:none") {
                    assert!(
                        selectors.contains(&"@media (min-width: 761px)"),
                        "Sign out must hide only over a phone's width: {selector}"
                    );
                    assert_eq!(selector, ".temper-settings-setting.temper-settings-sign-out");
                    hiding_rules += 1;
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    assert_eq!(hiding_rules, 1, "one desktop-only hiding rule");
}

#[test]
fn the_chat_log_follows_its_end_until_the_person_scrolls_away() {
    use skein_ui_web::page::{Event as PageEvent, Op};
    use temper_client_domain::{Action, Event, pages::Act};
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-solo-T6").expect("chat sample");
    let mut root = samples::domain(&sample);
    let (mut view, bounds, patches) = built(&sample);
    let log = find(view.tree(), Role::Log, "Conversation", sample.name);
    let mut page = Page::new(
        Look {
            names: &[temper_client_view::look::NAMES],
            pictures: &[],
            components: &[],
            elements: temper_client_view::look::ELEMENTS,
            draw,
        },
        &bounds.tree,
        b"temper",
        Primary::Control,
    );
    let mut ops = Vec::new();
    page.apply(&patches, &mut ops);
    assert!(ops.iter().any(|op| matches!(op, Op::ScrollEnd { node } if *node == log)), "initial chat follows its end");
    for (top, follow, words) in
        [(800, true, b"At the end".as_slice()), (100, false, b"Reading earlier turns"), (800, true, b"Back at the end")]
    {
        ops.clear();
        assert!(page.act(PageEvent::Scrolled { node: log, top, height: 1000, viewport: 200 }, &mut ops).is_none());
        let mut out = Queue::uncapped();
        driver::step(
            &mut root,
            Event::Act { action: Action::Pages(Act::MessageWords { words: words.into() }) },
            &mut out,
        );
        if words == b"At the end" {
            driver::step(&mut root, Event::Act { action: Action::Pages(Act::Send) }, &mut out);
        }
        let mut patches = Queue::with_capacity(bounds.tree.patches);
        temper_client_view::build(&mut view, &root, worlds::sample_now(sample.world, sample.name), &mut patches);
        let patches: Vec<_> = (0..patches.len()).map(|_| patches.pop().expect("queued patch")).collect();
        assert!(!patches.is_empty(), "the chat changes");
        page.apply(&patches, &mut ops);
        assert_eq!(ops.iter().any(|op| matches!(op, Op::ScrollEnd { node } if *node == log)), follow);
    }
}

#[test]
fn home_away_deliveries_draw_the_mock_words_and_off_span() {
    use temper_client_view::look;
    for (name, count, headline, expected_kicker, overturns, landed, overturned) in [
        ("home-decided", 2, "57 changes landed. One thing needs you.", "Tuesday, 08:04", 2, false, false),
        (
            "home-away",
            4,
            "57 changes landed. Four things were decided for you.",
            "While you were away · Mon 23:10 to now",
            4,
            false,
            false,
        ),
        (
            "home-away-one",
            1,
            "57 changes landed. One thing was decided for you.",
            "While you were away · Mon 23:10 to 08:03",
            0,
            true,
            false,
        ),
        (
            "home-away-overturned",
            4,
            "57 changes landed. Four things were decided for you.",
            "While you were away · Mon 23:10 to now",
            2,
            true,
            true,
        ),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).expect("Home away sample");
        let view = view(&sample);
        let tree = view.tree();
        assert!(
            tree.nodes()
                .iter()
                .any(|n| n.role == (Role::Heading { level: 1 }) && n.name.as_deref() == Some(headline.as_bytes()))
        );
        assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::DECISION)).count(), count);
        assert!(
            tree.nodes().iter().any(|n| n.name.as_deref() == Some("welcome-stall · 01:05".as_bytes())),
            "the decision task read supplies its title"
        );
        if landed {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some("landed 03:12 · a new goal can undo it".as_bytes())));
        }
        let kicker =
            tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::HOME_KICKER)).expect("away kicker");
        assert_eq!(kicker.name.as_deref(), Some(expected_kicker.as_bytes()));
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_ALLOWED)
            && n.name.as_deref() == Some("Allowed: implementation choices are decided while you’re away.".as_bytes())));
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_TOOK)
            && n.name.as_deref() == Some(b"In temper serve".as_slice())));
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Overturn".as_slice()))
                .count(),
            overturns
        );
        if overturned {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some(b"Overturned: it asks you again".as_slice())));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_OVERTURNED)));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some("landed 03:12 · a new goal can undo it".as_bytes())));
        }
    }
}

#[test]
fn agent_work_runs_its_line_and_reads_away_from_the_end() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "agent-work").expect("agent-work");
    assert_eq!(sample.address, temper_client_domain::Address::Task { task: 1 });
    let view = view(&sample);
    let styles = temper_client_view::look::WORK_RUNNING;
    let line = view.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&styles)).expect("running run");
    assert_eq!(line.description.as_deref(), Some(b"checking template 2".as_slice()));
    assert!(!view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&temper_client_view::look::TYPING)));
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("temper-agent-page"));
    assert!(html.contains("data-log-away"));
    assert!(html.contains("temper-latest"));
}

#[test]
fn the_browser_draws_each_model_name_once() {
    const ONCE: usize = 1; // One row of each model under the single account.
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-model-open").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert_eq!(html.matches(">Opus 5.5").count(), ONCE);
    assert_eq!(html.matches(">Sonnet 5.5").count(), ONCE);
}

#[test]
fn a_settings_cards_later_reply_follows_it() {
    for (name, expected) in [
        ("ask-chat-accepted", &["Card 1", "Reply"][..]),
        ("ask-chat-replaced", &["Card 1", "Reply", "Card 2", "Reply"][..]),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).unwrap();
        let view = view(&sample);
        let mut order = Vec::new();
        for node in view.tree().nodes() {
            if let Some(key) = node.key
                && key.owner == temper_client_view::frame::owner()
                && key.kind == 1
                && key.a == 10
            {
                order.push(if key.b == 1 { "Card 1" } else { "Card 2" });
            }
            if node.text.iter().any(|run| run.text.as_ref() == b"Tell me what you would change.") {
                order.push("Reply");
            }
        }
        assert_eq!(order, expected, "{name}");
    }
}

#[test]
fn finding_gallery_cards_keep_facts_forms_and_responsive_hooks() {
    use temper_client_domain::{
        Action,
        pages::{
            Act,
            findings::{Act as F, Id},
        },
    };
    use temper_client_view::look;
    const ID: Id = Id { project: 1, number: 7 };
    for sample in samples::samples().into_iter().filter(|sample| sample.name.starts_with("needs-finding")) {
        let root = samples::domain(&sample);
        assert_eq!(temper_client_domain::pages::parts::needs_facts(&root, worlds::now(samples::World::Solo)).open, 1);
        let (mut view, _, _) = built_root(&sample, ViewInputs::new(), &root);
        for (role, name) in [
            (Role::Text, "Finding"),
            (Role::Paragraph, "feedback · high · no area"),
            (Role::Paragraph, "Handed on by task 55: “Your area owns the view”"),
            (Role::Button, "Backlog"),
        ] {
            find(view.tree(), role, name, sample.name);
        }
        assert!(view.tree().nodes().iter().any(|node| {
            node.name
                .as_deref()
                .is_some_and(|name| name.starts_with("fabrig › 1/7 The phone card is clipped".as_bytes()))
        }));
        // The browser's single tree carries both the laptop list and the phone's one-card bar.
        for style in [look::ASK, look::ASK_CHOICES, look::NEEDS_BAR, look::NEEDS_HEAD] {
            assert!(
                view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&style)),
                "{}: responsive hook {style:?}",
                sample.name
            );
        }
        let backlog = find(view.tree(), Role::Button, "Backlog", sample.name);
        assert_eq!(
            temper_client_view::act(&mut view, backlog, skein_ui::Act::Press),
            Some(Action::Pages(Act::Finding(F::Backlog { id: ID })))
        );
        if sample.name == "needs-finding-hand" {
            find(view.tree(), Role::Button, "view", sample.name);
            let field = find(view.tree(), Role::TextField, "Why hand it on", sample.name);
            assert_eq!(
                temper_client_view::act(
                    &mut view,
                    field,
                    skein_ui::Act::Input { value: b"The view owns this".as_slice().into() }
                ),
                Some(Action::Pages(Act::Finding(F::Why { id: ID, words: b"The view owns this".as_slice().into() })))
            );
        }
        if sample.name == "needs-finding-drop" {
            let field = find(view.tree(), Role::TextField, "Reason to drop", sample.name);
            assert_eq!(
                temper_client_view::act(
                    &mut view,
                    field,
                    skein_ui::Act::Input { value: b"Duplicate".as_slice().into() }
                ),
                Some(Action::Pages(Act::Finding(F::Reason { id: ID, words: b"Duplicate".as_slice().into() })))
            );
        }
        if sample.name == "needs-finding-route" {
            assert!(
                !root.pages().needs().findings().goals(ID).is_empty(),
                "the live-goal menu has the project's goals"
            );
        }
        if sample.name == "needs-finding-refused" {
            find(view.tree(), Role::Paragraph, "not a live goal of the project", sample.name);
        }
    }
}

#[test]
fn role_talk_shows_provers_open_talk_sent_words_reply_and_leave() {
    use temper_client_view::look;
    let sample = samples::samples().into_iter().find(|sample| sample.name == "role-talk").expect("role-talk");
    assert_eq!(sample.address, temper_client_domain::Address::Task { task: 52 });
    assert!(!samples::samples().iter().any(|sample| sample.name == "night-role-talk"));
    let root = samples::domain(&sample);
    let task = root.pages().task().expect("role's page").document(root.client()).expect("task");
    assert_eq!(task.flags.ended_by, jig_client::contract::EndedBy::Standing);
    assert_eq!(task.talk.as_ref().unwrap().party, root.pages().me(root.client()).unwrap().party.number);
    let view = view(&sample);
    let tree = view.tree();
    find(tree, Role::Heading { level: 1 }, "T52 · Prover", sample.name);
    find(tree, Role::TextArea, "Talk", sample.name);
    find(tree, Role::Button, "Leave", sample.name);
    find(tree, Role::Status, "Sent", sample.name);
    let said: Vec<u8> =
        tree.nodes().iter().flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    let said = String::from_utf8(said).unwrap();
    for words in [
        "From now on, reproduce a flake twice before routing it.",
        "Add it to my rules?",
        "Talk with Prover · it answers here, and the talk stays open until you leave or 30 minutes pass with nothing said.",
        "Or in a terminal: temper ctl DIR role talk 52 TEXT --key KEY",
    ] {
        assert!(said.contains(words), "missing {words}: {said}");
    }
    let log = tree.nodes().iter().position(|node| node.role == Role::Log).unwrap();
    let reply = tree
        .nodes()
        .iter()
        .position(|node| node.text.iter().any(|run| run.text.as_ref() == b"Add it to my rules?"))
        .unwrap();
    assert!(reply > log && reply < log + usize::try_from(tree.nodes()[log].size).unwrap());
    assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&look::AGENT_PAGE)));
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("Talk to Prover"));
    assert!(html.contains("temper-agent-page"));
}
