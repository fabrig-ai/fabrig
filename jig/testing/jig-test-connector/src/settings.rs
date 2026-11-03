//! Connector-owned application settings (jig/docs/design/domain/connectors.md, section 17).

use alloc::{boxed::Box, collections::BTreeMap};
use skein_lib::{List, Queue, Token};

use crate::{Domain, Record, RecordKey, Request};

/// One application kind, with defaults given to its owning connector at each start.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::type_complexity, reason = "the configuration names each default by scope, number and bytes")]
pub struct SettingsKind {
    pub kind: u32,
    /// False owns scope zero; true owns nonzero project scopes.
    pub project: bool,
    pub value_bytes: u32,
    pub per_scope: u32,
    pub charters: bool,
    /// Scope, setting number and value.
    pub defaults: Box<[(u32, u32, Box<[u8]>)]>,
}

/// An item in the connector's vocabulary; the root translates the core's item.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingItem {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: SettingEdit,
}

/// A reset removes the override and follows this start's default.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingEdit {
    Set(Box<[u8]>),
    Remove,
    Reset,
}

/// A judgment in the ask's item order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingJudged {
    Accept { size: u32, charters: Box<[u32]> },
    Same,
    Refuse { why: Box<[u8]> },
}

/// A default's metadata compared with the defaults last saved.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingDefault {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub size: u32,
    pub charters: Box<[u32]>,
    pub differs: bool,
}

type Key = (u32, u32, u32);

/// The root bounds live scopes and proposals; the kind bounds each scope's values.
#[derive(Debug)]
pub(crate) struct State {
    defaults: BTreeMap<Key, Box<[u8]>>,
    changes: BTreeMap<Key, Option<Box<[u8]>>>,
    pub(crate) replaced: BTreeMap<(Key, u64), Box<[u8]>>,
    proposals: BTreeMap<(u64, Key), SettingEdit>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            defaults: BTreeMap::new(),
            changes: BTreeMap::new(),
            replaced: BTreeMap::new(),
            proposals: BTreeMap::new(),
        }
    }
}

fn owned(domain: &Domain, key: Key) -> bool {
    let Some(kind) = &domain.config.settings else { return false };
    key.0 == kind.kind && (key.1 != 0) == kind.project
}

fn fits(domain: &Domain, key: Key, value: Option<&[u8]>) -> bool {
    if !owned(domain, key) {
        return false;
    }
    let kind = domain.config.settings.as_ref().expect("owned kind");
    match value {
        Some(bytes) => u32::try_from(bytes.len()).unwrap_or(u32::MAX) <= kind.value_bytes,
        None => true,
    }
}

fn default_value(domain: &Domain, key: Key) -> Option<Box<[u8]>> {
    let kind = domain.config.settings.as_ref()?;
    if !owned(domain, key) {
        return None;
    }
    for (scope, number, value) in &kind.defaults {
        if (*scope, *number) == (key.1, key.2) {
            return Some(value.clone());
        }
    }
    None
}

pub(crate) fn value(domain: &Domain, key: Key) -> Option<Box<[u8]>> {
    if !owned(domain, key) {
        return None;
    }
    match domain.settings.changes.get(&key) {
        Some(value) => value.clone(),
        None => default_value(domain, key),
    }
}

fn charters(domain: &Domain, value: &[u8]) -> Box<[u32]> {
    let kind = domain.config.settings.as_ref().expect("metadata belongs to an owned kind");
    if kind.charters
        && let Some(bytes) = value.get(..4)
    {
        return Box::new([u32::from_be_bytes(bytes.try_into().expect("four charter bytes"))]);
    }
    Box::new([])
}

fn edited(domain: &Domain, key: Key, edit: &SettingEdit) -> Option<Box<[u8]>> {
    match edit {
        SettingEdit::Set(value) => Some(value.clone()),
        SettingEdit::Remove => None,
        SettingEdit::Reset => default_value(domain, key),
    }
}

fn chosen_edit(domain: &Domain, proposal: Option<u64>, item: &SettingItem) -> Option<SettingEdit> {
    match proposal {
        Some(proposal) => domain.settings.proposals.get(&(proposal, (item.kind, item.scope, item.number))).cloned(),
        None => Some(item.edit.clone()),
    }
}

pub(crate) fn judge(
    domain: &Domain,
    token: Token,
    proposal: Option<u64>,
    items: &[SettingItem],
    out: &mut Queue<Request>,
) {
    let mut judged = List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let key = (item.kind, item.scope, item.number);
        let result = match chosen_edit(domain, proposal, item) {
            Some(edit) => judge_edit(domain, key, &edit),
            None => SettingJudged::Refuse { why: Box::from(&b"proposal setting missing"[..]) },
        };
        judged.push(result).expect("one judgment per item");
    }
    out.push(Request::SettingsJudged { token, judged: judged.into_boxed() });
}

fn judge_edit(domain: &Domain, key: Key, edit: &SettingEdit) -> SettingJudged {
    let next = edited(domain, key, edit);
    if !fits(domain, key, next.as_deref()) {
        return SettingJudged::Refuse { why: Box::from(&b"setting outside kind bounds"[..]) };
    }
    if let Some(bytes) = &next
        && bytes.first() == Some(&b'!')
    {
        let rest = bytes.get(1..).expect("refusal has its marker");
        let length =
            9_u32.checked_add(u32::try_from(rest.len()).expect("bounded setting bytes")).expect("refusal fits");
        let mut why = List::with_capacity(length);
        for byte in b"refused: " {
            why.push(*byte).expect("refusal prefix fits");
        }
        for byte in rest {
            why.push(*byte).expect("refusal value fits");
        }
        return SettingJudged::Refuse { why: why.into_boxed() };
    }
    if next == value(domain, key) {
        return SettingJudged::Same;
    }
    match next {
        Some(bytes) => SettingJudged::Accept {
            size: u32::try_from(bytes.len()).expect("bounded setting bytes"),
            charters: charters(domain, &bytes),
        },
        None => SettingJudged::Accept { size: 0, charters: Box::new([]) },
    }
}

pub(crate) fn keep(
    domain: &mut Domain,
    revision: u64,
    proposal: Option<u64>,
    items: &[SettingItem],
    out: &mut Queue<Request>,
) {
    for item in items {
        let key = (item.kind, item.scope, item.number);
        let edit = chosen_edit(domain, proposal, item).expect("committed proposal item was kept");
        if let Some(Some(value)) = domain.settings.changes.get(&key) {
            let value = value.clone();
            domain.settings.replaced.insert((key, revision), value.clone());
            out.push(Request::Save {
                record: Record::SettingReplaced { kind: key.0, scope: key.1, number: key.2, revision, value },
            });
        }
        match edit {
            SettingEdit::Reset => {
                domain.settings.changes.remove(&key);
                out.push(Request::Erase { key: RecordKey::SettingChange { kind: key.0, scope: key.1, number: key.2 } });
            }
            SettingEdit::Set(_) | SettingEdit::Remove => {
                let value = edited(domain, key, &edit);
                assert!(fits(domain, key, value.as_deref()), "committed setting fits its kind");
                domain.settings.changes.insert(key, value.clone());
                out.push(Request::Save {
                    record: Record::SettingChange { kind: key.0, scope: key.1, number: key.2, value },
                });
            }
        }
    }
    if let Some(proposal) = proposal {
        drop_proposal(domain, proposal, out);
    }
}

pub(crate) fn keep_proposal(domain: &mut Domain, proposal: u64, items: &[SettingItem], out: &mut Queue<Request>) {
    for item in items {
        let key = (item.kind, item.scope, item.number);
        let next = edited(domain, key, &item.edit);
        assert!(fits(domain, key, next.as_deref()), "proposed setting fits its kind");
        domain.settings.proposals.insert((proposal, key), item.edit.clone());
        out.push(Request::Save {
            record: Record::SettingProposal {
                proposal,
                kind: key.0,
                scope: key.1,
                number: key.2,
                edit: item.edit.clone(),
            },
        });
    }
}

pub(crate) fn drop_proposal(domain: &mut Domain, proposal: u64, out: &mut Queue<Request>) {
    let keys = proposal_keys(&domain.settings);
    for (owner, key) in &keys {
        let (owner, key) = (*owner, *key);
        if owner == proposal {
            domain.settings.proposals.remove(&(owner, key));
            out.push(Request::Erase {
                key: RecordKey::SettingProposal { proposal, kind: key.0, scope: key.1, number: key.2 },
            });
        }
    }
}

pub(crate) fn defaults(domain: &Domain, out: &mut Queue<Request>) {
    let mut states = BTreeMap::new();
    if let Some(kind) = &domain.config.settings {
        for (scope, number, bytes) in &kind.defaults {
            let key = (kind.kind, *scope, *number);
            let mut count = 0_u32;
            for (other_scope, _, _) in &kind.defaults {
                if other_scope == scope {
                    count = count.checked_add(1).expect("bounded defaults");
                }
            }
            if !fits(domain, key, Some(bytes)) || count > kind.per_scope {
                refuse(key, out);
                return;
            }
            assert!(!states.contains_key(&key), "default identities are unique");
            states.insert(
                key,
                SettingDefault {
                    kind: kind.kind,
                    scope: *scope,
                    number: *number,
                    size: u32::try_from(bytes.len()).expect("bounded default bytes"),
                    charters: charters(domain, bytes),
                    differs: domain.settings.defaults.get(&key) != Some(bytes),
                },
            );
        }
    }
    for key in domain.settings.defaults.keys() {
        if !states.contains_key(key) {
            states.insert(
                *key,
                SettingDefault {
                    kind: key.0,
                    scope: key.1,
                    number: key.2,
                    size: 0,
                    charters: Box::new([]),
                    differs: true,
                },
            );
        }
    }
    let mut defaults = List::with_capacity(u32::try_from(states.len()).expect("bounded defaults"));
    for state in states.into_values() {
        defaults.push(state).expect("one entry per default");
    }
    out.push(Request::SettingsDefaults { defaults: defaults.into_boxed() });
}

pub(crate) fn save_defaults(domain: &mut Domain, out: &mut Queue<Request>) {
    for key in domain.settings.defaults.keys() {
        if default_value(domain, *key).is_none() {
            out.push(Request::Erase { key: RecordKey::SettingDefault { kind: key.0, scope: key.1, number: key.2 } });
        }
    }
    domain.settings.defaults.clear();
    if let Some(kind) = &domain.config.settings {
        for (scope, number, bytes) in &kind.defaults {
            domain.settings.defaults.insert((kind.kind, *scope, *number), bytes.clone());
            out.push(Request::Save {
                record: Record::SettingDefault {
                    kind: kind.kind,
                    scope: *scope,
                    number: *number,
                    value: bytes.clone(),
                },
            });
        }
    }
}

fn refuse(key: Key, out: &mut Queue<Request>) {
    out.push(Request::RestoreRefused { kind: key.0, scope: key.1, number: key.2 });
}

pub(crate) fn restore_default(domain: &mut Domain, key: Key, value: Box<[u8]>, out: &mut Queue<Request>) {
    let count = restored_scope_count(domain, key);
    if !fits(domain, key, Some(&value)) || count >= domain.config.settings.as_ref().expect("owned kind").per_scope {
        refuse(key, out);
        return;
    }
    domain.settings.defaults.insert(key, value);
}

pub(crate) fn restore_change(domain: &mut Domain, key: Key, value: Option<Box<[u8]>>, out: &mut Queue<Request>) {
    let count = restored_scope_count(domain, key);
    if !fits(domain, key, value.as_deref()) || count >= domain.config.settings.as_ref().expect("owned kind").per_scope {
        refuse(key, out);
        return;
    }
    domain.settings.changes.insert(key, value);
}

pub(crate) fn restore_replaced(
    domain: &mut Domain,
    key: Key,
    revision: u64,
    value: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let mut count = 0_u32;
    for (other, owner) in domain.settings.replaced.keys() {
        if *owner == revision && other.1 == key.1 && *other != key {
            count = count.checked_add(1).expect("bounded replaced values");
        }
    }
    if !fits(domain, key, Some(&value)) || count >= domain.config.settings.as_ref().expect("owned kind").per_scope {
        refuse(key, out);
        return;
    }
    domain.settings.replaced.insert((key, revision), value);
}

pub(crate) fn restore_proposal(
    domain: &mut Domain,
    proposal: u64,
    key: Key,
    edit: SettingEdit,
    out: &mut Queue<Request>,
) {
    let next = match &edit {
        SettingEdit::Set(value) => Some(value.as_ref()),
        SettingEdit::Remove | SettingEdit::Reset => None,
    };
    let mut count = 0_u32;
    for (owner, other) in domain.settings.proposals.keys() {
        if *owner == proposal && other.1 == key.1 && *other != key {
            count = count.checked_add(1).expect("bounded proposal items");
        }
    }
    if !fits(domain, key, next) || count >= domain.config.settings.as_ref().expect("owned kind").per_scope {
        refuse(key, out);
        return;
    }
    domain.settings.proposals.insert((proposal, key), edit);
}

pub(crate) fn release_project(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    if project == 0 {
        return;
    }
    if let Some(kind) = &mut domain.config.settings {
        let mut retained = List::with_capacity(u32::try_from(kind.defaults.len()).expect("bounded defaults"));
        for (scope, number, value) in &kind.defaults {
            if *scope != project {
                retained.push((*scope, *number, value.clone())).expect("retained defaults fit");
            }
        }
        kind.defaults = retained.into_boxed();
    }
    let mut changes = List::with_capacity(u32::try_from(domain.settings.changes.len()).expect("bounded changes"));
    for key in domain.settings.changes.keys() {
        changes.push(*key).expect("one key per change");
    }
    for key in &changes {
        let key = *key;
        if key.1 == project && project != 0 {
            domain.settings.changes.remove(&key);
            out.push(Request::Erase { key: RecordKey::SettingChange { kind: key.0, scope: key.1, number: key.2 } });
        }
    }
    let mut defaults = List::with_capacity(u32::try_from(domain.settings.defaults.len()).expect("bounded defaults"));
    for key in domain.settings.defaults.keys() {
        defaults.push(*key).expect("one key per default");
    }
    for key in &defaults {
        let key = *key;
        if key.1 == project && project != 0 {
            domain.settings.defaults.remove(&key);
            out.push(Request::Erase { key: RecordKey::SettingDefault { kind: key.0, scope: key.1, number: key.2 } });
        }
    }
    let proposals = proposal_keys(&domain.settings);
    for (proposal, key) in &proposals {
        let (proposal, key) = (*proposal, *key);
        if key.1 == project && project != 0 {
            domain.settings.proposals.remove(&(proposal, key));
            out.push(Request::Erase {
                key: RecordKey::SettingProposal { proposal, kind: key.0, scope: key.1, number: key.2 },
            });
        }
    }
}

fn proposal_keys(state: &State) -> List<(u64, Key)> {
    let mut keys = List::with_capacity(u32::try_from(state.proposals.len()).expect("bounded proposals"));
    for key in state.proposals.keys() {
        keys.push(*key).expect("one key per proposal item");
    }
    keys
}

fn restored_scope_count(domain: &Domain, key: Key) -> u32 {
    let mut count = 0_u32;
    for other in domain.settings.defaults.keys() {
        if other.1 == key.1 && *other != key {
            count = count.checked_add(1).expect("bounded defaults");
        }
    }
    for other in domain.settings.changes.keys() {
        if other.1 == key.1 && *other != key && !domain.settings.defaults.contains_key(other) {
            count = count.checked_add(1).expect("bounded changes");
        }
    }
    count
}
