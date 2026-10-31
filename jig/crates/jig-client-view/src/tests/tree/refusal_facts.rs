#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect complete semantic trees"
)]
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::{CLIENT, limits, shared_words, text};
use jig_client::contract::*;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role, Tree};
fn render(refusal: Refusal) -> Tree {
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    shared_words().refusal(&mut builder, &refusal, CLIENT.client.text);
    builder.close();
    let tree = builder.finish();
    assert!(tree.len() <= crate::refusal_nodes(&CLIENT, &(CLIENT).client).unwrap().checked_add(1).unwrap());
    crate::tests::tree::cards_access::check(&tree);
    tree
}
#[test]
#[expect(clippy::too_many_lines, reason = "all independent engine refusal categories are checked together")]
fn engine_refusals_keep_categories_supplied_hold_reasons_and_absent_ending_details() {
    let mut goldens = Goldens::new("tree/refusal_facts/engine_refusals_keep");
    for which in [
        AdmissionLimit::Words,
        AdmissionLimit::Identity,
        AdmissionLimit::Roles,
        AdmissionLimit::Arithmetic,
        AdmissionLimit::Duplicate,
        AdmissionLimit::Empty,
        AdmissionLimit::Batch,
        AdmissionLimit::ProjectLiveTasks,
        AdmissionLimit::LiveTasks,
        AdmissionLimit::Tree,
        AdmissionLimit::Depth,
        AdmissionLimit::Delegates,
        AdmissionLimit::Subscription,
        AdmissionLimit::Dependencies,
        AdmissionLimit::Cycle,
        AdmissionLimit::Spec,
        AdmissionLimit::Contract,
        AdmissionLimit::Authority,
        AdmissionLimit::Inputs,
        AdmissionLimit::Attempts,
        AdmissionLimit::LiveDelegates,
        AdmissionLimit::Restore,
        AdmissionLimit::Read,
        AdmissionLimit::Turn,
        AdmissionLimit::HoldKind,
        AdmissionLimit::HoldTaken,
        AdmissionLimit::Holds,
        AdmissionLimit::InboxFull,
        AdmissionLimit::Notes,
        AdmissionLimit::Revision,
        AdmissionLimit::Question,
        AdmissionLimit::Priority,
        AdmissionLimit::Policy,
        AdmissionLimit::Resource,
        AdmissionLimit::Service,
    ] {
        let tree = render(Refusal::Limit { which: Limit::Admission { which } });
        goldens.check(&tree);
        assert_eq!(tree.len(), 2);
    }
    for held in [
        None,
        Some(HeldRole::Owner),
        Some(HeldRole::Maintainer),
        Some(HeldRole::Member),
        Some(HeldRole::Observer),
        Some(HeldRole::Policy { role: 7 }),
        Some(HeldRole::Policy { role: u32::MAX }),
    ] {
        let tree = render(Refusal::Role { held });
        goldens.check(&tree);
        assert_eq!(tree.len(), 3);
    }
    for lacking in [
        Lacking::Approver,
        Lacking::Budget { needed: 8, left: 3 },
        Lacking::GrantKind { kind: 9 },
        Lacking::Funding,
        Lacking::TaskKind { executor: None },
        Lacking::Depth { needed: None, allowed: 7 },
        Lacking::Deadline { asked: None, latest: None },
        Lacking::Tasks { needed: None, left: 4 },
        Lacking::Tool,
        Lacking::Scope,
        Lacking::Models,
        Lacking::Reference,
        Lacking::ResourceAccess,
        Lacking::ResourceUnavailable,
        Lacking::Shape,
        Lacking::Again { required: true },
        Lacking::Again { required: false },
        Lacking::Account,
        Lacking::Writer,
        Lacking::Required { connector: 2, kind: 9 },
        Lacking::Failed { connector: 2, kind: 9 },
        Lacking::Unguarded { connector: 2, kind: 9 },
    ] {
        let tree = render(Refusal::Authority { lacking, proposable: None });
        assert!(tree.len() > 1);
        goldens.check(&tree);
    }
    for refusal in [Refusal::Taken { by: 7 }, Refusal::NotWaiting, Refusal::NoFurther] {
        assert!(render(refusal).len() > 1);
    }
    for how in [
        Ending::Failed { reason: None },
        Ending::Deleted,
        Ending::Cancelled { by: Actor::ProjectDeployment { project: 6 }, reason: None, at: Some(Wall::EPOCH) },
        Ending::Decided { by: 7, party: None, how: None, at: Wall::EPOCH },
    ] {
        let tree = render(Refusal::Ended { how });
        goldens.check(&tree);
    }
    let mut failures = List::with_capacity(2);
    for attempt in [1, 2] {
        failures
            .push(AttemptFailure {
                attempt,
                class: FailureClass::HostLost,
                reason: text(b"whole failure reason"),
                at: Wall::EPOCH,
            })
            .unwrap();
    }
    let tree = render(Refusal::Moved {
        now: Revision { value: 23 },
        what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(HoldReason::Tries { failures }))) },
    });
    goldens.check(&tree);
}

#[test]
fn sparse_current_hold_facts_keep_only_supplied_categories_and_numeric_identities() {
    let mut goldens = Goldens::new("tree/refusal_facts/sparse_current_hold");
    let mut reasons = Vec::from([
        HoldCategory::Circle,
        HoldCategory::Stalled,
        HoldCategory::Dependency { task: 123 },
        HoldCategory::Stopped,
        HoldCategory::Placed { hold: 123 },
        HoldCategory::Drift,
        HoldCategory::Effects,
        HoldCategory::EffectFailed,
        HoldCategory::Uncertain { entry: 123 },
        HoldCategory::Budget,
        HoldCategory::Deadline,
        HoldCategory::HoldsWaited,
        HoldCategory::ResourceUnavailable,
        HoldCategory::ResourceAccess,
        HoldCategory::PoolLost { pool: 123 },
        HoldCategory::Procedure,
    ]);
    for class in [
        HoldFailure::Transient,
        HoldFailure::Permanent,
        HoldFailure::Run,
        HoldFailure::Agent,
        HoldFailure::Lost,
        HoldFailure::Invalid,
    ] {
        reasons.push(HoldCategory::Failures { class });
    }
    for reason in reasons {
        let tree = render(Refusal::Moved {
            now: Revision { value: 23 },
            what: Moved::Hold { reason: Some(CurrentHold::Reason(reason)) },
        });
        goldens.check(&tree);
        assert!(tree.len() >= 4);
        assert!(!tree.nodes().iter().any(|node| node.role == Role::Time));

        match reason {
            HoldCategory::Exceeded(_) => unreachable!("bound phrases have their own table"),
            HoldCategory::Circle => goldens.check(&tree),
            HoldCategory::Placed { .. }
            | HoldCategory::Dependency { .. }
            | HoldCategory::Uncertain { .. }
            | HoldCategory::PoolLost { .. } => {
                goldens.check(&tree);
            }
            HoldCategory::Failures { .. }
            | HoldCategory::Stalled
            | HoldCategory::Stopped
            | HoldCategory::Drift
            | HoldCategory::Effects
            | HoldCategory::EffectFailed
            | HoldCategory::Budget
            | HoldCategory::Deadline
            | HoldCategory::HoldsWaited
            | HoldCategory::ResourceUnavailable
            | HoldCategory::ResourceAccess
            | HoldCategory::Procedure => {}
        }
    }
    for part in [
        Part::Panel { connector: 2, task: 123 },
        Part::Result { connector: 2, task: 123 },
        Part::Hold { connector: 2, task: 123 },
    ] {
        let tree = render(Refusal::Moved {
            now: Revision { value: 23 },
            what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(HoldReason::Procedure { part }))) },
        });
        goldens.check(&tree);
    }
}

#[test]
fn outbox_hold_facts_show_the_actual_entry_scope_without_a_proposal_identity() {
    let mut goldens = Goldens::new("tree/refusal_facts/outbox_hold_facts");
    for scope in [PartScope::Project(u32::MAX), PartScope::Deployment] {
        let tree = render(Refusal::Moved {
            now: Revision { value: 9 },
            what: Moved::Hold {
                reason: Some(CurrentHold::Document(Box::new(HoldReason::EffectUncertain {
                    part: Part::Outbox { connector: 2, scope, entry: u64::MAX },
                }))),
            },
        });
        goldens.check(&tree);
    }
}

#[test]
fn the_account_sign_in_limits_name_each_refusal() {
    let mut goldens = Goldens::new("tree/refusal_facts/the_account_sign");
    for which in
        [AdmissionLimit::Accounts, AdmissionLimit::HolderAccounts, AdmissionLimit::Sources, AdmissionLimit::SignIns]
    {
        let tree = render(Refusal::Limit { which: Limit::Admission { which } });
        goldens.check(&tree);
    }
}

#[test]
fn each_configured_bound_names_its_allowance_in_the_reference_view() {
    let mut goldens = Goldens::new("tree/refusal_facts/each_configured_bound");
    for bound in [
        Bound::Reasoning,
        Bound::Metadata,
        Bound::Items,
        Bound::Answer,
        Bound::Tools,
        Bound::HistoryItems,
        Bound::History,
        Bound::Window,
        Bound::Output,
    ] {
        let reason = HoldCategory::Exceeded(Exceeded { bound, value: u64::MAX });
        let tree = render(Refusal::Moved {
            now: Revision { value: 23 },
            what: Moved::Hold { reason: Some(CurrentHold::Reason(reason)) },
        });
        goldens.check(&tree);
    }
}

#[test]
fn project_name_refusals_say_the_failed_rule_and_projects_limit() {
    let mut goldens = Goldens::new("tree/refusal_facts/project_name_refusals");
    goldens.check(&render(Refusal::NameTaken));
    goldens.check(&render(Refusal::Limit { which: Limit::Admission { which: AdmissionLimit::Projects } }));
    for rule in [NameRule::Empty, NameRule::Long, NameRule::Control, NameRule::Space, NameRule::Digits] {
        goldens.check(&render(Refusal::Malformed { at: 0, why: Malformed::Name(rule) }));
    }
}

#[test]
fn account_move_refusals_show_the_supplied_reason() {
    let mut goldens = Goldens::new("tree/refusal_facts/account_move_refusals");
    for why in [
        AccountMoveWhy::NotSignedIn,
        AccountMoveWhy::NotOurs,
        AccountMoveWhy::Held,
        AccountMoveWhy::Cancelled,
        AccountMoveWhy::Moving,
    ] {
        goldens.check(&render(Refusal::AccountMove { why }));
    }
}
#[test]
fn moving_out_words_show_only_account_and_move_numbers() {
    let mut goldens = Goldens::new("tree/refusal_facts/moving_out_words");
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    let outcome = Outcome::Made {
        what: Made::MovingOut {
            account: 19,
            number: 23,
            record: skein_lib::Secret::new(Box::from(b"private moved record".as_slice())),
        },
    };
    shared_words().outcome(&mut builder, &outcome);
    builder.close();
    let tree = builder.finish();
    goldens.check(&tree);
}

#[test]
fn settings_refusals_name_each_setting_fault_and_retained_holder_fact() {
    let mut goldens = Goldens::new("tree/refusal_facts/settings_refusals_name");
    for setting in [
        Setting::Charter(SettingNumber { number: 2 }),
        Setting::Template(SettingNumber { number: 1 }),
        Setting::Policy(SettingNumber { number: 3 }),
        Setting::Rules,
    ] {
        for fault in [
            SettingFault::Shape,
            SettingFault::Count,
            SettingFault::Reference,
            SettingFault::Rules,
            SettingFault::Budget,
            SettingFault::Grace,
            SettingFault::Owner,
        ] {
            let tree =
                render(Refusal::Malformed { at: 0, why: Malformed::Settings(MalformedSettings { setting, fault }) });
            goldens.check(&tree);
        }
    }
    goldens.check(&render(Refusal::Stale(RefusalStale { newest: 29 })));
    let tasks = crate::tests::kit::list([11, 13]);
    let tree = render(Refusal::Full(RefusalFull { revision: 7, tasks, more: 17 }));
    goldens.check(&tree);
    goldens.check(&render(Refusal::Malformed { at: 0, why: Malformed::SettingsUnchanged }));
    goldens.check(&render(Refusal::Malformed { at: 0, why: Malformed::Running }));
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    shared_words().outcome(&mut builder, &Outcome::SettingsChanged(OutcomeSettingsChanged { revision: 31 }));
    builder.close();
    goldens.check(&builder.finish());
}
