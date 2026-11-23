//! Per-run fault draws and ledger (temper/docs/design/testing/worker.md, 3.1, 3.2 and 4).

use skein_lib::{Rng, Time, Token};
use skein_world::faults::{FaultLedger, Injection, Struck};
use skein_world::streams::{Stream, Streams};
use temper_worker_domain_checkout::git::{Done, Fault, Kind};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT};

#[derive(Debug, Default)]
pub(crate) struct Faults {
    pub ledger: FaultLedger,
    pub capacity: Option<u64>,
    rng: Option<Rng>,
    next_cut: Option<Kind>,
    capacity_at: Option<u64>,
    disk_full: Option<Injection>,
}

impl Faults {
    pub fn seed(&mut self, streams: Streams, iteration: u64) {
        let mut rng = streams.rng(Stream::Faults, "worker");
        self.capacity_at = Some(iteration + rng.between(24, 64));
        self.rng = Some(rng);
    }

    pub fn tick(&mut self, iteration: u64, disk: &temper_fake_checkout::Checkout) {
        if self.capacity_at == Some(iteration) {
            let headroom = self.rng.as_mut().expect("a seeded capacity").between(1, 32);
            self.capacity = Some(crate::translate::used(disk) + headroom);
            self.capacity_at = None;
        }
    }

    pub fn cut_next(&mut self, kind: Kind) {
        assert!(matches!(kind, Kind::Fetch | Kind::Counts | Kind::CheckOut | Kind::Merge));
        self.next_cut = Some(kind);
    }

    pub fn draws_cut(&mut self, kind: Kind) -> bool {
        if !matches!(kind, Kind::Fetch | Kind::Counts | Kind::CheckOut | Kind::Merge) {
            return false;
        }
        if self.next_cut == Some(kind) {
            self.next_cut = None;
            return true;
        }
        self.rng.as_mut().is_some_and(|rng| rng.chance(500))
    }

    pub fn cut(&mut self, owner: Token, kind: Kind, now: Time) {
        let entry = self.ledger.inject(FETCH_CUT, now, struck(owner, kind));
        self.ledger.stop(entry, now).expect("a cut is an instant");
    }

    pub fn answered(&mut self, owner: Token, kind: Kind, now: Time, used_before: u64, done: &Done) {
        let refused = matches!(done, Done::FailedWithOutput { fault: Fault::Broken, diagnostic }
            if diagnostic.output() == b"No space left on device");
        if refused && self.disk_full.is_none() {
            self.disk_full = Some(self.ledger.inject(DISK_FULL, now, struck(owner, kind)));
        } else if !matches!(done, Done::Failed { .. } | Done::FailedWithOutput { .. })
            && self.capacity.is_some_and(|capacity| used_before < capacity)
            && let Some(entry) = self.disk_full.take()
        {
            self.ledger.stop(entry, now).expect("the first success after freeing ends the interval");
        }
    }
}

fn struck(owner: Token, kind: Kind) -> Struck {
    Struck { request: Some(format!("hold {} {kind:?}", owner.raw()).to_ascii_lowercase()), ..Struck::default() }
}
