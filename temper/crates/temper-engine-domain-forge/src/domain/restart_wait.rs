//! Restart's unfinished terms, without changing them (domain/forge/README.md, 9;
//! protocol/README.md, 6.3). Tokens in each pending table share a kind tag;
//! their low sequence bits, and hence table order, are their order of making.
use super::{Domain, RestartStage};
use alloc::boxed::Box;
use skein_forge_client::What;
use skein_lib::{Decimal, List, bytes};

impl Domain {
    /// Waits for one restart stage, by kind, counted and naming its three oldest.
    /// An empty answer means none of the named kinds is still owed.
    #[must_use]
    pub fn restart_waits(&self, stage: RestartStage) -> Box<[u8]> {
        let mut words = Box::default();
        match stage {
            RestartStage::Restored => {}
            RestartStage::ReadAfresh => {
                if self.restart_fresh.is_none() {
                    return words;
                }
                let fresh = self.client.fresh_owed();
                let mut names = List::with_capacity(3);
                for resource in fresh.iter().take(3) {
                    let what = match &resource.what {
                        What::Repository => b"listing".as_slice().into(),
                        What::Branch(branch) => bytes::concat(&[b"branch ", branch]),
                        What::Pull(n) => bytes::concat(&[b"pull ", Decimal::of(*n).as_bytes()]),
                        What::Issue(n) => bytes::concat(&[b"issue ", Decimal::of(*n).as_bytes()]),
                    };
                    names
                        .push(bytes::concat(&[&repository(self, resource.repository), b" ", &what]))
                        .expect("three oldest reads");
                }
                add(
                    &mut words,
                    u64::try_from(fresh.len()).expect("bounded reads"),
                    b"fresh read",
                    b"fresh reads",
                    &names,
                );
                let mut names = List::with_capacity(3);
                for (_, pending) in &self.landing.pending {
                    if names.len() == 3 {
                        break;
                    }
                    names.push(task(pending.task)).expect("three oldest gathers");
                }
                add(&mut words, u64::from(self.landing.pending.len()), b"landing gather", b"landing gathers", &names);
                let mut names = List::with_capacity(3);
                for (_, pending) in &self.pending_ci {
                    if names.len() == 3 {
                        break;
                    }
                    let head = crate::commit_text(&pending.head);
                    let head = head.get(..12).expect("commit has twelve hex digits");
                    names
                        .push(bytes::concat(&[&repository(self, pending.repository), b" at ", head]))
                        .expect("three oldest CI reads");
                }
                add(&mut words, u64::from(self.pending_ci.len()), b"CI read", b"CI reads", &names);
                let mut names = List::with_capacity(3);
                for (_, pending) in &self.goal_file_reads {
                    if names.len() == 3 {
                        break;
                    }
                    names.push(task(pending.task)).expect("three oldest goal-file reads");
                }
                add(&mut words, u64::from(self.goal_file_reads.len()), b"goal-file read", b"goal-file reads", &names);
                let mut names = List::with_capacity(3);
                for (_, pending) in &self.lost {
                    if names.len() == 3 {
                        break;
                    }
                    names.push(task(pending.task)).expect("three oldest lost-writer reads");
                }
                add(&mut words, u64::from(self.lost.len()), b"lost writer's read", b"lost writers' reads", &names);
                let mut names = List::with_capacity(3);
                for (_, pending) in &self.makings {
                    if names.len() == 3 {
                        break;
                    }
                    names.push(task(pending.task)).expect("three oldest goal branches");
                }
                add(&mut words, u64::from(self.makings.len()), b"goal branch", b"goal branches", &names);
            }
            RestartStage::Settled => {
                let unsettled = self.client.unsettled();
                let mut names = List::with_capacity(3);
                for (entry, owner) in unsettled.iter().take(3) {
                    names
                        .push(bytes::concat(&[b"entry ", Decimal::of(*entry).as_bytes(), b" of ", &task(*owner)]))
                        .expect("three oldest uncertain writes");
                }
                add(
                    &mut words,
                    u64::try_from(unsettled.len()).expect("bounded entries"),
                    b"uncertain write",
                    b"uncertain writes",
                    &names,
                );
            }
        }
        words
    }
}
#[cfg(test)]
mod tests;

fn task(number: u64) -> Box<[u8]> {
    bytes::concat(&[b"task ", Decimal::of(number).as_bytes()])
}
fn repository(d: &Domain, repository: skein_forge::api::Repository) -> Box<[u8]> {
    let row = d.repositories.get(&repository).expect("restart read's adopted repository");
    bytes::concat(&[&row.owner, b"/", &row.name])
}
fn add(words: &mut Box<[u8]>, count: u64, singular: &[u8], plural: &[u8], names: &List<Box<[u8]>>) {
    if count == 0 {
        return;
    }
    let kind = crate::wait_kind(count, singular, plural, names.as_slice());
    let separator: &[u8] = if words.is_empty() { b"" } else { b", " };
    *words = bytes::concat(&[words, separator, &kind]);
}
