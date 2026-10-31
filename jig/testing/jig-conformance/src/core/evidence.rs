//! Evidence supplied by an adapter without linking its root or connectors.
use crate::referee as r;
use jig_domain_core as core;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Borrowed core records, submitted mutations, and connector evidence.
#[derive(Debug, Default)]
pub struct Store<'a> {
    pub applied: u64,
    pub rows: BTreeMap<core::Key, &'a core::Record>,
    pub pending: VecDeque<Commit<'a>>,
    pub effects: Effects,
    pub named: Vec<NamedRevision>,
}

/// One atomic decision in the application's store.
#[derive(Debug)]
pub struct Commit<'a> {
    pub number: u64,
    pub writes: Vec<Write<'a>>,
}

/// The core part of a submitted mutation.
#[derive(Debug, PartialEq, Eq)]
pub enum Write<'a> {
    Save(&'a core::Record),
    Erase(core::Key),
    Other,
}

/// Connector facts translated by the owning adapter.
#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub receipts: BTreeSet<r::Receipt>,
    pub attempts: BTreeMap<(u16, r::EffectKey), (u32, u64)>,
    pub held_effects: BTreeSet<(u16, r::EffectKey)>,
    pub settled_effects: BTreeSet<(u16, r::EffectKey)>,
    pub decisions: Vec<r::Decision>,
    pub proposals: BTreeMap<u64, (u16, Effect)>,
}

/// The effect fields needed to associate an accepted proposal with its decision.
#[derive(Clone, Debug)]
pub struct Effect {
    pub kind: u16,
    pub target: u64,
    pub purpose: u64,
}

/// Connector-owned parts of the independently configured referee policy.
#[derive(Clone, Debug, Default)]
pub struct ConnectorPolicy {
    pub kinds: BTreeMap<(u16, u16), r::Kind>,
    pub owned: Vec<(u16, r::Pattern)>,
}

/// A revision named explicitly in a connector's durable procedure step.
#[derive(Clone, Debug)]
pub struct NamedRevision {
    pub connector: u16,
    pub owner: u64,
    pub step: u16,
    pub task: u64,
    pub revision: u64,
    pub priority: bool,
}

/// The host fields an application's prepared assignment exposes to the referee.
#[derive(Clone, Copy, Debug)]
pub struct Assignment {
    pub task: u64,
    pub attempt: u64,
    pub conversation: u64,
    pub resumed: bool,
    pub budget: u64,
    pub settings: u32,
}
