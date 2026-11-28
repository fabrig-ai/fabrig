//! Byte and row fixtures for the root journal tests.
use skein_lib::{List, Wall};
use temper_engine_domain_limits::JournalLimits as Limits;
use temper_engine_domain_store::{Deployment, Record, TurnRecord, Write};

pub const LIMITS: Limits =
    Limits { commits: 2, held: 8, writes: 4, deliveries: 4, transcript_bytes: 64, result_bytes: 32, run_bytes: 1 };

#[must_use]
pub fn bytes(len: u32) -> Box<[u8]> {
    let mut bytes = List::with_capacity(len);
    for _ in 0..len {
        bytes.push(b'x').expect("one byte per reserved slot");
    }
    bytes.into_boxed()
}

#[must_use]
pub fn turn(number: u32, len: u32) -> Write {
    Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(TurnRecord {
        task: 1,
        attempt: 1,
        turn: number,
        spent: u64::from(number),
        read: Some(u64::from(number)),
        at: Wall::from_nanos(17),
        opens_window: number == 1,
        transcript: bytes(len),
    }))))
}

pub const DEPLOYMENT: Deployment = Deployment {
    projects: 0,
    restart_wait_ms: None,
    wait_floors: None,
    id: [19; 16],
    tasks: 0,
    people: 0,
    sign_ins: 0,
    messages: 0,
    runs: 0,
    calls: 0,
    connector_rows: 0,
    applications: 0,
    commits: 0,
};

/// Bind the decision driver to the root crate of the consuming test binary.
#[macro_export]
macro_rules! journal_fixture {
    ($root:path) => {
        pub mod step_journal_fixture {
            use ::skein_lib::{Queue, Token};
            use engine::{
                Counters, Decision, Delivery, Deployment, Family, Journal as SkeinJournal, JournalLimits as Limits,
                JournalOutput as Output, Write, accept_routed, committed as mark_committed, fresh as fresh_number,
                resume as release_ready,
            };
            use $crate::journal::{LIMITS, turn};
            use $root as engine;

            pub struct Journal {
                pub inner: SkeinJournal,
                pub counters: Counters,
                pub durable: u64,
            }

            impl Journal {
                pub fn bootstrap(id: [u8; 16], _limits: &Limits) -> Journal {
                    Journal { inner: SkeinJournal::uncapped(1, 0), counters: Counters::bootstrap(id), durable: 0 }
                }

                pub fn new(deployment: Deployment, _limits: &Limits) -> Journal {
                    Journal {
                        inner: SkeinJournal::uncapped(1, deployment.commits),
                        counters: Counters::new(deployment),
                        durable: deployment.commits,
                    }
                }

                pub fn deployment(&self) -> Deployment {
                    self.counters.deployment()
                }
                pub fn durable(&self) -> u64 {
                    self.durable
                }
                pub fn stopped(&self) -> bool {
                    self.inner.stopped()
                }
            }

            pub fn fresh(journal: &mut Journal, family: Family) -> Option<u64> {
                if journal.inner.stopped() { None } else { fresh_number(&mut journal.counters, family) }
            }
            #[expect(clippy::result_large_err, reason = "test wrapper preserves the decision's owned refusal")]
            pub fn accept(
                journal: &mut Journal,
                limits: &Limits,
                decision: Decision,
                out: &mut Queue<Output>,
            ) -> Result<(), Decision> {
                assert!(out.room() >= 1, "one root journal output reserved");
                accept_routed(&mut journal.inner, &mut journal.counters, limits, decision, true)?;
                if let Some(output) = engine::commit(&mut journal.inner, limits) {
                    out.push(output);
                }
                Ok(())
            }
            pub fn committed(journal: &mut Journal, number: u64) {
                mark_committed(&mut journal.inner, number);
                if !journal.inner.stopped() {
                    journal.durable = number;
                }
            }
            pub fn resume(journal: &mut Journal, out: &mut Queue<Output>) {
                release_ready(&mut journal.inner, out);
            }
            pub fn ack(number: u32) -> Delivery {
                let down =
                    jig_domain_host::LinkDown::TurnKept { run: Token::new(1), attempt: Token::new(1), turn: number };
                Delivery::Link { channel: Token::new(7), down }
            }

            pub fn decision(journal: &mut Journal, number: u32) -> Decision {
                let mut d = Decision::decide(&mut journal.inner, &LIMITS);
                d.write(&LIMITS, turn(number, 2)).unwrap();
                d.deliver(&LIMITS, ack(number)).unwrap();
                d
            }

            pub fn commit(out: &mut Queue<Output>) -> (u64, Box<[Write]>) {
                let Output::Commit { number, writes } = out.pop().unwrap() else {
                    panic!("commit");
                };
                (number, writes)
            }

            pub fn acknowledged(out: &mut Queue<Output>) -> u32 {
                let Output::Deliver(Delivery::Link { down: jig_domain_host::LinkDown::TurnKept { turn, .. }, .. }) =
                    out.pop().unwrap()
                else {
                    panic!("ack");
                };
                turn
            }
        }
    };
}
