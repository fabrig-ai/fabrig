//! Hand-made ways documents and page presses (temper/docs/design/client/browser.md, 7.2).
use super::{Address, Domain, Event, FirstRun, List, Queue, Request, Then, c, doc, step, world_in_layout};
use skein_lib::{RequestEnvelope, Token, Writer, bytes};
use temper_client_domain::{Action, pages::Act};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Classes, Way, classes_record};

const PROJECT: u32 = 1;
const REVISION: u32 = 1;
const NAMES: [&str; 3] = ["You approve", "Agents review", "Draft first"];
const LINES: [&str; 3] = [
    "An agent makes each change; nothing reaches main until you approve it.",
    "An agent makes each change, another reviews it, and code lands it.",
    "You and an agent write a design draft; agents build it on its own branch, and it lands whole.",
];
const REFUSED: &str =
    "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible";

pub(super) fn world(name: &str) -> Domain {
    world_in_layout(
        FirstRun { step: 7 },
        Address::Ask { project: PROJECT },
        Then::Shown,
        false,
        None,
        Some(name),
        false,
    )
    .0
}

pub(super) fn deliver(root: &mut Domain, watch: Token, project: u32, name: Option<&str>, out: &mut Queue<Request>) {
    let mut ways = List::with_capacity(3);
    for (index, (name, line)) in NAMES.into_iter().zip(LINES).enumerate() {
        ways.push(
            w::StockWay::new(
                &w::CEILINGS,
                w::StockWayParts {
                    name: name.as_bytes().into(),
                    line: line.as_bytes().into(),
                    path: path(index),
                    items: items(project, index),
                    delegation: false,
                },
            )
            .expect("stock way"),
        )
        .expect("three ways");
    }
    let yours = name == Some("ask-ways-yours");
    let panel = w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts {
            project,
            ways,
            current: if yours { None } else { Some(1) },
            own: if yours { own() } else { path(1) },
            may_change: name != Some("ask-ways-member"),
        },
    )
    .expect("ways panel");
    for up in [
        jig_client::Up::Watched { watch, event: c::WatchEvent::Opened },
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::ways_panel(&panel),
            },
        },
    ] {
        step(root, Event::Client { event: jig_client::Event::Up { up } }, out);
    }
}

fn act(root: &mut Domain, action: Act, out: &mut Queue<Request>) {
    step(root, Event::Act { action: Action::Pages(action) }, out);
}

pub(super) fn press(root: &mut Domain, name: &str, settings: Token, out: &mut Queue<Request>) {
    if matches!(name, "ask-ways-approve" | "ask-ways-card-person" | "ask-ways-refused") {
        act(root, Act::WayTab { project: PROJECT, tab: 0 }, out);
    } else if name == "ask-ways-draft" {
        act(root, Act::WayTab { project: PROJECT, tab: 2 }, out);
    }
    let stop = match name {
        "ask-ways-card" => Some(w::PathStep::Review),
        "ask-ways-card-person" => Some(w::PathStep::Approve),
        _ => None,
    };
    if let Some(stop) = stop {
        let ways = root.pages().stock_ways().expect("ways page");
        let tabs = ways.tabs();
        let tab = tabs.get(ways.shown().expect("shown tab")).expect("shown way");
        let stop = tab.path.stops().iter().position(|value| *value.step() == stop).expect("card stop");
        act(root, Act::WayStop { project: PROJECT, stop: u32::try_from(stop).expect("bounded stops") }, out);
    }
    if !matches!(name, "ask-ways-refused" | "ask-ways-kept") {
        return;
    }
    let kept = name == "ask-ways-kept";
    act(root, if kept { Act::KeepWay { project: PROJECT } } else { Act::UseWay { project: PROJECT, way: 0 } }, out);
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let Some(jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(request), .. }) =
            super::super::driver::down(request)
        {
            assert!(matches!(*request, c::Ask::ChangeSettings(_)), "way sends ChangeSettings");
            assert!(sent.replace(attempt).is_none(), "one way decision");
        }
    }
    let answer = if kept {
        c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: REVISION }))
    } else {
        c::Answer::Refused {
            refusal: c::Refusal::Judged {
                at: 0,
                setting: c::Setting::Application { kind: 2, scope: PROJECT, number: 1 },
                why: super::text(REFUSED),
            },
            at: None,
        }
    };
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: sent.expect("way press sends"),
                    envelope: RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
        out,
    );
    if kept {
        let mut settings_rows = List::with_capacity(1);
        settings_rows
            .push(c::SettingLine {
                setting: c::Setting::Application { kind: 2, scope: PROJECT, number: 1 },
                name: Some(doc("Landing rule")),
                follows: c::SettingFollows::Change,
                last: Some(REVISION),
                application: None,
            })
            .expect("owned rule");
        super::deliver(
            root,
            settings,
            c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: REVISION,
                settings: settings_rows,
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            })),
            out,
        );
    }
}

fn fact(label: &str, value: &str) -> w::Fact {
    w::Fact::new(&w::CEILINGS, w::FactParts { label: label.as_bytes().into(), value: value.as_bytes().into() })
        .expect("card fact")
}
fn card(who: &str, name: &str, facts: &[(&str, &str)]) -> w::Card {
    let mut rows = List::with_capacity(u32::try_from(facts.len()).expect("card facts"));
    for (label, value) in facts {
        rows.push(fact(label, value)).expect("card facts");
    }
    w::Card::new(&w::CEILINGS, w::CardParts { who: who.as_bytes().into(), name: name.as_bytes().into(), facts: rows })
        .expect("stop card")
}
fn agent(who: &str, name: &str, can: &str, asked: &str) -> w::Card {
    // The derivation keeps the sentence whole within 184 bytes, both quotation marks included.
    let asked =
        bytes::concat(&["“".as_bytes(), bytes::utf8_prefix(asked.as_bytes(), 184 - "“”".len()), "”".as_bytes()]);
    card(
        who,
        name,
        &[
            ("Model", "Claude Opus 5.5 · high"),
            ("Can", can),
            ("Asked to", std::str::from_utf8(&asked).expect("UTF-8 instructions")),
        ],
    )
}

#[expect(clippy::too_many_lines, reason = "the fixture gives each derived stop its own card")]
fn stop(step: w::PathStep) -> w::Stop {
    use w::{
        Actor::{Agent, Bar, Code, Person},
        PathStep::{
            Approve, Ask, Branch, Check, Draft, Merge, Plan, Prepare, Produce, ProduceMany, Prove, Repair, Resolve,
            Review, Turn, Update, Verify,
        },
    };
    let rule = w::Opens::Rule(
        w::OpensRule::new(&w::CEILINGS, w::OpensRuleParts { kind: 2, scope: PROJECT, number: 1 }).expect("rule"),
    );
    let coder = w::Opens::Charter(
        w::OpensCharter::new(&w::CEILINGS, w::OpensCharterParts { charter: 1 }).expect("coding charter"),
    );
    let (actor, words, opens, card) = match step {
        Ask => (
            Person,
            "you ask",
            w::Opens::Nothing,
            card("a person · asks for the change", "you", &[("Starts", "a chat")]),
        ),
        Draft => (
            Person,
            "draft",
            w::Opens::Nothing,
            card("a person · writes the draft with an agent", "you", &[("Starts", "a goal for a draft")]),
        ),
        Plan => (
            Agent,
            "plan",
            w::Opens::Charter(
                w::OpensCharter::new(&w::CEILINGS, w::OpensCharterParts { charter: 5 }).expect("coordinator"),
            ),
            agent(
                "an agent · cuts the draft into changes",
                "the coordinator",
                "read the draft; plan and delegate changes; never merge",
                "Cut the draft into changes and coordinate their work.",
            ),
        ),
        Produce | ProduceMany | Repair | Resolve => {
            let (words, who) = match step {
                Produce => ("produce", "an agent · makes the change"),
                ProduceMany => ("produce ×n", "agents · make the goal's changes, in parallel"),
                Repair => ("repair", "an agent · repairs what a check or a review found, then goes back through them"),
                Resolve => ("resolve", "an agent · settles a conflict with its branch"),
                _ => unreachable!("coding steps"),
            };
            (
                Agent,
                words,
                coder,
                agent(
                    who,
                    "the coding agent",
                    "read the code and the forge; change and push its branch; run commands; never merge or update",
                    "Make the change on its branch and report what changed and what was checked.",
                ),
            )
        }
        Check => (
            Code,
            "check",
            rule,
            card(
                "code · holds a change until CI passes",
                "CI",
                &[("Checks", "every check the repository's CI workflows report, at the change's head")],
            ),
        ),
        Review => (
            Agent,
            "review",
            w::Opens::Gate(
                w::OpensGate::new(&w::CEILINGS, w::OpensGateParts { number: 1, charter: 4 }).expect("review gate"),
            ),
            agent(
                "an agent · holds a change until it approves",
                "the reviewer",
                "read the change and its checks; never edit, push or merge",
                "Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.",
            ),
        ),
        Approve => (
            Person,
            "you approve",
            rule,
            card("a person · decides whether it lands", "owner", &[("Role", "owner"), ("Asks", "1 of them")]),
        ),
        Update => (
            Code,
            "update",
            w::Opens::Queue,
            card(
                "code · brings the change up to date with its branch",
                "the queue",
                &[("Does", "merges the branch's tip into the change, never by an agent")],
            ),
        ),
        Merge => (
            Code,
            "merge",
            w::Opens::Queue,
            card("code · lands the change", "the queue", &[("Does", "merges exactly the commit its checks saw")]),
        ),
        Verify => (
            Code,
            "verify",
            w::Opens::Tier(
                w::OpensTier::new(&w::CEILINGS, w::OpensTierParts { name: b"main checks".as_slice().into() })
                    .expect("main tier"),
            ),
            card("code · checks what landed", "main checks", &[("Runs", "the tier main checks after each landing")]),
        ),
        Branch => (Bar, "main", w::Opens::Nothing, card("a branch · where the change lands", "main", &[])),
        Prepare | Prove | Turn => unreachable!("the gallery ways use the queue"),
    };
    w::Stop::new(&w::CEILINGS, w::StopParts { actor, step, words: words.as_bytes().into(), opens, card })
        .expect("path stop")
}

fn branch(stops: Vec<w::Stop>) -> w::BranchPath {
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: Some(b"work/hello".as_slice().into()),
            branch: b"main".as_slice().into(),
            stops: List::from_boxed(stops.into_boxed_slice()),
        },
    )
    .expect("branch path")
}
fn path(index: usize) -> w::BranchPath {
    use w::PathStep::{
        Approve, Ask, Branch, Check, Draft, Merge, Plan, Produce, ProduceMany, Repair, Resolve, Review, Update, Verify,
    };
    let stops = match index {
        0 => vec![Ask, Produce, Check, Repair, Check, Approve, Update, Resolve, Merge, Branch],
        1 => vec![Ask, Produce, Check, Review, Repair, Check, Update, Resolve, Merge, Branch],
        2 => {
            vec![Draft, Plan, ProduceMany, Merge, Check, Repair, Branch, Check, Update, Resolve, Merge, Verify, Branch]
        }
        _ => unreachable!("three stock ways"),
    };
    let mut stops: Vec<_> = stops.into_iter().map(stop).collect();
    if index == 2 {
        let goal_check = stops.get_mut(4).expect("goal tier");
        let mut parts = goal_check.clone().into_parts();
        parts.opens = w::Opens::Tier(
            w::OpensTier::new(&w::CEILINGS, w::OpensTierParts { name: b"goal checks".as_slice().into() })
                .expect("goal tier"),
        );
        parts.card = card(
            "code · checks the goal's branch",
            "goal checks",
            &[("Runs", "the tier goal checks on the goal's branch")],
        );
        *goal_check = w::Stop::new(&w::CEILINGS, parts).expect("goal check");
        let repair = stops.get_mut(5).expect("goal repair");
        let mut parts = repair.clone().into_parts();
        parts.opens = w::Opens::Charter(
            w::OpensCharter::new(&w::CEILINGS, w::OpensCharterParts { charter: 5 }).expect("coordinator"),
        );
        parts.card = agent(
            "an agent · repairs a red on the goal's branch",
            "the coordinator",
            "read the draft; plan and delegate changes; never merge",
            "Cut the draft into changes and coordinate their work.",
        );
        *repair = w::Stop::new(&w::CEILINGS, parts).expect("goal repair");
        let goal = stops.get_mut(6).expect("goal landing");
        let mut parts = goal.clone().into_parts();
        parts.words = b"goal branch".as_slice().into();
        parts.card = card("a branch · holds the goal's changes until it lands whole", "goal branch", &[]);
        *goal = w::Stop::new(&w::CEILINGS, parts).expect("goal branch");
    }
    branch(stops)
}
fn own() -> w::BranchPath {
    use w::PathStep::{Ask, Branch, Merge, Produce};
    branch(vec![stop(Ask), stop(Produce), stop(Merge), stop(Branch)])
}

#[expect(clippy::too_many_lines, reason = "the fixture encodes each stock way's rules, visibility and classes")]
fn items(project: u32, index: usize) -> List<w::ProposalItem> {
    let mut items = List::with_capacity(6);
    for number in 1..=4 {
        let edit = if number == 1 || (index == 2 && number == 2) {
            let goal = number == 2;
            let mut segments = List::with_capacity(u32::from(goal));
            if goal {
                segments.push(b"temper".as_slice().into()).expect("goal prefix");
            }
            let last = if index == 2 && !goal {
                w::LandingLast::Exact(
                    w::LandingExact::new(&w::CEILINGS, w::LandingExactParts { value: b"main".as_slice().into() })
                        .expect("main"),
                )
            } else {
                w::LandingLast::Open(
                    w::LandingOpen::new(&w::CEILINGS, w::LandingOpenParts { value: Box::default() })
                        .expect("open branches"),
                )
            };
            let pattern = w::LandingPattern::new(&w::CEILINGS, w::LandingPatternParts { segments, last })
                .expect("branch pattern");
            let mut gates = List::with_capacity(u32::from(index == 1));
            if index == 1 {
                gates
                    .push(
                        w::RuleGate::new(
                            &w::CEILINGS,
                            w::RuleGateParts { number: 1, blocking: true, freshness: w::RuleFreshness::Clean },
                        )
                        .expect("review gate"),
                    )
                    .expect("one gate");
            }
            let mut approvals = List::with_capacity(u32::from(index == 0));
            if index == 0 {
                approvals
                    .push(
                        w::RuleApproval::new(
                            &w::CEILINGS,
                            w::RuleApprovalParts { role: 0, people: 1, freshness: w::RuleFreshness::Clean },
                        )
                        .expect("owner approval"),
                    )
                    .expect("one approval");
            }
            let rule = w::LandingRule::new(
                &w::CEILINGS,
                w::LandingRuleParts { pattern, enforce: true, ci: !goal, up_to_date: !goal, gates, approvals },
            )
            .expect("way rule");
            let mut writer = Writer::new(usize::try_from(rule.measure()).expect("rule bytes"));
            rule.encode(&mut writer).expect("measured rule");
            w::SettingEdit::Set(
                w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: writer.finish() })
                    .expect("rule value"),
            )
        } else {
            w::SettingEdit::Removed
        };
        items
            .push(
                w::ProposalItem::new(&w::CEILINGS, w::ProposalItemParts { kind: 2, scope: project, number, edit })
                    .expect("way item"),
            )
            .expect("four rules");
    }
    if index == 2 {
        let ways =
            w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { skein_bools: [true, true, false, false], way: 4 })
                .expect("draft goals");
        let mut writer = Writer::new(usize::try_from(ways.measure()).expect("ways bytes"));
        ways.encode(&mut writer).expect("measured ways");
        items
            .push(
                w::ProposalItem::new(
                    &w::CEILINGS,
                    w::ProposalItemParts {
                        kind: 7,
                        scope: project,
                        number: 1,
                        edit: w::SettingEdit::Set(
                            w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: writer.finish() })
                                .expect("ways value"),
                        ),
                    },
                )
                .expect("draft goals item"),
            )
            .expect("fifth item");
    }
    let way = match index {
        0 => Way::YouApprove,
        1 => Way::AgentsReview,
        2 => Way::DraftFirst,
        _ => unreachable!("three stock ways"),
    };
    let classes = classes_record(&Classes::stock(way)).expect("stock classes fit the record");
    let mut writer = Writer::new(usize::try_from(classes.measure()).expect("classes bytes"));
    classes.encode(&mut writer).expect("measured classes");
    items
        .push(
            w::ProposalItem::new(
                &w::CEILINGS,
                w::ProposalItemParts {
                    kind: 8,
                    scope: project,
                    number: 1,
                    edit: w::SettingEdit::Set(
                        w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: writer.finish() })
                            .expect("classes value"),
                    ),
                },
            )
            .expect("classes item"),
        )
        .expect("six stock items at most");
    items
}
