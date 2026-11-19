//! The landing kept at restore (temper/docs/design/domain/landing-settings.md, 1.1).
use super::{Bounds, Refusal};
use crate::{Follows, RepositoryGates, Tier};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;
use temper_engine_domain_forge_landing::{Combine, Gate, GateReport};
use temper_engine_domain_forge_tiers::Format;

pub const CARRIER: u32 = jig_client_protocol_wire::CEILINGS.application_value_bytes;

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
    w::ProjectLanding::new(
        &w::CEILINGS,
        w::ProjectLandingParts {
            repairs: super::repairs::record(v.repairs.as_ref()),
            by: match v.by {
                temper_engine_domain_forge_change::Way::Queue => w::SettingsLandingWay::Queue,
                temper_engine_domain_forge_change::Way::Turn => w::SettingsLandingWay::Turn,
            },
            batch: v.batch,
            combine_main: combine_record(v.combine_main),
            combine_other: combine_record(v.combine_other),
            merge_job: v.merge_job.clone(),
            close_job: v.close_job.clone(),
            landing_attempts: v.landing_attempts,
            check_timeout_ms: v.check_timeout_ms,
            gate_retries: v.gate_retries,
            gate_timeout_ms: v.gate_timeout_ms,
            tier_retries: v.tier_retries,
            verification_bound_ms: v.verification_bound_ms,
            wait: match v.wait {
                temper_engine_domain_forge_change::Wait::Cleared => w::SettingsLandingWait::Cleared,
                temper_engine_domain_forge_change::Wait::Landed => w::SettingsLandingWait::Landed,
            },
        },
    )
    .expect("admitted landing fits codec")
}
pub fn landing_from(v: w::ProjectLanding) -> ProjectLanding {
    let v = v.into_parts();
    ProjectLanding {
        repairs: super::repairs::from_record(v.repairs),
        by: match v.by {
            w::SettingsLandingWay::Queue => temper_engine_domain_forge_change::Way::Queue,
            w::SettingsLandingWay::Turn => temper_engine_domain_forge_change::Way::Turn,
        },
        batch: v.batch,
        combine_main: combine_from(v.combine_main),
        combine_other: combine_from(v.combine_other),
        merge_job: v.merge_job,
        close_job: v.close_job,
        landing_attempts: v.landing_attempts,
        check_timeout_ms: v.check_timeout_ms,
        gate_retries: v.gate_retries,
        gate_timeout_ms: v.gate_timeout_ms,
        tier_retries: v.tier_retries,
        verification_bound_ms: v.verification_bound_ms,
        wait: match v.wait {
            w::SettingsLandingWait::Cleared => temper_engine_domain_forge_change::Wait::Cleared,
            w::SettingsLandingWait::Landed => temper_engine_domain_forge_change::Wait::Landed,
        },
    }
}
fn combine_record(v: Combine) -> w::SettingsCombine {
    match v {
        Combine::Merge => w::SettingsCombine::Merge,
        Combine::Squash => w::SettingsCombine::Squash,
    }
}
fn combine_from(v: w::SettingsCombine) -> Combine {
    match v {
        w::SettingsCombine::Merge => Combine::Merge,
        w::SettingsCombine::Squash => Combine::Squash,
    }
}
fn format_record(v: Format) -> w::SettingsReportFormat {
    match v {
        Format::Junit => w::SettingsReportFormat::Junit,
        Format::Lines => w::SettingsReportFormat::Lines,
    }
}
fn format_from(v: w::SettingsReportFormat) -> Format {
    match v {
        w::SettingsReportFormat::Junit => Format::Junit,
        w::SettingsReportFormat::Lines => Format::Lines,
    }
}
fn ignored(v: &[Box<[u8]>]) -> List<Box<[u8]>> {
    let mut out = List::with_capacity(u32::try_from(v.len()).expect("counted paths"));
    for path in v {
        out.push(path.clone()).expect("counted paths");
    }
    out
}
fn report_record(v: &GateReport) -> w::SettingsGateReport {
    w::SettingsGateReport::new(
        &w::CEILINGS,
        w::SettingsGateReportParts {
            artifact: v.artifact.clone(),
            format: format_record(v.format),
            fuzzy: v.fuzzy.clone(),
        },
    )
    .expect("admitted report")
}
#[expect(clippy::manual_map, reason = "the domain matches optional report records without closures")]
pub fn gate_record(v: &Gate) -> w::SettingsGate {
    w::SettingsGate::new(
        &w::CEILINGS,
        w::SettingsGateParts {
            name: v.name.clone(),
            workflow: v.workflow.clone(),
            ignored: ignored(&v.ignored),
            blocking: v.blocking,
            base: v.base,
            report: match &v.report {
                Some(report) => Some(report_record(report)),
                None => None,
            },
        },
    )
    .expect("admitted gate")
}
pub fn tier_record(v: &Tier) -> w::SettingsTier {
    w::SettingsTier::new(
        &w::CEILINGS,
        w::SettingsTierParts {
            name: v.name.clone(),
            workflow: v.workflow.clone(),
            ignored: ignored(&v.ignored),
            artifact: v.artifact.clone(),
            format: format_record(v.format),
            follows: match &v.follows {
                Follows::Landing => w::SettingsFollows::Landing,
                Follows::Goals => w::SettingsFollows::Goals,
                Follows::Branch(name) => w::SettingsFollows::Branch(
                    w::SettingsBranch::new(&w::CEILINGS, w::SettingsBranchParts { name: name.clone() })
                        .expect("admitted branch"),
                ),
            },
            span_ms: match v.span {
                Some(span) => span.as_millis(),
                None => 0,
            },
            description: v.description.clone(),
        },
    )
    .expect("admitted tier")
}
fn gate_from(v: w::SettingsGate) -> Gate {
    let v = v.into_parts();
    Gate {
        name: v.name,
        workflow: v.workflow,
        ignored: v.ignored.into_boxed(),
        blocking: v.blocking,
        base: v.base,
        report: match v.report {
            Some(report) => {
                let r = report.into_parts();
                Some(GateReport { artifact: r.artifact, format: format_from(r.format), fuzzy: r.fuzzy })
            }
            None => None,
        },
    }
}
fn tier_from(v: w::SettingsTier) -> Tier {
    let v = v.into_parts();
    Tier {
        name: v.name,
        workflow: v.workflow,
        ignored: v.ignored.into_boxed(),
        artifact: v.artifact,
        format: format_from(v.format),
        follows: match v.follows {
            w::SettingsFollows::Landing => Follows::Landing,
            w::SettingsFollows::Goals => Follows::Goals,
            w::SettingsFollows::Branch(b) => Follows::Branch(b.into_parts().name),
        },
        span: match v.span_ms {
            0 => None,
            ms => Some(skein_lib::Duration::from_millis(ms)),
        },
        description: v.description,
    }
}
fn append(out: &mut Writer, bytes: &[u8]) {
    skein_codec::piece::write(out, bytes).expect("measured pieces");
}
fn encoded_gate(v: &Gate) -> Box<[u8]> {
    let r = gate_record(v);
    let mut out = Writer::new(skein_lib::usize_of(r.measure()));
    r.encode(&mut out).expect("measured gate");
    out.finish()
}
fn encoded_tier(v: &Tier) -> Box<[u8]> {
    let r = tier_record(v);
    let mut out = Writer::new(skein_lib::usize_of(r.measure()));
    r.encode(&mut out).expect("measured tier");
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
    w::RepositoryGates::new(
        &w::CEILINGS,
        w::RepositoryGatesParts {
            forge: v.repository.forge,
            repository: v.repository.repository,
            gates: gates.finish(),
            tiers: tiers.finish(),
        },
    )
    .expect("admitted repository gates")
}
pub fn gates_record(v: &ProjectGates) -> w::ProjectGates {
    let mut out = Writer::new(skein_lib::usize_of(
        gates_bytes(v).expect("admitted gates size").checked_sub(4).expect("record prefix"),
    ));
    for repo in &v.repositories {
        let r = repository_record(repo);
        let mut item = Writer::new(skein_lib::usize_of(r.measure()));
        r.encode(&mut item).expect("measured repository");
        append(&mut out, &item.finish());
    }
    w::ProjectGates::new(&w::CEILINGS, w::ProjectGatesParts { repositories: out.finish() })
        .expect("admitted project gates")
}
pub fn gates_from(v: w::ProjectGates) -> Option<ProjectGates> {
    let bytes = v.into_parts().repositories;
    let mut input = Reader::new(&bytes);
    let mut charged = 4;
    let mut repositories = List::uncapped();
    while let Some(bytes) = skein_codec::piece::split_within(&mut input, &mut charged, u64::from(CARRIER)).ok()? {
        let mut r = Reader::new(&bytes);
        let v = w::RepositoryGates::decode(&w::CEILINGS, &mut r).ok()?.into_parts();
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
            gates.push(gate_from(gate)).ok()?;
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
            tiers.push(tier_from(tier)).ok()?;
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
    if !(60_000..=86_400_000).contains(&v.gate_timeout_ms) {
        return Err(refuse(b"gate_timeout_ms", b"must be 60000 to 86400000"));
    }
    if v.landing_attempts == 0 {
        return Err(refuse(b"landing_attempts", b"must be 1 to 255"));
    }
    for (field, path) in [(b"merge_job".as_slice(), &v.merge_job), (b"close_job".as_slice(), &v.close_job)] {
        if let Some(path) = path
            && !job(path)
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
fn job(path: &[u8]) -> bool {
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
