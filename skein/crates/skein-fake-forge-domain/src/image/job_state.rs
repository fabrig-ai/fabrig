//! Optional versioned image extension; older images default to no requirements.
use super::{Domain, Reader, Refusal, Sink, codec, fields};
use crate::{Limits, actions::JobWait, api};
use skein_lib::List;

pub(super) fn put(out: &mut Sink, domain: &Domain) {
    out.put(b"JOBST001");
    out.u32(domain.repositories.len());
    for (_, repository) in &domain.repositories {
        out.len(repository.workflows.len());
        for workflow in &repository.workflows {
            out.len(workflow.requirements.len());
            for requirement in &workflow.requirements {
                out.bytes(&requirement.job);
                fields::put_contexts(out, &requirement.labels);
                fields::put_contexts(out, &requirement.needs);
            }
        }
        for runs in [&repository.runs, &repository.dispatched] {
            out.u32(runs.len());
            for (_, kept) in runs {
                out.u32(kept.jobs.len());
                for job in &kept.jobs {
                    fields::put_contexts(out, &job.labels);
                    fields::put_contexts(out, &job.needs);
                    out.u8(match job.wait {
                        None => 0,
                        Some(JobWait::Runner) => 1,
                        Some(JobWait::Needs) => 2,
                    });
                }
            }
        }
    }
}
pub(super) fn get(input: &mut Reader<'_>, domain: &mut Domain, limits: &Limits) -> Result<(), Refusal> {
    codec::require(codec::u32(input)? == domain.repositories.len(), "job-state repositories")?;
    let mut ids = List::with_capacity(domain.repositories.len());
    for (id, _) in &domain.repositories {
        ids.push(id).expect("repository slots");
    }
    for id in &ids {
        let repository = domain.repositories.get_mut(*id).expect("retained repository");
        codec::require(
            codec::u32(input)? == u32::try_from(repository.workflows.len()).expect("bounded workflows"),
            "job-state workflows",
        )?;
        for workflow in &mut repository.workflows {
            let count = codec::count(input, limits.contexts, "job requirements")?;
            let mut requirements = List::with_capacity(count);
            for _ in 0..count {
                let job = codec::bytes(input, limits.name_bytes, "job")?;
                codec::require(workflow.jobs.contains(&job), "job requirement names a job")?;
                let labels = fields::get_contexts(input, limits)?;
                let needs = fields::get_contexts(input, limits)?;
                for need in &needs {
                    codec::require(workflow.jobs.contains(need), "job need names a job")?;
                }
                requirements.push(api::JobRequirements { job, labels, needs }).expect("bounded requirements");
            }
            workflow.requirements = requirements.into_boxed();
        }
        for runs in [&mut repository.runs, &mut repository.dispatched] {
            codec::require(codec::u32(input)? == runs.len(), "job-state runs")?;
            let mut keys = List::with_capacity(runs.len());
            for (key, _) in runs.iter() {
                keys.push(*key).expect("run keys");
            }
            for key in &keys {
                let kept = runs.get_mut(key).expect("retained run");
                codec::require(codec::u32(input)? == kept.jobs.len(), "job-state jobs")?;
                for job in kept.jobs.iter_mut() {
                    job.labels = fields::get_contexts(input, limits)?;
                    job.needs = fields::get_contexts(input, limits)?;
                    job.wait = match codec::u8(input)? {
                        0 => None,
                        1 => Some(JobWait::Runner),
                        2 => Some(JobWait::Needs),
                        _ => return Err(Refusal::Malformed("job wait")),
                    };
                }
            }
        }
    }
    Ok(())
}
