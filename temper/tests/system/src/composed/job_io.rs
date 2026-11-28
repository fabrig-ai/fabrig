//! The composed world's job IO (temper/docs/design/testing/README.md, 3.2;
//! temper/docs/design/protocol/hooks.md, 4.3).
use super::{Checkout, Token, World, worker};
use temper_worker_domain::tooling::{self, Conflict, Event, Exit, Hooks, Request};

impl World {
    pub(super) fn job_operation(&mut self, owner: Token, op: Request) {
        let event = match op {
            Request::Write { owner: inner, path, bytes } => {
                assert_eq!(owner, inner);
                self.lanes[0].disk.write(&disk_path(&path), &bytes);
                Event::Written { owner, written: true }
            }
            Request::Spawn { owner: inner, program, mut args, dir, hooks, .. } => {
                assert_eq!(owner, inner);
                assert_eq!(args.first(), Some(&program));
                match command(&self.lanes[0].disk, owner, program, &dir, hooks) {
                    Ok(program) => {
                        args[0] = program;
                        if self.close_job.is_some() {
                            self.play_close_command(owner, &args[0], &args, &dir);
                        } else {
                            self.play_merge_command(owner, &args[0], &args, &dir);
                        }
                        return;
                    }
                    Err(event) => {
                        self.lanes[0].worker_events.push_back(worker::Event::Job { owner, event });
                        Event::Gone { owner }
                    }
                }
            }
            Request::Read { owner: inner, path, most } => {
                assert_eq!(owner, inner);
                let bytes = self.lanes[0]
                    .disk
                    .content(&disk_path(&path))
                    .filter(|bytes| bytes.len() <= usize::try_from(most).expect("result bound"))
                    .map(Into::into);
                Event::Read { owner, bytes }
            }
            Request::Kill { owner: inner } => {
                assert_eq!(owner, inner);
                self.lanes[0].worker_events.push_back(worker::Event::Job {
                    owner,
                    event: Event::Exited { owner, exit: Exit::Signal, tail: Box::new([]) },
                });
                Event::Gone { owner }
            }
            other => panic!("root routes job IO only: {other:?}"),
        };
        self.lanes[0].worker_events.push_back(worker::Event::Job { owner, event });
    }
}

// The fake git tree contains regular files and the directories they imply. File
// modes, sibling links and config reads are exercised at the worker service tier.
fn command(disk: &Checkout, owner: Token, program: Box<[u8]>, dir: &[u8], hooks: Hooks) -> Result<Box<[u8]>, Event> {
    let path = program.strip_prefix(dir).and_then(|path| path.strip_prefix(b"/")).expect("program in workspace");
    let Some((repository, hook)) = tooling::hook_path(path) else { return Ok(program) };
    let directory = disk_path(&[dir, b"/", repository, b"/.temper/hooks"].concat());
    let exited = |exit, tail| Event::Exited { owner, exit, tail };
    let fail = |why| exited(Exit::NotRun, tooling::hook_failure(hook, why));
    if hooks.conflict == Some(Conflict::Path) {
        return Err(fail(Conflict::Path.why()));
    }
    if !disk.exists(&directory) || disk.content(&directory).is_some() {
        if let Some(fallback) = hooks.fallback {
            if fallback.program.is_empty() {
                return Err(exited(Exit::Absent, Box::new([])));
            }
            if fallback.conflict == Some(Conflict::Path) {
                return Err(exited(Exit::NotRun, b"fallback command: left in conflict".as_slice().into()));
            }
            return Ok(fallback.program);
        }
        return Err(fail(if disk.exists(&directory) { b"parent is not a directory" } else { b"missing" }));
    }
    let at = disk_path(&program);
    if disk.content(&at).is_none() {
        return if disk.exists(&at) {
            Err(fail(b"not a regular file"))
        } else if hooks.fallback.is_some() {
            Err(exited(Exit::Absent, Box::new([])))
        } else {
            Err(fail(b"missing"))
        };
    }
    if let Some(conflict) = hooks.conflict {
        return Err(fail(conflict.why()));
    }
    Ok(program)
}

/// The job IO face and the fake git face spell the same workspace directory differently.
pub(super) fn disk_path(path: &[u8]) -> Vec<u8> {
    [b"ws/".as_slice(), path.strip_prefix(b"workspaces/").expect("job workspace path")].concat()
}
