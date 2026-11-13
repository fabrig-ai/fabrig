use super::note_line;
#[test]
fn joined_job_notes_delegate_to_each_halfs_words() {
    use jig_domain_host::{JobFailure, Preparation};
    use temper_service::{EngineNote, JobDiagnostic, JobDiagnosticEnding, Note, WorkerJobEnding, worker_alone};
    for (worker, prefix) in [
        (false, "temper serve: task 3 run 7 attempt 2 failed Unprepared(Transient): job 3: "),
        (true, "temper worker: run 7 attempt 2 job 3 merge failed Unprepared(Transient): "),
    ] {
        let why = b"fetch base: Broken\n\xff";
        let failure = JobFailure::Unprepared(Preparation::Transient);
        let note = if worker {
            Note::Worker(worker_alone::Note::WorkerJob {
                run: 7,
                attempt: 2,
                kind: 3,
                verb: b"merge".as_slice().into(),
                ending: WorkerJobEnding::Failed { failure, detail: why.as_slice().into() },
                why: why.as_slice().into(),
            })
        } else {
            Note::Engine(EngineNote::JobAnswered(JobDiagnostic {
                task: 3,
                run: 7,
                attempt: 2,
                kind: 3,
                ending: JobDiagnosticEnding::Failed { failure, detail: why.as_slice().into() },
            }))
        };
        assert_eq!(note_line("temper", note), format!("{prefix}fetch base: Broken\\n\\xff\n"));
    }
}
