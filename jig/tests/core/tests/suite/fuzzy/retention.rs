//! A separate horizon referee follows seeded ends, reads, forgets, clock moves and deletion.
use crate::{read_support::request, retention as f};
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Rng, Token, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    Turns,
    Record,
    Summary,
    Forgotten,
    Gone,
}
struct Model {
    record: tasks::TaskRecord,
    since: u64,
    next: Next,
    turns: bool,
    read_turns: bool,
    own: Vec<root::Key>,
}
impl Model {
    fn new(w: &mut World) -> Self {
        let record = f::finish(w);
        Self {
            since: record.ended_at.expect("ended").as_nanos(),
            next: Next::Turns,
            turns: true,
            read_turns: false,
            own: f::own(w, record.number),
            record,
        }
    }
    fn read(&mut self, seen: u64) {
        if self.full()
            && self.read_turns
            && seen >= self.record.result_position
            && self.record.flags.retention == tasks::Retention::ForgottenOnceRead
        {
            self.next = Next::Forgotten;
        }
    }
    fn pass(&mut self, now: u64, seen: u64) {
        if self.next == Next::Turns && now.saturating_sub(self.since) >= 14 * f::DAY {
            self.turns = false;
            self.next = Next::Record;
        }
        if self.next == Next::Record && now.saturating_sub(self.since) >= 30 * f::DAY {
            if !self.read_turns || seen < self.record.result_position {
                self.since = now;
            } else {
                self.turns = false;
                self.next = if self.record.flags.retention == tasks::Retention::ForgottenOnceRead {
                    Next::Gone
                } else {
                    Next::Summary
                };
            }
        }
        if self.next == Next::Summary && now.saturating_sub(self.since) >= 365 * f::DAY {
            self.next = Next::Gone;
        }
        if self.next == Next::Forgotten {
            self.next = Next::Gone;
            self.turns = false;
        }
    }
    fn full(&self) -> bool {
        matches!(self.next, Next::Turns | Next::Record)
    }
    fn check(&self, w: &mut World, seed: u64, step: u64) {
        let task = self.record.number;
        let context = (seed, step, task, self.next);
        assert_eq!(f::has(w, task), self.full(), "record horizon {context:?}");
        assert_eq!(f::summary(w, task), self.next != Next::Gone, "summary horizon {context:?}");
        assert_eq!(f::places(w, task), self.turns, "turns horizon {context:?}");
        assert_eq!(f::indexes(w, task) > 0, self.full() || self.next == Next::Summary, "indexes horizon {context:?}");
        let own = self.full() || self.next == Next::Forgotten;
        for key in &self.own {
            let retained = if matches!(key, root::Key::Core(core::Key::People(people::Key::EndedRead { .. }))) {
                self.full()
            } else {
                own
            };
            assert_eq!(w.store.rows.contains_key(key), retained, "own row horizon {context:?}: {key:?}");
        }
        if !own {
            assert_eq!(f::own(w, task), [], "no own row outlives its stage {context:?}");
        }
        for row in w.store.rows.values() {
            if let root::Record::Core(core::Record::Tasks(tasks::Stored::Update(update))) = row
                && update.task == task
            {
                let allowed = match update.object {
                    tasks::UpdateObject::Result => self.full(),
                    tasks::UpdateObject::Reply { .. } => own,
                    tasks::UpdateObject::Proposal { .. } => true,
                };
                assert!(allowed, "update outlives its stage {context:?}");
            }
        }
    }
}

#[test]
fn seeded_retention_never_erases_early_or_leaves_rows_past_their_last_stage() {
    for seed in 4100..4112 {
        let mut rng = Rng::new(seed);
        let mut w = f::world(seed, rng.below(2) == 0);
        let mut models = vec![Model::new(&mut w)];
        let mut seen = 0;
        let mut removed = false;
        let mut wall = w.clocks().1.as_nanos();
        for step in 0..20 {
            if step == 3 {
                let chat = jig_core_world::fixtures::chat(&w, 1, b"another end".as_slice().into());
                request(&mut w, 72, chat);
                models.push(Model::new(&mut w));
                wall = w.clocks().1.as_nanos();
            }
            match rng.below(4) {
                0 if !removed => {
                    let index =
                        usize::try_from(rng.below(u64::try_from(models.len()).expect("model count"))).expect("index");
                    let model = &mut models[index];
                    if model.full() {
                        f::read(&mut w, &model.record);
                        model.read_turns = true;
                        seen = seen.max(model.record.result_position);
                        for model in &mut models {
                            model.read(seen);
                        }
                    }
                }
                1 => {
                    wall += rng.below(40) * f::DAY;
                    w.set_clock(w.clocks().0, Wall::from_nanos(wall));
                }
                _ => {
                    f::pass(&mut w);
                    for model in &mut models {
                        model.pass(wall, seen);
                    }
                }
            }
            if step == 15 {
                wall += 366 * f::DAY;
                w.set_clock(w.clocks().0, Wall::from_nanos(wall));
                f::pass(&mut w);
                for model in &mut models {
                    model.pass(wall, seen);
                }
            }
            if step == 18 {
                w.send_all(vec![
                    root::Event::Core(core::Event::Act {
                        reply_to: ReplyTo::new(Token::new(74)),
                        connector: 1,
                        key: [74; 16],
                        act: Box::new(people::Act::RemoveProject { project: 1, grace: None }),
                        carried: None,
                    }),
                    root::Event::Timer(core::Timer::Retention),
                ]);
                assert!(matches!(w.act_answers.last(), Some((_, people::Outcome::ProjectRemoved { project: 1 }))));
                removed = true;
                for model in &mut models {
                    model.next = Next::Gone;
                    model.turns = false;
                }
            }
            for model in &models {
                model.check(&mut w, seed, step);
            }
        }
    }
}
