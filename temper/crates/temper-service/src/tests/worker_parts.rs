use super::fixtures;
use crate::{translate, worker_route};
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_lib::Token;
use temper_protocol_worker_wire::{self as payload, v1};
const RUN: Token = Token::new(43);
#[test]
fn a_resolution_past_its_workers_charter_allowance_starts_cut() {
    // Standard hub, smith's floor, and the reference agent
    // (temper/docs/design/domain/agent/tools.md; temper/docs/design/protocol/configuration/settings.md, 7).
    // The smaller hub proves its bound still holds.
    const STANDARD: u64 = 262_144;
    const FLOOR: u64 = 65_536;
    const REFERENCE: u64 = 127_480;
    // Both sides of a large generated file, past each receiving allowance.
    const PULL_BYTES: usize = 200_000;
    const STOCK_PAYLOAD: u32 = 10_662;
    const STOCK_OUTPUT: u32 = 8192;
    let stock = fixtures::parsed_agent(fixtures::stock_configuration(STOCK_PAYLOAD, STOCK_OUTPUT).unwrap());
    assert_eq!(crate::assignment::WORKER_CHARTER_BYTES, stock.host_limits.charter_bytes);
    // The smaller hub holds the fixture's fixed charter, title and one cut line.
    let mut minimum = temper_engine_smith::charter(&fixtures::resolution_with_pull(PULL_BYTES));
    minimum.brief.sections.first_mut().unwrap().text =
        b"[cut 200000 bytes; whole: read_pull/read_pull_files]\n".as_slice().into();
    let encoded = smith_protocol_translate::encode_charter(
        &minimum,
        &smith_charter::CEILINGS,
        &fixtures::encoding(&fixtures::engine()).endpoints,
    )
    .unwrap();
    let smaller_hub = minimum.owned_bytes().unwrap().max(skein_lib::len_u64(encoded.len()));
    assert!(smaller_hub < FLOOR);
    let rows = [
        ("engine_alone_under_standard", STANDARD, None, FLOOR, FLOOR, 0),
        ("local_agent_takes_more", STANDARD, Some(REFERENCE), REFERENCE, REFERENCE, FLOOR),
        ("hub_below_its_agent", smaller_hub, Some(REFERENCE), REFERENCE, smaller_hub, 0),
    ];
    let mut outcomes = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, hub, local, worker, bound, more_than) in rows {
        let mut settings = fixtures::engine();
        settings.jig.limits.hub.charter_bytes = hub;
        settings.agent_charter_bytes = local;
        let encoding = fixtures::encoding(&settings);
        let Some(jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, .. }) =
            crate::assignment::down(
                fixtures::resolution_with_pull(PULL_BYTES),
                &fixtures::forge_table(&fixtures::forges()),
                &encoding,
                &fixtures::values_with_identity(1, fixtures::FORGE_TOKEN),
                RUN,
                skein_lib::Time::ZERO,
            )
        else {
            panic!("{name}: encoded resolution")
        };
        let encoded = skein_lib::len_u64(assignment.assignment.charter.len());
        let live = assignment.assignment.live;
        let source = start(assignment.assignment, live);
        let mut agent = fixtures::agent();
        agent.host_limits.charter_bytes = worker;
        match worker_route::decode(RUN, source, &agent) {
            Ok(start) => {
                assert!(encoded <= bound, "{name}: encoded charter exceeds the receiving bound");
                assert!(encoded > more_than, "{name}: larger local allowance keeps more of the brief");
                let [section] = start.charter.value().brief.sections.as_ref() else {
                    panic!("{name}: pull section retained")
                };
                assert_eq!(section.title.as_ref(), b"Pull request", "{name}");
                assert!(section.text.ends_with(b" bytes; whole: read_pull/read_pull_files]\n"), "{name}");
                outcomes.push((name, Ok(()))).unwrap();
            }
            Err(why) => outcomes.push((name, Err(why))).unwrap(),
        }
    }
    assert_eq!(outcomes.as_slice(), &rows.map(|(name, ..)| (name, Ok(()))));
}

#[test]
fn an_engine_s_items_reach_the_worker_s_workspace_field_for_field() {
    let assignment = fixtures::assignment_with(fixtures::repository(3, b"acme", b"app"));
    let limits = fixtures::wire_limits();
    let items = translate::items(
        &assignment,
        &fixtures::forge_table(&fixtures::forges()),
        &crate::forge::tokens::Tokens::empty(),
        &limits,
    )
    .unwrap()
    .unwrap();
    for program in [None, Some(Box::from(b"/fake/git".as_slice()))] {
        let mut fields = items.clone().into_parts();
        let mut repositories = skein_lib::List::with_capacity(1);
        for repository in fields.repositories.into_boxed() {
            let mut fields = repository.into_parts();
            fields.git = program.clone();
            repositories.push(v1::Repository::new(&limits, fields).unwrap()).unwrap();
        }
        fields.repositories = repositories;
        let record = v1::Items::new(&limits, fields).unwrap();
        let parts = [Part { token: Token::new(9), bytes: payload::items(&record) }];
        let workspace = worker_route::items(&fixtures::worker_assignment(), &parts, &limits).unwrap();
        let mut expected = fixtures::one_repository();
        expected.repositories[0].git = program;
        expected.repositories[0].save = Some(b"temper/7/s7".as_slice().into());
        assert_eq!(workspace, expected);
    }
    assert_eq!(translate::workspace(None), wire::Workspace { repositories: Box::new([]) });
}

fn start(assignment: host::RunAssignment, live: host::LiveAsk) -> host::Start {
    host::Start {
        live,
        workspace: None,
        charter: assignment.charter,
        activation: 1,
        identity: assignment.identity,
        turns: Box::new([]),
        answered: Box::new([]),
        messages: Box::new([]),
        grants: Box::new([]),
    }
}

use temper_worker_domain::wire;
