//! Hand-made ways documents and page presses (temper/docs/design/client/web/gallery.md, 2).
use super::{Address, Domain, Event, FirstRun, List, Queue, Request, Then, c, doc, step, world_in_layout};
use skein_lib::{RequestEnvelope, Token, Writer, bytes};
use temper_client_domain::{Action, pages::Act};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Classes, Way, classes_record};

const PROJECT: u32 = 1;
const REVISION: u32 = 1;
const NAMES: [&str; 3] = ["You approve", "Agents review", "Draft first"];
const HEADLINES: [&str; 3] = [
    "You approve every change",
    "Agents review every change, and code lands it",
    "Draft first, then agents build it on its own branch",
];
const LINES: [&str; 3] = [
    "An agent makes each change; nothing reaches main until you approve it.",
    "An agent makes each change, another reviews it, and code lands it.",
    "You and an agent write a design draft; agents build it on its own branch, and it lands whole.",
];
const REFUSED: &str =
    "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible";

pub(super) fn world(name: &str) -> Domain {
    world_in_layout(
        if is_draft(name) { super::Reference } else { FirstRun { step: 7 } },
        if is_how(name) { Address::Settings { project: PROJECT } } else { Address::Ask { project: PROJECT } },
        Then::Shown,
        false,
        None,
        Some(name),
        false,
    )
    .0
}

pub(super) fn is_how(name: &str) -> bool {
    name == "how" || name.starts_with("how-")
}

pub(super) fn is_draft(name: &str) -> bool {
    matches!(name, "how-draft" | "how-reaches" | "how-follows")
}

pub(super) fn frame(me: &mut c::Me, name: &str) {
    if is_draft(name) {
        let project = me.projects.get_mut(0).expect("How project");
        let template = project.templates.iter_mut().find(|template| template.number == 4).expect("draft goal template");
        template.budget = c::TemplateBudget::Amount(80);
    }
}

pub(super) fn setting(name: &str) -> Box<[u8]> {
    let draft = is_draft(name);
    let ways = w::ProjectWays::new(
        &w::CEILINGS,
        w::ProjectWaysParts { drafts: draft, goals: draft, changes: false, way: if draft { 4 } else { 0 } },
    )
    .expect("How project ways");
    let mut writer = Writer::new(usize::try_from(ways.measure()).expect("ways bytes"));
    ways.encode(&mut writer).expect("measured ways");
    writer.finish()
}

pub(super) fn reaches_setting() -> Box<[u8]> {
    use temper_protocol_reaches::{Class, Item, Kind, PathSet, Subject};
    let classes = Classes {
        items: [
            (Subject::Kind(Kind::Wording), Class::Decide),
            (Subject::Kind(Kind::Retry), Class::Decide),
            (Subject::Kind(Kind::Implementation), Class::Decide),
            (Subject::Kind(Kind::Tests), Class::Decide),
            (Subject::Set(1), Class::AskAway),
            (Subject::Kind(Kind::Dependency), Class::Ask),
            (Subject::Kind(Kind::Departure), Class::Ask),
        ]
        .into_iter()
        .map(|(subject, class)| Item { subject, words: Box::default(), class })
        .collect(),
        sets: Box::new([PathSet {
            number: 1,
            name: "vsr’s protocol core".as_bytes().into(),
            patterns: ["include/vsr.h", "src/protocol.*", "src/transition.*", "docs/protocol.md"]
                .map(|pattern| pattern.as_bytes().into())
                .into(),
        }]),
    };
    let record = classes_record(&classes).expect("whole story's classes");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("classes bytes"));
    record.encode(&mut writer).expect("measured classes");
    writer.finish()
}

pub(super) fn deliver(root: &mut Domain, watch: Token, project: u32, name: Option<&str>, out: &mut Queue<Request>) {
    let mut ways = List::with_capacity(3);
    for (index, (way_name, line)) in NAMES.into_iter().zip(LINES).enumerate() {
        ways.push(
            w::StockWay::new(
                &w::CEILINGS,
                w::StockWayParts {
                    headline: HEADLINES[index].as_bytes().into(),
                    name: way_name.as_bytes().into(),
                    line: line.as_bytes().into(),
                    path: delivered_path(index, name),
                    items: items(project, index),
                    delegation: false,
                },
            )
            .expect("stock way"),
        )
        .expect("three ways");
    }
    let yours = matches!(name, Some("ask-ways-yours" | "how-yours"));
    let current = match name {
        Some("how-approve") => 0,
        Some("how-draft" | "how-reaches" | "how-follows") => 2,
        _ => 1,
    };
    let panel = w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts {
            words: if name == Some("how-yours") {
                Some(w::ProjectWords::new(
                    &w::CEILINGS,
                    w::ProjectWordsParts {
                        headline: b"Agents review everything, then one of you signs off".as_slice().into(),
                        lede: b"shop runs a way of its own: agents review every change, and a rule of yours adds a sign-off.".as_slice().into(),
                    },
                ).expect("project words"))
            } else {
                None
            },
            project,
            ways,
            current: if yours { None } else { Some(u32::try_from(current).expect("stock way")) },
            own: if yours { own() } else { delivered_path(current, name) },
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

pub(super) fn roles(root: &mut Domain, read: Token, out: &mut Queue<Request>) {
    let rows = [
        (
            2,
            "Chat",
            "For each chat: talks with you and asks for the changes",
            "read the code and the forge; start sub-agents; never merge",
            "Help the person with their work.",
        ),
        (
            7,
            "Ask temper",
            "When you ask: explains how work runs here, and proposes changes to it",
            "read temper’s records and the forge; propose settings; never change anything",
            "Explain how temper works, where work stands and how to configure it.",
        ),
        (
            12,
            "Drafter",
            "When you start a draft: writes it with you, in the terminal",
            "read the code and the forge; never merge",
            "Write this draft with the person, in their words.",
        ),
        (
            5,
            "Coordinator",
            "For each goal: designs it, cuts it into changes and lands it",
            "read the code and the forge; change and push its branch; start sub-agents; never merge",
            "Coordinate one goal: a draft built into code through changes into your own goal branch.",
        ),
        (
            1,
            "Coding agent",
            "For each change: makes it, repairs it and resolves its conflicts",
            "read the code and the forge; change and push its branch; run commands; never merge",
            "Make the change this task names, on its own branch.",
        ),
        (
            6,
            "Foreman",
            "When a coding agent asks: answers from the goal’s design, or passes it up",
            "read the code and the forge; never merge",
            "Settle the question in your brief from the change’s words, the design and its pushed head.",
        ),
        (
            9,
            "Status agent",
            "When you look at Home after landings: writes the lines at its top",
            "read the facts in its task; never merge",
            "Write the few lines at the top of the project’s Home, from the facts in your task.",
        ),
    ];
    let roles = rows
        .into_iter()
        .map(|(charter, name, who, can, asked)| {
            w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter, card: agent(who, name, can, asked) })
                .expect("project role")
        })
        .collect();
    let view = w::View::Roles(
        w::RolesView::new(&w::CEILINGS, w::RolesViewParts { roles: List::from_boxed(roles) }).expect("project roles"),
    );
    let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).expect("roles answer");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::view_answer(&answer)),
                    },
                },
            },
        },
        out,
    );
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
        "ask-ways-card" | "how-card" => Some(w::PathStep::Review),
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
            Approve, Ask, Branch, CatchUp, Check, Conflict, Draft, Merge, Plan, Prepare, Produce, ProduceMany, Prove,
            Repair, Resolve, Review, Turn, Verify,
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
        Plan | w::PathStep::Design | w::PathStep::CloseOut => (
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
                    "read the code and the forge; change and push its branch; run commands; never merge",
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
                &[
                    ("Workflows", "ci.yml"),
                    ("Checks", "every check the repository's CI workflows report, at the change's head"),
                ],
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
        Conflict => (
            Code,
            "conflict",
            w::Opens::Queue,
            card(
                "code · main has moved in a way that clashes with the change",
                "the queue",
                &[("When", "the change reaches the front of the queue: main is merged into it, never by an agent")],
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
        CatchUp | Prepare | Prove | Turn => unreachable!("the gallery ways use the queue"),
    };
    w::Stop::new(
        &w::CEILINGS,
        w::StopParts { needed: false, over: Box::default(), actor, step, words: words.as_bytes().into(), opens, card },
    )
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
        Approve, Ask, Branch, Check, Conflict, Draft, Merge, Plan, Produce, ProduceMany, Repair, Resolve, Review,
        Verify,
    };
    let stops = match index {
        0 => vec![Ask, Produce, Check, Repair, Check, Approve, Conflict, Resolve, Merge, Branch],
        1 => vec![Ask, Produce, Check, Review, Repair, Check, Conflict, Resolve, Merge, Branch],
        2 => {
            vec![
                Draft,
                Plan,
                ProduceMany,
                Merge,
                Check,
                Repair,
                Branch,
                Check,
                Conflict,
                Resolve,
                Merge,
                Verify,
                Branch,
            ]
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

fn delivered_path(index: usize, name: Option<&str>) -> w::BranchPath {
    let path = path(index);
    let Some(name) = name.filter(|name| is_how(name)) else { return path };
    let mut parts = path.into_parts();
    let mut stops = parts.stops.into_boxed().into_vec();
    if name == "how-approve" {
        stops.retain(|stop| {
            !matches!(stop.step(), w::PathStep::Check | w::PathStep::Verify)
                && (index != 0 || *stop.step() != w::PathStep::Repair)
        });
    } else if index != 2 {
        let main = stops.len().checked_sub(1).expect("main bar");
        stops.insert(main, stop(w::PathStep::Verify));
    }
    if name == "how-follows" {
        for stop in &mut stops {
            if *stop.step() == w::PathStep::Verify {
                let mut parts = stop.clone().into_parts();
                parts.card = card("code · checks what landed", "fuzz", &[("Checks", "fuzzing for 6 hours")]);
                *stop = w::Stop::new(&w::CEILINGS, parts).expect("landing tier");
            }
        }
    }
    parts.stops = List::from_boxed(stops.into_boxed_slice());
    w::BranchPath::new(&w::CEILINGS, parts).expect("How path")
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
                w::LandingRuleParts {
                    pattern,
                    enforce: true,
                    ci: !goal,
                    up_to_date: !goal,
                    gates,
                    approvals,
                    sets: List::with_capacity(0),
                },
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
        let ways = w::ProjectWays::new(
            &w::CEILINGS,
            w::ProjectWaysParts { drafts: true, goals: true, changes: false, way: 4 },
        )
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

fn follow_row(paths: &[&[u8]], start: w::FollowStart, lag: u32, verdict: Option<w::FollowVerdict>) -> w::FollowView {
    let repo = w::FollowRepository::new(&w::CEILINGS, w::FollowRepositoryParts { forge: 1, repository: 7 })
        .expect("gallery follow");
    let setting = w::Follow::new(
        &w::CEILINGS,
        w::FollowParts {
            source: w::FollowSource::new(
                &w::CEILINGS,
                w::FollowSourceParts {
                    repository: repo.clone(),
                    branch: b"main".as_slice().into(),
                    paths: List::from_boxed(paths.iter().map(|p| Box::from(*p)).collect()),
                },
            )
            .expect("gallery follow"),
            target: w::FollowTarget::new(
                &w::CEILINGS,
                w::FollowTargetParts { repository: repo, branch: b"main".as_slice().into() },
            )
            .expect("gallery follow"),
            owner: w::FollowOwner::Person(
                w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person: 4 }).expect("gallery follow"),
            ),
            start,
            producer: None,
            verification: w::FollowCheck::new(
                &w::CEILINGS,
                w::FollowCheckParts {
                    workflow: b".forgejo/workflows/vsr-tests.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: w::FollowFormat::Junit,
                },
            )
            .expect("gallery follow"),
        },
    )
    .expect("gallery follow");
    let parity = verdict.map(|verdict| {
        w::FollowParity::new(
            &w::CEILINGS,
            w::FollowParityParts { source: [0xab; 32], head: [0xcd; 32], verdict, run: b"check".as_slice().into() },
        )
        .expect("gallery follow")
    });
    w::FollowView::new(
        &w::CEILINGS,
        w::FollowViewParts {
            number: 1,
            setting,
            source: b"cowsql/vsr".as_slice().into(),
            source_branch: b"main".as_slice().into(),
            target: b"cowsql/vsr-rs".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            task: None,
            stopped: None,
            lag,
            entries: List::with_capacity(0),
            skips: List::with_capacity(0),
            parity,
        },
    )
    .expect("gallery follow")
}
pub(super) fn follows(root: &mut Domain, watch: Token, out: &mut Queue<Request>) {
    let row = follow_row(&[b"src", b"include"], w::FollowStart::Owner, 2, Some(w::FollowVerdict::Passed));
    let panel = w::FollowsPanel::new(
        &w::CEILINGS,
        w::FollowsPanelParts {
            project: PROJECT,
            follows: List::from_boxed(Box::new([row])),
            dependencies: List::with_capacity(0),
        },
    )
    .expect("gallery follow");
    for up in [
        jig_client::Up::Watched { watch, event: c::WatchEvent::Opened },
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::follows_panel(&panel),
            },
        },
    ] {
        step(root, Event::Client { event: jig_client::Event::Up { up } }, out);
    }
}
pub(super) fn how_project() -> c::Project {
    let mut project = super::team::project(4, true);
    project.members.get_mut(0).expect("Pat").name = doc("Pat");
    project.current_period = Some(c::PoolLedger {
        source: c::Funder::Period { project: PROJECT, period: 1 },
        parent: None,
        numbers: c::Numbers { budget: 1000, spent: 0, spent_below: 0, reserved: 0 },
        left: 1000,
        closed: false,
        began: Some(skein_lib::Wall::from_nanos(468_000_000_000_000)),
        next: Some(skein_lib::Wall::from_nanos(1_072_800_000_000_000)),
    });
    project
}
pub(super) fn standing() -> c::Tasks {
    let mut task = super::base_task(41, "Owns the tests");
    task.project = PROJECT;
    task.label = 3;
    task.chip.phase = c::Phase::Running;
    task.budget.numbers.budget = 40;
    task.budget.left = 28;
    c::Tasks { rows: List::from_boxed(Box::new([super::task_line(task)])), more: false }
}
