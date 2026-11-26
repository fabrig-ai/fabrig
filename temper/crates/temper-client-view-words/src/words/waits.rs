//! The words of a supplied wait (temper/docs/design/client/pages/waits.md, 2).
use super::{DAY, MINUTE, TimeForm, Times, Word, Words, shown};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{
    Decimal, List, Wall,
    bytes::{concat, copy_of},
};
use skein_ui::phrases::Hole;
use temper_client_domain::forge::words as resources;
use temper_client_protocol_wire::{FORGE, waits};

impl Words {
    /// Say this wait's object, age and own bound. Add its hearer only when requested.
    /// Names and deployment facts come from reads the caller already holds.
    #[must_use]
    pub fn wait(
        &self,
        wait: &c::Wait,
        times: &mut Times,
        reader: Option<u64>,
        members: &[c::ProjectMember],
        on_box: bool,
        include_hearer: bool,
    ) -> Box<[u8]> {
        let object = self.wait_object(&wait.object, times, reader, members, on_box);
        let age = times.say(self, TimeForm::Span, wait.since);
        let since = self.say(Word::WaitSince, &[Hole::Name(age)]);
        let bound = self.wait_bound(wait.bound, times);
        let dot = self.say(Word::ToolSeparator, &[]);
        if include_hearer && let Some(hearer) = wait.hears {
            let tail = if let Some(person) = reader
                && hearer == c::EscalationHolder::Person(person)
            {
                self.say(Word::WaitYouHear, &[])
            } else {
                let holder = self.wait_holder(hearer, reader, members);
                self.say(Word::WaitHearer, &[Hole::Name(holder)])
            };
            return concat(&[&object, &dot, &since, &dot, &bound, &dot, &tail]);
        }
        concat(&[&object, &dot, &since, &dot, &bound])
    }

    fn wait_object(
        &self,
        object: &c::WaitObject,
        times: &mut Times,
        reader: Option<u64>,
        members: &[c::ProjectMember],
        on_box: bool,
    ) -> Box<[u8]> {
        match object {
            c::WaitObject::Entry { kind, number, at } => {
                let entry = match kind {
                    c::WaitedFor::Question => b"question".as_slice(),
                    c::WaitedFor::Proposal => b"proposal",
                    c::WaitedFor::Escalation => b"escalation",
                    c::WaitedFor::PersonTask => b"person task",
                };
                let holder = self.wait_holder(*at, reader, members);
                self.say(Word::WaitEntry, &[Hole::Name(copy_of(entry)), Hole::Number(*number), Hole::Name(holder)])
            }
            c::WaitObject::Question { number, answerer } => {
                self.say(Word::WaitQuestionTo, &[Hole::Number(*number), Hole::Number(*answerer)])
            }
            c::WaitObject::Task(task) => self.say(Word::MsMakerTask, &[Hole::Number(*task)]),
            c::WaitObject::Resource { name, holder } => self.wait_resource(name, *holder),
            c::WaitObject::Run(run) => self.run_wait_object(*run, times, on_box),
            c::WaitObject::News { subscriptions, timer } => self.wait_news(subscriptions, *timer, times),
            c::WaitObject::Procedure { step, things } => self.wait_procedure(*step, things),
        }
    }

    fn wait_holder(&self, holder: c::EscalationHolder, reader: Option<u64>, members: &[c::ProjectMember]) -> Box<[u8]> {
        match holder {
            c::EscalationHolder::Person(person) => {
                if reader == Some(person) {
                    return self.say(Word::MsMakerYou, &[]);
                }
                for member in members {
                    if member.party == person && !member.name.bytes().is_empty() {
                        return shown(member.name.bytes(), self);
                    }
                }
                self.say(Word::MsMakerPerson, &[Hole::Number(person)])
            }
            c::EscalationHolder::Role { role, .. } => self.say(Word::WaitAtRole, &[Hole::Number(u64::from(role))]),
            c::EscalationHolder::Task(task) => self.say(Word::MsMakerTask, &[Hole::Number(task)]),
        }
    }

    fn wait_resource(&self, name: &c::ResourceName, holder: Option<u64>) -> Box<[u8]> {
        let path = name.path.as_slice();
        let most = self.path_bytes();
        let name = if name.connector == FORGE {
            let (word, holes) = resources::path(path, most);
            self.book().say(self.language, word.id(self.connector), &holes).expect("checked resource phrase").text
        } else {
            resources::joined(path, most)
        };
        let object = self.say(Word::WaitOnResource, &[Hole::Name(shown(&name, self))]);
        match holder {
            Some(task) => concat(&[&object, &self.say(Word::WaitResourceHeldBy, &[Hole::Number(task)])]),
            None => object,
        }
    }

    fn wait_news(&self, subscriptions: &List<u64>, armed_at: Option<Wall>, times: &mut Times) -> Box<[u8]> {
        let mut subs = Box::default();
        if !subscriptions.is_empty() {
            subs = self.say(Word::WaitNewsSubscriptions, &[]);
            let mut separator = b"".as_slice();
            for subscription in subscriptions {
                subs = concat(&[&subs, separator, Decimal::of(*subscription).as_bytes()]);
                separator = b", ";
            }
        }
        if let Some(at) = armed_at {
            let at = times.say(self, TimeForm::Clock, at);
            let alarm = self.say(Word::WaitNewsTimer, &[Hole::Name(at)]);
            return if subs.is_empty() { alarm } else { concat(&[&subs, b" and ", &alarm]) };
        }
        if subs.is_empty() { self.say(Word::WaitNewsUnarmed, &[]) } else { subs }
    }

    fn wait_procedure(&self, step: u32, things: &List<c::Thing>) -> Box<[u8]> {
        let mut text = match waits::step_name(step) {
            Some(name) => copy_of(name),
            None => concat(&[b"step ", Decimal::of(u64::from(step)).as_bytes()]),
        };
        let mut separator = b": ".as_slice();
        for thing in things {
            let name = match waits::thing_name(thing.what) {
                Some(name) => copy_of(name),
                None => concat(&[b"thing ", Decimal::of(u64::from(thing.what)).as_bytes()]),
            };
            let join = if thing.what == waits::THING_PULL { b" #".as_slice() } else { b" " };
            let number = Decimal::of(thing.number);
            let owner = match thing.owner {
                Some(owner) => self.say(Word::WaitThingOwner, &[Hole::Number(owner)]),
                None => Box::default(),
            };
            text = concat(&[&text, separator, &name, join, number.as_bytes(), &owner]);
            separator = b", ";
        }
        text
    }

    fn wait_bound(&self, bound: c::waits::Bound, times: &mut Times) -> Box<[u8]> {
        match bound {
            c::waits::Bound::None(reason) => match reason {
                c::Unbounded::NoDeadline => self.say(Word::WaitNoDeadline, &[]),
                c::Unbounded::Standing => self.say(Word::WaitStanding, &[]),
                c::Unbounded::Behind(task) => self.say(Word::WaitBehindTask, &[Hole::Number(task)]),
                c::Unbounded::OperatorHold => self.say(Word::WaitOperatorHold, &[]),
            },
            c::waits::Bound::At { at, then } if at > times.now => {
                let word = match then {
                    c::AtBound::PassUp => Word::WaitPassesUpIn,
                    c::AtBound::Hold => Word::WaitHoldsIn,
                    c::AtBound::Overdue => Word::WaitOverdueIn,
                    c::AtBound::GiveUp => Word::WaitGivesUpIn,
                };
                self.say(word, &[Hole::Name(self.time_span(remaining_minutes(times, at)))])
            }
            c::waits::Bound::At { at, then: c::AtBound::Overdue } => {
                let span = times.say(self, TimeForm::Span, at);
                self.say(Word::WaitOverdueFor, &[Hole::Name(span)])
            }
            c::waits::Bound::At { then: c::AtBound::PassUp | c::AtBound::Hold | c::AtBound::GiveUp, .. } => {
                self.say(Word::WaitDueNow, &[])
            }
        }
    }

    fn run_wait_object(&self, run: c::TaskRunWait, times: &mut Times, on_box: bool) -> Box<[u8]> {
        match run {
            c::TaskRunWait::Credential => self.say(Word::WaitCredential, &[]),
            c::TaskRunWait::Host => self.say(if on_box { Word::WaitCapacityBox } else { Word::WaitCapacity }, &[]),
            c::TaskRunWait::Worker { worker, until } => {
                let worker = self.say(Word::WaitWorker, &[Hole::Number(u64::from(worker))]);
                let until = times.say(self, TimeForm::Clock, until);
                self.say(Word::WaitReturningWorker, &[Hole::Name(worker), Hole::Name(until)])
            }
            c::TaskRunWait::Refused(why) => self.run_wait_refused(times, why),
            c::TaskRunWait::Account { source, why, .. } => self.account_wait_object(source, why),
        }
    }

    fn account_wait_object(&self, source: Option<u32>, why: c::TaskWaitWhy) -> Box<[u8]> {
        use c::TaskWaitWhy as R;
        match why {
            R::Refreshing => self.say(Word::WaitRefreshing, &[]),
            R::Host => self.say(Word::WaitHost, &[Hole::Name(self.say(Word::AccountRegion, &[]))]),
            R::Empty => self.say(Word::WaitEmpty, &[]),
            R::Invalid(_) => self.say(Word::WaitInvalid, &[]),
            R::Untooled { worker } => {
                let source = match source {
                    Some(number) => self.say(Word::WaitSource, &[Hole::Number(u64::from(number))]),
                    None => self.say(Word::AccountRegion, &[]),
                };
                self.say(
                    Word::WaitUntooled,
                    &[
                        Hole::Name(self.say(Word::WaitProgram, &[])),
                        Hole::Name(source),
                        Hole::Name(self.say(Word::WaitWorker, &[Hole::Number(u64::from(worker))])),
                    ],
                )
            }
            R::Source(state) => self.run_source_wait(state),
        }
    }
}

// Use the same whole minutes/days as spans, but in reverse: at an exact unit
// boundary the displayed count drops on the next nanosecond, not before it.
fn remaining_minutes(times: &mut Times, at: Wall) -> u64 {
    let left = at.as_nanos().saturating_sub(times.now.as_nanos());
    let unit = if left >= DAY { DAY } else { MINUTE };
    let tick_in = left.checked_rem(unit).expect("span unit is nonzero").saturating_add(1).min(left);
    let next = Wall::from_nanos(times.now.as_nanos().saturating_add(tick_in));
    times.next = Some(times.next.unwrap_or(next).min(next));
    left.checked_div(MINUTE).expect("minute is nonzero")
}
