//! The script supplies source moves; the referee reads durable task/forge facts and public lag.
use super::{BTreeSet, Commit, Key, Record, World, forge, proof, tasks};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Referee {
    pub moves: Vec<(Commit, bool)>,
    landed: BTreeMap<u64, Vec<Commit>>,
    order: Vec<Commit>,
    resolved: BTreeSet<Commit>,
}
impl Referee {
    pub fn kept(&self) -> usize {
        self.moves.iter().filter(|(_, kept)| *kept).count()
    }
    pub fn resolved(&self) -> usize {
        self.resolved.len()
    }
    pub fn check(&mut self, world: &mut World) {
        assert!(world.store.pending.is_empty(), "observe a durable settled step");
        let row = proof::follow(world).clone();
        let mut landed = BTreeMap::new();
        for record in world.store.rows.values() {
            if let Record::Forge { row } = record
                && let forge::Stored::Landed { task, .. } = row.as_ref()
            {
                let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge::Key::Change(*task))) else {
                    panic!("landed change retained");
                };
                let forge::Stored::Change(change) = row.as_ref() else {
                    panic!("change row");
                };
                if let Some(follows) = &change.follows {
                    landed.insert(*task, follows.commits.to_vec());
                }
            }
        }
        let newly_landed: Vec<_> = landed.iter().filter(|(task, _)| !self.landed.contains_key(task)).collect();
        assert!(newly_landed.len() <= 1, "one producer result per settled step makes landing order observable");
        for (_, commits) in newly_landed {
            self.order.extend(commits);
        }
        self.landed = landed;
        let source_positions: Vec<_> = self
            .order
            .iter()
            .map(|commit| {
                self.moves
                    .iter()
                    .position(|(after, kept)| after == commit && *kept)
                    .expect("only admitted source moves land")
            })
            .collect();
        assert!(
            source_positions.windows(2).all(|pair| pair[0] < pair[1]),
            "follow changes land in source order: {source_positions:?}"
        );

        let mut counts: BTreeMap<Commit, u32> = BTreeMap::new();
        self.resolved.clear();
        for commit in self.landed.values().flatten().copied().chain(row.skips.iter().map(|skip| skip.after)) {
            *counts.entry(commit).or_default() += 1;
            self.resolved.insert(commit);
        }
        for entry in &row.queue {
            *counts.entry(entry.after).or_default() += 1;
        }
        for (after, kept) in &self.moves {
            assert_eq!(
                counts.remove(after).unwrap_or(0),
                u32::from(*kept),
                "each source landing queued, followed or skipped once: {after:?}"
            );
        }
        assert!(counts.is_empty(), "no invented landing: {counts:?}");
        let expected_lag = self.kept() - self.resolved.len();
        assert_eq!(row.queue.len(), expected_lag, "lag is independently unresolved source landings");

        let mut owners: BTreeMap<Commit, u64> = BTreeMap::new();
        for record in world.store.rows.values() {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) = record {
                let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge::Key::Change(task.number)))
                else {
                    continue;
                };
                let forge::Stored::Change(change) = row.as_ref() else {
                    continue;
                };
                let Some(follows) = &change.follows else {
                    continue;
                };
                if let tasks::Phase::Held { why: tasks::Hold::Dependency(dependency), .. } = task.phase {
                    let ended =
                        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(dependency))));
                    assert!(
                        !matches!(ended, Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task))))
                        if matches!(task.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. }))),
                        "no follow change held for a cancelled dependency"
                    );
                }
                for dependency in &task.waiting_on {
                    let ended =
                        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(*dependency))));
                    assert!(
                        !matches!(ended, Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task))))
                        if matches!(task.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. }))),
                        "no cancelled dependency remains in the chain"
                    );
                }
                for after in &follows.commits {
                    assert!(owners.insert(*after, task.number).is_none(), "no landing has two live changes");
                }
            }
        }
        for entry in &row.queue {
            assert_eq!(
                owners.get(&entry.after),
                entry.change.as_ref(),
                "queued entry belongs to exactly one live change"
            );
        }
        proof::session(world);
        let document = world
            .oracle_document(world.signed_in.expect("owner session"), 2, forge::follows_subject(2))
            .expect("public lag read");
        let forge::Document::Follows(view) = document else {
            panic!("follows document");
        };
        let follow = view.follows.first().expect("accepted follow");
        assert_eq!(
            usize::try_from(follow.lag).expect("bounded lag"),
            expected_lag,
            "public lag equals unresolved source landings"
        );
        world.drain_cut();
    }
}
