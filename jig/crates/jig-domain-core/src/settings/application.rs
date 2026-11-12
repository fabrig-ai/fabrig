//! Application-owned settings (jig/docs/design/domain/engine/settings.md, 5).

use alloc::boxed::Box;

/// Where a kind's values are kept (jig/docs/design/domain/engine/settings.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KindScope {
    Deployment,
    Project,
}

/// A declared application kind and its bounds (jig/docs/design/domain/engine/settings.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Kind {
    pub number: u32,
    pub scope: KindScope,
    pub connector: u16,
    pub value_bytes: u32,
    pub per_scope: u32,
    pub charters: u8,
}

/// An edit whose value stays with its connector (jig/docs/design/domain/engine/settings.md, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Box<[u8]>),
    Remove,
    Reset,
}

/// One application setting handed to its connector (jig/docs/design/domain/engine/settings.md, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: Edit,
}

use super::{Change, Current, Failure, Fault, Setting};
use crate::{
    Core, Event, Limits, Request,
    effects::connector,
    routing::{Work, party::dispatch::Routed},
};
use jig_domain_people as people;
use skein_lib::{Env, Id, List, Map, Queue, Slab, Token};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Follows {
    Default,
    Changed { revision: u32 },
    Removed { revision: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub follows: Follows,
    pub size: u32,
    pub charters: Box<[u32]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Recorded {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: SizedEdit,
}
/// The state and size an application item replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Replaced {
    Changed { size: u32 },
    Default,
    Removed,
    NotOffered,
}
/// A revision's application item (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RevisionRecorded {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: SizedEdit,
    pub replaced: Replaced,
}
/// An application item a restore proposes (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Restored {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: RestoredEdit,
}
/// Where a restore takes an application's value (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RestoredEdit {
    Set { revision: u64 },
    Remove,
    Reset,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SizedEdit {
    Set { size: u32 },
    Remove,
    Reset,
}

pub(crate) fn capacity(kinds: &[Kind], limits: &Limits) -> Option<u32> {
    let mut count = 0_u32;
    for kind in kinds {
        let scopes = match kind.scope {
            KindScope::Deployment => 1,
            KindScope::Project => limits.authority.projects,
        };
        count = count.checked_add(kind.per_scope.checked_mul(scopes)?)?;
    }
    Some(count)
}

pub(crate) fn is_application(setting: Setting) -> bool {
    match setting {
        Setting::Application { .. } => true,
        Setting::Charter(_)
        | Setting::Template(_)
        | Setting::Policy(_)
        | Setting::Model(_)
        | Setting::Rules
        | Setting::ProjectPolicy { .. } => false,
    }
}

pub(crate) fn projects(items: &[super::Item]) -> Box<[u32]> {
    let mut names = Map::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let scope = match item.setting {
            Setting::ProjectPolicy { project } => project,
            Setting::Application { scope, .. } => scope,
            Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Model(_) | Setting::Rules => {
                people::DEPLOYMENT
            }
        };
        names.insert(scope, ()).expect("one scope per item");
    }
    let mut result = List::with_capacity(names.len());
    for (&scope, ()) in &names {
        result.push(scope).expect("unique scopes");
    }
    result.into_boxed()
}

/// The core checks names, scopes, bytes and retained slots before asking a connector.
pub(crate) fn check(core: &Core, items: &[super::Item]) -> Result<(), Failure> {
    for item in items {
        let Some(Name { kind, scope, number }) = named(item.setting) else { continue };
        let declared =
            kind_of(&core.kinds, kind).ok_or(Failure { setting: item.setting, fault: Fault::Shape, detail: None })?;
        let valid_scope = match declared.scope {
            KindScope::Deployment => scope == 0,
            KindScope::Project => scope != 0 && core.projects.as_slice().contains(&scope),
        };
        if number == 0 || !valid_scope {
            return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
        }
        match &item.edit {
            super::Edit::Set(super::Value::Application(bytes)) => {
                if bytes.len() > skein_lib::length::usize_of(declared.value_bytes) {
                    return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
                }
            }
            super::Edit::Set(
                super::Value::Overlay(_) | super::Value::Charter(_)
                | super::Value::Template(_)
                | super::Value::Policy(_)
                | super::Value::Model(_)
                | super::Value::Rules(_),
            ) => {
                return Err(Failure { setting: item.setting, fault: Fault::Shape, detail: None });
            }
            super::Edit::Removed | super::Edit::Reset => {}
        }
        let mut count = 0_u32;
        for (&setting, _) in &core.current.applications {
            if same_scope(setting, kind, scope) {
                count = count.checked_add(1).expect("bounded settings count");
            }
        }
        for addition in items {
            if same_scope(addition.setting, kind, scope) && !core.current.applications.contains_key(&addition.setting) {
                count = count.checked_add(1).expect("bounded settings count");
            }
        }
        if count > declared.per_scope {
            return Err(Failure { setting: item.setting, fault: Fault::Count, detail: None });
        }
    }
    Ok(())
}
fn same_scope(setting: Setting, kind: u32, scope: u32) -> bool {
    match setting {
        Setting::Application { kind: held, scope: held_scope, .. } => kind == held && scope == held_scope,
        Setting::Charter(_)
        | Setting::Template(_)
        | Setting::Policy(_)
        | Setting::Model(_)
        | Setting::Rules
        | Setting::ProjectPolicy { .. } => false,
    }
}
#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
pub(crate) fn kind_of(kinds: &[Kind], number: u32) -> Option<&Kind> {
    for kind in kinds {
        if kind.number == number {
            return Some(kind);
        }
    }
    None
}
#[derive(Clone, Copy, Debug)]
struct Name {
    kind: u32,
    scope: u32,
    number: u32,
}
fn named(setting: Setting) -> Option<Name> {
    match setting {
        Setting::Application { kind, scope, number } => Some(Name { kind, scope, number }),
        Setting::Charter(_)
        | Setting::Template(_)
        | Setting::Policy(_)
        | Setting::Model(_)
        | Setting::Rules
        | Setting::ProjectPolicy { .. } => None,
    }
}
fn translate(item: &super::Item) -> Option<Item> {
    let Name { kind, scope, number } = named(item.setting)?;
    let edit = match &item.edit {
        super::Edit::Set(super::Value::Application(bytes)) => Edit::Set(bytes.clone()),
        super::Edit::Removed => Edit::Remove,
        super::Edit::Reset => Edit::Reset,
        super::Edit::Set(
            super::Value::Overlay(_) | super::Value::Charter(_)
            | super::Value::Template(_)
            | super::Value::Policy(_)
            | super::Value::Model(_)
            | super::Value::Rules(_),
        ) => unreachable!("checked application value"),
    };
    Some(Item { kind, scope, number, edit })
}

/// The change and its keyed people continuation live only until the same step's judges answer.
#[derive(Hash, Debug)]
struct Decision {
    request: Token,
    person: u64,
    change: Change,
    room: u32,
    proposal: Option<(people::Proposer, u32)>,
}
#[derive(Hash, Debug)]
struct Group {
    connector: u16,
    positions: Box<[u32]>,
    judged: Option<Box<[connector::Judged]>>,
}
#[derive(Hash, Debug)]
pub(crate) struct Judging {
    decision: Option<Decision>,
    groups: Slab<Option<Group>>,
}
impl Judging {
    pub(crate) fn new(kinds: &[Kind]) -> Self {
        Self { decision: None, groups: Slab::with_capacity(u32::try_from(kinds.len()).expect("declared kinds")) }
    }
}

/// Read the active decision's templates under the policy it would leave for this project.
pub(crate) fn project_templates(core: &Core, owner: Token, project: u32) -> Box<[crate::Template]> {
    assert!(core.settings_judging.groups.get(Id::from_token(owner)).is_some(), "active settings judge owner");
    let decision = core.settings_judging.decision.as_ref().expect("active settings decision");
    let items = &decision.change.items;
    let overrides = match super::check::edit(items, Setting::ProjectPolicy { project }) {
        Some(super::Edit::Set(super::Value::Policy(policy))) => policy.templates.as_ref(),
        Some(
            super::Edit::Set(
                super::Value::Overlay(_) | super::Value::Charter(_)
                | super::Value::Template(_)
                | super::Value::Model(_)
                | super::Value::Rules(_)
                | super::Value::Application(_),
            )
            | super::Edit::Removed
            | super::Edit::Reset,
        ) => unreachable!("checked project policy"),
        None => match core.template_overrides.get(&project) {
            Some(entries) => entries.as_ref(),
            None => &[],
        },
    };
    super::templates::merged(&super::check::templates(&core.current, items), overrides)
}

pub(crate) fn start(
    core: &mut Core,
    routed: &mut Routed,
    request: Token,
    person: u64,
    change: Change,
    room: u32,
    proposal: Option<(people::Proposer, u32)>,
) {
    assert!(core.settings_judging.decision.is_none(), "one settings decision within a step");
    let number = core.current.newest().checked_add(1).expect("revision preflighted");
    let mut connectors = List::with_capacity(u32::try_from(core.kinds.len()).expect("declared kinds"));
    for item in &change.items {
        let Some(Name { kind, .. }) = named(item.setting) else { continue };
        let connector = kind_of(&core.kinds, kind).expect("checked kind").connector;
        if !connectors.as_slice().contains(&connector) {
            connectors.push(connector).expect("one connector per kind");
        }
    }
    for connector in connectors.into_boxed() {
        let mut items = List::with_capacity(u32::try_from(change.items.len()).expect("bounded items"));
        let mut positions = List::with_capacity(items.capacity());
        for (at, item) in change.items.iter().enumerate() {
            let Some(item) = translate(item) else { continue };
            if kind_of(&core.kinds, item.kind).expect("checked kind").connector == connector {
                items.push(item).expect("group items");
                positions.push(u32::try_from(at).expect("bounded item position")).expect("group positions");
            }
        }
        let owner = core
            .settings_judging
            .groups
            .insert(Some(Group { connector, positions: positions.into_boxed(), judged: None }))
            .expect("one group per connector")
            .token();
        routed.out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::JudgeSettings {
                owner,
                revision: u64::from(number),
                proposal: proposal_number(core, proposal),
                items: items.into_boxed(),
            }),
        });
    }
    core.settings_judging.decision = Some(Decision { request, person, change, room, proposal });
}

#[derive(Debug)]
pub(crate) struct Kept {
    pub item: Item,
    pub state: State,
}

pub(crate) fn judged(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    owner: Token,
    answers: Box<[connector::Judged]>,
) -> Queue<Request> {
    let group = core
        .settings_judging
        .groups
        .get_mut(Id::from_token(owner))
        .expect("root returned an unknown settings judge owner")
        .as_mut()
        .expect("root answered a finished settings judge");
    assert!(group.judged.is_none(), "root answered settings connector {} twice", group.connector);
    assert_eq!(
        answers.len(),
        group.positions.len(),
        "root returned the wrong settings answer count for connector {}",
        group.connector
    );
    group.judged = Some(answers);
    for (_, group) in &core.settings_judging.groups {
        if group.as_ref().expect("active settings judge").judged.is_none() {
            return Queue::uncapped();
        }
    }
    let decision = core.settings_judging.decision.take().expect("root answered a settings decision");
    let mut judged = List::with_capacity(u32::try_from(decision.change.items.len()).expect("bounded items"));
    for _ in &decision.change.items {
        judged.push(None).expect("one answer slot per item");
    }
    let mut owners = List::with_capacity(core.settings_judging.groups.len());
    for (id, _) in &core.settings_judging.groups {
        owners.push(id).expect("active owners");
    }
    for id in owners.into_boxed() {
        let group = core.settings_judging.groups.get_mut(id).expect("active owner").take().expect("active group");
        for (&at, answer) in group.positions.iter().zip(group.judged.expect("answered group")) {
            *judged.get_mut(at).expect("group item position") = Some(answer);
        }
        core.settings_judging.groups.retire(id);
    }
    core.settings_judging.groups.reclaim();
    let mut routed = Routed::new(work);
    let result = accepted(core, env, &decision.change, judged.into_boxed());
    let result = match result {
        Ok(kept) => crate::routing::settings::dispatch::decide_judged(
            core,
            env,
            &mut routed,
            decision.person,
            crate::routing::settings::dispatch::Judged {
                change: decision.change,
                room: decision.room,
                kept,
                proposal: decision.proposal,
            },
        ),
        Err(why) => Err(why),
    };
    if let Some(key) = decision.proposal {
        finish_proposal(core, &mut routed, decision.request, decision.person, key, result);
        return routed.finish(work);
    }
    let outcome = match result {
        Ok(revision) => people::Outcome::SettingsChanged { revision },
        Err(why) => people::Outcome::Refused(why),
    };
    routed.work.push(Work::Event(Event::People(people::Event::Decided { request: decision.request, outcome })));
    routed.finish(work)
}

fn accepted(
    core: &Core,
    env: &Env<Limits>,
    change: &Change,
    judged: Box<[Option<connector::Judged>]>,
) -> Result<Box<[Kept]>, people::Refusal> {
    // A connector's first refusal is selected before checking any accepted metadata.
    let bound = u32::try_from(change.items.len()).expect("bounded items");
    let mut accepted = List::with_capacity(bound);
    for (at, (item, answer)) in change.items.iter().zip(judged).enumerate() {
        match answer {
            Some(connector::Judged::Refuse { why }) => {
                return Err(people::Refusal::Judged {
                    at: u32::try_from(at).expect("bounded position"),
                    setting: item.setting,
                    why: words(&why, env.limits.people.words),
                });
            }
            Some(connector::Judged::Accept { size, charters }) => {
                accepted.push((item, size, charters)).expect("accepted answers");
            }
            None | Some(connector::Judged::Same) => {}
        }
    }
    let mut kept = List::with_capacity(bound);
    for (item, size, charters) in accepted.into_boxed() {
        if let Some(item) = translate(item) {
            let kind = kind_of(&core.kinds, item.kind).expect("checked kind");
            if size > kind.value_bytes || charters.len() > usize::from(kind.charters) {
                return Err(crate::routing::settings::dispatch::malformed_refusal(
                    &change.items,
                    Failure { setting: setting_of(&item), fault: Fault::Shape, detail: None },
                ));
            }
            let revision = core.current.newest().checked_add(1).expect("revision preflighted");
            let state = match item.edit {
                Edit::Set(_) => State { follows: Follows::Changed { revision }, size, charters },
                Edit::Remove => State { follows: Follows::Removed { revision }, size: 0, charters: Box::default() },
                Edit::Reset => State { follows: Follows::Default, size, charters },
            };
            kept.push(Kept { item, state }).expect("accepted items");
        }
    }
    Ok(kept.into_boxed())
}
#[expect(
    clippy::disallowed_methods,
    reason = "the core checks and cuts connector refusal words at its internal byte boundary"
)]
pub(crate) fn words(bytes: &[u8], bound: u32) -> Box<[u8]> {
    if core::str::from_utf8(bytes).is_err() {
        return Box::default();
    }
    let end = bytes.len().min(skein_lib::length::usize_of(bound));
    // At most three trailing bytes can be a partial UTF-8 scalar.
    for cut in 0..4 {
        if let Some(end) = end.checked_sub(cut) {
            let prefix = bytes.get(..end).expect("bounded refusal words");
            if core::str::from_utf8(prefix).is_ok() {
                return skein_lib::bytes::copy_of(prefix);
            }
        }
    }
    Box::default()
}

pub(crate) fn references(current: &Current, items: &[super::Item], kept: &[Kept]) -> Result<(), Failure> {
    for (&setting, state) in &current.applications {
        let mut replaced = false;
        for accepted in kept {
            replaced |= setting == setting_of(&accepted.item);
        }
        if !replaced {
            state_references(current, items, setting, state)?;
        }
    }
    for accepted in kept {
        state_references(current, items, setting_of(&accepted.item), &accepted.state)?;
    }
    Ok(())
}
fn state_references(current: &Current, items: &[super::Item], setting: Setting, state: &State) -> Result<(), Failure> {
    for &charter in &state.charters {
        let mut offered = current.charter(charter).is_some();
        for item in items {
            if item.setting == Setting::Charter(charter) {
                offered = match &item.edit {
                    super::Edit::Set(_) => true,
                    super::Edit::Removed => false,
                    super::Edit::Reset => current.default_value(item.setting).is_some(),
                };
            }
        }
        if !offered {
            return Err(Failure { setting, fault: Fault::Reference, detail: None });
        }
    }
    Ok(())
}
fn setting_of(item: &Item) -> Setting {
    Setting::Application { kind: item.kind, scope: item.scope, number: item.number }
}

/// Values leave with `KeepSettings`; only metadata and sized edits enter jig's records.
pub(crate) fn keep(
    core: &mut Core,
    out: &mut Queue<Request>,
    kept: Box<[Kept]>,
    revision: u32,
    proposal: Option<u64>,
) -> Box<[RevisionRecorded]> {
    let bound = u32::try_from(kept.len()).expect("bounded accepted items");
    let mut records = List::with_capacity(bound);
    let proposed: &[Recorded] = match core.current.head.proposal {
        Some(key) if proposal.is_some() => {
            &core.settings_proposals.get(&key).expect("accepted proposal set").applications
        }
        Some(_) | None => &[],
    };
    let groups =
        bound.checked_add(u32::try_from(proposed.len()).expect("bounded proposal items")).expect("connector groups");
    let mut connectors = Map::with_capacity(groups);
    // Even a connector whose items are all unchanged must erase its kept proposal.
    for item in proposed {
        let connector = kind_of(&core.kinds, item.kind).expect("kept proposal kind").connector;
        connectors.insert(connector, ()).expect("one connector per proposal item");
    }
    for accepted in &kept {
        let connector = kind_of(&core.kinds, accepted.item.kind).expect("checked kind").connector;
        connectors.insert(connector, ()).expect("one connector per accepted item");
    }
    // Preserve change order inside each connector's keep.
    for (&connector, ()) in &connectors {
        let mut items = List::with_capacity(bound);
        for accepted in &kept {
            if kind_of(&core.kinds, accepted.item.kind).expect("checked kind").connector == connector {
                items.push(accepted.item.clone()).expect("accepted group");
            }
        }
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::KeepSettings {
                revision: u64::from(revision),
                proposal,
                items: items.into_boxed(),
            }),
        });
    }
    for accepted in kept {
        let setting = setting_of(&accepted.item);
        let edit = match accepted.item.edit {
            Edit::Set(_) => SizedEdit::Set { size: accepted.state.size },
            Edit::Remove => SizedEdit::Remove,
            Edit::Reset => SizedEdit::Reset,
        };
        let replaced = match core.current.applications.get(&setting) {
            Some(State { follows: Follows::Default, .. }) => Replaced::Default,
            Some(State { follows: Follows::Changed { .. }, size, .. }) => Replaced::Changed { size: *size },
            Some(State { follows: Follows::Removed { .. }, .. }) => Replaced::Removed,
            None => Replaced::NotOffered,
        };
        records
            .push(RevisionRecorded {
                kind: accepted.item.kind,
                scope: accepted.item.scope,
                number: accepted.item.number,
                edit,
                replaced,
            })
            .expect("sized edits");
        drop(
            core.current
                .applications
                .insert(setting, accepted.state.clone())
                .expect("checked application settings count"),
        );
        out.push(Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(
            super::Record::Application { setting, state: accepted.state },
        )))));
    }
    records.into_boxed()
}

fn proposal_number(core: &Core, proposal: Option<(people::Proposer, u32)>) -> Option<u64> {
    match proposal {
        Some(key) => core.settings_proposals.get(&key).expect("accepted proposal set").key,
        None => None,
    }
}

/// Values cross to their connectors only when the tasks child keeps the proposal.
pub(crate) fn keep_proposal(core: &Core, set: &mut super::Proposal, proposal: u64, out: &mut Queue<Request>) {
    let bound = u32::try_from(set.items.len()).expect("bounded proposal items");
    let mut connectors = Map::with_capacity(bound);
    let mut applications = List::with_capacity(bound);
    let mut jig = List::with_capacity(bound);
    for item in &set.items {
        match translate(item) {
            Some(item) => {
                let connector = kind_of(&core.kinds, item.kind).expect("checked proposal kind").connector;
                connectors.insert(connector, ()).expect("one connector per item");
                let edit = match item.edit {
                    Edit::Set(bytes) => {
                        SizedEdit::Set { size: u32::try_from(bytes.len()).expect("checked value bytes") }
                    }
                    Edit::Remove => SizedEdit::Remove,
                    Edit::Reset => SizedEdit::Reset,
                };
                applications
                    .push(Recorded { kind: item.kind, scope: item.scope, number: item.number, edit })
                    .expect("bounded proposal items");
            }
            None => jig.push(item.clone()).expect("bounded jig items"),
        }
    }
    for (&connector, ()) in &connectors {
        let mut items = List::with_capacity(bound);
        for item in &set.items {
            if let Some(item) = translate(item)
                && kind_of(&core.kinds, item.kind).expect("checked kind").connector == connector
            {
                items.push(item).expect("connector proposal items");
            }
        }
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::KeepProposalSettings { proposal, items: items.into_boxed() }),
        });
    }
    set.items = jig.into_boxed();
    set.applications = applications.into_boxed();
}

pub(crate) fn drop_proposal(core: &Core, items: &[Recorded], proposal: u64, out: &mut Queue<Request>) {
    let mut connectors = Map::with_capacity(u32::try_from(items.len()).expect("bounded proposal items"));
    for item in items {
        let connector = kind_of(&core.kinds, item.kind).expect("kept proposal kind").connector;
        connectors.insert(connector, ()).expect("one connector per item");
    }
    for (&connector, ()) in &connectors {
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::DropProposalSettings { proposal }),
        });
    }
}

/// Empty set bytes identify values retained under the proposal at its connector.
pub(crate) fn proposal_change(
    core: &Core,
    set: super::Proposal,
    running: super::Running,
) -> Result<Change, people::Refusal> {
    let bound = set.items.len().checked_add(set.applications.len()).expect("bounded proposal items");
    let mut items = List::with_capacity(u32::try_from(bound).expect("bounded proposal items"));
    for item in set.items {
        items.push(item).expect("jig proposal items");
    }
    for item in set.applications {
        let setting = Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
        let refusal = people::Refusal::NotWellFormed {
            at: items.len(),
            why: people::Malformed::Settings { setting, fault: Fault::Shape, detail: None },
        };
        let Some(kind) = kind_of(&core.kinds, item.kind) else { return Err(refusal) };
        let edit = match item.edit {
            SizedEdit::Set { size } => {
                if size > kind.value_bytes {
                    return Err(refusal);
                }
                super::Edit::Set(super::Value::Application(Box::default()))
            }
            SizedEdit::Remove => super::Edit::Removed,
            SizedEdit::Reset => super::Edit::Reset,
        };
        items.push(super::Item { setting, edit }).expect("application proposal items");
    }
    Ok(Change { against: set.against, items: items.into_boxed(), running })
}

fn finish_proposal(
    core: &mut Core,
    routed: &mut Routed,
    request: Token,
    person: u64,
    key: (people::Proposer, u32),
    result: Result<u32, people::Refusal>,
) {
    use jig_domain_tasks as tasks;
    let revision = match result {
        Ok(revision) => revision,
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
            return;
        }
    };
    let proposal = u64::from(key.1);
    let event = match key.0 {
        people::Proposer::Party(proposer) => {
            let request = core.fly_goal(request, crate::GoalRoute::Deciding { proposer, proposal, by: person });
            tasks::Event::DecidePersonProposal {
                reply_to: skein_lib::ReplyTo::new(request),
                proposer,
                proposal,
                by: tasks::Party::Person(person),
                message: None,
                revision: Some(revision),
                decision: tasks::ProposalDecision::Accept,
            }
        }
        people::Proposer::Task(proposer) => {
            let message = crate::fresh(&mut core.counters, crate::Family::Message).expect("preflighted outcome number");
            let request = core.fly_person_proposal(crate::PersonProposalRoute::Deciding {
                request,
                proposer,
                proposal,
                by: person,
                moved: None,
            });
            tasks::Event::DecideSettingsProposal {
                reply_to: Some(skein_lib::ReplyTo::new(request)),
                decision: tasks::SettingsDecision {
                    proposer,
                    proposal,
                    message,
                    by: tasks::Party::Person(person),
                    outcome: tasks::SettingsOutcome::Accepted { revision },
                    reason: Box::default(),
                },
            }
        }
    };
    routed.work.push(Work::Event(Event::Tasks(event)));
}

#[derive(Hash, Debug)]
pub(crate) struct StartDefaults {
    defaults: Box<[connector::DefaultState]>,
    dropped: Box<[connector::DroppedDefault]>,
}

/// Replies may precede settings restore; preparation waits for every owner (domain/engine/settings.md, 5–7).
#[derive(Hash, Debug)]
pub(crate) enum Starting {
    Waiting { replies: Map<u16, StartDefaults> },
    Prepared { connectors: Box<[u16]>, edits: Box<[Recorded]>, writes: Box<[crate::Write]> },
    Saved,
}
impl Starting {
    pub(crate) fn new(connectors: u32) -> Self {
        Self::Waiting { replies: Map::with_capacity(connectors) }
    }

    pub(crate) fn received(&self, connector: u16) -> bool {
        match self {
            Self::Waiting { replies } => replies.contains_key(&connector),
            Self::Prepared { .. } | Self::Saved => true,
        }
    }
}

fn application_failure(name: Name, why: crate::start::ApplicationWhy) -> crate::start::Failure {
    let Name { kind, scope, number } = name;
    crate::start::Failure::Application { kind, scope, number, why }
}

pub(crate) fn restore(core: &mut Core, setting: Setting, state: State) -> bool {
    use crate::start::ApplicationWhy;
    let Some(name) = named(setting) else { return false };
    let Name { kind, scope, .. } = name;
    let Some(declared) = kind_of(&core.kinds, kind) else {
        core.refuse_restore(application_failure(name, ApplicationWhy::Undeclared));
        return false;
    };
    assert!(!core.current.applications.contains_key(&setting), "duplicate application state {setting:?}");
    let mut count = 0_u32;
    for (&held, _) in &core.current.applications {
        if same_scope(held, kind, scope) {
            count = count.checked_add(1).expect("application settings count");
        }
    }
    if count >= declared.per_scope || core.current.applications.len() == core.current.applications.capacity() {
        core.refuse_restore(application_failure(name, ApplicationWhy::Count));
        return false;
    }
    core.current.applications.insert(setting, state).expect("stored application slot preflighted");
    true
}

pub(crate) fn defaults(
    core: &mut Core,
    connector: u16,
    defaults: Box<[connector::DefaultState]>,
    dropped: Box<[connector::DroppedDefault]>,
) {
    use crate::start::ApplicationWhy;
    let mut owns = false;
    for kind in &core.kinds {
        owns |= kind.connector == connector;
    }
    for default in &defaults {
        let name = Name { kind: default.kind, scope: default.scope, number: default.number };
        let Some(kind) = kind_of(&core.kinds, default.kind) else {
            core.refuse_restore(application_failure(name, ApplicationWhy::Undeclared));
            return;
        };
        assert!(kind.connector == connector, "default returned by its owning connector");
        let scope = match kind.scope {
            KindScope::Deployment => default.scope == 0,
            KindScope::Project => default.scope != 0,
        };
        let why = if !scope {
            Some(ApplicationWhy::Scope)
        } else if default.number == 0
            || default.size > kind.value_bytes
            || default.charters.len() > usize::from(kind.charters)
        {
            Some(ApplicationWhy::Shape)
        } else {
            let mut count = 0_u32;
            for other in &defaults {
                if other.kind == default.kind && other.scope == default.scope {
                    count = count.checked_add(1).expect("default count");
                }
            }
            if count > kind.per_scope { Some(ApplicationWhy::Count) } else { None }
        };
        if let Some(why) = why {
            core.refuse_restore(application_failure(name, why));
            return;
        }
    }
    for name in &dropped {
        let Some(kind) = kind_of(&core.kinds, name.kind) else {
            core.refuse_restore(application_failure(
                Name { kind: name.kind, scope: name.scope, number: name.number },
                ApplicationWhy::Undeclared,
            ));
            return;
        };
        assert!(kind.connector == connector, "dropped default returned by its owning connector {connector}");
        for default in &defaults {
            assert!(
                (name.kind, name.scope, name.number) != (default.kind, default.scope, default.number),
                "connector {connector} both reports and drops a default"
            );
        }
    }
    assert!(owns, "defaults answer from a connector with no declared kind");
    match &mut core.settings_restore.applications {
        Starting::Waiting { replies } => {
            assert!(!replies.contains_key(&connector), "one defaults answer per kind-owning connector");
            replies
                .insert(connector, StartDefaults { defaults, dropped })
                .expect("one defaults answer per kind-owning connector");
        }
        Starting::Prepared { .. } | Starting::Saved => unreachable!("defaults precede their start's decision"),
    }
}

pub(crate) fn defaults_received(core: &Core) -> bool {
    for kind in &core.kinds {
        if !core.settings_restore.applications.received(kind.connector) {
            return false;
        }
    }
    true
}

/// Apply default metadata only after stored states are loaded, retaining every change.
pub(crate) fn prepare_start(core: &mut Core) -> bool {
    assert!(defaults_received(core), "all default owners answered before the release decision");
    let replies = match core::mem::replace(&mut core.settings_restore.applications, Starting::Saved) {
        Starting::Waiting { replies } => replies,
        prepared @ Starting::Prepared { .. } => {
            core.settings_restore.applications = prepared;
            return true;
        }
        Starting::Saved => return true,
    };
    let bound = core.current.applications.capacity().checked_mul(2).expect("application default union");
    let mut edits = List::with_capacity(bound);
    let mut writes = List::with_capacity(bound);
    let mut dropped_states = List::with_capacity(core.current.applications.len());
    for (&setting, state) in &core.current.applications {
        match state.follows {
            Follows::Default => {}
            Follows::Changed { .. } | Follows::Removed { .. } => continue,
        }
        let mut dropped = false;
        for (_, reply) in &replies {
            for name in &reply.dropped {
                dropped |= setting == Setting::Application { kind: name.kind, scope: name.scope, number: name.number };
            }
        }
        if dropped && let Some(name) = named(setting) {
            dropped_states.push((setting, name)).expect("stored defaults");
        }
    }
    for (setting, name) in dropped_states.into_boxed() {
        drop(core.current.applications.remove(&setting));
        edits.push(recorded(name, SizedEdit::Remove)).expect("default union");
        writes
            .push(crate::Write::Erase(crate::Key::Core(crate::CoreKey::Settings(super::Key::Changed(setting)))))
            .expect("default union");
    }
    if !apply_start_defaults(core, &replies, &mut edits, &mut writes) {
        return false;
    }
    if !start_references(core, &replies) {
        return false;
    }
    let mut connectors = List::with_capacity(replies.len());
    for (&connector, _) in &replies {
        connectors.push(connector).expect("default owners");
    }
    core.settings_restore.applications = Starting::Prepared {
        connectors: connectors.into_boxed(),
        edits: edits.into_boxed(),
        writes: writes.into_boxed(),
    };
    true
}

fn apply_start_defaults(
    core: &mut Core,
    replies: &Map<u16, StartDefaults>,
    edits: &mut List<Recorded>,
    writes: &mut List<crate::Write>,
) -> bool {
    for (_, reply) in replies {
        for default in &reply.defaults {
            let setting = default_setting(default);
            let state = State { follows: Follows::Default, size: default.size, charters: default.charters.clone() };
            let previous = core.current.applications.get(&setting);
            if let Some(previous) = previous {
                match previous.follows {
                    Follows::Default => {}
                    Follows::Changed { .. } | Follows::Removed { .. } => continue,
                }
            }
            let differs = default.differs || previous.is_none();
            let write = previous != Some(&state);
            if core.current.applications.contains_key(&setting) {
                core.current.applications.insert(setting, state.clone()).expect("existing application slot");
            } else {
                // A new default shares the kind's retained slots with changed and removed states.
                if !restore(core, setting, state.clone()) {
                    return false;
                }
            }
            if differs {
                edits
                    .push(recorded(
                        Name { kind: default.kind, scope: default.scope, number: default.number },
                        SizedEdit::Set { size: default.size },
                    ))
                    .expect("default union");
            }
            if write {
                writes
                    .push(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(
                        super::Record::Application { setting, state },
                    ))))
                    .expect("default union");
            }
        }
    }
    true
}

fn start_references(core: &mut Core, replies: &Map<u16, StartDefaults>) -> bool {
    // Defaults' references are held even when a change is over their value.
    for (_, reply) in replies {
        for default in &reply.defaults {
            let state = State { follows: Follows::Default, size: default.size, charters: default.charters.clone() };
            if let Err(failure) = state_references(&core.current, &[], default_setting(default), &state) {
                let _refused = core.refuse_settings(failure);
                return false;
            }
        }
    }
    if let Err(failure) = references(&core.current, &[], &[]) {
        let _refused = core.refuse_settings(failure);
        return false;
    }
    true
}

fn default_setting(default: &connector::DefaultState) -> Setting {
    Setting::Application { kind: default.kind, scope: default.scope, number: default.number }
}
fn recorded(name: Name, edit: SizedEdit) -> Recorded {
    let Name { kind, scope, number } = name;
    Recorded { kind, scope, number, edit }
}
pub(crate) fn defaults_changed(core: &Core) -> bool {
    match &core.settings_restore.applications {
        Starting::Prepared { edits, .. } => !edits.is_empty(),
        Starting::Waiting { .. } => unreachable!("defaults prepared before their release"),
        Starting::Saved => false,
    }
}
pub(crate) fn save_start(core: &mut Core, revision: Option<u32>, out: &mut Queue<Request>) -> Box<[RevisionRecorded]> {
    match core::mem::replace(&mut core.settings_restore.applications, Starting::Saved) {
        Starting::Prepared { connectors, edits, writes } => {
            for write in writes {
                out.push(Request::Write(write));
            }
            for connector in connectors {
                out.push(Request::Ask {
                    connector,
                    ask: crate::Ask::Effect(connector::Ask::SaveSettingsDefaults {
                        revision: match revision {
                            Some(number) => Some(u64::from(number)),
                            None => None,
                        },
                    }),
                });
            }
            let mut records = List::with_capacity(u32::try_from(edits.len()).expect("bounded release items"));
            for item in edits {
                records
                    .push(RevisionRecorded {
                        kind: item.kind,
                        scope: item.scope,
                        number: item.number,
                        edit: item.edit,
                        replaced: Replaced::Default,
                    })
                    .expect("release application items");
            }
            records.into_boxed()
        }
        Starting::Saved => Box::default(),
        Starting::Waiting { .. } => unreachable!("defaults prepared before saving"),
    }
}

pub(crate) fn changed(current: &Current, setting: Setting) -> bool {
    match current.applications.get(&setting) {
        Some(state) => match state.follows {
            Follows::Default => false,
            Follows::Changed { .. } | Follows::Removed { .. } => true,
        },
        None => current.changed(setting),
    }
}

pub(crate) fn drop_project(core: &mut Core, project: u32, out: &mut Queue<Request>) {
    let mut erased = List::with_capacity(core.current.applications.len());
    for (&setting, _) in &core.current.applications {
        let Some(Name { scope, .. }) = named(setting) else { continue };
        if scope == project {
            erased.push(setting).expect("project settings bounded by all states");
        }
    }
    for setting in erased.into_boxed() {
        drop(core.current.applications.remove(&setting));
        out.push(Request::Write(crate::Write::Erase(crate::Key::Core(crate::CoreKey::Settings(super::Key::Changed(
            setting,
        ))))));
    }
}
