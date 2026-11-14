//! The outside actors, the drain boundary and the kill boundary.
use super::{CoreRecord, HarnessProcess, Proc, Record, STATE, Story, continued, ctl, golden, referee, stories};
use skein_io::kernel::ServiceSignal;
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
#[expect(clippy::struct_excessive_bools, reason = "independent stop, fault, client and upgrade observations")]
pub(super) struct Judge {
    finding: Option<String>,
    controls: skein_world::Controls,
    store: Rc<RefCell<referee::Store>>,
    forge: referee::forge::Forge,
    transport: temper_fake_git_hosted::Typed,
    upgrade: bool,
    stopped: bool,
    released: bool,
    client_started: bool,
    ledger: Rc<RefCell<Vec<skein_fake_forge_domain::Observation>>>,
    finished: bool,
    epoch: u32,
    ticks: u32,
}
impl Judge {
    pub(super) fn new(
        controls: skein_world::Controls,
        store: Rc<RefCell<referee::Store>>,
        transport: temper_fake_git_hosted::Typed,
        upgrade: bool,
        ledger: Rc<RefCell<Vec<skein_fake_forge_domain::Observation>>>,
    ) -> Self {
        Self {
            finding: None,
            controls,
            store,
            forge: referee::forge::Forge::new(transport.clone(), Story::PlaygroundA).keeping_ledger(),
            transport,
            upgrade,
            stopped: false,
            released: false,
            client_started: false,
            ledger,
            finished: false,
            epoch: 0,
            ticks: 0,
        }
    }

    fn after_kill(&mut self, first: &Proc, others: &mut [Proc]) -> Result<(), String> {
        if !(STATE.with(|s| s.borrow().killed)) {
            return Err(String::from("only a killed service is replaced by a closer"));
        }
        for proc in others.iter_mut() {
            if let Proc::Other(HarnessProcess::Client(person)) = proc {
                person.finish();
            }
            if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                caller.closed = true;
            }
            if let Proc::Other(HarnessProcess::Ctl(caller)) = proc {
                caller.drain();
            }
        }
        if !STATE.with(|s| s.borrow().kill_told) {
            for (index, proc) in others.iter().enumerate() {
                if matches!(proc, Proc::Agent(_)) && !proc.is_empty() {
                    self.controls.signal(index + 1, ServiceSignal::Terminate);
                    self.controls.signal(index + 1, ServiceSignal::Terminate);
                }
            }
            STATE.with(|s| s.borrow_mut().kill_told = true);
        }
        self.stopped = true;
        self.stop_peers(others)?;
        self.finished = first.is_empty() && others.iter().all(Host::is_empty);

        Ok(())
    }
    fn arm_write(&mut self, others: &mut [Proc]) {
        if !self.upgrade
            && !STATE.with(|s| s.borrow().armed)
            && golden::in_flight(&self.store.borrow()).is_some()
            && self.forge.forge_complete()
        {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                if !s.armed {
                    let store = self.store.borrow();
                    let task = golden::in_flight(&store).expect("mid-run task");
                    s.task = task.number;
                    s.attempt = task.attempt;
                    for proc in others.iter_mut() {
                        if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                            caller.commands.push_back(ctl::Command::new(&[
                                b"task",
                                b"message",
                                s.task.to_string().as_bytes(),
                                b"remember tail",
                                b"--key",
                                b"remember-tail",
                                b"--json",
                            ]));
                        }
                    }
                    s.armed = true;
                }
            });
        }
        if !self.upgrade && STATE.with(|s| s.borrow().armed && !s.borrow().faulted) && self.store.borrow().rows.values().any(|r| matches!(r,temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Entry {entry,..} if entry.attempt.is_some() && STATE.with(|s| entry.task == s.borrow().task)))) {
            self.transport.forge.borrow_mut().config.timeouts = 1000;
            STATE.with(|s| s.borrow_mut().faulted = true);
        }
    }
    fn start_client(&mut self, others: &mut [Proc]) {
        for proc in others.iter_mut() {
            if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                let task = STATE.with(|s| s.borrow().task.to_string());
                caller.commands.push_back(ctl::Command::new(&[
                    b"task",
                    b"message",
                    task.as_bytes(),
                    b"remember tail",
                    b"--key",
                    b"remember-tail",
                    b"--json",
                ]));
                caller.commands.push_back(ctl::Command::new(&[
                    b"task",
                    b"message",
                    b"1",
                    b"Resume paused chat",
                    b"--key",
                    b"resume-paused",
                    b"--json",
                ]));
                caller.commands.push_back(ctl::Command::new(&[
                    b"chat",
                    b"start",
                    b"After upgrade",
                    b"--project",
                    b"1",
                    b"--key",
                    b"after-upgrade",
                    b"--json",
                ]));
                caller.closed = true;
            }
        }
        self.client_started = true;
    }
    fn stop_peers(&mut self, procs: &mut [Proc]) -> Result<(), String> {
        let ended = procs.iter().all(|p| !matches!(p, Proc::Agent(a) if !a.is_empty()));
        for p in procs {
            match p {
                Proc::Peer(peer) => {
                    if ended && !self.released {
                        peer.release(stories::STOP_HOLD).map_err(|why| format!("mid-run hold released: {why:?}"))?;
                        self.released = true;
                    }
                    peer.shutdown();
                }
                Proc::Forge(peer) => peer.shutdown(),
                _ => {}
            }
        }

        Ok(())
    }
}
impl Judge {
    fn act_checked(&mut self, _: Time, procs: &mut [Proc]) -> Result<(), String> {
        self.ticks += 1;
        let epoch = STATE.with(|s| s.borrow().epoch);
        if epoch != self.epoch {
            self.epoch = epoch;
            self.client_started = false;
            self.stopped = false;
            self.finished = false;
            self.released = false;
        }
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else {
            self.after_kill(&first[0], others)?;
            return Ok(());
        };
        if self.ticks >= 100_000 {
            return Err(format!("upgrade bounded turns: {:?}", service.service.wait_state()));
        }
        if !self.upgrade {
            for proc in others.iter() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc
                    && let Some(answer) = caller.answers.first()
                {
                    STATE.with(|s| s.borrow_mut().kept_answer.clone_from(&answer.output));
                }
            }
        }
        if !self.upgrade
            && let Some(link) = service.service.links().pop()
        {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.sign_in(&link);
                }
            }
        }
        self.arm_write(others);
        let checked = STATE.with(|s| s.borrow().checked);
        if self.upgrade && checked && !self.client_started {
            self.start_client(others);
        }
        let complete = if self.upgrade {
            checked && self.store.borrow().agent_records.iter().any(|r| matches!(r, referee::AgentRecord::Start {task, attempt, ..} if STATE.with(|s| *task == s.borrow().task && *attempt > s.borrow().attempt)))
                && others.iter().any(|p| matches!(p, Proc::Other(HarnessProcess::Calls(c)) if c.answers.len() == 3))
                && self.store.borrow().rows.values().any(|r| matches!(r, temper_engine_domain::Record::Core(Record::Core(CoreRecord::Turn(turn))) if turn.task == 1 && skein_lib::bytes::find(&turn.transcript,b"Paused chat resumed").is_some()))
                && !self.store.borrow().rows.values().any(|r| matches!(r, temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Flight {task,..} if STATE.with(|s| *task == s.borrow().task))))
        } else {
            STATE.with(|s| s.borrow().issue)
                && others.iter().any(|p| matches!(p, Proc::Other(HarnessProcess::Calls(c)) if c.answers.len() == 1))
        };
        if complete && !self.stopped {
            for proc in others.iter() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    for answer in &caller.answers {
                        match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
                            (left, right) if left != right => {
                                let finding = format!("ctl: {}", String::from_utf8_lossy(&answer.error));
                                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                            }
                            _ => {}
                        }
                    }
                    if self.upgrade {
                        STATE.with(|s| {
                            match (&(caller.answers[0].output), &(s.borrow().kept_answer)) {
                                (left, right) if left != right => {
                                    let finding = String::from("replayed key is answered from its record");
                                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                                }
                                _ => {}
                            }

                            Ok(())
                        })?;
                    } else {
                        STATE.with(|s| s.borrow_mut().kept_answer.clone_from(&caller.answers[0].output));
                    }
                }
            }
            if self.upgrade {
                continued(&self.store.borrow())?;
            }

            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.finish();
                }
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    caller.closed = true;
                }
            }
            if !self.upgrade {
                if !(self.store.borrow().rows.values().any(|r| matches!(r,temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Flight {task,..} if STATE.with(|s| *task == s.borrow().task))))) { return Err(String::from("write still in flight at the stop boundary")); }
                STATE.with(|s| s.borrow_mut().write_at = Some(service.observer.inner.submitted));
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped {
            self.stop_peers(others)?;
            self.finished = service.is_empty() && others.iter().all(Host::is_empty);
        }

        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.inner.finding
            {
                return Err(why.clone());
            }
        }

        // Keep the fake's write ledger outside the service and peer lifetimes.
        // Forge's branch referee consumes the same observations after this copy.
        if self.stopped {
            self.finished = procs.iter().all(Host::is_empty);
        }
        self.forge.observe(procs)?;
        self.ledger.borrow_mut().extend(self.forge.take_ledger());
        let issue = self.ledger.borrow().iter().any(|o| matches!(o, skein_fake_forge_domain::Observation::Opened {title, ..} if title.as_ref() == b"Upgrade once"));
        if issue {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                if !s.issue {
                    s.issue = true;
                    let Proc::Service(service) = &procs[0] else { return Err(String::from("service")) };
                    s.write_at = Some(service.observer.inner.submitted);
                    self.transport.forge.borrow_mut().config.timeouts = 0;
                }

                Ok(())
            })?;
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(60_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.finished && now >= Time::from_nanos(60_000_000_000)).then(|| {
            format!(
                "upgrade stalled: checked {}, issue {}, tasks {:?}",
                STATE.with(|s| s.borrow().checked),
                STATE.with(|s| s.borrow().issue),
                self.store
                    .borrow()
                    .rows
                    .values()
                    .filter_map(|row| {
                        if let temper_engine_domain::Record::Core(Record::Tasks(jig_domain_tasks::Stored::Live(task))) =
                            row
                        {
                            Some((task.number, task.attempt, task.turn, &task.phase))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.finished)
    }
}
