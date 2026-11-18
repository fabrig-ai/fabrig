//! Production restore adoption on the world's disk, with the same per-kernel-call cuts as opening.
use super::{Change, World, answers, change, root};
use jig_protocol_store::{
    CheckpointName, Event, Store,
    host::{Adoption, Drivers, Lane, Submission},
};
use skein_io::kernel::{Complete, Error, Op};
use skein_lib::{Queue, Token};
use skein_sim::{Answer, Ask};
impl World {
    /// Run a stopped adoption; callers stop first and reopen through the ordinary world's path afterwards.
    pub fn stopped_adoption(&mut self, mode: Adoption, fail_at: Option<u32>) -> Vec<Event> {
        let mut drivers = Drivers::new(&self.env.limits, self.starts);
        self.store = Store::on_directory(&self.env.limits, root::APPLICATION, &[], drivers.seeded());
        self.store.owner_sets(self.sets);
        drivers.adopt_mode(self.root.fd, 0, Token::new(987), self.drawn, mode).expect("stopped adoption");
        self.operations = 0;
        self.changes.clear();
        self.written.clear();
        let mut above = Queue::uncapped();
        for _ in 0..100_000 {
            drivers.iterate(&mut self.store, &self.env, &mut above);
            let mut submitted = Queue::uncapped();
            while let Some(Submission { lane, mut submit }) = drivers.pop_submission() {
                let tag = u8::try_from(lane.index() + 1).expect("lane tag");
                submit.op = Token::tagged(tag, submit.op.raw()).expect("local op");
                if let Op::Cancel { target } = &mut submit.kind {
                    *target = Token::tagged(tag, target.raw()).expect("local cancel");
                }
                submitted.push(submit);
            }
            self.sim.submit(self.pid, &mut submitted);
            let mut calls = Queue::uncapped();
            let mut results = Queue::uncapped();
            self.sim.calls(&mut calls);
            while let Some(call) = calls.pop() {
                if self.cut == Some(self.operations) {
                    self.halted = true;
                    return answers(&mut above);
                }
                if let Some(change) = change(&call.ask) {
                    self.changes.push((self.operations, change));
                }
                if let Ask::Write { bytes, .. } | Ask::Append { bytes, .. } = &call.ask {
                    self.written.push((self.operations, bytes.clone()));
                }
                let failed = fail_at == Some(self.operations);
                self.operations += 1;
                if failed {
                    results.push(Answer { ticket: call.ticket, result: Err(Error::NoSpace) });
                } else if let Some(error) = self.link_error.filter(|_| matches!(call.ask, Ask::HardLink { .. })) {
                    results.push(Answer { ticket: call.ticket, result: Err(error) });
                } else {
                    skein_fake_machine::step(&mut self.machine, call, &mut results);
                }
            }
            self.sim.answer(&mut results);
            let mut completed = Queue::<Complete>::uncapped();
            self.sim.reap(self.pid, &mut completed);
            while let Some(mut complete) = completed.pop() {
                let index = usize::from(complete.op.tag()) - 1;
                complete.op = Token::new(complete.op.untagged());
                if let Op::Cancel { target } = &mut complete.kind {
                    *target = Token::new(target.untagged());
                }
                drivers.complete(*Lane::ALL.get(index).expect("lane"), complete);
            }
            if !above.is_empty() {
                let result = answers(&mut above);
                assert_eq!(result.len(), 1, "one adoption terminal");
                return result;
            }
        }
        panic!("stopped adoption did not settle");
    }
    pub fn restore_checkpoint(&mut self, name: CheckpointName, draw: [u8; 16]) -> Vec<Event> {
        self.stopped_adoption(Adoption::Restore { name, draw }, None)
    }
    /// An independent reader for source/deployment lock refusals.
    pub fn hold_path(&mut self, path: &[u8]) -> skein_fake_machine::Opened {
        let root = self.machine.reopen_root(0);
        let file = self.machine.open(root, path, skein_fake_machine::How::DirectoryNoFollow).expect("held directory");
        self.machine.close(root);
        self.machine.lock(file).expect("independent lock");
        file
    }
    pub fn release_path(&mut self, file: skein_fake_machine::Opened) {
        self.machine.close(file);
    }
    /// Cut points, including the rename inside the intent's durable replace.
    #[must_use]
    pub fn restore_cuts(&self) -> Vec<u32> {
        self.changes
            .iter()
            .filter_map(|(at, kind)| {
                matches!(kind, Change::Write | Change::Sync | Change::Link | Change::Rename).then_some(*at)
            })
            .collect()
    }
}
