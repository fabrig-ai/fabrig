use super::*;

#[test]
fn task_owner_holds_keep_every_placed_fact_and_their_independent_revisions() {
    let limits = fixtures::limits();
    for full in [false, true] {
        let number = if full { u64::MAX } else { 0 };
        let owner = if full {
            w::HoldOwner::Task(w::HoldOwnerTask::new(&w::CEILINGS, w::HoldOwnerTaskParts { task: number }).unwrap())
        } else {
            w::HoldOwner::Party(
                w::HoldOwnerParty::new(&w::CEILINGS, w::HoldOwnerPartyParts { person: number }).unwrap(),
            )
        };
        let scope = if full {
            w::HoldScope::Project
        } else {
            w::HoldScope::Subtree(
                w::HoldScopeSubtree::new(&w::CEILINGS, w::HoldScopeSubtreeParts { task: number }).unwrap(),
            )
        };
        let placed = w::PlacedLine::new(
            &w::CEILINGS,
            w::PlacedLineParts {
                number,
                owner,
                scope,
                mode: if full { w::HoldMode::Drain } else { w::HoldMode::Stop },
                reason: if full { Box::from([255; 32]) } else { Box::new([]) },
                placed_at: number,
                revision: number,
                lifted: full,
                may_lift: full,
            },
        )
        .unwrap();
        let mut owner = jig_client_test_records::hold_record().into_parts();
        owner.placed = Some(placed.clone());
        let owner = w::OwnerChipHold::new(&w::CEILINGS, owner).unwrap();
        let mut task = fixtures::task_record().into_parts();
        task.hold = Some(owner);
        let task = w::Task::new(&w::CEILINGS, task).unwrap();
        let hold = translate::documents::task(task, &limits).unwrap().hold.unwrap();
        {
            let value = hold.placed().expect("placed accessor");
            assert_eq!(value.number, number);
            assert_eq!(value.revision.value, number);
            assert_eq!(value.placed_at.as_nanos(), number);
            assert_eq!((value.lifted, value.may_lift), (full, full));
            assert_eq!(value.reason.as_ref(), placed.reason());
            assert_eq!(
                value.owner,
                if full { c::HoldOwner::Task { task: number } } else { c::HoldOwner::Party { person: number } }
            );
            assert_eq!(value.mode, if full { c::HoldMode::Drain } else { c::HoldMode::Stop });
            assert_eq!(value.scope, if full { c::HoldScope::Project } else { c::HoldScope::Subtree { task: number } });
            assert_ne!(hold.revision().value, number);
        }
    }
}
