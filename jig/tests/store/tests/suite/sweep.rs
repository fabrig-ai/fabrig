//! Held-prefix file-operation sweeps (domain/testing.md, 4 and 9).
use jig_store_world::World;
use skein_fork::{Exit, Split, passed, split};
use std::{
    cell::RefCell,
    io::{Read, Write},
    os::unix::net::UnixStream,
    rc::Rc,
};

/// The byte is a checked outcome, not shared state from the child's world.
pub(crate) fn fork_case(world: &mut World, cut: u32, check: impl FnOnce(&mut World) -> bool) -> bool {
    let (mut parent, mut child) = UnixStream::pair().expect("sweep outcome channel");
    match split() {
        Split::Child => {
            let outcome = check(world);
            child.write_all(&[u8::from(outcome)]).expect("send cut outcome");
            passed();
            unreachable!("a sweep child exits");
        }
        Split::Parent(exit) => {
            assert_eq!(exit, Exit::Code(0), "cut {cut} failed");
            let mut outcome = [0];
            parent.read_exact(&mut outcome).expect("child's checked outcome");
            assert!(outcome[0] <= 1);
            outcome[0] == 1
        }
    }
}

pub(crate) struct Sweep {
    expected: Vec<u32>,
    passed: Rc<RefCell<Vec<(u32, bool)>>>,
}

impl Sweep {
    pub(crate) fn start(
        world: &mut World,
        expected: Vec<u32>,
        mut check: impl FnMut(&mut World, u32) -> bool + 'static,
    ) -> Self {
        Self::start_with(world, expected, move |world, cut| {
            fork_case(world, cut, |world| {
                world.cut_at(cut);
                check(world, cut)
            })
        })
    }

    /// A cut with two recovery paths must split here, in the parent.
    pub(crate) fn start_with(
        world: &mut World,
        expected: Vec<u32>,
        mut check: impl FnMut(&mut World, u32) -> bool + 'static,
    ) -> Self {
        assert_ne!(expected.len(), 0);
        assert!(expected.windows(2).all(|pair| pair[0] < pair[1]), "reference cuts are ordered and unique");
        let passed = Rc::new(RefCell::new(Vec::new()));
        let outcomes = Rc::clone(&passed);
        let cuts = expected.clone();
        world.on_operation(move |world| {
            let next = outcomes.borrow().len();
            if let Some(&cut) = cuts.get(next) {
                assert!(world.operation_count() <= cut, "missed reference cut {cut}");
                if world.operation_count() == cut {
                    let outcome = check(world, cut);
                    outcomes.borrow_mut().push((cut, outcome));
                }
            }
        });
        Self { expected, passed }
    }

    pub(crate) fn finish(self, world: &mut World) -> Vec<bool> {
        world.operation_boundary();
        world.stop_observing_operations();
        let passed = self.passed.borrow();
        assert_eq!(passed.iter().map(|(cut, _)| *cut).collect::<Vec<_>>(), self.expected, "all reference cuts passed");
        passed.iter().map(|(_, outcome)| *outcome).collect()
    }
}
