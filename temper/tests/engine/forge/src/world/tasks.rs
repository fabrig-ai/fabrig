//! Tasks explicitly handed over by the scripted parent, never reconstructed from forge rows.
use super::{Env, Queue, ReplyTo, Time, Token, Wall, World, top};
use jig_domain_tasks as tasks;

const LIMITS: tasks::Limits = jig_tasks_kit::LIMITS;

pub(super) fn domain(seed: u64) -> tasks::Domain {
    let mut domain = jig_tasks_kit::domain(&LIMITS, seed);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let mut out = Queue::uncapped();
    for event in [
        tasks::Event::Check,
        tasks::Event::Restored,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(0)), project: 1, period: 0, budget: 100_000 },
    ] {
        tasks::step(&mut domain, &env, event, &mut out);
        out.clear();
    }
    domain
}

pub(super) fn hold(domain: &mut tasks::Domain, now: Time, wall: Wall, number: u64) {
    if domain.task(number).is_some() {
        return;
    }
    let env = Env { now, wall, limits: LIMITS };
    let mut out = Queue::uncapped();
    tasks::step(
        domain,
        &env,
        tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(number)),
            creator: tasks::Party::Person(jig_tasks_kit::PERSON),
            batch: Box::new([jig_tasks_kit::task(number, &[])]),
        },
        &mut out,
    );
    assert!(domain.task(number).is_some(), "scripted task {number} admitted: {out:?}");
}

impl World {
    /// The scripted core holds a task before giving its work to the connector.
    pub fn hold_task(&mut self, number: u64) {
        hold(&mut self.tasks, self.env.now, self.env.wall, number);
    }

    fn forget_task(&mut self, number: u64) {
        let held = self.tasks.view_tasks();
        self.tasks = domain(self.seed);
        for task in held {
            if task.number != number {
                self.hold_task(task.number);
            }
        }
    }

    pub(super) fn handed_tasks(&mut self, event: &top::Event) {
        match event {
            top::Event::Unnamed { task } | top::Event::DropErased { task } => self.forget_task(*task),
            top::Event::Change { row } => {
                self.hold_task(row.task);
                if let Some(goal) = row.goal {
                    self.hold_task(goal);
                }
            }
            top::Event::Landing { row } => self.hold_task(row.landing.task),
            top::Event::Project { view, .. } => self.hold_task(view.goal),
            top::Event::ProjectDesired { goal, .. } | top::Event::FailProjection { goal, .. } => self.hold_task(*goal),
            top::Event::Subscribe { subscription } => self.hold_task(subscription.task),
            top::Event::Enqueue { entry } | top::Event::EnqueueCall { entry, .. } => self.hold_task(entry.task),
            top::Event::Claim { task, holders, .. } => {
                self.hold_task(*task);
                for holder in holders {
                    self.hold_task(*holder);
                }
            }
            top::Event::Delegated { task, child, .. } => {
                self.hold_task(*task);
                self.hold_task(*child);
            }
            top::Event::SettleEffects { task, root, .. } | top::Event::Release { task, root, .. } => {
                self.hold_task(*task);
                self.hold_task(*root);
            }
            top::Event::Hold { task, .. }
            | top::Event::Names { task, .. }
            | top::Event::TierTask { task, .. }
            | top::Event::StepTier { task, .. }
            | top::Event::StepChange { task, .. }
            | top::Event::StepLanding { task, .. } => self.hold_task(*task),
            // Restore and replies do not declare a task: a corrupt row must not create its own target.
            _ => {}
        }
    }
}
