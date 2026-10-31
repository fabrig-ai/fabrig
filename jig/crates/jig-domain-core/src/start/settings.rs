//! Settings fits and the release revision belong to the start (jig/docs/design/domain/engine.md, 6, 17.7).
use crate::{Core, Limits, Request, settings, start::Failure};
use skein_lib::{Env, Queue};

impl Core {
    pub(crate) fn start_settings_finish(&mut self, env: &Env<Limits>) -> Option<Failure> {
        let _finished = self.finish_restored_settings(env);
        self.start_collect_failure();
        self.start_failure()
    }

    pub(super) fn start_settings_commit(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> Option<Failure> {
        let room = settings::writes_bound(&env.limits).expect("configured settings writes").saturating_sub(1);
        let mut writes = self.write_restored_settings(env, room);
        self.start_collect_failure();
        if self.start_failure().is_some() {
            return self.start_failure();
        }
        for _ in 0..writes.len() {
            let request = writes.pop().expect("settings release writes");
            out.push(request);
        }
        None
    }
}
