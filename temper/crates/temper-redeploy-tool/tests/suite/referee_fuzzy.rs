//! The pure referee's findings, generated with an independent failure ledger (redeploy.md, 6).
use temper_redeploy_tool::{
    fake::{WIRES, description},
    record::{Grace, Opening, Record, TaskBefore, Window},
    redeploy::Mode,
    referee::judgement::{FailedEnd, Readings, evaluate},
};
const BEGAN: u64 = 1_794_564_000_000_000_000;
struct Generator(u64);
impl Generator {
    fn below(&mut self, limit: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % limit
    }
}
fn task(number: u64, label: u64, held: bool, class: Option<&str>) -> TaskBefore {
    TaskBefore {
        task: number,
        project: number % 3 + 1,
        label,
        phase: if held { "held" } else { "running" }.into(),
        held: class.map(str::to_owned),
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "Keep the seeded scenario and its independent failure ledger together")]
fn generated_findings_fail_exactly_the_span_openings_window_holds_and_reads() {
    let mut coverage = [false; 7];
    let mut outcomes = [false; 2];
    for seed in 1..=1024 {
        let mut random = Generator(seed);
        let count = usize::try_from(random.below(4) + 1).unwrap();
        let d = description(count);
        let mut record = Record::new("abcdef012345", Mode::Engine);
        record.began = BEGAN;
        let mut read = Readings::default();
        let mut failures = 0;
        let mut expected = Vec::new();
        for worker in &d.workers {
            record.graces.push(Grace { worker: worker.name.clone(), grace_ms: 300_000 + random.below(1000) });
            match random.below(4) {
                0 => {
                    failures += 1;
                    expected.push(format!("worker {} did not open", worker.name));
                }
                state => {
                    let refused = state == 1;
                    if refused {
                        failures += 1;
                        expected.push(format!("worker {} refused: wires: no bridge", worker.name));
                    }
                    record.openings.push(Opening {
                        worker: worker.name.clone(),
                        at: BEGAN,
                        line: if refused {
                            format!(
                                "temper-engine serve: worker {} refused at its opening: wires: no bridge",
                                worker.name
                            )
                        } else {
                            format!("temper-engine serve: worker {} opened: {WIRES}", worker.name)
                        },
                        refused,
                    });
                }
            }
        }
        let grace = record.graces.iter().map(|grace| grace.grace_ms).min().unwrap() * 1_000_000;
        match random.below(4) {
            0 => (),
            side => {
                let duration = match side {
                    1 => grace - 1,
                    2 => grace,
                    _ => grace + 1,
                };
                record.window = Some(Window { from: BEGAN, to: BEGAN + duration, worker: "not-the-smallest".into() });
                if side != 1 {
                    failures += 1;
                }
            }
        }
        for number in 1..=random.below(8) + 1 {
            let label = if random.below(2) == 0 { 2 } else { 0 };
            let name = if label == 2 { "goal" } else { "task" };
            let presence = random.below(3);
            let held_before = random.below(2) == 0;
            if presence != 0 {
                record.before.tasks.push(task(number, label, held_before, None));
            }
            let classes = ["run", "lost", "infrastructure", "agent", "transient", "permanent", "invalid"];
            let class_index = usize::try_from(random.below(7)).unwrap();
            coverage[class_index] = true;
            let class = classes[class_index];
            let hold_kind = random.below(3);
            if presence != 1 {
                let class = if hold_kind == 1 { Some(class) } else { None };
                read.snapshot.tasks.push(task(number, label, hold_kind != 0, class));
                if hold_kind != 0 {
                    let reason = class.map_or_else(|| "dependency".to_owned(), |class| format!("failures {class}"));
                    read.holds.insert(number, reason.clone());
                    if presence == 0 || !held_before {
                        expected.push(format!("{name} {number} held: {reason}"));
                        if hold_kind == 1 && class_index != 0 {
                            failures += 1;
                        }
                    }
                }
            }
            let age = random.below(3);
            read.endings.push(FailedEnd {
                task: number,
                at: match age {
                    0 => BEGAN - 1,
                    1 => BEGAN,
                    _ => BEGAN + 1,
                },
                attempt: 3,
                class: class.into(),
            });
            if age != 0 {
                let own = if class_index == 0 { ", its own" } else { "" };
                expected.push(format!("{name} {number} attempt 3 failed: {class}{own}"));
                if class_index != 0 {
                    failures += 1;
                }
            }
        }
        for index in 0..random.below(3) {
            read.unread.push(format!("task activity {index}: offline"));
            failures += 1;
        }
        record.before.cut = random.below(2) == 0;
        read.snapshot.cut = random.below(2) == 0;
        let result = evaluate(&d, &record, &read);
        assert_eq!(result.failures, failures, "seed {seed}: {result:?}");
        assert_eq!(
            result.lines.last().unwrap(),
            &if failures == 0 { "pass".into() } else { format!("fail: {failures}") },
            "seed {seed}"
        );
        for line in expected {
            assert!(result.lines.contains(&line), "seed {seed}: missing {line}: {result:?}");
        }
        outcomes[usize::from(failures == 0)] = true;
    }
    assert!(coverage.into_iter().all(|seen| seen), "all seven failure classes");
    assert!(outcomes.into_iter().all(|seen| seen), "both verdicts");
}
