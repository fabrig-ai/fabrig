//! Whole requester preferences in asking briefs (`jig/docs/design/domain/engine/briefs.md`, 2.1).

use crate::settings_support as support;
use jig_core_world::{
    client_peer::ClientPeer,
    fixtures,
    world::{Boot, Setup, World},
};
use jig_domain_brief::{Core as Section, GatherPlaced};
use jig_domain_core as core;
use jig_domain_link::Hello;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, List, Token, Wall};
use std::fmt::Write as _;

const P: u64 = support::OWNER;
const Q: u64 = 2; // The second configured owner, signed in after P.
// The party-scope bounds in domain/engine/briefs.md, 2.1.
const PARTY_ENTRIES: usize = 12;
const PARTY_BYTES: usize = 120;

type Kept = (u64, u64, Vec<u8>);

fn two_people() -> Vec<Kept> {
    vec![
        (P, 90, b"Second preference".to_vec()),
        (Q, 5, b"Q's preference".to_vec()),
        (P, 10, b"First preference".to_vec()),
    ]
}

fn start(kept: &[Kept], charter_tools: jig_domain_tasks::Tools) -> (World, [u64; 2]) {
    let (mut config, limits) = support::run_configuration();
    config.core.defaults.charters.first_mut().expect("profile charter").policy.tools = charter_tools;
    let mut owners = config.core.owners.to_vec();
    let mut other = owners.first().expect("configured owner").clone();
    other.identity.subject = b"owner-b".as_slice().into();
    owners.push(other);
    config.core.owners = owners.into_boxed_slice();
    let mut rows = jig_test_profile::fixtures::account_rows(&config.core.settings.llm_providers).into_vec();
    for (party, name, description) in kept {
        let scope = notes::Scope::Party { party: *party };
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            scope: scope.clone(),
            name: *name,
            description: description.clone().into(),
            revision: 1,
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
            scope,
            name: *name,
            description: description.clone().into(),
            body: Box::default(),
            author: notes::Author::Party { party: *party },
            revision: 1,
            references: List::with_capacity(0),
            changed: Wall::from_nanos(*name),
        }))));
    }
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        rows: Some(rows.into_boxed_slice()),
        boot: Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let signed_p = support::sign_in(&mut world);
    world.send(fixtures::sign_in(Token::new(support::SEED), 1, b"owner-b", &[], &[]));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { sign_in: signed_q, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("client peer").delivered.pop().expect("second sign in")
    else {
        panic!("Q signs in")
    };
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::default(),
            hosting: Box::default(),
        },
    })));
    (world, [signed_p, signed_q])
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table keeps the preference gates and whole-section cases together")]
fn an_asking_run_reads_its_persons_preferences_whole() {
    struct Row {
        label: &'static str,
        person: usize,
        asking: bool,
        charter_tools: jig_domain_tasks::Tools,
        kept: Vec<Kept>,
        section: Section,
        expected: Option<Vec<u8>>,
    }
    let long_line = vec![b'x'; PARTY_BYTES];
    let mut whole = String::new();
    for place in 1..=PARTY_ENTRIES {
        writeln!(&mut whole, "{place}. {}", "x".repeat(PARTY_BYTES)).expect("whole preferences");
    }
    let rows = [
        Row {
            label: "P, name order",
            person: 0,
            asking: true,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: two_people(),
            section: Section::Preferences,
            expected: Some(b"1. First preference\n2. Second preference\n".to_vec()),
        },
        Row {
            label: "P, none",
            person: 0,
            asking: true,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: vec![],
            section: Section::Preferences,
            expected: Some(b"None kept.\n".to_vec()),
        },
        Row {
            label: "P, full",
            person: 0,
            asking: true,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: (1..=PARTY_ENTRIES)
                .rev()
                .map(|name| (P, u64::try_from(name).expect("bounded entries"), long_line.clone()))
                .collect(),
            section: Section::Preferences,
            expected: Some(whole.into_bytes()),
        },
        Row {
            label: "Q, own lines",
            person: 1,
            asking: true,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: two_people(),
            section: Section::Preferences,
            expected: Some(b"1. Q's preference\n".to_vec()),
        },
        Row {
            label: "P, no asking family",
            person: 0,
            asking: false,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: two_people(),
            section: Section::Preferences,
            expected: None,
        },
        Row {
            label: "P, charter without asking family",
            person: 0,
            asking: true,
            charter_tools: jig_domain_tasks::Tools(2047),
            kept: two_people(),
            section: Section::Preferences,
            expected: None,
        },
        Row {
            label: "P, separate notes index",
            person: 0,
            asking: true,
            charter_tools: core::RunPolicy::EVERY_FAMILY,
            kept: two_people(),
            section: Section::NotesIndex,
            expected: Some(b"No notes\n".to_vec()),
        },
    ];
    for row in rows {
        let (mut world, signed) = start(&row.kept, row.charter_tools);
        let mut goal = fixtures::chat(&world, 1, b"Explain the work".as_slice().into());
        let people::Ask::Create { batch, .. } = &mut goal else { panic!("goal creation") };
        let member = batch.first_mut().expect("one task");
        member.flags.ended_by = people::EndedBy::Result;
        member.authority.tools = 2047 | if row.asking { core::ToolFamilies::JIG.ask.0 } else { 0 };
        support::ask(&mut world, signed[row.person], 101, goal);
        let assignment = world.assigned.as_ref().unwrap_or_else(|| {
            panic!(
                "{}: brief received at the root; answers {:?}; trace {:?}",
                row.label, world.people_answers, world.trace
            )
        });
        let received = assignment.brief.iter().find_map(|section| {
            if let GatherPlaced::Core { kind, text, .. } = section
                && *kind == row.section
            {
                Some(text.to_vec())
            } else {
                None
            }
        });
        assert_eq!(received, row.expected, "{}", row.label);
        fixtures::finish_chat(&mut world, 0, b"Explained");
        world.close();
    }
}
