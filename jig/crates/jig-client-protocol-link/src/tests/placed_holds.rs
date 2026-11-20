use super::*;

#[test]
fn task_owner_holds_keep_every_placed_fact_and_their_independent_revisions() {
    let limits = fixtures::limits();
    for full in [false, true] {
        let number = if full { u64::MAX } else { 0 };
        let owner = if full {
            w::HoldOwner::Task(w::HoldOwnerTask { task: number })
        } else {
            w::HoldOwner::Party(w::HoldOwnerParty { person: number })
        };
        let scope =
            if full { w::HoldScope::Project } else { w::HoldScope::Subtree(w::HoldScopeSubtree { task: number }) };
        let placed = w::PlacedLine {
            number,
            owner,
            scope,
            mode: if full { w::HoldMode::Drain } else { w::HoldMode::Stop },
            reason: if full { Box::from([255; 32]) } else { Box::new([]) },
            placed_at: number,
            revision: number,
            lifted: full,
            may_lift: full,
        };
        let mut owner = jig_client_test_records::hold_record();
        owner.placed = Some(placed.clone());
        let owner = owner;
        let mut task = fixtures::task_record();
        task.hold = Some(owner);
        let task = task;
        let hold = translate::documents::task(task, &limits).unwrap().hold.unwrap();
        {
            let value = hold.placed().expect("placed accessor");
            assert_eq!(value.number, number);
            assert_eq!(value.revision.value, number);
            assert_eq!(value.placed_at.as_nanos(), number);
            assert_eq!((value.lifted, value.may_lift), (full, full));
            assert_eq!(value.reason.as_ref(), placed.reason.as_ref());
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
