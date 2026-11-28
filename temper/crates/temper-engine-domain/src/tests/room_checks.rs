//! Charter values are checked before reaching the core
//! (temper/docs/design/domain/agent/tools.md).
use super::resume_fixture;
use crate::room::{Measure, Refusal, Short, Unfit};
use crate::{Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request};
use alloc::boxed::Box;
use jig_domain_core::{
    self as core,
    settings::{self as settings, Edit, Item, Setting, Value},
};
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token, Wall};

// A synthetic host part isolates the admission boundary from smith's measured table.
const ZERO: crate::room::Cost = crate::room::Cost { rendered: 0, owned: 0, encoded: 0 };
const HOST: crate::room::HostPart = crate::room::HostPart {
    bound: Some(3_048),
    allowance: u64::MAX,
    fixed: ZERO,
    model: ZERO,
    always: ZERO,
    families: [ZERO; 12],
    asking: ZERO,
    asking_settings: ZERO,
    inspect: 0,
    modify: 0,
    shell: 0,
    agents: 0,
    floor: ZERO,
    ..crate::room::HostPart::EMPTY
};
const CHARTER: u32 = 1;
const REPLY: Token = Token::new(9);
// 3,048 - (999 instruction bytes + two line-ending bytes) = 2,047.
const SHORT_INSTRUCTIONS: u32 = 999;
const WORDS: &[u8] = b"charter 1 leaves its brief 2,047 bytes of its host part's 3,048; it needs at least 2,048";

fn root(host: &crate::room::HostPart) -> (Domain, Env<Limits>) {
    let mut config = resume_fixture::config(7);
    config.host_part = *host;
    resume_fixture::root_with_config(config)
}
fn short_charter() -> crate::Charter {
    let mut charter = resume_fixture::charter();
    charter.policy.instructions = super::bytes(SHORT_INSTRUCTIONS);
    charter
}
fn parts(bytes: u32) -> Box<[core::CharterPart]> {
    Box::new([core::CharterPart { on: jig_domain_tasks::Reasons::FIRST, instructions: super::bytes(bytes) }])
}

fn overlay(instructions: u32, part: Option<u32>) -> settings::Overlay {
    let mut overlay = settings::Overlay::empty();
    overlay.instructions = super::bytes(instructions);
    overlay.parts = match part {
        Some(bytes) => parts(bytes),
        None => Box::new([]),
    };
    overlay.shell = Some(false);
    overlay.resume = Some(false);
    overlay
}

fn overlay_items(overlay: settings::Overlay) -> Box<[Item]> {
    Box::new([Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Overlay(Box::new(overlay))) }])
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture replaces items at the two party settings entrances")]
fn with_items(input: PartyInput, items: Box<[Item]>) -> PartyInput {
    match input {
        PartyInput::ChangeSettings { reply_to, sign_in, key, digest, kinds, mut change } => {
            change.items = items;
            PartyInput::ChangeSettings { reply_to, sign_in, key, digest, kinds, change }
        }
        PartyInput::ProposeSettings { reply_to, sign_in, key, digest, kinds, mut change } => {
            change.items = items;
            PartyInput::ProposeSettings { reply_to, sign_in, key, digest, kinds, change }
        }
        _ => unreachable!("a settings entrance"),
    }
}

fn items() -> Box<[Item]> {
    Box::new([
        Item { setting: Setting::Rules, edit: Edit::Reset },
        Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Charter(Box::new(short_charter()))) },
    ])
}
fn change() -> PartyInput {
    PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Change { against: 1, items: items(), running: settings::Running::Stays }),
    }
}
fn proposal() -> PartyInput {
    PartyInput::ProposeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Proposed { against: 1, items: items(), reason: b"a charter".as_slice().into() }),
    }
}
fn refused(input: PartyInput) -> people::Refusal {
    let (mut root, env) = root(&HOST);
    crate::try_step(&mut root, &env, Event::Party(input)).unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut root, &env, &mut out);
    crate::release(&mut root, &env, &mut out);
    let Some(Request::Party(PartyRequest::Deliver(Delivery::WebReply { to, reply: people::Reply::Refused(why) }))) =
        out.pop()
    else {
        panic!("charter refused at the settings entrance");
    };
    assert_eq!(to.into_token(), REPLY);
    assert!(out.is_empty());
    why
}
fn judged() -> people::Refusal {
    people::Refusal::Judged { at: 1, setting: Setting::Charter(CHARTER), why: WORDS.into() }
}

#[test]
fn a_partys_change_counts_every_part() {
    fn item(charter: crate::Charter) -> Box<[Item]> {
        Box::new([Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Charter(Box::new(charter))) }])
    }
    // Instructions alone leave 4,000 bytes; the part and its blank line leave 998.
    let host = crate::room::HostPart { bound: Some(5_001), ..HOST };
    let mut charter = short_charter();
    charter.policy.resume = false;
    let mut limits = resume_fixture::limits();
    limits.journal.run_bytes = 8_192;
    let (mut root, env) = owner_root_with_limits(&limits);
    root.host_part = host;
    charter.parts = parts(3_000);
    assert_eq!(
        settled(&mut root, &env, with_items(change(), item(charter.clone()))),
        people::Reply::Refused(people::Refusal::Judged {
            at: 0,
            setting: Setting::Charter(CHARTER),
            why: b"charter 1 leaves its brief 998 bytes of its host part's 5,001; it needs at least 2,048"
                .as_slice()
                .into(),
        })
    );
    charter.parts = Box::new([]);
    assert_eq!(
        settled(&mut root, &env, with_items(change(), item(charter))),
        people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
    );
}

#[test]
fn a_partys_change_judges_overlays_over_the_default() {
    party_overlays(false);
}

#[test]
fn a_partys_proposal_judges_overlays_over_the_default() {
    party_overlays(true);
}

fn party_overlays(propose: bool) {
    let default_bytes = u32::try_from(resume_fixture::charter().policy.instructions.len()).unwrap();
    let added = SHORT_INSTRUCTIONS.checked_sub(default_bytes).unwrap().checked_sub(2).unwrap();
    let fitting = added.checked_sub(1).unwrap();
    for (overlay, short) in [
        (overlay(added, None), true),
        (overlay(0, Some(added)), true),
        (overlay(fitting, None), false),
        (overlay(0, Some(fitting)), false),
        (overlay(0, None), false),
    ] {
        let (mut root, env) = owner_root();
        root.host_part = HOST;
        let input = with_items(if propose { proposal() } else { change() }, overlay_items(overlay));
        let reply = settled(&mut root, &env, input);
        if short {
            assert_eq!(
                reply,
                people::Reply::Refused(people::Refusal::Judged {
                    at: 0,
                    setting: Setting::Charter(CHARTER),
                    why: WORDS.into(),
                })
            );
        } else if propose {
            let people::Reply::Outcome(people::Outcome::Proposed { .. }) = reply else { panic!("{reply:?}") };
        } else {
            assert_eq!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 }));
        }
    }
}

#[test]
fn an_overlay_is_judged_over_the_default_not_the_current_change() {
    let (mut root, env) = owner_root();
    assert_eq!(
        settled(&mut root, &env, change()),
        people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
    );
    root.host_part = HOST;
    // The current whole charter is short; a fields-only overlay replaces it with the fitting default.
    assert_eq!(crate::route::room_refusal(&root, &overlay_items(overlay(0, None))), None);
}

fn restored_overlay(
    host: &crate::room::HostPart,
    overlay: settings::Overlay,
) -> (Domain, Env<Limits>, resume_fixture::StartStore) {
    let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
    root.host_part = *host;
    let mut old_default = resume_fixture::charter();
    old_default.policy.instructions = Box::new([]);
    let old = settings::overlaid(&old_default, &overlay);
    assert_eq!(host.check(&old), Ok(()), "the overlay fit before the default grew");
    let mut store = resume_fixture::StartStore::new();
    let mut deployment = root.engine.core().counters.deployment();
    deployment.commits = 1;
    store.keep(crate::Record::Core(core::Record::Core(core::CoreRecord::Deployment(deployment))));
    for setting in root.engine.core().current.offered().into_boxed() {
        let value = if setting == Setting::Charter(CHARTER) {
            Value::Charter(Box::new(old_default.clone()))
        } else {
            root.engine.core().current.default_value(setting).unwrap().clone()
        };
        store.keep(crate::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Default {
            setting,
            default: settings::Default { value, revision: 1 },
        }))));
    }
    store.keep(crate::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
        setting: Setting::Charter(CHARTER),
        changed: settings::Changed { value: Some(Value::Overlay(Box::new(overlay))), revision: 1 },
    }))));
    // A persisted head makes this a restored settings set, not the first start.
    store.keep(crate::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Revision {
        history: false,
        revision: settings::Revision {
            head: settings::Head {
                number: 1,
                against: 0,
                at: Wall::EPOCH,
                by: settings::Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
            applications: Box::new([]),
        },
    }))));
    (root, env, store)
}

fn finish_load_core(root: &mut Domain, env: &Env<Limits>, mut store: resume_fixture::StartStore) {
    store.advance(root, env, resume_fixture::Until::Phase(core::start::Phase::LoadInbox));
}

#[test]
fn a_stored_overlay_is_judged_over_the_new_default_after_restore() {
    for fits in [false, true] {
        let default = resume_fixture::charter();
        let added = SHORT_INSTRUCTIONS - u32::try_from(default.policy.instructions.len()).unwrap() - 2;
        let (mut root, env, store) = restored_overlay(&HOST, overlay(0, Some(added - u32::from(fits))));
        finish_load_core(&mut root, &env, store);
        if fits {
            assert_eq!(root.start_stop(), None);
            assert_ne!(root.start_phase(), Some(core::start::Phase::LoadCore));
        } else {
            assert_eq!(
                root.start_stop(),
                Some(crate::StartStop::Room(Refusal::Rendered(Short {
                    charter: CHARTER,
                    room: 2_047,
                    bound: HOST.bound.unwrap()
                })))
            );
            assert_eq!(root.start_phase(), Some(core::start::Phase::LoadCore));
            assert!(!root.ready());
        }
    }
}

#[test]
fn a_partys_change_refuses_the_short_charter_item() {
    assert_eq!(refused(change()), judged());
}

#[test]
fn a_partys_proposal_refuses_the_short_charter_item() {
    assert_eq!(refused(proposal()), judged());
}

#[test]
fn a_reset_carries_its_previously_admitted_value() {
    let (mut root, env) = owner_root();
    let _changed = settled(&mut root, &env, change());
    // Even the default leaves zero bytes under this new host bound.
    root.host_part = crate::room::HostPart { bound: Some(0), ..HOST };
    let reply = settled(
        &mut root,
        &env,
        PartyInput::ChangeSettings {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [3; 16],
            digest: [4; 32],
            kinds: Box::new([]),
            change: Box::new(settings::Change {
                against: 2,
                items: Box::new([Item { setting: Setting::Charter(CHARTER), edit: Edit::Reset }]),
                running: settings::Running::Stays,
            }),
        },
    );
    assert_eq!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 3 }));
    assert_eq!(root.engine.core().current.charter(CHARTER), Some(&resume_fixture::charter()));
}

const FIXED_WORDS: &[u8] = b"charter 1 on a first wake is 66,012 bytes owned before its brief, past its allowance of 65,536; trim its text, parts or tool families";
fn fixed_host(charter: &crate::Charter) -> crate::room::HostPart {
    let base = crate::room::HostPart { bound: None, allowance: 65_536, ..HOST };
    let bytes = base.on_wake(charter, jig_domain_tasks::Reasons::FIRST).owned;
    crate::room::HostPart { fixed: crate::room::Cost { owned: 66_012_u64.checked_sub(bytes).unwrap(), ..ZERO }, ..base }
}

#[test]
fn a_root_without_an_agent_checks_fixed_but_not_rendered_room() {
    let (mut root, env) = owner_root();
    root.host_part = crate::room::HostPart { bound: None, allowance: 65_536, ..HOST };
    assert_eq!(
        settled(&mut root, &env, change()),
        people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
    );
    let (mut root, env) = owner_root();
    root.host_part = fixed_host(&short_charter());
    assert_eq!(
        settled(&mut root, &env, change()),
        people::Reply::Refused(people::Refusal::Judged {
            at: 1,
            setting: Setting::Charter(CHARTER),
            why: FIXED_WORDS.into(),
        })
    );
}

#[test]
fn fixed_parts_refuse_party_changes_proposals_and_default_overlays() {
    for propose in [false, true] {
        for overlaid in [false, true] {
            let (mut root, env) = owner_root();
            let mut input = if propose { proposal() } else { change() };
            let at;
            if overlaid {
                let overlay = overlay(0, Some(100));
                let charter = settings::overlaid(&resume_fixture::charter(), &overlay);
                root.host_part = fixed_host(&charter);
                input = with_items(input, overlay_items(overlay));
                at = 0;
            } else {
                root.host_part = fixed_host(&short_charter());
                at = 1;
            }
            assert_eq!(
                settled(&mut root, &env, input),
                people::Reply::Refused(people::Refusal::Judged {
                    at,
                    setting: Setting::Charter(CHARTER),
                    why: FIXED_WORDS.into(),
                })
            );
        }
    }
}

fn deployment_charter(name: &[u8]) -> crate::Charter {
    crate::Charter { name: Some(name.into()), ..resume_fixture::charter() }
}

#[test]
fn a_settings_decision_replaces_the_forges_deployment_charters() {
    let mut limits = resume_fixture::limits();
    limits.tasks.charters = 2;
    let (mut root, env) = owner_root_with_limits(&limits);
    assert!(root.forge.deployment_charters().offered.is_empty(), "the stock charter is unnamed");
    assert_eq!(root.forge.deployment_charters().limit, limits.tasks.charters);
    for (against, name) in [(1_u8, b"implementer".as_slice()), (2, b"renamed".as_slice())] {
        let charter = crate::Charter { number: 33, ..deployment_charter(name) };
        let input = PartyInput::ChangeSettings {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [against; 16],
            digest: [against; 32],
            kinds: Box::new([]),
            change: Box::new(settings::Change {
                against: u32::from(against),
                items: Box::new([Item {
                    setting: Setting::Charter(charter.number),
                    edit: Edit::Set(Value::Charter(Box::new(charter))),
                }]),
                running: settings::Running::Stays,
            }),
        };
        assert_eq!(
            settled(&mut root, &env, input),
            people::Reply::Outcome(people::Outcome::SettingsChanged { revision: u32::from(against) + 1 })
        );
        let held = &root.forge.deployment_charters().offered;
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].name.as_ref(), name);
    }
}

#[test]
fn the_forge_has_restored_deployment_charters_before_its_first_read() {
    use super::resume::{repository, restored_start_store};
    use resume_fixture::Until;
    use temper_engine_domain_forge as forge;
    let (mut store, mut env) = restored_start_store();
    env.limits.tasks.charters = 2;
    store.keep(changed_row(crate::Charter { number: 33, ..deployment_charter(b"restored") }));
    let repo = forge::Repository { home: true, role: forge::Role::Adopted, ..repository() };
    store.keep(crate::Record::Forge { row: Box::new(forge::Stored::Repository(repo)) });
    let mut root = Domain::new(resume_fixture::config(7), &env.limits).unwrap();
    assert!(root.forge.deployment_charters().offered.is_empty());
    crate::step(&mut root, &env, Event::Restart { pending: true });
    assert!(store.advance(&mut root, &env, Until::ForgeRead).is_some(), "the home starts reading");
    let held = root.forge.deployment_charters();
    assert_eq!(held.offered.len(), 1);
    assert_eq!(held.offered[0].name.as_ref(), b"restored");
}

fn composed(
    fields: &crate::Charter,
    words: &settings::Overlay,
    instructions: u32,
    project_parts: u32,
) -> crate::Charter {
    let mut project = fields.clone();
    project.policy.instructions = super::bytes(instructions);
    let part = core::CharterPart { on: jig_domain_tasks::Reasons::FIRST, instructions: b"part".as_slice().into() };
    let mut parts = skein_lib::List::with_capacity(project_parts);
    for _ in 0..project_parts {
        parts.push(part.clone()).unwrap();
    }
    project.parts = parts.into_boxed();
    settings::overlaid(&project, words)
}

#[test]
fn deployment_charter_room_is_exact_for_whole_and_overlaid_settings() {
    use crate::room::{BRIEF_ROOM, Cost};
    use jig_domain_core_vocabulary::limits::{parts_owned_bytes, policy_owned_bytes};
    use jig_domain_tasks::Reasons;
    const BYTES: u32 = 256;
    const PARTS: u32 = 3;
    const FLOOR: Cost = Cost { owned: 13, encoded: 19, rendered: 0 };
    const UNBOUNDED: crate::room::HostPart = crate::room::HostPart { bound: None, floor: FLOOR, ..HOST };
    for overlaid in [false, true] {
        let mut config = resume_fixture::config(7);
        let mut stock = deployment_charter(b"implementer");
        stock.parts = parts(100); // The project's parts replace these, even under an overlay.
        config.core.defaults.charters = Box::new([stock.clone()]);
        config.host_part = UNBOUNDED;
        let (mut root, env) = owner_root_with_config(config, &resume_fixture::limits());
        let words = if overlaid {
            let mut overlay = overlay(37, Some(41));
            overlay.parts = Box::new([
                overlay.parts[0].clone(),
                core::CharterPart { on: Reasons::TIMER, instructions: super::bytes(17) },
            ]);
            assert_eq!(
                settled(&mut root, &env, with_items(change(), overlay_items(overlay.clone()))),
                people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
            );
            overlay
        } else {
            let mut whole = stock;
            whole.policy.instructions = b"These words are set whole.".as_slice().into();
            assert_eq!(
                settled(
                    &mut root,
                    &env,
                    with_items(
                        change(),
                        Box::new([Item {
                            setting: Setting::Charter(CHARTER),
                            edit: Edit::Set(Value::Charter(Box::new(whole))),
                        }])
                    )
                ),
                people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })
            );
            settings::Overlay::empty()
        };
        let core = root.engine.core();
        let fields = core.charter(CHARTER).unwrap();
        let at = composed(fields, &words, BYTES, 0);
        let past = composed(fields, &words, BYTES + 1, 0);
        let jig_at =
            u32::try_from(policy_owned_bytes(&at.policy).unwrap() + parts_owned_bytes(&at.parts).unwrap()).unwrap();
        let rendered_at = UNBOUNDED.non_brief(&at) + BRIEF_ROOM;
        let first = UNBOUNDED.on_wake(&at, Reasons::FIRST);
        let extra = 123;
        let owned = crate::room::HostPart {
            allowance: first.owned + FLOOR.owned + extra,
            fixed: Cost { owned: extra, ..ZERO },
            ..UNBOUNDED
        };
        let encoded = crate::room::HostPart {
            allowance: first.encoded + FLOOR.encoded + extra,
            fixed: Cost { encoded: extra, ..ZERO },
            ..UNBOUNDED
        };
        let rendered = crate::room::HostPart { bound: Some(rendered_at), ..UNBOUNDED };
        let project_parts = PARTS - u32::try_from(words.parts.len()).unwrap();
        for (constraint, host, run_bytes) in [
            ("jig", UNBOUNDED, jig_at),
            ("rendered", rendered, env.limits.journal.run_bytes),
            ("owned", owned, env.limits.journal.run_bytes),
            ("encoded", encoded, env.limits.journal.run_bytes),
        ] {
            let mut limits = env.limits;
            limits.journal.run_bytes = run_bytes;
            limits.tasks.charter_parts = PARTS;
            let deployment = crate::deployment_charters::snapshot(core, &limits, &host);
            let offered = &deployment.offered[0];
            assert_eq!(
                (offered.bytes, offered.parts),
                (Some(u64::from(BYTES)), project_parts),
                "{constraint}, overlay {overlaid}"
            );
            let jig = crate::limits::core_limits(&limits);
            let check = |charter: &crate::Charter| {
                settings::check::shape(Setting::Charter(CHARTER), &Value::Charter(Box::new(charter.clone())), &jig)
            };
            assert_eq!(check(&at), Ok(()));
            assert_eq!(host.check(&at), Ok(()));
            assert!(check(&past).is_err() || host.check(&past).is_err(), "{constraint}, overlay {overlaid}");
            let at_parts = composed(fields, &words, 0, project_parts);
            assert_eq!(check(&at_parts), Ok(()));
            assert_eq!(host.check(&at_parts), Ok(()));
            let past_parts = composed(fields, &words, 0, project_parts + 1);
            assert!(check(&past_parts).is_err() && host.check(&past_parts).is_ok());
        }
    }
}

fn changed_row(charter: crate::Charter) -> crate::Record {
    crate::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
        setting: Setting::Charter(charter.number),
        changed: settings::Changed { value: Some(Value::Charter(Box::new(charter))), revision: 1 },
    })))
}

#[test]
fn a_stored_short_charter_refuses_before_the_next_start_phase() {
    let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
    root.host_part = HOST;
    let mut store = resume_fixture::StartStore::new();
    store.keep(changed_row(short_charter()));
    finish_load_core(&mut root, &env, store);
    let mut out = Queue::uncapped();
    crate::release(&mut root, &env, &mut out);
    assert_eq!(
        root.start_stop(),
        Some(crate::StartStop::Room(Refusal::Rendered(Short { charter: CHARTER, room: 2_047, bound: 3_048 })))
    );
    assert_eq!(root.start_phase(), Some(core::start::Phase::LoadCore));
    assert!(!root.ready());
    assert_eq!(out.pop(), Some(Request::Stop));
    crate::release(&mut root, &env, &mut out);
    assert!(out.is_empty(), "the retained refusal stops once");
}

#[test]
fn a_stored_unfit_charter_refuses_before_rendered_room() {
    // The rendered check either also fails or is skipped without a local agent.
    for bound in [HOST.bound, None] {
        let (mut root, env) = super::resume::at_phase(false, core::start::Phase::LoadCore);
        let charter = short_charter();
        root.host_part = crate::room::HostPart { bound, ..fixed_host(&charter) };
        let mut store = resume_fixture::StartStore::new();
        store.keep(changed_row(charter));
        finish_load_core(&mut root, &env, store);
        assert_start_unfit(
            &root,
            Unfit {
                charter: CHARTER,
                wake: jig_domain_tasks::Reasons::FIRST,
                measure: Measure::Owned,
                bytes: 66_012,
                allowance: 65_536,
                floor: 0,
            },
        );
    }
}

#[test]
fn a_stored_overlay_short_of_the_brief_floor_refuses_start() {
    const PART: u32 = 80;
    // Default instructions (39), join (2), part (80), model name (8).
    const FIXED: u64 = 129;
    const ALLOWANCE: u64 = 1_024;
    const FLOOR: u64 = ALLOWANCE - FIXED + 1;
    let host =
        crate::room::HostPart { allowance: ALLOWANCE, floor: crate::room::Cost { encoded: FLOOR, ..ZERO }, ..HOST };
    let (mut root, env, store) = restored_overlay(&host, overlay(0, Some(PART)));
    finish_load_core(&mut root, &env, store);
    assert_start_unfit(
        &root,
        Unfit {
            charter: CHARTER,
            wake: jig_domain_tasks::Reasons::FIRST,
            measure: Measure::Encoded,
            bytes: FIXED,
            allowance: ALLOWANCE,
            floor: FLOOR,
        },
    );
}

fn assert_start_unfit(root: &Domain, unfit: Unfit) {
    assert_eq!(root.start_stop(), Some(crate::StartStop::Room(Refusal::Fixed(unfit))));
    assert_eq!(root.start_phase(), Some(core::start::Phase::LoadCore));
    assert!(!root.ready());
}

#[test]
fn a_stored_charter_that_fits_both_checks_opens() {
    // The exact rendered room boundary and the exact fixed-part allowance.
    const INSTRUCTIONS: u32 = SHORT_INSTRUCTIONS - 1;
    const MODEL_NAME: u64 = 8; // The fixture uses `scripted`.
    let (mut store, env) = super::resume::restored_start_store();
    let mut charter = resume_fixture::charter();
    charter.policy.instructions = super::bytes(INSTRUCTIONS);
    let row = changed_row(charter);
    store.rows.insert(row.key(), row);
    let mut config = resume_fixture::config(7);
    config.host_part = crate::room::HostPart { allowance: u64::from(INSTRUCTIONS) + MODEL_NAME, ..HOST };
    let mut root = Domain::new(config, &env.limits).unwrap();
    crate::step(&mut root, &env, Event::Restart { pending: false });
    store.run(&mut root, &env, false);
    assert!(root.ready());
    assert_eq!(root.start_stop(), None);
    assert_eq!(root.engine.core().current.charter(CHARTER).unwrap().policy.instructions, super::bytes(INSTRUCTIONS));
}

fn run_root() -> (Domain, Env<Limits>, u64) {
    let (source, _, task) = super::live_task(None, jig_domain_tasks::Executor::Agent { charter: CHARTER });
    let mut row = source.engine.core().tasks.task(task).unwrap().clone();
    row.authority.tools = core::RunPolicy::EVERY_FAMILY;
    let mut config = resume_fixture::config(7);
    config.host_part = HOST;
    config.core.defaults.rules.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
    config.core.seed_projects[0].policy.policy.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
    let (mut root, env) = resume_fixture::root_with_records(
        config,
        &resume_fixture::limits(),
        &[
            crate::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Ledger(
                *source.engine.core().tasks.funding(row.funder).unwrap(),
            ))),
            crate::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row)))),
        ],
    );
    root.engine
        .core_mut()
        .proofs
        .insert(
            task,
            core::RunProof {
                kept_part: 0,
                worker: None,
                task,
                attempt: 1,
                conversation: 1,
                endpoint: None,
                credentials: Box::new([]),
                accounts: Box::new([]),
                placement: Some(Token::new(7)),
                account_failure: None,
                account_terminal: false,
                window: None,
                crossed: false,
                host: jig_domain_fleet::HostKind::Worker,
                offered: None,
                outstanding: Box::new([]),
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
    (root, env, task)
}

#[test]
fn a_runs_proposal_judges_overlays_over_the_default() {
    let default_bytes = u32::try_from(resume_fixture::charter().policy.instructions.len()).unwrap();
    let added = SHORT_INSTRUCTIONS - default_bytes - 2;
    for (overlay, short) in [
        (overlay(added, None), true),
        (overlay(0, Some(added)), true),
        (overlay(added - 1, None), false),
        (overlay(0, Some(added - 1)), false),
        (overlay(0, None), false),
    ] {
        use crate::route::host_route::{Flight, FlightRender};
        let (mut root, env, task) = run_root();
        let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
        root.call_renders
            .insert(
                REPLY,
                Flight {
                    task,
                    attempt: 1,
                    key: Some(key),
                    note_decoding: false,
                    job: None,
                    page: None,
                    render: FlightRender::Plain,
                    name: b"proposal".as_slice().into(),
                    tool: b"propose_settings".as_slice().into(),
                },
            )
            .unwrap();
        let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
        crate::route::relay_payload(
            &mut root,
            &env,
            &mut decision,
            ReplyTo::new(REPLY),
            key,
            crate::Call {
                completion: 1,
                position: 0,
                tool: crate::Tool::ProposeSettings {
                    more: false,
                    against: 1,
                    items: overlay_items(overlay),
                    reason: b"an overlay".as_slice().into(),
                    kinds: Box::new([]),
                },
            },
        );
        if short {
            let Some(crate::Work::Fleet(jig_domain_fleet::Event::Relayed { to, answer })) = root.work.pop() else {
                panic!("the short overlay is refused at the run's entrance");
            };
            assert_eq!(to.into_token(), REPLY);
            let Some(crate::Payload::SettledCall(call)) = root.hub_link.take_payload(answer) else {
                panic!("a settled host answer");
            };
            assert_eq!(call.answer, core::SettledAnswer::Host { error: true, body: WORDS.into() });
        } else {
            let Some(crate::Work::Core(core::Event::NamedAction {
                action: core::NamedAction::ProposeSettings { .. },
                ..
            })) = root.work.pop()
            else {
                panic!("the fitting overlay reaches jig")
            };
        }
        assert!(root.work.is_empty());
    }
}

#[test]
fn a_runs_measure_reaches_jig_as_a_revisioned_named_act() {
    let (mut root, env, task) = run_root();
    let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::relay_payload(
        &mut root,
        &env,
        &mut decision,
        ReplyTo::new(REPLY),
        key,
        crate::Call { completion: 1, position: 0, tool: crate::Tool::Measure { target: 41, revision: 2 } },
    );
    let Some(crate::Work::Core(core::Event::NamedAction { to, key: named, action })) = root.work.pop() else {
        panic!("expected a named act in jig")
    };
    assert_eq!((to.into_token(), named), (REPLY, key));
    let core::NamedAction::Measure { target, revision } = action else { panic!("expected jig's Measure") };
    assert_eq!((target, revision), (41, 2));
}

#[test]
fn a_runs_short_proposal_is_answered_and_its_next_call_is_decided() {
    run_refusal(&HOST, WORDS);
}

#[test]
fn a_runs_unfit_proposal_is_answered_and_its_next_call_is_decided() {
    run_refusal(&fixed_host(&short_charter()), FIXED_WORDS);
}

fn run_refusal(host: &crate::room::HostPart, words: &[u8]) {
    use crate::route::host_route::{Flight, FlightRender};
    let (mut root, env, task) = run_root();
    root.host_part = *host;
    let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
    root.call_renders
        .insert(
            REPLY,
            Flight {
                task,
                attempt: 1,
                key: Some(key),
                note_decoding: false,
                job: None,
                page: None,
                render: FlightRender::Plain,
                name: b"proposal".as_slice().into(),
                tool: b"propose_settings".as_slice().into(),
            },
        )
        .unwrap();
    let mut decision = crate::Decision::decide(&mut root.journal, &env.limits.journal);
    crate::route::relay_payload(
        &mut root,
        &env,
        &mut decision,
        ReplyTo::new(REPLY),
        key,
        crate::Call {
            completion: 1,
            position: 0,
            tool: crate::Tool::ProposeSettings {
                more: false,
                against: 1,
                items: items(),
                reason: b"a charter".as_slice().into(),
                kinds: Box::new([]),
            },
        },
    );
    let Some(crate::Work::Fleet(jig_domain_fleet::Event::Relayed { to, answer })) = root.work.pop() else {
        panic!("the host error goes straight back to the run");
    };
    assert_eq!(to.into_token(), REPLY);
    let Some(crate::Payload::SettledCall(call)) = root.hub_link.take_payload(answer) else {
        panic!("a settled host answer");
    };
    assert_eq!(call.answer, core::SettledAnswer::Host { error: true, body: words.into() });
    assert!(!root.engine.core().call_parts.contains_key(&key));
    assert_eq!(root.engine.core().current.newest(), 1);
    let next = crate::CallKey { completion: 2, ..key };
    crate::route::relay_payload(
        &mut root,
        &env,
        &mut decision,
        ReplyTo::new(REPLY),
        next,
        crate::Call { completion: 2, position: 0, tool: crate::Tool::Unavailable },
    );
    crate::route::route_into(&mut root, &env, &mut decision);
    assert_eq!(root.engine.core().call_parts.get(&next), Some(&core::CallPart::Unavailable));
    assert!(root.engine.core().current_proof(task, 1));
    assert!(root.engine.core().tasks.task(task).is_some());
}

fn owner_root() -> (Domain, Env<Limits>) {
    owner_root_with_limits(&resume_fixture::limits())
}

pub(crate) fn owner_root_with_limits(limits: &Limits) -> (Domain, Env<Limits>) {
    owner_root_with_config(resume_fixture::config(7), limits)
}

fn owner_root_with_config(mut config: crate::Config, limits: &Limits) -> (Domain, Env<Limits>) {
    config.core.owners[0].project = people::DEPLOYMENT;
    let records = [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: config.core.owners[0].identity.clone(),
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: 1, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ]
    .map(core::Record::People)
    .map(crate::Record::Core);
    resume_fixture::root_with_records(config, limits, &records)
}

// Release the root's decisions against a durable store; the party is the only outside reader.
#[expect(clippy::wildcard_enum_match_arm, reason = "this fixture only commits settings and receives party replies")]
pub(crate) fn settled(root: &mut Domain, env: &Env<Limits>, input: PartyInput) -> people::Reply {
    // Each change needs fewer than 64 journal and child continuation handoffs.
    const HANDOFFS: u32 = 64;
    crate::try_step(root, env, Event::Party(input)).unwrap();
    let mut out = Queue::uncapped();
    for _ in 0..HANDOFFS {
        crate::release(root, env, &mut out);
        if let Some(request) = out.pop() {
            match request {
                Request::Store(crate::StoreRequest::Commit { number, .. }) => {
                    crate::try_step(root, env, Event::Store(crate::StoreInput::Committed { number, fill: 0 })).unwrap();
                }
                Request::Party(PartyRequest::Deliver(Delivery::WebReply { reply, .. })) => return reply,
                other => panic!("unexpected settings output: {other:?}"),
            }
        }
    }
    panic!("settings reply did not settle");
}

#[test]
fn an_accepted_stored_proposal_keeps_its_admitted_charter() {
    let (mut root, env) = owner_root();
    let people::Reply::Outcome(people::Outcome::Proposed { proposal: number, .. }) =
        settled(&mut root, &env, proposal())
    else {
        panic!("the proposal is stored");
    };
    root.host_part = HOST;
    let reply = settled(
        &mut root,
        &env,
        PartyInput::Ask {
            reply_to: ReplyTo::new(REPLY),
            sign_in: 1,
            key: [3; 16],
            models: Box::new([]),
            ask: people::Ask::DecideProposal {
                project: people::DEPLOYMENT,
                proposer: people::Proposer::Party(1),
                proposal: number,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            },
        },
    );
    let people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. }) =
        reply
    else {
        panic!("the stored proposal is accepted: {reply:?}");
    };
    assert_eq!(
        root.engine.core().current.charter(CHARTER).unwrap().policy.instructions.len(),
        usize::try_from(SHORT_INSTRUCTIONS).unwrap()
    );
}

#[test]
fn applying_a_charter_reserves_its_brief_floor_with_or_without_an_agent() {
    let charter = short_charter();
    let base = crate::room::HostPart { allowance: 65_536, bound: None, ..HOST };
    let bytes = base.on_wake(&charter, jig_domain_tasks::Reasons::FIRST).owned;
    for bound in [Some(100_000), None] {
        for propose in [false, true] {
            for (fixed, fits) in [(54_320, false), (51_656, true)] {
                let (mut root, env) = owner_root();
                root.host_part = crate::room::HostPart {
                    bound,
                    fixed: crate::room::Cost { owned: fixed - bytes, ..ZERO },
                    floor: crate::room::Cost { owned: 13_880, encoded: 13_500, ..ZERO },
                    ..base
                };
                let input = if propose { proposal() } else { change() };
                let reply = settled(&mut root, &env, input);
                if fits && propose {
                    let people::Reply::Outcome(people::Outcome::Proposed { .. }) = reply else { panic!("{reply:?}") };
                } else if fits {
                    assert_eq!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 }));
                } else {
                    assert_eq!(reply, people::Reply::Refused(people::Refusal::Judged {
                        at: 1,
                        setting: Setting::Charter(CHARTER),
                        why: b"charter 1 on a first wake is 54,320 bytes owned before its brief, leaving its brief 11,216 of the 13,880 its floor needs; trim its text, parts or tool families".as_slice().into(),
                    }));
                }
            }
        }
    }
}
