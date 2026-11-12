//! Keep old generation bytes opaque until the patched generation is durably active.
use super::{Commit, Store, codec, core, in_generation, keys, snapshots};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Generation {
    pub(super) number: u64,
    pub(super) rows: BTreeMap<Vec<u8>, Vec<u8>>,
    touched: BTreeMap<Vec<u8>, u64>,
    snapshot: Option<u64>,
    commits: BTreeMap<u64, Box<[skein_kv::Op]>>,
}

fn name(path: &[u8]) -> Option<&[u8]> {
    let suffix = in_generation(path)?;
    Some(&path[..path.len() - suffix.len() - 1])
}

fn behind(rows: &BTreeMap<Vec<u8>, Vec<u8>>, binary: codec::component::Levels) -> bool {
    let key = keys::encode_own(&keys::Own::Format, 64).expect("format key");
    let Some(bytes) = rows.get(key.as_ref()) else { return false };
    let found = codec::component::Format::decode(bytes).expect("observed format row").levels;
    [
        (found.skein, binary.skein),
        (found.smith, binary.smith),
        (found.jig, binary.jig),
        (found.application, binary.application),
    ]
    .into_iter()
    .any(|(found, wanted)| found.unwrap_or(0) < wanted.unwrap_or(0))
}

impl Store {
    pub(super) fn generation(&self, path: &[u8]) -> Option<&Generation> {
        self.generations.get(name(path)?)
    }

    pub(super) fn is_active(&self, path: &[u8]) -> bool {
        self.active.as_deref().is_none_or(|active| name(path) == Some(active))
    }

    fn binary(&self) -> codec::component::Levels {
        self.levels.unwrap_or_else(|| jig_protocol_store::levels(&temper_engine_service::store::PATCH_SETS))
    }

    /// Retain every raw operation before deciding whether its generation can be decoded.
    pub(super) fn record(&mut self, path: &[u8], number: u64, ops: &[skein_kv::Op], snapshot: bool) -> bool {
        let name = name(path).expect("store generation").to_vec();
        let binary = self.binary();
        let generation = self.generations.entry(name.clone()).or_default();
        if snapshot {
            // Copying can expose logs before their older snapshot. Preserve every
            // later write or erasure while filling in that snapshot's other rows.
            generation.rows.retain(|key, _| generation.touched.get(key).is_some_and(|written| *written > number));
            generation.snapshot = Some(number);
        } else {
            generation.commits.insert(number, ops.into());
        }
        for op in ops {
            let key = match op {
                skein_kv::Op::Put { key, .. } | skein_kv::Op::Erase { key } => key,
            };
            if snapshot && generation.touched.get(key.as_ref()).is_some_and(|written| *written > number) {
                continue;
            }
            if !snapshot {
                generation.touched.insert(key.to_vec(), number);
            }
            match op {
                skein_kv::Op::Put { key, value } => {
                    generation.rows.insert(key.to_vec(), value.to_vec());
                }
                skein_kv::Op::Erase { key } => {
                    generation.rows.remove(key.as_ref());
                }
            }
        }
        generation.number = generation.number.max(number);
        let old = behind(&generation.rows, binary);
        if self.active.is_none() {
            self.active = Some(name.clone());
        }
        if self.active.as_ref() == Some(&name) && old {
            self.deferred = Some(name);
        }
        self.is_active(path) && !old
    }

    pub(super) fn snapshot(
        &mut self,
        path: &[u8],
        limits: &codec::View,
        names: &[core::settings::ModelName],
        connector: &temper_protocol_store::Bounds,
    ) -> Vec<Commit> {
        let Some(bytes) = self.files.get(path) else { return Vec::new() };
        let Some((number, ops)) = snapshots::read(bytes) else { return Vec::new() };
        if self
            .generation(path)
            .is_some_and(|generation| generation.snapshot.is_some_and(|snapshot| snapshot >= number))
        {
            return Vec::new();
        }
        if !self.record(path, number, &ops, true) {
            return Vec::new();
        }
        if self.deferred.is_some() {
            return self.active_file(limits, names, connector);
        }
        if self.applied >= number {
            return Vec::new();
        }
        let commit = self.commit(number, ops, limits, names, connector);
        self.apply(&commit, limits);
        self.recovered.extend(1..=number);
        vec![commit]
    }

    pub(super) fn active_file(
        &mut self,
        limits: &codec::View,
        names: &[core::settings::ModelName],
        connector: &temper_protocol_store::Bounds,
    ) -> Vec<Commit> {
        let Some(bytes) = self.files.get(b"active".as_slice()) else { return Vec::new() };
        let Some(number) = std::str::from_utf8(bytes).ok().and_then(|text| text.strip_suffix('\n')) else {
            return Vec::new();
        };
        let Ok(number) = number.parse::<u64>() else { return Vec::new() };
        let active = format!("generations/{number}").into_bytes();
        if self.active.as_ref() == Some(&active) && self.deferred.as_ref() != Some(&active) {
            return Vec::new();
        }
        self.active = Some(active.clone());
        let Some(generation) = self.generations.get(&active) else { return Vec::new() };
        if behind(&generation.rows, self.binary()) {
            self.deferred = Some(active);
            return Vec::new();
        }
        let number = generation.number;
        let history = (self.observed_through > 0).then(|| {
            generation
                .commits
                .range((self.observed_through + 1)..)
                .map(|(number, ops)| (*number, ops.clone()))
                .collect::<Vec<_>>()
        });
        let ops = generation
            .rows
            .iter()
            .map(|(key, value)| skein_kv::Op::Put { key: key.as_slice().into(), value: value.as_slice().into() })
            .collect();
        self.rows.clear();
        self.lengths.clear();
        self.pending.clear();
        self.recovered.clear();
        self.agent_records.clear();
        self.installation = None;
        self.wires = None;
        self.applied = 0;
        self.restored = None;
        self.restoring = true;
        self.deferred = None;
        let commit = self.commit(number, ops, limits, names, connector);
        self.apply(&commit, limits);
        let mut committed = Vec::new();
        if let Some(history) = history {
            // This build already observed the source's prefix. After the switch,
            // report only later current-generation commits, in their captured order;
            // historical rows before that prefix remain opaque, not transcoded.
            let mut replay = Store {
                applied: self.observed_through,
                lengths: self.observed_lengths.clone(),
                files: self
                    .files
                    .iter()
                    .filter(|(path, _)| path.starts_with(b"transcripts/"))
                    .map(|(path, bytes)| (path.clone(), bytes.clone()))
                    .collect(),
                restoring: true,
                ..Store::default()
            };
            for (number, ops) in history {
                assert_eq!(number, replay.applied + 1, "current generation's captured commit order");
                let commit = replay.commit(number, ops, limits, names, connector);
                replay.apply(&commit, limits);
                committed.push(commit);
            }
            if number >= self.observed_through {
                assert_eq!(replay.applied, number, "whole current generation recovered");
            }
        } else {
            committed.push(commit);
        }
        self.recovered.extend(1..=number);
        self.restored = Some(self.rows.clone());
        self.restoring = false;
        committed
    }
}
