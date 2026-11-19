//! Milestone reads cross the project's family boundary (domain/milestones.md, 7).
use super::*;

#[test]
fn milestone_reads_keep_their_scope_totals_and_absence() {
    let reading = w::Reading::new(
        &w::CEILINGS,
        w::ReadingParts {
            read: w::Read::Milestone(
                w::MilestoneRead::new(&w::CEILINGS, w::MilestoneReadParts { project: 7, task: 812 }).unwrap(),
            ),
        },
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::ForgeRead { reply, sign_in, scope, query }) =
        away_family(kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            record: wire::reading(&reading),
            window: jig_domain_core::documents::Window { rows: 0, after: None },
            scope: people::ApplicationScope::Project(7),
        })
    else {
        panic!("milestone read")
    };
    assert_eq!(
        (reply, sign_in, scope, query),
        (REPLY, 3, people::ApplicationScope::Project(7), root::ForgeRead::Milestone { project: 7, task: 812 })
    );
    let result = family::to_root(
        &jig_protocol_config::Directories::working(),
        kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            record: wire::reading(&reading),
            window: jig_domain_core::documents::Window { rows: 0, after: None },
            scope: people::ApplicationScope::Project(8),
        },
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None, push: None },
        None,
        &w::CEILINGS,
    );
    let Err(reply) = result else { panic!("mismatched scope refused") };
    let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *reply else {
        panic!("empty read answer")
    };
    assert_eq!((reply, family, record), (REPLY, wire::FORGE, None));
    let totals = forge::MilestoneTotals {
        task: 812,
        done: 1,
        failed: 2,
        closed: 3,
        live: 4,
        ended: Some(skein_lib::Wall::from_nanos(89)),
        last_landing: Some(forge::MilestoneLanding { at: skein_lib::Wall::from_nanos(87), goal: 815, change: 816 }),
        spent: 100,
        reds: 0,
    };
    let bytes = family::forge_read(root::ForgeReadAnswer::Milestone(Some(totals)), b"", &w::CEILINGS).unwrap();
    let result = wire::decode_milestone_answer(&bytes, &w::CEILINGS).unwrap().into_parts().result;
    let w::MilestoneResult::Totals(totals) = result else { panic!("totals") };
    let totals = totals.into_parts();
    assert_eq!(
        (totals.task, totals.done, totals.failed, totals.closed, totals.live, totals.ended, totals.spent),
        (812, 1, 2, 3, 4, Some(89), 100)
    );
    let landing = totals.last_landing.unwrap().into_parts();
    assert_eq!((landing.at, landing.goal, landing.change), (87, 815, 816));
    let bytes = family::forge_read(root::ForgeReadAnswer::Milestone(None), b"", &w::CEILINGS).unwrap();
    assert_eq!(
        wire::decode_milestone_answer(&bytes, &w::CEILINGS).unwrap().into_parts().result,
        w::MilestoneResult::NotMilestone
    );
}
