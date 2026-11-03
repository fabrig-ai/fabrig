//! Application-owned settings (jig/docs/design/domain/engine.md, 17.5).

use alloc::boxed::Box;

/// Where a kind's values are kept (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KindScope {
    Deployment,
    Project,
}

/// A declared application kind and its bounds (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Kind {
    pub number: u32,
    pub scope: KindScope,
    pub connector: u16,
    pub value_bytes: u32,
    pub per_scope: u32,
    pub charters: u8,
}

/// An edit whose value stays with its connector (jig/docs/design/domain/engine.md, 17.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Box<[u8]>),
    Remove,
    Reset,
}

/// One application setting handed to its connector (jig/docs/design/domain/engine.md, 17.5).
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
        Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => false,
    }
}

pub(crate) fn projects(items: &[super::Item]) -> Box<[u32]> {
    let mut names = Map::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let scope = match item.setting {
            Setting::Application { scope, .. } => scope,
            Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => people::DEPLOYMENT,
        };
        names.insert(scope, ()).expect("one scope per item");
    }
    let mut result = List::with_capacity(names.len());
    for (&scope, ()) in &names {
        result.push(scope).expect("unique scopes");
    }
    result.into_boxed()
}

/// The core checks names, scopes, bytes and the count the overlay would leave before asking a connector.
pub(crate) fn check(core: &Core, items: &[super::Item]) -> Result<(), Failure> {
    for item in items {
        let (kind, scope, number) = match item.setting {
            Setting::Application { kind, scope, number } => (kind, scope, number),
            Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => continue,
        };
        let declared = kind_of(&core.kinds, kind).ok_or(Failure { setting: item.setting, fault: Fault::Shape })?;
        let valid_scope = match declared.scope {
            KindScope::Deployment => scope == 0,
            KindScope::Project => scope != 0 && core.projects.as_slice().contains(&scope),
        };
        if number == 0 || !valid_scope {
            return Err(Failure { setting: item.setting, fault: Fault::Shape });
        }
        match &item.edit {
            super::Edit::Set(super::Value::Application(bytes)) => {
                if bytes.len() > skein_lib::length::usize_of(declared.value_bytes) {
                    return Err(Failure { setting: item.setting, fault: Fault::Shape });
                }
            }
            super::Edit::Set(
                super::Value::Charter(_) | super::Value::Template(_) | super::Value::Policy(_) | super::Value::Rules(_),
            ) => {
                return Err(Failure { setting: item.setting, fault: Fault::Shape });
            }
            super::Edit::Removed | super::Edit::Reset => {}
        }
        let mut count = 0_u32;
        for (&setting, state) in &core.current.applications {
            if same_scope(setting, kind, scope) && leaves_present(items, setting, state) {
                count = count.checked_add(1).expect("bounded settings count");
            }
        }
        for addition in items {
            if same_scope(addition.setting, kind, scope) && !core.current.applications.contains_key(&addition.setting) {
                match addition.edit {
                    super::Edit::Set(_) | super::Edit::Reset => {
                        count = count.checked_add(1).expect("bounded settings count");
                    }
                    super::Edit::Removed => {}
                }
            }
        }
        if count > declared.per_scope {
            return Err(Failure { setting: item.setting, fault: Fault::Count });
        }
    }
    Ok(())
}
fn same_scope(setting: Setting, kind: u32, scope: u32) -> bool {
    match setting {
        Setting::Application { kind: held, scope: held_scope, .. } => kind == held && scope == held_scope,
        Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => false,
    }
}
fn leaves_present(items: &[super::Item], setting: Setting, state: &State) -> bool {
    for item in items {
        if item.setting == setting {
            return match item.edit {
                super::Edit::Removed => false,
                super::Edit::Set(_) | super::Edit::Reset => true,
            };
        }
    }
    match state.follows {
        Follows::Removed { .. } => false,
        Follows::Changed { .. } | Follows::Default => true,
    }
}
#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
fn kind_of(kinds: &[Kind], number: u32) -> Option<&Kind> {
    for kind in kinds {
        if kind.number == number {
            return Some(kind);
        }
    }
    None
}
fn translate(item: &super::Item) -> Item {
    let Setting::Application { kind, scope, number } = item.setting else { unreachable!("application item") };
    let edit = match &item.edit {
        super::Edit::Set(super::Value::Application(bytes)) => Edit::Set(bytes.clone()),
        super::Edit::Removed => Edit::Remove,
        super::Edit::Reset => Edit::Reset,
        super::Edit::Set(
            super::Value::Charter(_) | super::Value::Template(_) | super::Value::Policy(_) | super::Value::Rules(_),
        ) => unreachable!("checked application value"),
    };
    Item { kind, scope, number, edit }
}

/// The change and its keyed people continuation live only until the same step's judges answer.
#[derive(Debug)]
struct Decision {
    request: Token,
    person: u64,
    change: Change,
    room: u32,
    proposal: Option<(people::Proposer, u32)>,
}
#[derive(Debug)]
struct Group {
    connector: u16,
    positions: Box<[u32]>,
    judged: Option<Box<[connector::Judged]>>,
}
#[derive(Debug)]
pub(crate) struct Judging {
    decision: Option<Decision>,
    groups: Slab<Option<Group>>,
}
impl Judging {
    pub(crate) fn new(kinds: &[Kind]) -> Self {
        Self { decision: None, groups: Slab::with_capacity(u32::try_from(kinds.len()).expect("declared kinds")) }
    }
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
        let kind = match item.setting {
            Setting::Application { kind, .. } => kind,
            Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => continue,
        };
        let connector = kind_of(&core.kinds, kind).expect("checked kind").connector;
        if !connectors.as_slice().contains(&connector) {
            connectors.push(connector).expect("one connector per kind");
        }
    }
    for connector in connectors.into_boxed() {
        let mut items = List::with_capacity(u32::try_from(change.items.len()).expect("bounded items"));
        let mut positions = List::with_capacity(items.capacity());
        for (at, item) in change.items.iter().enumerate() {
            let kind = match item.setting {
                Setting::Application { kind, .. } => kind,
                Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => continue,
            };
            if kind_of(&core.kinds, kind).expect("checked kind").connector == connector {
                items.push(translate(item)).expect("group items");
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
                proposal: proposal_number(proposal),
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
            decision.change,
            decision.room,
            kept,
            decision.proposal,
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
    // A connector's first refusal is selected in the change's order, independently of answer order.
    for (at, answer) in judged.iter().enumerate() {
        if let Some(connector::Judged::Refuse { why }) = answer {
            let why = words(why, env.limits.people.words);
            return Err(people::Refusal::Judged {
                at: u32::try_from(at).expect("bounded position"),
                setting: change.items.get(at).expect("item position").setting,
                why,
            });
        }
    }
    let mut kept = List::with_capacity(u32::try_from(change.items.len()).expect("bounded items"));
    for (item, answer) in change.items.iter().zip(judged) {
        let Some(answer) = answer else { continue };
        match answer {
            connector::Judged::Same => {}
            connector::Judged::Refuse { .. } => unreachable!("refusals checked first"),
            connector::Judged::Accept { size, charters } => {
                let item = translate(item);
                let kind = kind_of(&core.kinds, item.kind).expect("checked kind");
                if size > kind.value_bytes || charters.len() > usize::from(kind.charters) {
                    return Err(crate::routing::settings::dispatch::malformed_refusal(Failure {
                        setting: Setting::Application { kind: item.kind, scope: item.scope, number: item.number },
                        fault: Fault::Shape,
                    }));
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
    }
    Ok(kept.into_boxed())
}
#[expect(
    clippy::disallowed_methods,
    reason = "the core checks and cuts connector refusal words at its internal byte boundary"
)]
fn words(bytes: &[u8], bound: u32) -> Box<[u8]> {
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
            return Err(Failure { setting, fault: Fault::Reference });
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
) -> Box<[Recorded]> {
    let bound = u32::try_from(kept.len()).expect("bounded accepted items");
    let mut records = List::with_capacity(bound);
    let mut connectors = Map::with_capacity(bound);
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
        records
            .push(Recorded { kind: accepted.item.kind, scope: accepted.item.scope, number: accepted.item.number, edit })
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

fn proposal_number(proposal: Option<(people::Proposer, u32)>) -> Option<u64> {
    match proposal {
        Some((_, number)) => Some(u64::from(number)),
        None => None,
    }
}

/// Values cross to their connectors only when the tasks child keeps the proposal.
pub(crate) fn keep_proposal(core: &Core, set: &mut super::Proposal, proposal: u32, out: &mut Queue<Request>) {
    let bound = u32::try_from(set.items.len()).expect("bounded proposal items");
    let mut connectors = Map::with_capacity(bound);
    let mut applications = List::with_capacity(bound);
    let mut jig = List::with_capacity(bound);
    for item in &set.items {
        let (kind, scope, number) = match item.setting {
            Setting::Application { kind, scope, number } => (kind, scope, number),
            Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => {
                jig.push(item.clone()).expect("bounded jig items");
                continue;
            }
        };
        {
            let connector = kind_of(&core.kinds, kind).expect("checked proposal kind").connector;
            connectors.insert(connector, ()).expect("one connector per item");
            let edit = match &item.edit {
                super::Edit::Set(super::Value::Application(bytes)) => {
                    SizedEdit::Set { size: u32::try_from(bytes.len()).expect("checked value bytes") }
                }
                super::Edit::Removed => SizedEdit::Remove,
                super::Edit::Reset => SizedEdit::Reset,
                super::Edit::Set(
                    super::Value::Charter(_)
                    | super::Value::Template(_)
                    | super::Value::Policy(_)
                    | super::Value::Rules(_),
                ) => unreachable!("checked proposal value"),
            };
            applications.push(Recorded { kind, scope, number, edit }).expect("bounded proposal items");
        }
    }
    for (&connector, ()) in &connectors {
        let mut items = List::with_capacity(bound);
        for item in &set.items {
            match item.setting {
                Setting::Application { kind, .. } => {
                    if kind_of(&core.kinds, kind).expect("checked kind").connector == connector {
                        items.push(translate(item)).expect("connector proposal items");
                    }
                }
                Setting::Charter(_) | Setting::Template(_) | Setting::Policy(_) | Setting::Rules => {}
            }
        }
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::KeepProposalSettings {
                proposal: u64::from(proposal),
                items: items.into_boxed(),
            }),
        });
    }
    set.items = jig.into_boxed();
    set.applications = applications.into_boxed();
}

pub(crate) fn drop_proposal(core: &Core, items: &[Recorded], proposal: u32, out: &mut Queue<Request>) {
    let mut connectors = Map::with_capacity(u32::try_from(items.len()).expect("bounded proposal items"));
    for item in items {
        let connector = kind_of(&core.kinds, item.kind).expect("kept proposal kind").connector;
        connectors.insert(connector, ()).expect("one connector per item");
    }
    for (&connector, ()) in &connectors {
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::DropProposalSettings { proposal: u64::from(proposal) }),
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
        let kind = kind_of(&core.kinds, item.kind)
            .ok_or(crate::routing::settings::dispatch::malformed_refusal(Failure { setting, fault: Fault::Shape }))?;
        let edit = match item.edit {
            SizedEdit::Set { size } => {
                if size > kind.value_bytes {
                    return Err(crate::routing::settings::dispatch::malformed_refusal(Failure {
                        setting,
                        fault: Fault::Shape,
                    }));
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
