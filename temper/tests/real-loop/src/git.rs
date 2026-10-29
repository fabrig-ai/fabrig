//! Hosted fake git on the real filesystem, sharing the caller's pipe lifecycle.
use crate::world::{Kind, Proc};
use skein_world::{HostedProgram, Inherited, StartupRoot};
use std::cell::RefCell;
use temper_fake_git_hosted::Typed;

thread_local! { static SHARED: RefCell<Option<Typed>> = const { RefCell::new(None) }; }
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
        path: format!("/proc/self/fd/{}", spawn.root.raw()).into_bytes().into(),
    }]
}

fn routed(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    match temper_fake_git::check(spawn) {
        Ok(_) | Err(temper_fake_git::Refusal::Usage) => {}
        Err(temper_fake_git::Refusal::Broken(rule)) => panic!("git's spawn broke {rule}"),
    }
    Proc::new(Kind::GitRoute(Box::new(temper_fake_git::Git::new(
        spawn,
        temper_fake_git_hosted::start(inherited),
        temper_fake_git::Route::new(crate::github::protocol_limits()),
    ))))
}
pub(crate) fn route_program() -> HostedProgram<Proc> {
    HostedProgram { make: routed, ..program() }
}
