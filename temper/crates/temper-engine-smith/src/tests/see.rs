//! Tokens, whole texts and bounded parts (domain/agent/ask-temper.md, 1).
#![expect(
    clippy::disallowed_methods,
    reason = "test fixtures reconstruct UTF-8 parts and decode their decimal offsets"
)]
#![expect(clippy::disallowed_types, reason = "test fixtures join expected text in strings")]
use super::decided;
use crate::{ADDRESSES, SEE_PART, see_answer};
use jig_domain_core::settings::Setting;
use temper_engine_domain::{SeeAnswer, Tool};

fn rendered(line: &str, from: u32, answer: SeeAnswer, maximum: u32) -> (String, bool) {
    let (body, error) =
        see_answer(line.as_bytes(), from, &answer, &[], maximum, crate::ForgeReadBounds::STOCK).into_parts();
    assert!(body.len() <= usize::try_from(maximum).unwrap());
    (String::from_utf8(body.into_vec()).unwrap(), error)
}

fn person(read: temper_engine_domain::SeeRead) -> SeeAnswer {
    SeeAnswer::Person(Box::new(temper_engine_domain::SeePerson {
        read,
        names: temper_engine_domain::SeeNames {
            project: temper_engine_domain::SeeName { number: 7, name: b"Shop".as_slice().into() },
            people: Box::new([]),
            charters: Box::new([]),
        },
    }))
}

#[test]
fn tokens_round_trip_and_malformed_tokens_stay_unknown() {
    for (line, from, token) in [
        ("settings policy 1", 6120, "settings+policy+1@6120"),
        ("a+b@c%d", 1, "a%2Bb%40c%25d@1"),
        ("settings charter-words 4", 12, "settings+charter-words+4@12"),
        ("settings project-words", 12, "settings+project-words@12"),
        ("settings project-areas", 12, "settings+project-areas@12"),
        ("settings project-triager", 12, "settings+project-triager@12"),
    ] {
        assert_eq!(crate::see::token(line.as_bytes(), from).as_ref(), token.as_bytes());
        let input = format!(r#"{{"address":"more {token}"}}"#);
        assert_eq!(decided(b"see", input.as_bytes()).unwrap().tool, Tool::See { line: line.as_bytes().into(), from });
    }
    assert_eq!(
        decided(b"see", br#"{"address":"more\tsettings+policy+1@6120"}"#).unwrap().tool,
        Tool::See { line: b"settings policy 1".as_slice().into(), from: 6120 }
    );
    assert_eq!(
        decided(b"see", br#"{"address":" settings  policy 1 "}"#).unwrap().tool,
        Tool::See { line: b"settings  policy 1".as_slice().into(), from: 0 }
    );
    for token in
        ["a", "a@", "a@4294967296", "a%20b@1", "a%@1", "a%2b@1", "+a@1", "a+@1", "a++b@1", "@1", "a@-1", "a@1@2"]
    {
        let line = format!("more {token}");
        let input = format!(r#"{{"address":"{line}"}}"#);
        assert_eq!(
            decided(b"see", input.as_bytes()).unwrap().tool,
            Tool::See { line: line.as_bytes().into(), from: 0 },
            "{token}"
        );
    }
}

#[test]
fn parts_keep_lines_characters_offsets_and_the_reply_bound() {
    let long = "short line\n".repeat(700);
    let (body, error) = crate::see::page(b"help settings", 0, long.as_bytes(), b"help", 20_000).into_parts();
    assert!(!error);
    let text = core::str::from_utf8(&body).unwrap();
    let (part, tail) = text.split_once("more: ").unwrap();
    assert!(part.len() <= usize::try_from(SEE_PART).unwrap());
    assert!(part.ends_with('\n'));
    assert_eq!(part, &long[..part.len()]);
    assert!(tail.starts_with(&format!("help+settings@{}\n", part.len())));
    let (body, error) =
        crate::see::page(b"help settings", u32::try_from(part.len()).unwrap(), long.as_bytes(), b"help", 20_000)
            .into_parts();
    assert!(!error);
    assert!(core::str::from_utf8(&body).unwrap().starts_with(&long[part.len()..]));
    let long = format!("{}é{}", "x".repeat(6143), "z".repeat(100));
    let (body, error) = crate::see::page(b"file a/b x", 0, long.as_bytes(), b"help", 20_000).into_parts();
    assert!(!error);
    assert!(core::str::from_utf8(&body).unwrap().contains("more: file+a/b+x@6143\n"));
    let (body, error) = crate::see::page(b"file a/b x", 6144, long.as_bytes(), b"help", 20_000).into_parts();
    assert!(!error);
    assert!(body.starts_with(b"zz"));
    let (body, error) =
        crate::see::page(b"help", 0, b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n", b"help NAME", 40).into_parts();
    assert!(!error);
    assert!(body.len() <= 40);
    assert!(core::str::from_utf8(&body).unwrap().contains("more: help@"));
    assert!(body.ends_with(b"see: help NAME\n"));
    let (body, error) = crate::see::page(b"help", 0, long.as_bytes(), b"help NAME", 1).into_parts();
    assert!(error);
    assert!(body.len() <= 1);
    let short_lines = "x\n".repeat(100);
    let (body, error) = crate::see::page(b"help", 0, short_lines.as_bytes(), b"help NAME", 214).into_parts();
    assert!(!error, "{}", String::from_utf8_lossy(&body));
    assert!(body.len() <= 214);
    assert!(core::str::from_utf8(&body).unwrap().contains("more: help@"));
    for from in [3, 4] {
        let (body, error) = crate::see::page(b"help", from, b"abc", b"help NAME", 1000).into_parts();
        assert!(error);
        assert_eq!(
            body.as_ref(),
            format!("see help: nothing from byte {from}; see help reads it from its start.").as_bytes()
        );
    }
}

fn settings(answer: jig_domain_core::SettingsAnswer) -> SeeAnswer {
    SeeAnswer::Settings(Box::new(temper_engine_domain::SeeSettings {
        requested: match &answer {
            jig_domain_core::SettingsAnswer::Revision(_) => Some(1),
            jig_domain_core::SettingsAnswer::Inventory(_)
            | jig_domain_core::SettingsAnswer::Setting(_)
            | jig_domain_core::SettingsAnswer::Revisions(_)
            | jig_domain_core::SettingsAnswer::Unknown => None,
        },
        answer,
        repositories: Box::new([]),
        people: Box::new([]),
        forge_people: Box::new([]),
        offer: Box::new([]),
        application: None,
    }))
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one contract table covers the text and following addresses of each answer kind"
)]
fn errors_and_pages_use_the_address_form() {
    for (line, answer, expected, error) in [
        (
            "task 41 activity",
            person(temper_engine_domain::SeeRead::Activity {
                number: 41,
                activity: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Busy),
            }),
            "activity: not read\nsee: task N turn A.T · task N · task N tree\n".to_string(),
            false,
        ),
        (
            "task 41 turn 1.2",
            person(temper_engine_domain::SeeRead::Turn {
                number: 41,
                attempt: 1,
                turn: 2,
                read: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Busy),
            }),
            "turn 1.2 of task 41: not read\nsee: task N activity · task N\n".to_string(),
            false,
        ),
        (
            "changes",
            person(temper_engine_domain::SeeRead::Changes { changes: None, landings: None }),
            "changes of project 7, Shop: not read\nsee: change N · settings landing · ways\n".to_string(),
            false,
        ),
        ("bogus", SeeAnswer::Unknown, format!("see: no address bogus; addresses: {ADDRESSES}"), true),
        (
            "help search x",
            SeeAnswer::Search { words: b"".as_slice().into() },
            "see help search x: nothing to search for".to_string(),
            true,
        ),
    ] {
        assert_eq!(rendered(line, 0, answer, 20_000), (expected, error));
    }

    assert_eq!(rendered("settings charter 99",0,settings(jig_domain_core::SettingsAnswer::Unknown),20_000),
        ("see settings charter 99: not read: no such setting. This says nothing about the settings: see settings lists them.".into(),true));
    for (answer, expected) in [
        (temper_engine_domain::CallAnswer::Unavailable, "This engine tool is unavailable."),
        (
            temper_engine_domain::CallAnswer::ToolDenied {
                answer: jig_domain_authority::Answer::Refuse,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            },
            "Authority refused: tool family settings",
        ),
    ] {
        assert_eq!(
            rendered("settings", 0, SeeAnswer::Call { tool: b"settings", answer: Box::new(answer) }, 20_000),
            (format!("see settings: {expected}"), true)
        );
    }
    let (text, error) = rendered(
        "file a/b x",
        0,
        SeeAnswer::Call { tool: b"read_file", answer: Box::new(temper_engine_domain::CallAnswer::Controlled) },
        20_000,
    );
    assert_eq!((text, error), ("Done\nsee: file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME\n".into(), false));
    assert_eq!(
        rendered("proposal t1/1", 0, SeeAnswer::Proposal(Box::new(temper_engine_domain::SeeProposal::Unknown)), 20_000),
        ("see proposal t1/1: not read: no such proposal of settings. This says nothing about the settings: see settings lists those waiting.".into(), true)
    );
    for (answer, next) in [
        (
            jig_domain_core::SettingsAnswer::Inventory(jig_domain_core::documents::Settings {
                newest: 1,
                settings: skein_lib::List::with_capacity(0),
                held: skein_lib::List::with_capacity(0),
                pending: skein_lib::List::with_capacity(0),
            }),
            "settings charter N · settings template N · settings policy N · settings model N · settings forge-charter N · settings rules · settings landing-rule N [project] · settings landing · settings gates OWNER/NAME · settings project-ways · settings project-questions · settings project-classes · revisions · help settings",
        ),
        (
            jig_domain_core::SettingsAnswer::Revisions(jig_domain_core::reads::RevisionsPage {
                rows: skein_lib::List::with_capacity(0),
                next: None,
                forgotten: false,
            }),
            "revision N · settings · help revision",
        ),
        (
            jig_domain_core::SettingsAnswer::Revision(jig_domain_core::reads::RevisionAnswer::Unknown),
            "revisions · proposal tN/K · proposal pN/K · settings",
        ),
    ] {
        let (text, error) = rendered("settings", 0, settings(answer), 20_000);
        assert!(!error, "{text}");
        assert!(text.ends_with(&format!("see: {next}\n")), "{text}");
    }
    for (setting, next) in [
        (Setting::Charter(1), "settings · revisions · help settings-fields"),
        (Setting::Template(1), "settings · revisions · help settings-fields"),
        (Setting::Policy(1), "settings · revisions · help settings-fields"),
        (Setting::Model(1), "settings · revisions · help settings-fields"),
        (Setting::Rules, "settings · revisions · help settings-fields"),
        (Setting::Application { kind: 1, scope: 0, number: 1 }, "settings landing · settings · help landing-rule"),
        (Setting::Application { kind: 2, scope: 7, number: 1 }, "settings landing · settings · help landing-rule"),
        (Setting::Application { kind: 3, scope: 0, number: 1 }, "settings · revisions · help settings-fields"),
        (Setting::Application { kind: 6, scope: 7, number: 1 }, "settings · revisions · help settings-fields"),
        (Setting::Application { kind: 7, scope: 7, number: 1 }, "settings · revisions · help settings-fields"),
        (Setting::Application { kind: 11, scope: 0, number: 4 }, "settings template N · ways · help settings-fields"),
    ] {
        let answer = jig_domain_core::SettingsAnswer::Setting(jig_domain_core::reads::SettingDocument {
            overlay: None,
            setting,
            revision: 1,
            value: None,
            application: None,
            proposed: None,
        });
        let (text, error) = rendered("settings", 0, settings(answer), 20_000);
        assert!(!error, "{text}");
        assert!(text.ends_with(&format!("see: {next}\n")), "{text}");
    }
    for (line, name, next) in
        [("help", "index", "help NAME · help search WORDS"), ("help settings", "settings", "help · help search WORDS")]
    {
        let page = temper_docs::find(name.as_bytes()).unwrap();
        let mut from = 0;
        let mut whole = String::new();
        loop {
            let (text, error) = rendered(line, from, SeeAnswer::Page { name: name.as_bytes().into() }, 1000);
            assert!(!error, "{text}");
            assert!(text.ends_with(&format!("see: {next}\n")));
            let end = text.find("\nmore: ").or_else(|| text.find("\nsee: ")).unwrap() + 1;
            whole.push_str(&text[..end]);
            if let Some(token) = text.lines().find_map(|line| line.strip_prefix("more: ")) {
                from = token.rsplit_once('@').unwrap().1.parse().unwrap();
            } else {
                break;
            }
        }
        assert_eq!(
            whole,
            format!(
                "{}: {}\n{}{}",
                page.name,
                page.title,
                page.text,
                if page.text.ends_with('\n') { "" } else { "\n" }
            )
        );
    }
    let index = temper_docs::find(b"index").unwrap();
    assert_eq!(
        rendered("help", 0, SeeAnswer::Page { name: b"index".as_slice().into() }, 20_000),
        (format!("index: {}\n{}see: help NAME · help search WORDS\n", index.title, index.text), false)
    );
    let (text, error) =
        rendered("help no-such-page", 0, SeeAnswer::Page { name: b"no-such-page".as_slice().into() }, 20_000);
    assert!(!error);
    assert!(text.starts_with("no page no-such-page; the sections holding its words:\n"));
    let (text, error) =
        rendered("help search landing rule", 0, SeeAnswer::Search { words: b"landing rule".as_slice().into() }, 20_000);
    assert!(!error);
    assert!(text.contains("landing-rule: "), "{text}");
    assert!(text.ends_with("see: help NAME\n"));
    let (text, error) =
        rendered("help search xyzzy", 0, SeeAnswer::Search { words: b"xyzzy-no-sections".as_slice().into() }, 20_000);
    assert_eq!((text, error), ("nothing matches\nsee: help NAME\n".into(), false));
    let (text, error) =
        rendered("help search temper", 0, SeeAnswer::Search { words: b"temper".as_slice().into() }, 20_000);
    assert!(!error);
    assert!(text.contains("more sections match; narrow the words\n"));
    for words in ["one two three four five six seven eight nine".to_string(), "x".repeat(33)] {
        let expected = if words.contains(' ') { "at most 8 words" } else { "a word is at most 32 bytes" };
        assert_eq!(
            rendered("help search", 0, SeeAnswer::Search { words: words.into_bytes().into() }, 20_000),
            (format!("see help search: {expected}"), true)
        );
    }
}

#[test]
fn landing_and_gates_read_the_configured_values() {
    use temper_engine_domain::{SeeLanding, SeeRepository};
    use temper_engine_domain_forge::{AfterLanding, Follows, RepositoryGates, Tier};
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Format, Settings, Suspects};
    let none = SeeRepository {
        name: b"org/empty".as_slice().into(),
        checks: RepositoryGates {
            repository: skein_forge::api::Repository { forge: 1, repository: 1 },
            gates: Box::new([]),
            tiers: Box::new([]),
        },
    };
    let not_setting = "not a setting yet: whoever runs temper here sets it in temper's configuration (its landing member); no proposal changes it.";
    for (by, merge_job, after, repositories, text) in [
        (Way::Turn, None, None, Box::<[SeeRepository]>::default(),
         "landing: by turn\nmerge job: none\nrepositories:\n  none\n".to_string()),
        (Way::Queue, Some(b"merge".as_slice().into()),
         Some(AfterLanding { verdict: Settings { retries: 1, suspects: Suspects::TellAll }, bound: skein_lib::Duration::from_millis(5_400_000) }),
         Box::new([SeeRepository { name: none.name.clone(), checks: none.checks.clone() }]),
         "landing: by queue\nmerge job: merge\nafter landing: 1 tier retry, a verification's bound of 1h30m, suspects tell all\nrepositories:\n  org/empty: no gates or tiers\n".to_string()),
    ] {
        assert_eq!(rendered("settings landing", 0, SeeAnswer::Landing(Box::new(SeeLanding {
            by, merge_job, after, repositories,
        })), 20_000), (format!("{text}{not_setting}\nsee: settings gates OWNER/NAME · settings landing-rule N [project] · help landing\n"), false));
    }
    let checks = RepositoryGates {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        gates: Box::new([temper_engine_domain_forge_landing::Gate {
            name: b"tests".as_slice().into(),
            workflow: b"tests.yml".as_slice().into(),
            ignored: Box::new([b"docs".as_slice().into(), b"notes".as_slice().into()]),
            blocking: false,
        }]),
        tiers: [Follows::Landing, Follows::Branch(b"stable".as_slice().into()), Follows::Goals]
            .map(|follows| Tier {
                name: b"smoke".as_slice().into(),
                workflow: b"smoke.yml".as_slice().into(),
                artifact: b"result".as_slice().into(),
                format: Format::Junit,
                ignored: Box::new([b"docs".as_slice().into()]),
                follows,
                span: None,
                description: b"the slow suites".as_slice().into(),
            })
            .into(),
    };
    for (repository, text) in [
        (none, "gates of org/empty:\n  none\ntiers of org/empty:\n  none\n"),
        (
            SeeRepository { name: b"org/repo".as_slice().into(), checks },
            "gates of org/repo:\n  tests: workflow tests.yml, not blocking, ignoring docs, notes\ntiers of org/repo:\n  smoke: workflow smoke.yml, artifact result, follows the landing branch, checks the slow suites, ignoring docs\n  smoke: workflow smoke.yml, artifact result, follows stable, checks the slow suites, ignoring docs\n  smoke: workflow smoke.yml, artifact result, follows goals' branches, checks the slow suites, ignoring docs\n",
        ),
    ] {
        assert_eq!(
            rendered("settings gates org/repo", 0, SeeAnswer::Gates(Box::new(repository)), 20_000),
            (format!("{text}{not_setting}\nsee: settings landing · file OWNER/NAME PATH\n"), false)
        );
    }
}

fn branch_path() -> temper_engine_domain::ways::Path {
    use temper_engine_domain::ways::{Actor, Card, Opens, Path, Step, Stop};
    let stops = [
        (Actor::Person, Step::Ask, "ask"),
        (Actor::Agent, Step::Produce, "make"),
        (Actor::Agent, Step::Repair, "repair"),
        (Actor::Code, Step::Merge, "merge"),
        (Actor::Bar, Step::Branch, "main"),
    ]
    .map(|(actor, step, words)| Stop {
        needed: step == Step::Repair,
        over: if step == Step::Repair {
            b"tests, review".as_slice().into()
        } else if step == Step::Merge {
            b"into goal".as_slice().into()
        } else {
            Box::default()
        },
        actor,
        step,
        words: words.as_bytes().into(),
        opens: Opens::Nothing,
        card: Card { who: Box::new([]), name: Box::new([]), facts: Box::new([]) },
    });
    Path { repository: None, branch: b"main".as_slice().into(), stops: Box::new(stops) }
}

#[test]
fn ways_text_names_the_current_path_and_the_readers_permission() {
    use temper_engine_domain::ways::{WayEntry, WaysDocument};
    const PATH: &str = "ask (a person) → make (an agent) → repair (an agent, when needed; tests, review) → merge (code; into goal) → main (a branch)";
    const FOLLOWING: &str = "see: settings rules · settings landing · settings project-ways · settings project-words · settings charter-words N · help settings\n";
    for (current, may_change, first, second, own, permission) in [
        (Some(0), true, "One · now", "Two", "", "You may change this project's settings."),
        (Some(1), false, "One", "Two · now", "", "You may not change this project's settings; an owner may."),
        (None, true, "One", "Two", "Yours · now\n", "You may change this project's settings."),
        (None, false, "One", "Two", "Yours · now\n", "You may not change this project's settings; an owner may."),
    ] {
        let document = WaysDocument {
            rules: temper_engine_domain::ways::PathRules::empty(),
            project: 1,
            ways: [("One", "A way to work."), ("Two", "Another way to work."), ("No headline", "")]
                .map(|(name, headline)| WayEntry {
                    name: name.as_bytes().into(),
                    headline: headline.as_bytes().into(),
                    line: b"A way.".as_slice().into(),
                    path: branch_path(),
                    items: Box::new([]),
                    delegation: false,
                    roles: Box::default(),
                })
                .into(),
            current,
            own: branch_path(),
            words: None,
            may_change,
        };
        let own = if own.is_empty() { String::new() } else { format!("\n{own}{PATH}\n") };
        let expected = format!(
            "{first}\nA way to work.\nA way.\n{PATH}\n\n{second}\nAnother way to work.\nA way.\n{PATH}\n\nNo headline\nA way.\n{PATH}\n{own}\n{permission}\n{FOLLOWING}"
        );
        assert_eq!(rendered("ways", 0, SeeAnswer::Ways(Box::new(document)), 20_000), (expected, false));
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the step test holds the table to the stock instructions")]
fn the_address_table_keeps_the_instructions_forms_and_short_lines() {
    use crate::see::addresses::{GROUPS, forms};
    let mut out = skein_lib::Writer::new(ADDRESSES.len());
    forms(&mut out).expect("forms fit ADDRESSES");
    let text = out.finish();
    assert_eq!(text.as_ref(), ADDRESSES.as_bytes());
    let instructions = include_str!("../../../temper-service/src/stock/ask.txt");
    let line = instructions.split_once("see reads by address, in your project:\n").unwrap().1.lines().next().unwrap();
    assert_eq!(ADDRESSES, line);
    let mut seen = alloc::collections::BTreeSet::new();
    for group in GROUPS {
        assert!(!group.rows.is_empty(), "{}: no addresses", group.form);
        for row in group.rows {
            assert!(seen.insert(row.form), "{}: repeated address", row.form);
            assert!(!row.line.is_empty() && row.line.len() <= 160, "{}: line bound", row.form);
            assert!(!row.next.is_empty(), "{}: no next addresses", row.form);
        }
    }
}

#[test]
fn ways_text_prints_path_rules_and_empty_cells_as_none() {
    use temper_engine_domain::ways::{PathRule, PathRules, WaysDocument};
    let document = WaysDocument {
        project: 1,
        ways: Box::new([]),
        current: None,
        own: branch_path(),
        words: None,
        may_change: true,
        rules: PathRules {
            rows: Box::new([
                PathRule {
                    paths: b"protocol/**".as_slice().into(),
                    review: b"an agent".as_slice().into(),
                    sign_off: b"you".as_slice().into(),
                    lands: b"once you sign off".as_slice().into(),
                },
                PathRule {
                    paths: b"everything else".as_slice().into(),
                    review: Box::new([]),
                    sign_off: Box::new([]),
                    lands: b"on its own".as_slice().into(),
                },
            ]),
            note: b"A sign-off is for one head.".as_slice().into(),
        },
    };
    let (text, cut) = rendered("ways", 0, SeeAnswer::Ways(Box::new(document)), 20_000);
    assert!(!cut);
    assert_eq!(
        text,
        concat!(
            "Yours · now\nask (a person) → make (an agent) → repair (an agent, when needed; tests, review) → merge (code; into goal) → main (a branch)\n",
            "\nRules by path\nprotocol/**: review an agent; sign-off you; lands once you sign off\n",
            "everything else: review none; sign-off none; lands on its own\nA sign-off is for one head.\n",
            "\nYou may change this project's settings.\n",
            "see: settings rules · settings landing · settings project-ways · settings project-words · settings charter-words N · help settings\n",
        )
    );
}
