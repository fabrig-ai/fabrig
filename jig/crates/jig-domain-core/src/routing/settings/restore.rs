//! A paged proposal of what later revisions replaced (jig/docs/design/domain/engine.md, 17.10).
use crate::routing::{
    Work,
    party::dispatch::{Party, Routed},
};
use crate::{Core, Event, Limits, Request, effects::connector, reads, settings as s};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Decimal, Env, Id, List, Map, Queue, Token, Writer};

pub(crate) fn start(core: &mut Core, env: &Env<Limits>, party: Party, revision: u32, reason: Box<[u8]>) -> Routed {
    let mut routed = Routed::new(&mut Queue::uncapped());
    let newest = core.current.newest();
    let why = if core.restoring.pending.is_some() || revision == 0 || revision > newest {
        Some(people::Refusal::Stale { newest })
    } else if revision < reads::settings::oldest(core, &env.limits) {
        Some(people::Refusal::Forgotten { oldest: reads::settings::oldest(core, &env.limits) })
    } else if revision == newest {
        Some(empty())
    } else {
        None
    };
    if let Some(why) = why {
        refuse(&mut routed, party.request, why);
        return routed;
    }
    core.restoring.pending = Some(s::Restoration {
        party,
        revision,
        newest,
        reason,
        items: Map::with_capacity(jig_domain_tasks::SETTINGS_ITEMS.checked_add(1).expect("one policy")),
        left_out: Box::default(),
        proposal: None,
    });
    load(core, env, &mut routed, newest);
    routed
}

fn empty() -> people::Refusal {
    people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged }
}
fn refuse(routed: &mut Routed, request: Token, why: people::Refusal) {
    routed
        .work
        .push(Work::Event(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) })));
}
fn failed(core: &Core, routed: &mut Routed, why: people::Refusal) {
    refuse(routed, core.restoring.pending.as_ref().expect("active restore").party.request, why);
}
fn load(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, last: u32) {
    let pending = core.restoring.pending.as_ref().expect("active restore");
    let first =
        pending.revision.saturating_add(1).max(last.saturating_sub(env.limits.documents.rows.saturating_sub(1)));
    match reads::settings::restore_load(core, env, first, last, pending.party.person) {
        Some(mut out) => {
            for _ in 0..out.len() {
                routed.out.push(out.pop().expect("page load output"));
            }
        }
        None => busy(core, routed),
    }
}

fn busy(core: &Core, routed: &mut Routed) {
    routed.work.push(Work::Event(Event::People(people::Event::Decided {
        request: core.restoring.pending.as_ref().expect("active restore").party.request,
        outcome: people::Outcome::Again(people::Again::Busy),
    })));
}

pub(crate) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    range: reads::Range,
    result: reads::Loaded,
) -> Queue<Request> {
    let mut routed = Routed::new(work);
    let Some(pending) = core.restoring.pending.as_ref() else { return routed.finish(work) };
    if pending.newest != core.current.newest() {
        failed(core, &mut routed, people::Refusal::Stale { newest: core.current.newest() });
        return routed.finish(work);
    }
    let result = match result {
        reads::Loaded::Busy => {
            busy(core, &mut routed);
            return routed.finish(work);
        }
        other @ (reads::Loaded::Page { .. }
        | reads::Loaded::Gone
        | reads::Loaded::TooLinked { .. }
        | reads::Loaded::TurnProblem { .. }) => other,
    };
    let result = page(core, env, &range, result);
    match result {
        Ok(first) => {
            if first > core.restoring.pending.as_ref().expect("active restore").revision.saturating_add(1) {
                load(core, env, &mut routed, first.saturating_sub(1));
            } else {
                propose(core, env, &mut routed);
            }
        }
        Err(why) => failed(core, &mut routed, why),
    }
    routed.finish(work)
}

fn page(
    core: &mut Core,
    env: &Env<Limits>,
    range: &reads::Range,
    result: reads::Loaded,
) -> Result<u32, people::Refusal> {
    let reads::Scope::SettingsHistory { first, last, .. } = range.scope else {
        unreachable!("restore loads only history")
    };
    let rows = match result {
        reads::Loaded::Page { rows, next: None } if rows.len() <= skein_lib::length::usize_of(range.rows) => rows,
        reads::Loaded::Page { .. }
        | reads::Loaded::Busy
        | reads::Loaded::Gone
        | reads::Loaded::TooLinked { .. }
        | reads::Loaded::TurnProblem { .. } => return Err(people::Refusal::Unknown),
    };
    let mut records = Map::with_capacity(range.rows);
    for row in rows {
        if !reads::admits(range.party, &range.scope, &row.value.key())
            || row.owner.is_some()
            || !row.ancestors.is_empty()
            || row.related.is_some()
            || !row.linked.is_empty()
            || row.inbox.is_some()
            || row.unjoined.is_some()
            || row.ended_read.is_some()
        {
            return Err(people::Refusal::Unknown);
        }
        let Some(revision) = reads::settings::record_revision(&row.value) else { return Err(people::Refusal::Unknown) };
        if !reads::settings::valid_revision(revision, &env.limits) {
            return Err(people::Refusal::Unknown);
        }
        if records.insert(revision.head.number, revision.clone()).ok() != Some(None) {
            return Err(people::Refusal::Unknown);
        }
    }
    for number in (first..=last).rev() {
        let Some(revision) = records.remove(&number) else { return Err(people::Refusal::Unknown) };
        for item in revision.items {
            let edit = match item.replaced {
                s::Replaced::Value(value) => s::Edit::Set(value),
                s::Replaced::Default | s::Replaced::NotOffered => s::Edit::Reset,
                s::Replaced::Removed => s::Edit::Removed,
            };
            fold(core, &env.limits, item.setting, s::RestoreEdit::Core(edit))?;
        }
        for item in revision.applications {
            let edit = match item.replaced {
                s::application::Replaced::Changed { .. } => {
                    s::application::RestoredEdit::Set { revision: u64::from(number) }
                }
                s::application::Replaced::Default | s::application::Replaced::NotOffered => {
                    s::application::RestoredEdit::Reset
                }
                s::application::Replaced::Removed => s::application::RestoredEdit::Remove,
            };
            fold(
                core,
                &env.limits,
                s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number },
                s::RestoreEdit::Application(edit),
            )?;
        }
    }
    Ok(first)
}

fn count_failure(setting: s::Setting) -> people::Refusal {
    super::dispatch::malformed_refusal(s::Failure { setting, fault: s::Fault::Count })
}
fn fold(core: &mut Core, limits: &Limits, setting: s::Setting, edit: s::RestoreEdit) -> Result<(), people::Refusal> {
    let pending = core.restoring.pending.as_mut().expect("active restore");
    if !pending.items.contains_key(&setting) {
        let mut count = 0_u32;
        let mut regular = 0_u32;
        let mut kind_count = 0_u32;
        let kind = application_kind(setting);
        for (&held, _) in &pending.items {
            if kind.is_some() && application_kind(held) == kind {
                kind_count = kind_count.saturating_add(1);
            }
            if group_name(held) == group_name(setting) {
                count = count.saturating_add(1);
            }
            if !s::is_project_policy(held) {
                regular = regular.saturating_add(1);
            }
        }
        let bound = match setting {
            s::Setting::Charter(_) => limits.tasks.charters,
            s::Setting::Template(_) => limits.templates,
            s::Setting::Policy(_) => limits.authority.projects,
            s::Setting::Rules | s::Setting::ProjectPolicy { .. } => 1,
            s::Setting::Application { kind, .. } => {
                let mut bound = None;
                for declared in &core.kinds {
                    if declared.number == kind {
                        let scopes = match declared.scope {
                            s::KindScope::Deployment => 1,
                            s::KindScope::Project => limits.authority.projects,
                        };
                        let whole = declared.per_scope.checked_mul(scopes).expect("configured kind count");
                        if kind_count >= whole {
                            return Err(count_failure(setting));
                        }
                        bound = Some(declared.per_scope);
                    }
                }
                bound.ok_or(super::dispatch::malformed_refusal(s::Failure { setting, fault: s::Fault::Shape }))?
            }
        };
        if count >= bound || (!s::is_project_policy(setting) && regular >= jig_domain_tasks::SETTINGS_ITEMS) {
            return Err(count_failure(setting));
        }
    }
    if pending.items.insert(setting, edit).is_err() {
        return Err(count_failure(setting));
    }
    Ok(())
}
fn application_kind(setting: s::Setting) -> Option<u32> {
    match setting {
        s::Setting::Application { kind, .. } => Some(kind),
        s::Setting::Charter(_)
        | s::Setting::Template(_)
        | s::Setting::Policy(_)
        | s::Setting::Rules
        | s::Setting::ProjectPolicy { .. } => None,
    }
}
fn group_name(setting: s::Setting) -> s::Setting {
    match setting {
        s::Setting::Charter(_) => s::Setting::Charter(0),
        s::Setting::Template(_) => s::Setting::Template(0),
        s::Setting::Policy(_) => s::Setting::Policy(0),
        s::Setting::Rules => s::Setting::Rules,
        s::Setting::ProjectPolicy { .. } => s::Setting::ProjectPolicy { project: 0 },
        s::Setting::Application { kind, scope, .. } => s::Setting::Application { kind, scope, number: 0 },
    }
}

fn deleted(core: &Core, setting: s::Setting) -> bool {
    let project = match setting {
        s::Setting::ProjectPolicy { project } => project,
        s::Setting::Application { scope, .. } => scope,
        s::Setting::Charter(_) | s::Setting::Template(_) | s::Setting::Policy(_) | s::Setting::Rules => return false,
    };
    project != people::DEPLOYMENT && !core.open_project(project)
}

fn proposal_items(core: &Core) -> Box<[s::Item]> {
    let pending = core.restoring.pending.as_ref().expect("active restore");
    let mut items = List::with_capacity(pending.items.len());
    for (&setting, edit) in &pending.items {
        if pending.left_out.contains(&setting) {
            continue;
        }
        let edit = match edit {
            s::RestoreEdit::Core(edit) => edit.clone(),
            s::RestoreEdit::Application(edit) => match edit {
                s::application::RestoredEdit::Set { .. } => s::Edit::Set(s::Value::Application(Box::default())),
                s::application::RestoredEdit::Remove => s::Edit::Removed,
                s::application::RestoredEdit::Reset => s::Edit::Reset,
            },
        };
        items.push(s::Item { setting, edit }).expect("bounded restore items");
    }
    items.into_boxed()
}

fn propose(core: &mut Core, env: &Env<Limits>, routed: &mut Routed) {
    let pending = core.restoring.pending.as_ref().expect("active restore");
    let mut left_out = List::with_capacity(pending.items.len());
    for (&setting, _) in &pending.items {
        if deleted(core, setting) {
            left_out.push(setting).expect("bounded excluded items");
        }
    }
    core.restoring.pending.as_mut().expect("active restore").left_out = left_out.into_boxed();
    let items = proposal_items(core);
    let against = core.current.newest();
    super::dispatch::prepare_proposal(core, env, against, items);
    if let Err(why) = super::dispatch::proposal_check(core, env, against) {
        failed(core, routed, why);
        return;
    }
    let mut connectors = Map::with_capacity(u32::try_from(core.kinds.len()).expect("declared kinds"));
    let pending = core.restoring.pending.as_ref().expect("active restore");
    for (&setting, edit) in &pending.items {
        if pending.left_out.contains(&setting) {
            continue;
        }
        match edit {
            s::RestoreEdit::Core(_) => {}
            s::RestoreEdit::Application(_) => {
                let s::Setting::Application { kind, .. } = setting else { unreachable!("application item") };
                connectors.insert(connector_of(core, kind), ()).expect("declared connectors");
            }
        }
    }
    if connectors.is_empty() {
        admit(core, env, routed);
        return;
    }
    let proposal = core.current.keys;
    core.current.keys = proposal.checked_add(1).expect("proposal preflighted");
    core.restoring.pending.as_mut().expect("active restore").proposal = Some(proposal);
    routed.out.push(Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(
        s::Record::Extent { oldest: core.current.oldest, keys: core.current.keys },
    )))));
    for (&connector, ()) in &connectors {
        let pending = core.restoring.pending.as_ref().expect("active restore");
        let mut items = List::with_capacity(pending.items.len());
        for (&setting, edit) in &pending.items {
            if pending.left_out.contains(&setting) {
                continue;
            }
            match edit {
                s::RestoreEdit::Core(_) => {}
                s::RestoreEdit::Application(edit) => {
                    let s::Setting::Application { kind, scope, number } = setting else {
                        unreachable!("application item")
                    };
                    if connector_of(core, kind) == connector {
                        items
                            .push(s::application::Restored { kind, scope, number, edit: edit.clone() })
                            .expect("bounded group");
                    }
                }
            }
        }
        let items = items.into_boxed();
        let owner = core
            .restoring
            .groups
            .insert(Some(s::RestoreGroup { connector, items: items.clone(), judged: None }))
            .expect("one group per connector")
            .token();
        routed.out.push(Request::Ask {
            connector,
            ask: crate::Ask::Effect(connector::Ask::RestoreSettings { owner, proposal, items }),
        });
    }
}
fn connector_of(core: &Core, number: u32) -> u16 {
    for kind in &core.kinds {
        if kind.number == number {
            return kind.connector;
        }
    }
    unreachable!("checked kind")
}

pub(crate) fn restored(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    owner: Token,
    judged: Box<[connector::Judged]>,
) -> Queue<Request> {
    let group = core
        .restoring
        .groups
        .get_mut(Id::from_token(owner))
        .expect("root answered a restore owner")
        .as_mut()
        .expect("active restore owner");
    assert!(group.judged.is_none(), "one restore answer per connector");
    assert_eq!(judged.len(), group.items.len(), "one restore answer per item");
    group.judged = Some(judged);
    for (_, group) in &core.restoring.groups {
        if group.as_ref().expect("active restore group").judged.is_none() {
            return Queue::uncapped();
        }
    }
    let mut routed = Routed::new(work);
    if let Err(why) = answers(core, env) {
        failed(core, &mut routed, why);
    } else {
        admit(core, env, &mut routed);
    }
    routed.finish(work)
}

fn answers(core: &Core, env: &Env<Limits>) -> Result<(), people::Refusal> {
    let pending = core.restoring.pending.as_ref().expect("active restore");
    if pending.newest != core.current.newest() {
        return Err(people::Refusal::Stale { newest: core.current.newest() });
    }
    let mut at = 0_u32;
    // Select a refusal in proposal order before judging any returned metadata.
    for (&setting, _) in &pending.items {
        if pending.left_out.contains(&setting) {
            continue;
        }
        for (_, group) in &core.restoring.groups {
            let group = group.as_ref().expect("active group");
            for (item, answer) in group.items.iter().zip(group.judged.as_ref().expect("answered group").iter()) {
                if setting != (s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number }) {
                    continue;
                }
                match answer {
                    connector::Judged::Refuse { why } => {
                        return Err(people::Refusal::Judged {
                            at,
                            setting,
                            why: s::application::words(why, env.limits.people.words),
                        });
                    }
                    connector::Judged::Same | connector::Judged::Accept { .. } => {}
                }
            }
        }
        at = at.checked_add(1).expect("bounded item count");
    }
    for (_, group) in &core.restoring.groups {
        let group = group.as_ref().expect("active group");
        for (item, answer) in group.items.iter().zip(group.judged.as_ref().expect("answered group").iter()) {
            let setting = s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
            match answer {
                connector::Judged::Refuse { .. } => unreachable!("refusals checked first"),
                connector::Judged::Same => {
                    return Err(super::dispatch::malformed_refusal(s::Failure { setting, fault: s::Fault::Shape }));
                }
                connector::Judged::Accept { size, charters } => {
                    for kind in &core.kinds {
                        if kind.number == item.kind
                            && (*size > kind.value_bytes || charters.len() > usize::from(kind.charters))
                        {
                            return Err(super::dispatch::malformed_refusal(s::Failure {
                                setting,
                                fault: s::Fault::Shape,
                            }));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
fn admit(core: &mut Core, env: &Env<Limits>, routed: &mut Routed) {
    let pending = core.restoring.pending.as_ref().expect("active restore");
    let party = pending.party;
    let against = pending.newest;
    let reason = reason(pending.revision, &pending.reason, env.limits.people.words);
    let items = proposal_items(core);
    super::dispatch::prepare_proposal(core, env, against, items);
    super::dispatch::propose_admitted(core, env, routed, party, against, reason);
}

/// The connectors have retained their values; the offering carries only their sized edits.
pub(crate) fn proposal_value(core: &Core, request: Token, value: &mut s::Proposal) {
    let Some(pending) = &core.restoring.pending else { return };
    if pending.party.request != request {
        return;
    }
    value.key = pending.proposal;
    let mut jig = List::with_capacity(u32::try_from(value.items.len()).expect("bounded items"));
    for item in &value.items {
        if !s::application::is_application(item.setting) {
            jig.push(item.clone()).expect("jig items");
        }
    }
    let mut applications = List::with_capacity(pending.items.len());
    for (_, group) in &core.restoring.groups {
        let group = group.as_ref().expect("active group");
        for (item, answer) in group.items.iter().zip(group.judged.as_ref().expect("answered group").iter()) {
            let connector::Judged::Accept { size, .. } = answer else { unreachable!("checked restore answer") };
            let edit = match item.edit {
                s::application::RestoredEdit::Set { .. } => s::application::SizedEdit::Set { size: *size },
                s::application::RestoredEdit::Remove => s::application::SizedEdit::Remove,
                s::application::RestoredEdit::Reset => s::application::SizedEdit::Reset,
            };
            applications
                .push(s::application::Recorded { kind: item.kind, scope: item.scope, number: item.number, edit })
                .expect("bounded applications");
        }
    }
    value.items = jig.into_boxed();
    value.applications = applications.into_boxed();
}

/// Complete the original keyed request and release every volatile connector continuation.
pub(crate) fn outcome(
    core: &mut Core,
    request: Token,
    outcome: people::Outcome,
    out: &mut Queue<Request>,
) -> people::Outcome {
    let Some(pending) = &core.restoring.pending else { return outcome };
    if pending.party.request != request {
        return outcome;
    }
    let pending = core.restoring.pending.take().expect("matched restore request");
    let people::Outcome::Proposed { proposal, to } = outcome else {
        finish_groups(core, pending.proposal, out);
        return outcome;
    };
    finish_groups(core, None, out);
    people::Outcome::Restored { proposal, to, left_out: pending.left_out }
}
fn finish_groups(core: &mut Core, drop_proposal: Option<u64>, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(core.restoring.groups.len());
    for (id, group) in &core.restoring.groups {
        if let Some(proposal) = drop_proposal {
            out.push(Request::Ask {
                connector: group.as_ref().expect("active group").connector,
                ask: crate::Ask::Effect(connector::Ask::DropProposalSettings { proposal }),
            });
        }
        names.push(id).expect("active groups");
    }
    for id in names.into_boxed() {
        drop(core.restoring.groups.get_mut(id).expect("active group").take());
        core.restoring.groups.retire(id);
    }
    core.restoring.groups.reclaim();
}

fn reason(revision: u32, words: &[u8], bound: u32) -> Box<[u8]> {
    let number = Decimal::of(u64::from(revision));
    let prefix = b"restores revision ";
    let len = prefix
        .len()
        .checked_add(number.as_bytes().len())
        .expect("revision prefix")
        .checked_add(2)
        .expect("separator")
        .checked_add(words.len())
        .expect("bounded reason");
    let mut writer = Writer::new(len);
    for part in [prefix.as_slice(), number.as_bytes(), b": ".as_slice(), words] {
        writer.put(part).expect("sized restore reason");
    }
    s::application::words(&writer.finish(), bound)
}
