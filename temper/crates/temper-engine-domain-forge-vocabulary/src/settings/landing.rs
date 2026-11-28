//! The landing kept at restore (temper/docs/design/domain/landing-settings.md, 1.1).
use super::{Bounds, Refusal};
use crate::{
    Follows, RepositoryGates, Tier,
    generated::client_wire::{
        project_landing_to_domain, project_landing_to_record, settings_gate_to_domain, settings_gate_to_record,
        settings_tier_to_domain, settings_tier_to_record,
    },
};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::{Duration, List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain_forge_landing::{Combine, Gate};

pub const CARRIER: u32 = jig_client_protocol_wire::CEILINGS.application_value_bytes;

/// Whether a project's checks can flake (temper/docs/design/domain/forge/judgement.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flakes {
    /// `"none"`.
    Never,
    /// `{"rerun": N}`, N at most the deployment's `checkout.rerun_processes`.
    Rerun(u32),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectLanding {
    pub by: temper_engine_domain_forge_change::Way,
    pub batch: u16,
    pub combine_main: Combine,
    pub combine_other: Combine,
    pub merge_job: Option<Box<[u8]>>,
    pub close_job: Option<Box<[u8]>>,
    pub landing_attempts: u8,
    pub check_timeout_ms: Option<u64>,
    pub hook_timeout_max_seconds: u32,
    /// Unset is `rerun` at the deployment's figure.
    pub flakes: Option<Flakes>,
    pub gate_retries: u8,
    pub gate_timeout_ms: u64,
    pub tier_retries: u8,
    pub verification_bound_ms: u64,
    pub wait: temper_engine_domain_forge_change::Wait,
    pub repairs: Option<super::RepairSettings>,
}
impl Default for ProjectLanding {
    fn default() -> Self {
        Self {
            by: temper_engine_domain_forge_change::Way::Queue,
            batch: 1,
            combine_main: Combine::Merge,
            combine_other: Combine::Squash,
            merge_job: None,
            close_job: None,
            landing_attempts: 3,
            check_timeout_ms: None,
            hook_timeout_max_seconds: 3_600,
            flakes: None,
            gate_retries: 2,
            gate_timeout_ms: 3_600_000,
            tier_retries: 2,
            verification_bound_ms: 7_200_000,
            wait: temper_engine_domain_forge_change::Wait::Cleared,
            repairs: None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectGates {
    pub repositories: Box<[RepositoryGates]>,
}

pub fn landing_record(v: &ProjectLanding) -> w::ProjectLanding {
    project_landing_to_record(v).expect("admitted repair model counts")
}
pub fn landing_from(v: w::ProjectLanding) -> ProjectLanding {
    project_landing_to_domain(v)
}
pub fn gate_record(v: &Gate) -> w::SettingsGate {
    settings_gate_to_record(v).expect("counted paths")
}
pub fn tier_record(v: &Tier) -> w::SettingsTier {
    settings_tier_to_record(v).expect("counted paths")
}

// Zero in a tier's span field means no span; other values are whole milliseconds.
#[expect(clippy::ref_option, reason = "the wire span is a required scalar translated from a borrowed domain field")]
pub(crate) fn span_to_record(span: &Option<Duration>) -> u64 {
    match span {
        Some(span) => span.as_millis(),
        None => 0,
    }
}
pub(crate) fn span_to_domain(ms: u64) -> Option<Duration> {
    match ms {
        0 => None,
        ms => Some(Duration::from_millis(ms)),
    }
}

fn append(out: &mut Writer, bytes: &[u8]) {
    skein_codec::piece::write(out, bytes).expect("measured pieces");
}
fn encoded_gate(v: &Gate) -> Box<[u8]> {
    let r = gate_record(v);
    let mut out = Writer::new(skein_lib::usize_of(r.measure(&w::CEILINGS).expect("admitted settings within ceilings")));
    r.write(&w::CEILINGS, &mut out).expect("measured gate");
    out.finish()
}
fn encoded_tier(v: &Tier) -> Box<[u8]> {
    let r = tier_record(v);
    let mut out = Writer::new(skein_lib::usize_of(r.measure(&w::CEILINGS).expect("admitted settings within ceilings")));
    r.write(&w::CEILINGS, &mut out).expect("measured tier");
    out.finish()
}
fn repository_record(v: &RepositoryGates) -> w::RepositoryGates {
    let mut gates = Writer::new(skein_lib::usize_of(gates_size(&v.gates).expect("admitted gates size")));
    for gate in &v.gates {
        append(&mut gates, &encoded_gate(gate));
    }
    let mut tiers = Writer::new(skein_lib::usize_of(tiers_size(&v.tiers).expect("admitted tiers size")));
    for tier in &v.tiers {
        append(&mut tiers, &encoded_tier(tier));
    }
    w::RepositoryGates {
        forge: v.repository.forge,
        repository: v.repository.repository,
        gates: gates.finish(),
        tiers: tiers.finish(),
    }
}
pub fn gates_record(v: &ProjectGates) -> w::ProjectGates {
    let mut out = Writer::new(skein_lib::usize_of(
        gates_bytes(v).expect("admitted gates size").checked_sub(4).expect("record prefix"),
    ));
    for repo in &v.repositories {
        let r = repository_record(repo);
        let mut item =
            Writer::new(skein_lib::usize_of(r.measure(&w::CEILINGS).expect("admitted settings within ceilings")));
        r.write(&w::CEILINGS, &mut item).expect("measured repository");
        append(&mut out, &item.finish());
    }
    w::ProjectGates { repositories: out.finish() }
}
pub fn gates_from(v: w::ProjectGates) -> Option<ProjectGates> {
    let bytes = v.repositories;
    let mut input = Reader::new(&bytes);
    let mut charged = 4;
    let mut repositories = List::uncapped();
    while let Some(bytes) = skein_codec::piece::split_within(&mut input, &mut charged, u64::from(CARRIER)).ok()? {
        let mut r = Reader::new(&bytes);
        let v = w::RepositoryGates::decode(&w::CEILINGS, &mut r).ok()?;
        if !r.is_empty() {
            return None;
        }
        let mut input = Reader::new(&v.gates);
        let mut sum = 0;
        let mut gates = List::uncapped();
        while let Some(bytes) = skein_codec::piece::split_within(&mut input, &mut sum, u64::from(CARRIER)).ok()? {
            let mut r = Reader::new(&bytes);
            let gate = w::SettingsGate::decode(&w::CEILINGS, &mut r).ok()?;
            if !r.is_empty() {
                return None;
            }
            gates.push(settings_gate_to_domain(gate)).ok()?;
        }
        let mut input = Reader::new(&v.tiers);
        let mut sum = 0;
        let mut tiers = List::uncapped();
        while let Some(bytes) = skein_codec::piece::split_within(&mut input, &mut sum, u64::from(CARRIER)).ok()? {
            let mut r = Reader::new(&bytes);
            let tier = w::SettingsTier::decode(&w::CEILINGS, &mut r).ok()?;
            if !r.is_empty() {
                return None;
            }
            tiers.push(settings_tier_to_domain(tier)).ok()?;
        }
        repositories
            .push(RepositoryGates {
                repository: skein_forge::api::Repository { forge: v.forge, repository: v.repository },
                gates: gates.into_boxed(),
                tiers: tiers.into_boxed(),
            })
            .ok()?;
    }
    Some(ProjectGates { repositories: repositories.into_boxed() })
}
// Measure without constructing a codec record: an oversized value is a judgement, not a panic.
fn text_size(v: &[u8]) -> Option<u32> {
    u32::try_from(v.len()).ok()?.checked_add(4)
}
fn check_size(name: &[u8], workflow: &[u8], ignored: &[Box<[u8]>]) -> Option<u32> {
    let mut size = text_size(name)?.checked_add(text_size(workflow)?)?.checked_add(4)?;
    for path in ignored {
        size = size.checked_add(text_size(path)?)?;
    }
    Some(size)
}
fn gates_size(v: &[Gate]) -> Option<u32> {
    let mut size = 0_u32;
    for gate in v {
        let mut n = check_size(&gate.name, &gate.workflow, &gate.ignored)?.checked_add(3)?;
        if let Some(r) = &gate.report {
            n = n.checked_add(text_size(&r.artifact)?)?.checked_add(2)?;
            if let Some(f) = &r.fuzzy {
                n = n.checked_add(text_size(f)?)?;
            }
        }
        size = size.checked_add(4)?.checked_add(n)?;
    }
    Some(size)
}
fn tiers_size(v: &[Tier]) -> Option<u32> {
    let mut size = 0_u32;
    for tier in v {
        let mut n = check_size(&tier.name, &tier.workflow, &tier.ignored)?
            .checked_add(text_size(&tier.artifact)?)?
            .checked_add(10)?
            .checked_add(text_size(&tier.description)?)?;
        if let Follows::Branch(branch) = &tier.follows {
            n = n.checked_add(text_size(branch)?)?;
        }
        size = size.checked_add(4)?.checked_add(n)?;
    }
    Some(size)
}
pub fn gates_bytes(v: &ProjectGates) -> Option<u32> {
    let mut size = 4_u32;
    for repo in &v.repositories {
        size = size.checked_add(18)?.checked_add(gates_size(&repo.gates)?)?.checked_add(tiers_size(&repo.tiers)?)?;
    }
    Some(size)
}
fn refuse(field: &'static [u8], rule: &'static [u8]) -> Refusal {
    Refusal::Landing { field, rule }
}
pub fn judge_landing(v: &ProjectLanding, b: &Bounds) -> Result<(), Refusal> {
    if v.batch == 0 {
        return Err(refuse(b"batch", b"must be at least 1"));
    }
    if u32::from(v.batch) > b.landing_members {
        return Err(refuse(b"batch", b"passes forge.landing_members"));
    }
    if let Some(ms) = v.check_timeout_ms
        && !(1_000..=86_400_000).contains(&ms)
    {
        return Err(refuse(b"check_timeout_ms", b"must be 1000 to 86400000"));
    }
    if !(1..=86_400).contains(&v.hook_timeout_max_seconds) {
        return Err(refuse(b"hook_timeout_max_seconds", b"must be 1 to 86400"));
    }
    if let Some(Flakes::Rerun(processes)) = v.flakes
        && processes > b.rerun_processes
    {
        return Err(refuse(b"flakes", b"rerun passes checkout.rerun_processes"));
    }
    if !(60_000..=86_400_000).contains(&v.gate_timeout_ms) {
        return Err(refuse(b"gate_timeout_ms", b"must be 60000 to 86400000"));
    }
    if v.landing_attempts == 0 {
        return Err(refuse(b"landing_attempts", b"must be 1 to 255"));
    }
    for (field, path) in [(b"merge_job".as_slice(), &v.merge_job), (b"close_job".as_slice(), &v.close_job)] {
        if let Some(path) = path
            && !job_path(path)
        {
            return Err(refuse(field, b"must be a relative path of 1 to 4096 bytes without NUL or .. segments"));
        }
    }
    if let Some(repairs) = &v.repairs {
        if v.wait != temper_engine_domain_forge_change::Wait::Landed {
            return Err(refuse(b"repairs", b"needs wait landed"));
        }
        if !(1..=16).contains(&repairs.goals) {
            return Err(refuse(b"repairs.repair_goals", b"must be 1 to 16"));
        }
        if !(1..=8).contains(&repairs.attempts) {
            return Err(refuse(b"repairs.repair_attempts", b"must be 1 to 8"));
        }
        if !(3_600_000..=604_800_000).contains(&repairs.deadline_ms) {
            return Err(refuse(b"repairs.repair_deadline_ms", b"must be 3600000 to 604800000"));
        }
    }
    Ok(())
}
/// A relative tooling path, within the setting's path bound.
#[must_use]
pub fn job_path(path: &[u8]) -> bool {
    if path.is_empty() || path.len() > 4096 || path.first() == Some(&b'/') || path.contains(&0) {
        return false;
    }
    segments(path, false)
}
fn check(name: &[u8], workflow: &[u8], ignored: &[Box<[u8]>], b: &Bounds) -> Result<(), Refusal> {
    if name.is_empty() || name.len() > 32 {
        return Err(refuse(b"name", b"must be 1 to 32 bytes"));
    }
    for byte in name {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
            return Err(refuse(b"name", b"must be letters, digits, - or _"));
        }
    }
    if workflow.is_empty() || workflow.len() > 255 || workflow.contains(&b'/') {
        return Err(refuse(b"workflow", b"must be a filename of 1 to 255 bytes"));
    }
    if !super::within(ignored.len(), b.ignored_paths) {
        return Err(refuse(b"ignored", b"passes forge.ignored_paths"));
    }
    for path in ignored {
        if !super::within(path.len(), b.name_bytes) {
            return Err(refuse(b"ignored", b"passes forge.name_bytes"));
        }
        if path.is_empty() {
            return Err(refuse(b"ignored", b"must be whole path segments"));
        }
        if !segments(path, true) {
            return Err(refuse(b"ignored", b"must be whole path segments"));
        }
    }
    Ok(())
}
fn artifact(value: &[u8]) -> Result<(), Refusal> {
    if value.is_empty() || value.len() > 32 || value.contains(&b'/') {
        Err(refuse(b"artifact", b"must be 1 to 32 bytes without /"))
    } else {
        Ok(())
    }
}
pub fn judge_gates(v: &ProjectGates, b: &Bounds) -> Result<(), Refusal> {
    if match gates_bytes(v) {
        Some(size) => size > CARRIER,
        None => true,
    } {
        return Err(Refusal::LandingCarrier);
    }
    if !super::within(v.repositories.len(), b.repositories) {
        return Err(refuse(b"repositories", b"passes forge.repositories"));
    }
    for (i, repo) in v.repositories.iter().enumerate() {
        for previous in v.repositories.iter().take(i) {
            if previous.repository == repo.repository {
                return Err(refuse(b"repository", b"given twice in repositories"));
            }
        }
        if !super::within(repo.gates.len(), b.landing_gates) {
            return Err(refuse(b"gates", b"passes forge.gates"));
        }
        if !super::within(repo.tiers.len(), b.tiers) {
            return Err(refuse(b"tiers", b"passes forge.tiers.tiers"));
        }
        for (i, gate) in repo.gates.iter().enumerate() {
            check(&gate.name, &gate.workflow, &gate.ignored, b)?;
            for previous in repo.gates.iter().take(i) {
                if previous.name == gate.name {
                    return Err(refuse(b"gates.name", b"given twice"));
                }
            }
            if let Some(r) = &gate.report {
                artifact(&r.artifact)?;
                if let Some(f) = &r.fuzzy
                    && (f.is_empty() || f.len() > 64)
                {
                    return Err(refuse(b"report.fuzzy", b"must be 1 to 64 bytes"));
                }
            }
        }
        for (i, tier) in repo.tiers.iter().enumerate() {
            check(&tier.name, &tier.workflow, &tier.ignored, b)?;
            artifact(&tier.artifact)?;
            for previous in repo.tiers.iter().take(i) {
                if previous.name == tier.name {
                    return Err(refuse(b"tiers.name", b"given twice"));
                }
            }
            for gate in &repo.gates {
                if gate.name == tier.name {
                    return Err(refuse(b"tiers.name", b"also names a gate"));
                }
            }
            if !tier.description.is_empty() && !super::words_fit(&tier.description, 120) {
                return Err(refuse(b"description", b"must be 1 to 120 bytes of UTF-8 without controls"));
            }
            if let Follows::Branch(branch) = &tier.follows
                && (branch.is_empty() || branch.len() > 1_048_576)
            {
                return Err(refuse(b"branch", b"must be 1 to 1048576 bytes"));
            }
        }
    }
    Ok(())
}

fn segments(value: &[u8], whole: bool) -> bool {
    let mut length = 0_usize;
    let mut dots = true;
    for byte in value {
        if *byte == b'/' {
            if (whole && length == 0) || (dots && (length == 2 || (whole && length == 1))) {
                return false;
            }
            length = 0;
            dots = true;
        } else {
            length = length.saturating_add(1);
            dots &= *byte == b'.';
        }
    }
    !(whole && length == 0) && !(dots && (length == 2 || (whole && length == 1)))
}

/// Retained nested allocations, priced with the setting's row at admission.
pub fn gates_heap(v: &ProjectGates) -> Option<u64> {
    let mut held =
        skein_lib::len_u64(v.repositories.len()).checked_mul(u64::try_from(size_of::<RepositoryGates>()).ok()?)?;
    for repository in &v.repositories {
        held = held
            .checked_add(
                skein_lib::len_u64(repository.gates.len()).checked_mul(u64::try_from(size_of::<Gate>()).ok()?)?,
            )?
            .checked_add(
                skein_lib::len_u64(repository.tiers.len()).checked_mul(u64::try_from(size_of::<Tier>()).ok()?)?,
            )?;
        for gate in &repository.gates {
            held = held.checked_add(check_heap(&gate.name, &gate.workflow, &gate.ignored)?)?;
            if let Some(report) = &gate.report {
                held = held.checked_add(skein_lib::len_u64(report.artifact.len()))?;
                if let Some(fuzzy) = &report.fuzzy {
                    held = held.checked_add(skein_lib::len_u64(fuzzy.len()))?;
                }
            }
        }
        for tier in &repository.tiers {
            held = held
                .checked_add(check_heap(&tier.name, &tier.workflow, &tier.ignored)?)?
                .checked_add(skein_lib::len_u64(tier.artifact.len()))?
                .checked_add(skein_lib::len_u64(tier.description.len()))?;
            if let Follows::Branch(branch) = &tier.follows {
                held = held.checked_add(skein_lib::len_u64(branch.len()))?;
            }
        }
    }
    Some(held)
}
fn check_heap(name: &[u8], workflow: &[u8], paths: &[Box<[u8]>]) -> Option<u64> {
    let mut held = skein_lib::len_u64(name.len())
        .checked_add(skein_lib::len_u64(workflow.len()))?
        .checked_add(skein_lib::len_u64(paths.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?)?;
    for path in paths {
        held = held.checked_add(skein_lib::len_u64(path.len()))?;
    }
    Some(held)
}
