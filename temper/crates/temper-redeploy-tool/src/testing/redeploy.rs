//! Shared redeploy value fixtures (tools/crates/design.md, 3.2).
use crate::fake::{Deployment, NEW, OLD, description};
use std::{cell::RefCell, io::Write, rc::Rc};

pub const CHECKPOINT: &str = "00000000000000000001-0123456789abcdef";
#[derive(Clone, Default)]
pub struct Printed(pub Rc<RefCell<Vec<u8>>>);
impl Write for Printed {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn staged(count: usize) -> Deployment {
    let mut hosts = Deployment::new(description(count));
    hosts.stage_all(NEW);
    hosts
}

pub fn all_old(hosts: &Deployment) {
    for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
        assert!(machine.running);
        assert!(machine.installed.values().all(|build| build == OLD));
    }
    assert!(hosts.lines.iter().all(|call| !call.line.starts_with("swap ")));
}
