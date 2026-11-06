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
    let promised: Vec<&str> = include_str!("../names.txt").lines().collect();
    assert_eq!(names, promised);
}

#[test]
fn a_later_settings_card_replaces_a_withdrawn_one_without_call_objects() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "configure-chat-replaced").unwrap();
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
    ] {
        assert_eq!(names.style(skein_ui::Style { owner: names.owner, token }), Some(expected));
    }
}

#[test]
fn navigation_links_have_addresses_and_install_action_links_keep_their_binding() {
    for sample in samples::samples() {
        let mut view = view(&sample);
        let mut links = Vec::new();
        for node in view.tree().nodes() {
            if node.role == Role::Link {
                assert!(node.link.is_some() || node.external.is_some(), "{}: link {:?}", sample.name, node.name);
                links.push(node.id);
            }
        }
        for node in links {
            if let Some(address) = temper_client_view::href(&mut view, node) {
                assert!(!address.is_empty(), "{}: empty href for {node:?}", sample.name);
            } else if let Some(temper_client_domain::Action::Forge(temper_client_domain::forge::Act::Install {
                forge,
                ..
            })) = temper_client_view::act(&mut view, node, skein_ui::Act::Press)
            {
                // The existing picker link starts an asynchronous install request;
                // it has no static destination until the forge answers.
                assert_eq!(sample.name, "github-picker");
                assert_eq!(forge, 2);
            } else if let Some(temper_client_domain::Action::Forge(temper_client_domain::forge::Act::EndPlayground)) =
                temper_client_view::act(&mut view, node, skein_ui::Act::Press)
            {
                let reset = view.tree().nodes().iter().find(|shown| shown.id == node).expect("bound reset link");
                assert!(matches!(reset.name.as_deref(), Some(b"set up temper" | b"Set up temper")));
            } else if let Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Needs(
                temper_client_domain::pages::needs::Act::Open { object },
            ))) = temper_client_view::act(&mut view, node, skein_ui::Act::Press)
            {
                assert!(
                    matches!(sample.address, temper_client_domain::Address::Home { .. })
                        || matches!(sample.name, "rail-away-on" | "rail-away-off")
                );
                assert!(
                    samples::domain(&sample)
                        .pages()
                        .inbox()
                        .expect("Home inbox")
                        .cards
                        .iter()
                        .any(|card| card.object == object)
                );
            } else {
                panic!("{}: link {node:?} has neither a destination nor its action", sample.name);
            }
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
    Pressed { name: &'static str },
    Text { text: &'static str },
    Named { role: Role, name: &'static str },
    Described { role: Role, name: &'static str, description: &'static str },
    Absent { role: Role, name: &'static str },
    NoRole { role: Role },
    Value { role: Role, name: &'static str, value: &'static str },
    Chats { live: u32, closed: u32 },
    Goals { rows: &'static [&'static str] },
    Plan { live: u32, title: &'static str },
}
const HOLDS: [Holds; samples::N] = [
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
            Part::Text { text: "Sending resumes this chat." },
            Part::Absent { role: Role::Button, name: "Pause" },
            Part::Absent { role: Role::Status, name: "Applied" },
        ],
    },
    Holds {
        name: "chat-closed",
        heading: "T6 · Why is tally sum slow on big files?",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "Closed at 21:05" },
            Part::Named { role: Role::Status, name: "Closed" },
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
        heading: "tally",
        parts: &[
            Part::Named { role: Role::Menu, name: "More ways to start work" },
            Part::Described { role: Role::MenuItem, name: "Chat", description: "Ask a question or ask for a change." },
            Part::Described {
                role: Role::MenuItem,
                name: "Goal",
                description: "Agents plan a larger change and land it.",
            },
            Part::Named { role: Role::MenuItem, name: "All ways to start work" },
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
        name: "home-away",
        heading: "57 changes landed. Four things were decided for you.",
        parts: &[
            Part::Named { role: Role::Region, name: "Decided for you · 4" },
            Part::Named { role: Role::Button, name: "Limits" },
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
        heading: "41 changes landed overnight. Three things need you.",
        parts: &[
            Part::Named { role: Role::Button, name: "Away: off" },
            Part::Text { text: "Away is off: questions wait for you again." },
        ],
    },
    Holds {
        name: "rail-away-on",
        heading: "41 changes landed overnight. Three things need you.",
        parts: &[
            Part::Named { role: Role::Button, name: "Away until 08:00" },
            Part::Text { text: "Away until 08:00: agents decide within Hello’s limits." },
            Part::Named { role: Role::Button, name: "Limits" },
        ],
    },
    Holds { name: "start-goal", heading: "Talk to an agent", parts: &[Part::NoRole { role: Role::Dialog }] },
    Holds {
        name: "start-goal-draft",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Dialog, name: "Start a goal from away-mode.md" },
            Part::Value { role: Role::TextField, name: "Name", value: "away-mode" },
            Part::Named { role: Role::ChoiceGroup, name: "How it runs" },
            Part::Named { role: Role::ChoiceGroup, name: "When it needs you" },
            Part::Named { role: Role::Choice, name: "Follow the away switch" },
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
            Part::Text { text: "Plan, tasks, land whole · follows the away switch" },
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
        heading: "6 of 9 tasks landed.",
        parts: &[Part::Text { text: "running" }, Part::Named { role: Role::Figure, name: "6 landed of 9" }],
    },
    Holds {
        name: "chat-draft-empty",
        heading: "T1 · Away mode, draft",
        parts: &[
            Part::Named { role: Role::Region, name: "untitled.md" },
            Part::Named { role: Role::Paragraph, name: "The draft appears here as you write it together." },
            Part::Absent { role: Role::Button, name: "+ Draft" },
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
        name: "builders",
        heading: "Where agents work.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Builders · 3" },
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
            Part::Named { role: Role::Heading { level: 2 }, name: "Builders · 0" },
            Part::Text { text: "No builder is connected." },
        ],
    },
    Holds {
        name: "builders-not-owner",
        heading: "Where agents work.",
        parts: &[Part::Text { text: "Builders are seen by the deployment’s owners." }],
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
        name: "forges-setting-up",
        heading: "Where your code lives.",
        parts: &[
            Part::Described {
                role: Role::Disclosure,
                name: "Forgejo on this machine",
                description: "Forge 3 · Forgejo · Setting up: preparing\nwork/hello",
            },
            Part::Named { role: Role::Link, name: "Open its set-up ›" },
        ],
    },
    Holds {
        name: "forges-fetching",
        heading: "Where your code lives.",
        parts: &[Part::Described {
            role: Role::Disclosure,
            name: "Forgejo on this machine",
            description: "Forge 3 · Forgejo · Setting up: fetching Forgejo · 42%\nwork/hello",
        }],
    },
    Holds {
        name: "forges-fetching-bytes",
        heading: "Where your code lives.",
        parts: &[Part::Described {
            role: Role::Disclosure,
            name: "Forgejo on this machine",
            description: "Forge 3 · Forgejo · Setting up: fetching Forgejo · 41 MB\nwork/hello",
        }],
    },
    Holds {
        name: "forges-failed",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Button, name: "Set up again" },
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
        name: "needs-one",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[Part::Named { role: Role::Button, name: "Later" }],
    },
    Holds {
        name: "needs-saw-open",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[Part::Named { role: Role::Button, name: "Later" }],
    },
    Holds {
        name: "needs-reply-open",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[
            Part::Named { role: Role::Button, name: "Later" },
            Part::Named { role: Role::TextArea, name: "Your reply" },
        ],
    },
    Holds {
        name: "needs-second-answered",
        heading: "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?",
        parts: &[
            Part::Named { role: Role::Button, name: "Later" },
            Part::Named { role: Role::Button, name: "Undo" },
            Part::Text { text: "Answered: In jig’s store" },
        ],
    },
    Holds {
        name: "needs-later",
        heading: "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?",
        parts: &[
            Part::Named { role: Role::Button, name: "Later" },
            Part::Text { text: "Later: it stays in Needs you, after the others." },
        ],
    },
    Holds {
        name: "needs-last",
        heading: "The repository has no license. Name one in the README?",
        parts: &[Part::Named { role: Role::Button, name: "Later" }, Part::Text { text: "The last one" }],
    },
    Holds {
        name: "needs-done",
        heading: "Nothing needs you.",
        parts: &[Part::Named { role: Role::Link, name: "Home" }],
    },
    Holds {
        name: "needs-proposal",
        heading: "Accept how temper will run your work.",
        parts: &[
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Disclosure, name: "Reject" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Named { role: Role::Button, name: "Later" },
            Part::Text { text: "Funding after: 34" },
        ],
    },
    Holds {
        name: "needs-held",
        heading: "first-run-projects",
        parts: &[
            Part::Named { role: Role::Button, name: "Release" },
            Part::Named { role: Role::Disclosure, name: "Leave held" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Named { role: Role::Button, name: "Later" },
            Part::Text { text: "Both halves of the store task run. The Ready page waits for this decision." },
        ],
    },
    Holds {
        name: "needs-funds",
        heading: "31 of 40 used.",
        parts: &[
            Part::Named { role: Role::Button, name: "Release" },
            Part::Named { role: Role::Disclosure, name: "Leave held" },
            Part::Named { role: Role::TextArea, name: "Reason" },
            Part::Named { role: Role::Button, name: "Open" },
            Part::Named { role: Role::Button, name: "Later" },
            Part::Absent { role: Role::Button, name: "Add $10" },
            Part::Text { text: "first-run-projects" },
        ],
    },
    Holds {
        name: "needs-plain-words",
        heading: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
        parts: &[
            Part::Named { role: Role::Button, name: "Later" },
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
        name: "home-three-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-two-left",
        heading: "Hello is ready.",
        parts: &[
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
        name: "home-one-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Text { text: "work/hello is in Hello" },
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-with-goals",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Link, name: "Goals" },
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
            Part::Named { role: Role::Button, name: "Reply" },
            Part::Absent { role: Role::Heading { level: 2 }, name: "To begin" },
        ],
    },
    Holds {
        name: "home-at-work",
        heading: "One thing needs you.",
        parts: &[
            Part::Named { role: Role::Link, name: "Needs you 1" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 1" },
            Part::Named { role: Role::Paragraph, name: "The repository has no license. Name one in the README?" },
            Part::Named { role: Role::Button, name: "Reply" },
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
            Part::Named { role: Role::Link, name: "Needs you 3" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 3" },
            Part::Named { role: Role::Link, name: "Answer in a row" },
            Part::Named {
                role: Role::Link,
                name: "The repository has no license. Name one in the README? · a question · 4 min",
            },
            Part::Named {
                role: Role::Link,
                name: "The repository has no license. Name one in the README? · a question · 4 min",
            },
            Part::Named { role: Role::Link, name: "All 3 chats" },
        ],
    },
    Holds {
        name: "home-nothing-needs",
        heading: "Nothing needs you.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you" },
            Part::Named {
                role: Role::Paragraph,
                name: "Nothing needs you. Work carries on, and this page tells you what changed when you come back.",
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
                text: "Chats can change their code now, on your Forgejo; nothing goes back where they were copied from.",
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
                text: "Chats can change its code now, on your Forgejo; nothing goes back where it was copied from.",
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
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you" },
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
        name: "home-needs-under-begin",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "To begin" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Needs you · 1" },
            Part::Named { role: Role::Paragraph, name: "The repository has no license. Name one in the README?" },
            Part::Named { role: Role::Link, name: "All 1 chats" },
        ],
    },
    Holds {
        name: "home-begin-settings-left",
        heading: "Hello is ready.",
        parts: &[
            Part::Named {
                role: Role::Paragraph,
                name: "Two steps left. Skip them if you like; they stay in Settings.",
            },
            Part::Named { role: Role::Button, name: "Start" },
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Absent { role: Role::Link, name: "New chat" },
        ],
    },
    Holds {
        name: "home-begin-one-chats",
        heading: "Hello is ready.",
        parts: &[
            Part::Named { role: Role::Paragraph, name: "One step left. Skip it if you like; it stays in Chats." },
            Part::Named { role: Role::Link, name: "New chat" },
            Part::Absent { role: Role::Button, name: "Start" },
            Part::Absent { role: Role::Button, name: "Adopt" },
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
            Part::Text { text: "96 of 112 tasks landed." },
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
            Part::Text { text: "0 of 1 tasks landed." },
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
            Part::Named {
                role: Role::Paragraph,
                name: "Fix the dropped watch in jig’s store, or keep it alive in temper serve?",
            },
            Part::Named { role: Role::Button, name: "In jig’s store" },
            Part::Named { role: Role::Disclosure, name: "What the agent saw" },
            Part::Named { role: Role::Button, name: "Reply" },
            Part::Named { role: Role::Button, name: "Later" },
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
            Part::Named { role: Role::Button, name: "MIT" },
            Part::Named { role: Role::Button, name: "Later" },
            Part::Named {
                role: Role::Link,
                name: "Add a README: The repository has no license. Name one in the README? Answer",
            },
            Part::Absent { role: Role::Link, name: "Answer in a row" },
        ],
    },
    Holds {
        name: "welcome-model-forge-under-way",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Progress, name: "Setting up: preparing" },
            Part::Named { role: Role::Complementary, name: "Forge" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
        ],
    },
    Holds {
        name: "welcome-model-forge-fetching",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Progress, name: "Fetching Forgejo" },
            Part::Named { role: Role::Complementary, name: "Forge" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Text { text: "42%" },
            Part::Text { text: "Fetching Forgejo 17.0.0 and its runner 13.2.0 · 42%" },
        ],
    },
    Holds {
        name: "welcome-model-forge-fetching-bytes",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Progress, name: "Fetching Forgejo" },
            Part::Named { role: Role::Complementary, name: "Forge" },
            Part::Named { role: Role::Disclosure, name: "Each step" },
            Part::Text { text: "41 MB" },
            Part::Text { text: "Fetching Forgejo 17.0.0 and its runner 13.2.0 · 41 MB" },
        ],
    },
    Holds {
        name: "welcome-model-forge-done",
        heading: "Connect a model",
        parts: &[
            Part::NoRole { role: Role::Progress },
            Part::Named { role: Role::Complementary, name: "Forge" },
            Part::Absent { role: Role::Disclosure, name: "Each step" },
        ],
    },
    Holds {
        name: "welcome-model-forge-failed",
        heading: "Connect a model",
        parts: &[
            Part::NoRole { role: Role::Progress },
            Part::Named { role: Role::Complementary, name: "Forge" },
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
            Part::Named { role: Role::Status, name: "This sign-in has ended." },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
            Part::Absent { role: Role::TextField, name: "Address of the page that couldn’t be reached" },
        ],
    },
    Holds {
        name: "welcome-model-declined",
        heading: "Signing in to ChatGPT",
        parts: &[Part::Named { role: Role::Button, name: "Continue with ChatGPT" }],
    },
    Holds {
        name: "welcome-model-renewed",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Link, name: "Continue" }],
    },
    Holds {
        name: "welcome-model-refused",
        heading: "Signing in to ChatGPT",
        parts: &[Part::Named { role: Role::Button, name: "Continue with ChatGPT" }],
    },
    Holds {
        name: "welcome-paste-waiting",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-written",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-too-long",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-sending",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-finishing",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Status, name: "Finishing your sign-in…" }],
    },
    Holds {
        name: "welcome-paste-address",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-state",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-too-many",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Button, name: "Continue with ChatGPT" }],
    },
    Holds {
        name: "welcome-paste-ended",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Button, name: "Continue with ChatGPT" }],
    },
    Holds {
        name: "welcome-paste-lost",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "welcome-paste-unsent",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Address of the page that couldn’t be reached" }],
    },
    Holds {
        name: "forges-github-install",
        heading: "Where your code lives.",
        parts: &[Part::Named { role: Role::Button, name: "Install on GitHub" }],
    },
    Holds {
        name: "forge-admin-details",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Region, name: "Your first repository" },
            Part::Named { role: Role::Heading { level: 3 }, name: "Your first repository" },
            Part::Named { role: Role::TextField, name: "Runner label" },
        ],
    },
    Holds {
        name: "forge-setup-failed",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Status, name: "failed" },
            Part::Named { role: Role::List, name: "Each step" },
            Part::Named { role: Role::Button, name: "Set up again" },
        ],
    },
    Holds {
        name: "welcome-setup-failed",
        heading: "Almost ready",
        parts: &[
            Part::Named { role: Role::Status, name: "failed" },
            Part::Named { role: Role::List, name: "Each step" },
            Part::Named { role: Role::Button, name: "Set up again" },
        ],
    },
    Holds {
        name: "welcome-model-claude",
        heading: "Your Claude subscription",
        parts: &[
            Part::Text { text: "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat" },
            Part::Named { role: Role::Button, name: "I’ve signed in" },
            Part::Absent { role: Role::TextField, name: "Name" },
        ],
    },
    Holds {
        name: "welcome-ready-checking",
        heading: "Ready",
        parts: &[
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
        parts: &[Part::Named { role: Role::TextField, name: "Key, pasted once" }],
    },
    Holds {
        name: "welcome-model-connected",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Status, name: "ChatGPT plan · sam@example.com" },
            Part::Named { role: Role::Link, name: "Continue" },
        ],
    },
    Holds {
        name: "welcome-model-away",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "welcome-model-signing-in",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "welcome-model-signin-ready",
        heading: "Connect a model",
        parts: &[
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
        heading: "Connect a forge",
        parts: &[Part::Named { role: Role::Button, name: "Install on GitHub" }],
    },
    Holds {
        name: "github-picker",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "GitHub is connected" },
            Part::Named { role: Role::Link, name: "Adopt a repository" },
        ],
    },
    Holds {
        name: "github-adopted",
        heading: "Connect a forge",
        parts: &[Part::Named { role: Role::Link, name: "Adopt a repository" }],
    },
    Holds {
        name: "welcome-forge-choose",
        heading: "Connect a forge",
        parts: &[
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
            Part::Named { role: Role::Dialog, name: "Forgejo on this machine" },
            Part::Named { role: Role::Choice, name: "Nothing for now" },
            Part::Named { role: Role::Button, name: "Run on this machine" },
        ],
    },
    Holds {
        name: "welcome-forge-under-way",
        heading: "Connect a forge",
        parts: &[
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
            Part::Named { role: Role::Heading { level: 2 }, name: "ChatGPT" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Claude" },
            Part::Named { role: Role::Paragraph, name: "Sign in, in a new tab · about a minute" },
            Part::Named { role: Role::Button, name: "Continue with ChatGPT" },
        ],
    },
    Holds {
        name: "forge-details",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::TextField, name: "Organization" },
            Part::Named { role: Role::TextField, name: "Repository" },
            Part::Absent { role: Role::TextField, name: "Into a new project, named" },
        ],
    },
    Holds {
        name: "forge-setting-up-middle",
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Status, name: "under way" },
            Part::Named { role: Role::List, name: "Each step" },
        ],
    },
    Holds {
        name: "welcome-ready-copies",
        heading: "Ready",
        parts: &[
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
        heading: "Connect a forge",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Setting up Forgejo on this machine" },
            Part::Named { role: Role::Status, name: "under way" },
            Part::Named { role: Role::Paragraph, name: "work/hello" },
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
            Part::Named { role: Role::Region, name: "Forgejo on this machine" },
            Part::Named { role: Role::Region, name: "Your Forgejo" },
        ],
    },
    Holds {
        name: "welcome-go-on",
        heading: "Welcome to temper",
        parts: &[
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
            Part::Named { role: Role::List, name: "Steps" },
            Part::Named { role: Role::Region, name: "Try the playground" },
            Part::Named { role: Role::Button, name: "Try the playground" },
            Part::Named { role: Role::Region, name: "Set up temper" },
            Part::Named { role: Role::Link, name: "Set up temper" },
        ],
    },
    Holds {
        name: "configure",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Region, name: "Ways to begin" },
            Part::Value { role: Role::TextArea, name: "Describe how you work, or what to change", value: "" },
        ],
    },
    Holds {
        name: "configure-reference",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Group, name: "How temper builds temper, beside the chat" },
            Part::Pressed { name: "Split" },
            Part::Named { role: Role::Region, name: "How temper builds temper" },
            Part::Text {
                text: "Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change.",
            },
        ],
    },
    Holds {
        name: "configure-reference-draft",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Group, name: "How temper builds temper, beside the chat" },
            Part::Pressed { name: "Draft" },
            Part::Named { role: Role::Region, name: "How temper builds temper" },
            Part::Text {
                text: "Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change.",
            },
        ],
    },
    Holds {
        name: "configure-typing",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Region, name: "Ways to begin" },
            Part::Value {
                role: Role::TextArea,
                name: "Describe how you work, or what to change",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "configure-sending",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Region, name: "Ways to begin" },
            Part::Value {
                role: Role::TextArea,
                name: "Describe how you work, or what to change",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "configure-refused",
        heading: "Configure temper",
        parts: &[
            Part::Named { role: Role::Region, name: "Ways to begin" },
            Part::Value {
                role: Role::TextArea,
                name: "Describe how you work, or what to change",
                value: "I work alone on small changes.",
            },
        ],
    },
    Holds {
        name: "configure-chat-document",
        heading: "T10 · Configure temper",
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
        name: "configure-chat-document-split",
        heading: "T10 · Configure temper",
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
        name: "configure-chat",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Named { role: Role::Navigation, name: "Where" },
            Part::Named { role: Role::TextArea, name: "Message" },
            Part::Named { role: Role::Button, name: "Close" },
            Part::Absent { role: Role::Disclosure, name: "Model" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "configure-chat-proposal-deployment",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run your projects" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "configure-chat-proposal-two-projects",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally and 2" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "configure-chat-proposal",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Named { role: Role::Heading { level: 5 }, name: "temper enforces" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: answered by the goal" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Button, name: "Accept" },
            Part::Named { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "configure-chat-accepted",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: answered by the goal" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "✓ Accepted · new work follows it" },
            Part::Absent { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "configure-chat-rejected",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: answered by the goal" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Rejected · nothing changes" },
            Part::Absent { role: Role::Paragraph, name: "Waiting for an owner" },
            Part::Absent { role: Role::Button, name: "Accept" },
            Part::Absent { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "configure-chat-withdrawn",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: answered by the goal" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Withdrawn · nothing changes" },
            Part::Absent { role: Role::Paragraph, name: "Waiting for an owner" },
            Part::Absent { role: Role::Button, name: "Accept" },
            Part::Absent { role: Role::Button, name: "Change something" },
        ],
    },
    Holds {
        name: "configure-chat-replaced",
        heading: "T10 · Add a --json flag to tally report",
        parts: &[
            Part::Text { text: "How temper will run tally" },
            Part::Text { text: "Drafts: on" },
            Part::Text { text: "Questions: answered by the goal" },
            Part::Text { text: "Way: Plan, tasks, land whole" },
            Part::Named { role: Role::Paragraph, name: "Replaced by the proposal below" },
            Part::Named { role: Role::Button, name: "Accept" },
        ],
    },
    Holds {
        name: "configure-chats-tag",
        heading: "Talk to an agent",
        parts: &[Part::Text { text: "Configuring" }, Part::Chats { live: SOLO_LIVE, closed: SOLO_CLOSED }],
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
            Part::Named { role: Role::Navigation, name: "Settings" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Repositories · 1" },
            Part::Named { role: Role::Text, name: "robin/tally" },
            Part::Named { role: Role::Link, name: "Adopt a repository" },
        ],
    },
    Holds {
        name: "repositories-none",
        heading: "What gallery works on.",
        parts: &[
            Part::Named { role: Role::Text, name: "No repository yet" },
            Part::Text { text: "Chats in gallery talk and plan, but change no code until a repository is in." },
            Part::Named { role: Role::Link, name: "Adopt a repository" },
        ],
    },
    Holds {
        name: "forges-in-settings",
        heading: "Where your code lives.",
        parts: &[
            Part::Named { role: Role::Navigation, name: "Settings" },
            Part::Named { role: Role::Heading { level: 2 }, name: "Your forges · 1" },
        ],
    },
    Holds {
        name: "accounts-in-settings",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 1" },
            Part::Named { role: Role::Navigation, name: "Settings" },
            Part::Named { role: Role::Text, name: "Signed in as Robin" },
            Part::Named { role: Role::Button, name: "Sign out" },
        ],
    },
    Holds {
        name: "settings-away",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "While you’re away, agents may decide" },
            Part::Named { role: Role::Check, name: "Implementation choices within a task" },
            Part::Named { role: Role::Select, name: "Away until" },
        ],
    },
    Holds {
        name: "settings",
        heading: "How work runs in tally.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Ways of working" },
            Part::Named { role: Role::Check, name: "Away switch" },
            Part::Named { role: Role::Link, name: "Builders" },
            Part::Named { role: Role::Text, name: "Configure temper" },
            Part::Named { role: Role::Link, name: "Forges" },
            Part::Named { role: Role::Link, name: "Accounts" },
            Part::Named { role: Role::Link, name: "Start" },
            Part::Named { role: Role::Navigation, name: "Settings" },
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
        name: "settings-away-off",
        heading: "How work runs in tally.",
        parts: &[Part::Named { role: Role::Check, name: "Away switch" }],
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
            Part::Named { role: Role::Link, name: "Go on" },
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
        heading: "0 of 2 tasks landed.",
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
            Part::Named { role: Role::Heading { level: 3 }, name: "Claude" },
            Part::Named { role: Role::Item, name: "Claude Code login claude-robin · Ready" },
            Part::Named { role: Role::Item, name: "API key · Ready" },
            Part::Named { role: Role::Paragraph, name: "laptop" },
        ],
    },
    Holds {
        name: "chats-first-run-1",
        heading: "Talk to an agent",
        parts: &[
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
        name: "adoption",
        heading: "Adopt a repository into factory",
        parts: &[Part::Value { role: Role::TextField, name: "Owner", value: "" }],
    },
    Holds {
        name: "adoption-filled",
        heading: "Adopt a repository into factory",
        parts: &[
            Part::Named { role: Role::Button, name: "Adopt" },
            Part::Value { role: Role::TextField, name: "Owner", value: "acme" },
        ],
    },
    Holds {
        name: "adoption-refused",
        heading: "Adopt a repository into factory",
        parts: &[Part::Named { role: Role::Alert, name: "" }],
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
            Part::Named { role: Role::Status, name: "Closed" },
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
        name: "configure-model-open",
        heading: "Configure temper",
        parts: &[Part::Described { role: Role::Disclosure, name: "Model", description: "Claude Opus 5.5 · high" }],
    },
    Holds { name: "chat-working", heading: "T6", parts: &[Part::Named { role: Role::Slot, name: "Writing" }] },
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
            Part::Named { role: Role::Text, name: "No account yet" },
            Part::Named { role: Role::Link, name: "Connect a model" },
        ],
    },
    Holds {
        name: "accounts-reference",
        heading: "What agents run on.",
        parts: &[
            Part::Named { role: Role::Heading { level: 2 }, name: "Your accounts · 2" },
            Part::Named { role: Role::Heading { level: 3 }, name: "Claude" },
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
        parts: &[Part::Named { role: Role::Link, name: "Continue with Google" }],
    },
    Holds {
        name: "signed-out",
        heading: "",
        parts: &[Part::Named {
            role: Role::Paragraph,
            name: "Signed out. To sign in, open a sign-in link temper printed.",
        }],
    },
    Holds {
        name: "out-of-date",
        heading: "",
        parts: &[Part::Named {
            role: Role::Paragraph,
            name: "This temper is out of date: run the temper the engine runs.",
        }],
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
        parts: &[Part::Named { role: Role::Region, name: "Choose" }],
    },
    Holds {
        name: "connect-model-key-details",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Key (pasted once)" }],
    },
    Holds {
        name: "connect-model-program-details",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::TextField, name: "Name" }],
    },
    Holds {
        name: "connect-model-key-ready",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Status, name: "Added your OpenAI API key" }],
    },
    Holds {
        name: "connect-model-signin-ready",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Heading { level: 2 }, name: "Signed in to ChatGPT as pat@example.com" }],
    },
    Holds {
        name: "connect-model-declined",
        heading: "Connect a model",
        parts: &[Part::Named { role: Role::Alert, name: "You signed in without plan usage." }],
    },
    Holds {
        name: "connect-model-renewed",
        heading: "Connect a model",
        parts: &[Part::Named {
            role: Role::Status,
            name: "That’s pat@example.com again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
        }],
    },
    Holds {
        name: "connect-model-refused",
        heading: "Connect a model",
        parts: &[Part::Named {
            role: Role::Alert,
            name: "Another person here holds that ChatGPT account. To use one of yours, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
        }],
    },
    Holds {
        name: "connect-model-signing-in",
        heading: "Connect a model",
        parts: &[
            Part::Named { role: Role::Status, name: "Waiting for ChatGPT’s sign-in, in the new tab" },
            Part::Named { role: Role::Link, name: "Open it again" },
            Part::Named { role: Role::Disclosure, name: "Signed in, but the page couldn’t be reached?" },
        ],
    },
    Holds {
        name: "connect-model-key-lost",
        heading: "Connect a model",
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
        heading: "3 of 9 tasks landed.",
        parts: &[Part::Text { text: "Harness" }, Part::Named { role: Role::Disclosure, name: "3 landed" }],
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
const SPENDS: [(&str, Role, &str); 46] = [
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
#[test]
fn each_chat_reads_its_allotment() {
    for sample in samples::samples() {
        if (sample.name.starts_with("chat-") || matches!(sample.name, "chats-reference" | "chats-solo"))
            && !matches!(sample.address, temper_client_domain::Address::NewChat { .. })
        {
            let (_, role, spend) = SPENDS.iter().find(|row| row.0 == sample.name).expect("sample's spend row");
            let view = view(&sample);
            find(view.tree(), *role, spend, sample.name);
        }
    }
}
#[test]
fn no_state_reads_of_zero() {
    for sample in samples::samples() {
        let view = view(&sample);
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
}
// A browser's button, tab and option show their name as their text (skein/docs/design/ui-web.md, 6), so text under one
// is more words; recommended/default tags and All models' count are intentional children (temper/docs/design/client/README.md, 3.5).
#[test]
fn no_named_control_draws_a_text_child_in_any_state() {
    for sample in samples::samples() {
        let view = view(&sample);
        let nodes = view.tree().nodes();
        for (index, node) in nodes.iter().enumerate() {
            if node.name.is_none() || !matches!(node.role, Role::Button | Role::Tab | Role::Option) {
                continue;
            }
            let mut text = Vec::new();
            let mut tagged = Vec::new();
            let mut tag_end = 0;
            for (offset, inner) in nodes.iter().enumerate().skip(index).take(usize::try_from(node.size).unwrap()) {
                if inner.styles.as_slice().contains(&temper_client_view::look::MODEL_TAG) {
                    tag_end = offset + usize::try_from(inner.size).unwrap();
                }
                for run in &inner.text {
                    text.extend_from_slice(&run.text);
                    if offset < tag_end {
                        tagged.extend_from_slice(&run.text);
                    }
                }
            }
            let model_tag = node.role == Role::Button
                && text == tagged
                && (tagged == b"default"
                    || (node.styles.as_slice().contains(&temper_client_view::look::MODEL_ALL)
                        && tagged.iter().all(u8::is_ascii_digit)));
            assert!(
                text.is_empty() || text == b"recommended" || model_tag,
                "{}: {:?} {:?} also draws the text {:?}",
                sample.name,
                node.role,
                node.name.as_deref().map(String::from_utf8_lossy),
                String::from_utf8_lossy(&text),
            );
        }
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
#[test]
#[expect(clippy::too_many_lines, reason = "the gallery table walk checks each kind of named part")]
fn each_state_holds_what_its_name_says() {
    for sample in samples::samples() {
        let mut row = None;
        for holds in &HOLDS {
            if holds.name == sample.name {
                assert!(row.is_none(), "duplicate holds row");
                row = Some(holds);
            }
        }
        let row = row.expect("every sample has a HOLDS row");
        let view = view(&sample);
        let tree = view.tree();
        let mut main = TreeFace::new(tree);
        let h1 = in_main(Role::Heading { level: 1 }, row.heading);
        if row.heading.is_empty() {
            assert_eq!(main.find(&h1), Found::Absent, "{} has no main heading", sample.name);
        } else {
            assert!(matches!(main.find(&h1), Found::Node(_)), "{}: heading {}", sample.name, row.heading);
        }
        // Count-only names may repeat; described parts follow conversation order.
        let mut described = tree.nodes().iter();
        for part in row.parts {
            match part {
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
                Part::Named { role, name } => {
                    find(tree, *role, name, sample.name);
                }
                Part::Absent { role, name } => {
                    assert_eq!(
                        main.find(&in_main(*role, name)),
                        Found::Absent,
                        "{}: absent {role:?} {name:?}",
                        sample.name
                    );
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
fn print_writes_every_page_and_the_bundle() {
    let scratch = Scratch::new("temper-client-gallery-print");
    let out = scratch.path().join("gallery");
    print(&out).expect("print gallery");
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
    for file in ["brand.svg", "menu.svg", "send.svg"] {
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
        if matches!(sample.address, temper_client_domain::Address::Welcome { .. }) {
            assert!(!page.contains("<summary>Menu</summary>"), "welcome has a bare frame");
            assert!(page.contains("<title>Welcome · first run · temper</title>"), "{}: welcome title", sample.name);
        } else if page.contains("class=\"temper-rail\"") {
            assert!(page.contains("class=\"temper-tabbar\""), "{}: project sections", sample.name);
        } else {
            assert!(page.contains("class=\"temper-menu\""), "{}: menu disclosure", sample.name);
            assert!(page.contains("<summary>Menu</summary>"), "{}: menu name", sample.name);
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
    let signin = fs::read_to_string(out.join("connect-model-signing-in.html")).expect("sign-in page");
    assert!(signin.contains("href=\"https://issuer.example/visit/gallery-sign-in\""), "external sign-in href");
    let closing = fs::read_to_string(out.join("chat-closing.html")).expect("confirmation page");
    assert!(closing.contains("<dialog open "), "static confirmation opens");
    let chats = fs::read_to_string(out.join("chats-solo.html")).expect("chats page");
    assert!(chats.contains("<title>Talk to an agent about robin/tally. · temper</title>"), "shell heading title");
    let signed_out = fs::read_to_string(out.join("signed-out.html")).expect("signed-out page");
    assert!(signed_out.contains("<title>temper</title>"), "shell title without heading");
    // Reprinting replaces the whole owned directory, including stale files.
    let stale = out.join("stale.html");
    fs::write(&stale, b"obsolete").expect("stale page");
    print(&out).expect("replace gallery");
    assert!(!stale.exists(), "old output removed");
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
        ("connect-model-key-details", Some("connect-model.html?world=reference&way=key&bare")),
        ("connect-model-program-details", Some("connect-model.html?world=reference&way=program&bare")),
        ("connect-model-signing-in", None),
        ("connect-model-key-ready", None),
        ("connect-model-signin-ready", Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare")),
        (
            "connect-model-declined",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-declined"),
        ),
        (
            "connect-model-renewed",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-renewed"),
        ),
        (
            "connect-model-refused",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-refused"),
        ),
        ("connect-model-key-lost", None),
    ];
    rows.sort_by_key(|row| row.0);
    let models: Vec<_> =
        samples::samples().into_iter().filter(|sample| sample.name.starts_with("connect-model-")).collect();
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
        ("welcome-model-choose", vec!["Continue with ChatGPT"]),
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
        .position(|node| node.styles.as_slice().contains(&look::FLOW_PROGRESS))
        .expect("progress");
    let nodes = view.tree().nodes();
    let end = progress + usize::try_from(nodes[progress].size).expect("subtree size");
    let items: Vec<_> = nodes[progress..end].iter().filter(|node| node.role == Role::Item).collect();
    assert_eq!(items.len(), 5);
    assert!(items[0].styles.as_slice().contains(&look::FLOW_DONE));
    assert!(items[1].states.as_slice().contains(&skein_ui::State::Current));
    assert!(items[2].styles.is_empty());
    assert!(items[2].states.is_empty());
}

#[test]
fn configuring_gallery_states_list_the_template_and_send_readiness() {
    for (name, sending, disabled, refusal) in [
        ("configure", false, true, false),
        ("configure-typing", false, false, false),
        ("configure-sending", true, true, false),
        ("configure-refused", false, false, true),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).expect("sample");
        let root = samples::domain(&sample);
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
                .any(|t| t.number == 3)
        );
        let chats = root.pages().chats().expect("configure");
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
            (5, b"Plan, then wait for me".as_slice()),
            (6, b"One agent, one branch".as_slice()),
        ]
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
    for (name, off) in [
        ("settings-drafts-off", "Chats can write drafts"),
        ("settings-goals-off", "Goals"),
        ("settings-changes-off", "Changes"),
        ("settings-away-off", "Away switch"),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("switch-off sample");
        let view = view(&sample);
        let tree = view.tree();
        for label in ["Chats can write drafts", "Goals", "Changes", "Away switch"] {
            let id = find(tree, Role::Check, label, name);
            let check = tree.nodes().iter().find(|node| node.id == id).expect("switch node");
            assert_eq!(check.states.as_slice().contains(&skein_ui::State::Pressed), label != off, "{name}: {label}");
        }
        for label in ["Goals", "Changes"] {
            let present = tree
                .nodes()
                .iter()
                .any(|node| node.role == Role::Link && node.name.as_deref() == Some(label.as_bytes()));
            assert_eq!(present, label != off, "{name}: {label} navigation");
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
    for (name, disclosure) in [("chat-model-open", "Model"), ("chat-work-open", "Checked settings 3×, read 7 docs")] {
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
    assert!(html.contains("Ask a question or ask for a change."));
    assert!(html.contains("Agents plan a larger change and land it."));
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
    assert!(person.contains("role=\"menuitem\">Sign out</div>"));
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
    for (name, count, headline) in [
        ("home-away", 4, "57 changes landed. Four things were decided for you."),
        ("home-away-one", 1, "57 changes landed. One thing was decided for you."),
        ("home-away-overturned", 4, "57 changes landed. Four things were decided for you."),
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
        if count == 1 {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some("landed 03:12 · a new goal can undo it".as_bytes())));
        }
        let kicker =
            tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::HOME_KICKER)).expect("away kicker");
        assert_eq!(
            kicker.name.as_deref(),
            Some(if count == 4 {
                "While you were away · Mon 23:10 to now".as_bytes()
            } else {
                "While you were away · Mon 23:10 to 08:03".as_bytes()
            })
        );
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_TOOK)
            && n.name.as_deref() == Some(b"In temper serve".as_slice())));
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Overturn".as_slice()))
                .count(),
            match name {
                "home-away" => 4,
                "home-away-overturned" => 2,
                _ => 0,
            }
        );
        if name == "home-away-overturned" {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some(b"Overturned: it asks you again".as_slice())));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_OVERTURNED)));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some("landed 03:12 · a new goal can undo it".as_bytes())));
        }
    }
}

#[test]
fn settings_away_draws_the_default_choices() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "settings-away").expect("Away settings");
    assert_eq!(samples::mockup(&sample).as_deref(), Some("../greenfield/settings.html#away"));
    let view = view(&sample);
    for (role, name, state, expected) in [
        (Role::Check, "Implementation choices within a task", skein_ui::State::Pressed, true),
        (Role::Check, "Retries and splitting a failing task", skein_ui::State::Pressed, true),
        (Role::Check, "A design document’s wording", skein_ui::State::Pressed, true),
        (Role::Check, "Adding a dependency", skein_ui::State::Pressed, false),
        (Role::Option, "08:00, the end of quiet hours", skein_ui::State::Selected, true),
    ] {
        let id = find(view.tree(), role, name, sample.name);
        let node = view.tree().nodes().iter().find(|node| node.id == id).expect("Away control");
        assert_eq!(node.states.as_slice().contains(&state), expected, "{name}");
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
        ("configure-chat-accepted", &["Card 1", "Reply"][..]),
        ("configure-chat-replaced", &["Card 1", "Reply", "Card 2", "Reply"][..]),
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
