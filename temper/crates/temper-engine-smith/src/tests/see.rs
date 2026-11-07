//! Tokens, whole texts and bounded parts (domain/agent.md, 8.4.1).
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
    let (body, error) = see_answer(line.as_bytes(), from, &answer, &[], maximum).into_parts();
    assert!(body.len() <= usize::try_from(maximum).unwrap());
    (String::from_utf8(body.into_vec()).unwrap(), error)
}

#[test]
fn tokens_round_trip_and_malformed_tokens_stay_unknown() {
    for (line, from, token) in
        [("settings policy 1", 6120, "settings+policy+1@6120"), ("a+b@c%d", 1, "a%2Bb%40c%25d@1")]
    {
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
    for (line, answer, expected) in [
        ("me", SeeAnswer::NotYet, "see me: not there yet.".to_string()),
        ("bogus", SeeAnswer::Unknown, format!("see: no address bogus; addresses: {ADDRESSES}")),
        (
            "help search x",
            SeeAnswer::Search { words: b"".as_slice().into() },
            "see help search x: nothing to search for".to_string(),
        ),
    ] {
        assert_eq!(rendered(line, 0, answer, 20_000), (expected, true));
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
    let repository = temper_engine_domain::SeeRepository {
        name: b"a/b".as_slice().into(),
        checks: temper_engine_domain_forge::RepositoryGates {
            repository: skein_forge::api::Repository { forge: 1, repository: 1 },
            gates: Box::new([]),
            tiers: Box::new([]),
        },
    };
    for (line, answer) in [
        (
            "settings landing",
            SeeAnswer::Landing(Box::new(temper_engine_domain::SeeLanding {
                by: temper_engine_domain_forge_change::Way::Queue,
                merge_job: None,
                after: None,
                repositories: Box::new([]),
            })),
        ),
        ("settings gates a/b", SeeAnswer::Gates(Box::new(repository))),
        ("proposal t1/1", SeeAnswer::Proposal(Box::new(temper_engine_domain::SeeProposal::Unknown))),
    ] {
        assert_eq!(rendered(line, 0, answer, 20_000), (format!("see {line}: not there yet."), true));
    }
    for (answer, next) in [
        (
            jig_domain_core::SettingsAnswer::Inventory(jig_domain_core::documents::Settings {
                newest: 1,
                settings: skein_lib::List::with_capacity(0),
                held: skein_lib::List::with_capacity(0),
                pending: skein_lib::List::with_capacity(0),
            }),
            "settings charter N · settings template N · settings policy N · settings model N · settings forge-charter N · settings rules · settings landing-rule N [project] · settings landing · settings gates OWNER/NAME · settings project-ways · settings project-questions · revisions · help settings",
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
    ] {
        let answer = jig_domain_core::SettingsAnswer::Setting(jig_domain_core::reads::SettingDocument {
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
            let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
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
