//! Dispatch-only gates and manual merges of the process forge (testing/fakes.md, 2).
use alloc::boxed::Box;
use jig_protocol_config::Path;
use skein_fake_forge_domain::api;
use skein_lib::{Duration, List};

use crate::read::{self, Refusal};

pub(super) const WORKFLOWS: u32 = 8;
pub(super) const RUNS: u32 = 32;

pub(super) fn decode(d: &skein_json::Document, at: u32, p: &Path, ci_ms: u32) -> Result<Box<[api::Workflow]>, Refusal> {
    let items = match jig_protocol_config::read::list(d, at, WORKFLOWS) {
        Ok(items) => items,
        Err(why) => return Err(read::kind(p, why)),
    };
    let mut workflows = List::<api::Workflow>::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let mut o = read::object(d, *at, &p)?;
        let file = read::field(&mut o, d, b"file", 64, &p)?;
        read::nonempty(&file, &p.member(b"file"))?;
        if file.contains(&b'/') {
            return Err(read::why(&p.member(b"file"), b"must be a file name without /"));
        }
        for previous in &workflows {
            if previous.file == file {
                return Err(read::why(&p.member(b"file"), b"must be unique"));
            }
        }
        let jobs_at = read::required(&mut o, d, b"jobs", &p)?;
        let jobs = jobs(d, jobs_at, &p.member(b"jobs"), file.len())?;
        let cue_at = read::required(&mut o, d, b"cue", &p)?;
        let ci = super::ci(d, cue_at, &p.member(b"cue"), false)?;
        read::finish(o, d, &p)?;
        let latency = Duration::from_millis(u64::from(ci_ms));
        workflows
            .push(api::Workflow {
                file,
                jobs,
                latency_min: latency,
                latency_max: latency,
                silent: 0,
                passes: 1000,
                cue: super::checks(ci.as_ref()).cue,
                fails: Box::new([]),
                flaky: Box::new([]),
                report: None,
                select: None,
                requirements: Box::new([]),
            })
            .expect("bounded workflows");
    }
    Ok(workflows.into_boxed())
}

fn jobs(d: &skein_json::Document, at: u32, p: &Path, file_bytes: usize) -> Result<Box<[Box<[u8]>]>, Refusal> {
    let items = match jig_protocol_config::read::list(d, at, 8) {
        Ok(items) => items,
        Err(why) => return Err(read::kind(p, why)),
    };
    if items.is_empty() {
        return Err(read::why(p, b"must be non-empty"));
    }
    let mut jobs = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let job = read::text(d, *at, super::limits().limits.name_bytes, &p)?;
        let mut word = !job.is_empty();
        for b in &job {
            word &= b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-';
        }
        if !word {
            return Err(read::why(&p, b"expected a word"));
        }
        // Domain workflow admission bounds file, separator and job together.
        if job.len().checked_add(file_bytes).expect("bounded context").checked_add(3).expect("context separator")
            > skein_lib::usize_of(super::limits().limits.name_bytes)
        {
            return Err(read::why(&p, b"past its bound"));
        }
        if jobs.as_slice().contains(&job) {
            return Err(read::why(&p, b"must be unique"));
        }
        jobs.push(job).expect("bounded jobs");
    }
    Ok(jobs.into_boxed())
}

pub(super) fn manual_merges(d: &skein_json::Document, at: u32, p: &Path) -> Result<api::ManualMerges, Refusal> {
    let mut o = read::object(d, at, p)?;
    let allow_manual_merge = boolean(&mut o, d, b"allow", p)?;
    let autodetect_manual_merge = boolean(&mut o, d, b"autodetect", p)?;
    read::finish(o, d, p)?;
    Ok(api::ManualMerges { allow_manual_merge, autodetect_manual_merge })
}

fn boolean(
    o: &mut jig_protocol_config::read::Object,
    d: &skein_json::Document,
    name: &[u8],
    p: &Path,
) -> Result<bool, Refusal> {
    match o.member(d, name) {
        Some(at) => match jig_protocol_config::read::boolean(d, at) {
            Ok(value) => Ok(value),
            Err(why) => Err(read::kind(&p.member(name), why)),
        },
        None => Ok(false),
    }
}
