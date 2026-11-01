//! An empty start keeps settings before seeding (temper/docs/design/domain/testing.md, 2.3).
use super::{Machine, Proc, STORE, Story, fixtures, referee, service, service_roots, stories};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};

struct Judge {
    controls: skein_world::Controls,
    stopping: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let Proc::Service(host) = &mut procs[0] else { panic!("initial service") };
        assert!(self.stopping || !host.service.is_empty(), "empty start refused: {:?}", host.service.end());
        if !self.stopping && host.service.links().pop().is_some() {
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.stopping = true;
        }
    }
    fn observe(&mut self, _now: Time, _procs: &[Proc]) {}
    fn passed(&self) -> bool {
        self.stopping
    }
}

pub(crate) fn run(disk: skein_fake_machine::Machine, story: Story) -> skein_fake_machine::Machine {
    stories::set(story);
    stories::setup_settings(super::Phase::Story);
    super::REFUSE_START.with(|refused| refused.set(false));
    let previous = STORE.with(|slot| slot.replace(Some(Rc::new(RefCell::new(referee::Store::new())))));
    let mut world = skein_world::World::new_controlled(
        fixtures::SEED,
        skein_sim::Config::calm(),
        skein_world::Memory::Unchecked,
        |controls| Judge { controls, stopping: false },
    )
    .with_machine(Machine(disk));
    world.spawn_restartable(service_roots(), service);
    let mut outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "empty start drains");
    STORE.with(|slot| *slot.borrow_mut() = previous);
    std::mem::replace(&mut outcome.machine.0, skein_fake_machine::Machine::for_user(1000))
}
