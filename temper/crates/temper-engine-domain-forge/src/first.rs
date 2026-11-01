//! Kept first-repository intent (temper/docs/plan/playground-reset-r4/notes/contracts.md, 2 and D5).
use crate::{Domain, First, FirstState, Forge, Limits, Request, SetUpAnswer, SetUpRefusal, Stored};
use skein_lib::{Duration, Env, Queue, Token};

pub(crate) fn keep(d: &mut Domain, forge: u16, intent: First, out: &mut Queue<Request>) {
    if let Some(record) = d.forge_records.get_mut(&forge) {
        record.first = Some(crate::record::FirstIntent {
            org: intent.org.clone(),
            repository: intent.repository.clone(),
            project: intent.project.clone(),
            label: intent.label.clone(),
        });
        out.push(Request::Save { record: Stored::Forge(record.clone()), release: None, read_afresh: None });
    }
    d.first.insert(forge, intent).expect("configured forges bound first repositories");
}

pub(crate) fn made(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    if let Some(record) = d.forge_records.get_mut(&forge)
        && record.first.take().is_some()
    {
        out.push(Request::Save { record: Stored::Forge(record.clone()), release: None, read_afresh: None });
    }
}

pub(crate) fn restored(d: &mut Domain, record: &Forge) {
    d.first.remove(&record.number);
    if let Some(intent) = &record.first {
        d.first
            .insert(
                record.number,
                First {
                    playground: record.playground,
                    org: intent.org.clone(),
                    repository: intent.repository.clone(),
                    project: intent.project.clone(),
                    label: intent.label.clone(),
                    state: FirstState::Making,
                },
            )
            .expect("restored forges bound first repositories");
    }
}

impl Domain {
    /// Any retained repository takes its project (temper/docs/plan/playground-reset-r4/notes/contracts.md, D5).
    #[must_use]
    pub fn project_has_repository(&self, project: u32) -> bool {
        for (_, row) in &self.repositories {
            if row.project == project {
                return true;
            }
        }
        false
    }

    /// Retain a `SetUp` refusal through its ordinary bounded answer path (contracts, D5).
    pub fn refuse_set_up(
        &mut self,
        env: &Env<Limits>,
        request: Token,
        token: Token,
        retention: Duration,
        why: SetUpRefusal,
        out: &mut Queue<Request>,
    ) {
        if self.setup_admit(env, request, token, retention, out) {
            self.setup_answer(env, request, SetUpAnswer::Refused(why), out);
        }
    }
}
