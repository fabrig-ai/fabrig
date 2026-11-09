//! Address grammar in the chat's project (domain/agent.md, 8.4.1).
use crate::route::see::{Address, address};
use jig_domain_core::settings::Setting;
use jig_domain_people::Proposer;

#[test]
fn addresses_parse_in_the_chats_project() {
    for (line, expected) in [
        ("settings", Address::Inventory),
        ("settings charter 1", Address::Setting(Setting::Charter(1))),
        ("settings template 5", Address::Setting(Setting::Template(5))),
        ("settings policy 1", Address::Setting(Setting::Policy(1))),
        ("settings model 1", Address::Setting(Setting::Model(1))),
        ("settings rules", Address::Setting(Setting::Rules)),
        ("settings landing-rule 1", Address::Setting(Setting::Application { kind: 1, scope: 0, number: 1 })),
        ("settings landing-rule 3 project", Address::Setting(Setting::Application { kind: 2, scope: 7, number: 3 })),
        ("settings forge-charter 1", Address::Setting(Setting::Application { kind: 3, scope: 0, number: 1 })),
        ("settings template-goal 4", Address::Setting(Setting::Application { kind: 11, scope: 0, number: 4 })),
        ("settings project-ways", Address::Setting(Setting::Application { kind: 7, scope: 7, number: 1 })),
        ("settings project-questions", Address::Setting(Setting::Application { kind: 6, scope: 7, number: 1 })),
        ("settings landing", Address::Landing),
        ("settings gates qa/pilot", Address::Gates { repository: b"qa/pilot".as_slice().into() }),
        ("revisions", Address::Revisions),
        ("revision 2", Address::Revision { number: 2 }),
        ("proposal t4/12", Address::Proposal { proposer: Proposer::Task(4), number: 12 }),
        ("proposal p2/9", Address::Proposal { proposer: Proposer::Party(2), number: 9 }),
        ("help", Address::Page { name: b"index".as_slice().into() }),
        ("help settings", Address::Page { name: b"settings".as_slice().into() }),
        ("help search landing  rule", Address::Search { words: b"landing rule".as_slice().into() }),
        (
            "file qa/pilot README.md",
            Address::File {
                repository: b"qa/pilot".as_slice().into(),
                path: b"README.md".as_slice().into(),
                branch: None,
            },
        ),
        (
            "file qa/pilot README.md main",
            Address::File {
                repository: b"qa/pilot".as_slice().into(),
                path: b"README.md".as_slice().into(),
                branch: Some(b"main".as_slice().into()),
            },
        ),
        (" \tsettings  charter 1\n", Address::Setting(Setting::Charter(1))),
        ("me", Address::NotYet),
        ("project", Address::NotYet),
        ("goals", Address::NotYet),
        ("goals ended", Address::NotYet),
        ("tasks", Address::NotYet),
        ("task 41", Address::NotYet),
        ("task 41 tree", Address::NotYet),
        ("task 41 activity", Address::NotYet),
        ("task 41 turn 1.2", Address::NotYet),
        ("change 1", Address::NotYet),
        ("changes", Address::NotYet),
        ("inbox", Address::NotYet),
        ("ways", Address::Ways),
        ("Settings", Address::Unknown),
        ("settings charter", Address::Unknown),
        ("settings charter 0", Address::Unknown),
        ("settings charter x", Address::Unknown),
        ("settings charter 4294967296", Address::Unknown),
        ("revision", Address::Unknown),
        ("revision 0", Address::Unknown),
        ("help Bad_Name", Address::Unknown),
        ("help search", Address::Unknown),
        ("file qa/pilot", Address::Unknown),
        ("file qapilot README.md", Address::Unknown),
        ("file a/b/c x", Address::Unknown),
        ("proposal x4/12", Address::Unknown),
        ("proposal t0/12", Address::Unknown),
        ("proposal p2/0", Address::Unknown),
        ("more a@1", Address::Unknown),
        ("task 0", Address::Unknown),
        ("task 41 turn 1.x", Address::Unknown),
        ("", Address::Unknown),
        (" \t\n", Address::Unknown),
        ("change 4294967296", Address::Unknown),
    ] {
        assert_eq!(address(line.as_bytes(), 7), expected, "{line}");
    }
}
