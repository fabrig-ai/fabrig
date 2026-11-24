//! Hosted fake git on the real filesystem, sharing the caller's pipe lifecycle.
use crate::world::{Kind, Proc};
use skein_world::{HostedProgram, Inherited, StartupRoot};
use std::cell::RefCell;
use temper_fake_git_hosted::Typed;

thread_local! { static SHARED: RefCell<Option<Typed>> = const { RefCell::new(None) }; }
pub(crate) fn warm_runtime() {
    SHARED.with(|_| {});
}
pub fn share(transport: Option<Typed>) {
    SHARED.with(|shared| *shared.borrow_mut() = transport);
}
#[must_use]
pub fn transport() -> Typed {
    SHARED.with(|shared| shared.borrow().as_ref().expect("world's forge").clone())
}
fn git(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    Proc::new(Kind::Git(Box::new(temper_fake_git_hosted::git(spawn, inherited, transport()))))
}
pub(crate) fn program() -> HostedProgram<Proc> {
    HostedProgram { program: b"/usr/bin/git".as_slice().into(), make: git, instances: 1, operations: 1 }
}
pub(crate) fn roots(spawn: &skein_io::kernel::Spawn) -> Vec<StartupRoot> {
    vec![StartupRoot {
        name: b"worker".as_slice().into(),
        path: skein_shell::keeper::descriptor_path(spawn.root)
            .unwrap_or_else(|error| panic!("git worker root {:?}: {error:?}", spawn.root)),
    }]
}
