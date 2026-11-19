//! Settings taken at the next start (temper/docs/design/domain/landing-settings.md, 1.1).
use crate::file::{Decoder, Error, Fields, error, field, optional};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;
mod repairs;
const MOST: u32 = jig_client_protocol_wire::CEILINGS.application_value_bytes;
fn number(d: &Decoder<'_>, fields: &Fields, name: &[u8], default: u64) -> Result<u64, Error> {
    match optional(fields, name) {
        Some(at) => match d.number(at) {
            Ok(v) => Ok(v),
            Err(why) => Err(why.at(name)),
        },
        None => Ok(default),
    }
}
fn text(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<Option<Box<[u8]>>, Error> {
    match optional(fields, name) {
        Some(at) => match d.string(at) {
            Ok(v) => Ok(Some(v.into())),
            Err(why) => Err(why.at(name)),
        },
        None => Ok(None),
    }
}
fn required_text(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<Box<[u8]>, Error> {
    match d.string(field(fields, name)?) {
        Ok(v) => Ok(v.into()),
        Err(why) => Err(why.at(name)),
    }
}
fn boolean(d: &Decoder<'_>, fields: &Fields, name: &[u8], default: bool) -> Result<bool, Error> {
    match optional(fields, name) {
        Some(at) => match d.boolean(at) {
            Ok(v) => Ok(v),
            Err(why) => Err(why.at(name)),
        },
        None => Ok(default),
    }
}
fn combine(
    d: &Decoder<'_>,
    fields: &Fields,
    name: &[u8],
    default: w::SettingsCombine,
) -> Result<w::SettingsCombine, Error> {
    match text(d, fields, name)?.as_deref() {
        None => Ok(default),
        Some(b"merge") => Ok(w::SettingsCombine::Merge),
        Some(b"squash") => Ok(w::SettingsCombine::Squash),
        Some(_) => Err(error(name, b"merge or squash required")),
    }
}
pub(super) fn landing(d: &Decoder<'_>, at: u32) -> Result<w::ProjectLanding, Error> {
    let f = d.object(
        at,
        &[
            b"by",
            b"batch",
            b"combine",
            b"merge_job",
            b"close_job",
            b"landing_attempts",
            b"check_timeout_ms",
            b"gate_retries",
            b"gate_timeout_ms",
            b"tier_retries",
            b"verification_bound_ms",
            b"wait",
            b"repairs",
        ],
    )?;
    let by = match text(d, &f, b"by")?.as_deref() {
        None | Some(b"queue") => w::SettingsLandingWay::Queue,
        Some(b"turn") => w::SettingsLandingWay::Turn,
        Some(_) => return Err(error(b"by", b"queue or turn required")),
    };
    let wait = match text(d, &f, b"wait")?.as_deref() {
        None | Some(b"cleared") => w::SettingsLandingWait::Cleared,
        Some(b"landed") => w::SettingsLandingWait::Landed,
        Some(_) => return Err(error(b"wait", b"cleared or landed required")),
    };
    let (combine_main, combine_other) = match optional(&f, b"combine") {
        None => (w::SettingsCombine::Merge, w::SettingsCombine::Squash),
        Some(at) => {
            let c = d.object(at, &[b"main", b"other"])?;
            (combine(d, &c, b"main", w::SettingsCombine::Merge)?, combine(d, &c, b"other", w::SettingsCombine::Squash)?)
        }
    };
    let check_timeout_ms = match optional(&f, b"check_timeout_ms") {
        Some(at) => Some(d.number(at)?),
        None => None,
    };
    Ok(
        match w::ProjectLanding::new(
            &d.kinds,
            w::ProjectLandingParts {
                by,
                wait,
                repairs: repairs::read(d, &f)?,
                batch: u16::try_from(number(d, &f, b"batch", 1)?).or(Err(error(b"batch", b"u16 bound")))?,
                combine_main,
                combine_other,
                merge_job: text(d, &f, b"merge_job")?,
                close_job: text(d, &f, b"close_job")?,
                landing_attempts: u8::try_from(number(d, &f, b"landing_attempts", 3)?)
                    .or(Err(error(b"landing_attempts", b"u8 bound")))?,
                check_timeout_ms,
                gate_retries: u8::try_from(number(d, &f, b"gate_retries", 2)?)
                    .or(Err(error(b"gate_retries", b"u8 bound")))?,
                gate_timeout_ms: number(d, &f, b"gate_timeout_ms", 3_600_000)?,
                tier_retries: u8::try_from(number(d, &f, b"tier_retries", 2)?)
                    .or(Err(error(b"tier_retries", b"u8 bound")))?,
                verification_bound_ms: number(d, &f, b"verification_bound_ms", 7_200_000)?,
            },
        ) {
            Ok(value) => value,
            Err(why) => return Err(record_bound(why)),
        },
    )
}
fn paths(d: &Decoder<'_>, f: &Fields) -> Result<List<Box<[u8]>>, Error> {
    let Some(at) = optional(f, b"ignored") else {
        return Ok(List::with_capacity(0));
    };
    let ats = d.array(at, MOST, b"ignored")?;
    let mut out = List::with_capacity(ats.len());
    for at in &ats {
        out.push(d.string(*at)?.into()).expect("counted paths");
    }
    Ok(out)
}
fn format(d: &Decoder<'_>, f: &Fields) -> Result<w::SettingsReportFormat, Error> {
    match d.string(field(f, b"format")?)? {
        b"junit" => Ok(w::SettingsReportFormat::Junit),
        b"lines" => Ok(w::SettingsReportFormat::Lines),
        _ => Err(error(b"format", b"junit or lines required")),
    }
}
fn report(d: &Decoder<'_>, at: u32) -> Result<w::SettingsGateReport, Error> {
    let f = d.object(at, &[b"artifact", b"format", b"fuzzy"])?;
    Ok(
        match w::SettingsGateReport::new(
            &d.kinds,
            w::SettingsGateReportParts {
                artifact: required_text(d, &f, b"artifact")?,
                format: format(d, &f)?,
                fuzzy: text(d, &f, b"fuzzy")?,
            },
        ) {
            Ok(value) => value,
            Err(why) => return Err(record_bound(why)),
        },
    )
}
fn gate(d: &Decoder<'_>, at: u32) -> Result<Box<[u8]>, Error> {
    let f = d.object(at, &[b"name", b"workflow", b"ignored", b"blocking", b"base", b"report"])?;
    let report = match optional(&f, b"report") {
        Some(at) => Some(report(d, at)?),
        None => None,
    };
    let r = match w::SettingsGate::new(
        &d.kinds,
        w::SettingsGateParts {
            name: required_text(d, &f, b"name")?,
            workflow: required_text(d, &f, b"workflow")?,
            ignored: paths(d, &f)?,
            blocking: super::boolean(d, &f, b"blocking")?,
            base: boolean(d, &f, b"base", false)?,
            report,
        },
    ) {
        Ok(value) => value,
        Err(why) => return Err(record_bound(why)),
    };
    let mut out = Writer::new(skein_lib::usize_of(r.measure()));
    r.encode(&mut out).expect("measured gate");
    Ok(out.finish())
}
fn tier(d: &Decoder<'_>, at: u32) -> Result<Box<[u8]>, Error> {
    let f = d.object(
        at,
        &[b"name", b"workflow", b"ignored", b"artifact", b"format", b"branch", b"span_ms", b"description"],
    )?;
    let follows = match text(d, &f, b"branch")? {
        None => w::SettingsFollows::Landing,
        Some(b) if b.as_ref() == b"goal" => w::SettingsFollows::Goals,
        Some(name) => w::SettingsFollows::Branch(
            w::SettingsBranch::new(&d.kinds, w::SettingsBranchParts { name })
                .or(Err(error(b"branch", b"wire value bound")))?,
        ),
    };
    let description = text(d, &f, b"description")?;
    if let Some(value) = &description
        && value.is_empty()
    {
        return Err(error(b"description", b"must be 1 to 120 bytes"));
    }
    let r = match w::SettingsTier::new(
        &d.kinds,
        w::SettingsTierParts {
            name: required_text(d, &f, b"name")?,
            workflow: required_text(d, &f, b"workflow")?,
            ignored: paths(d, &f)?,
            artifact: required_text(d, &f, b"artifact")?,
            format: format(d, &f)?,
            follows,
            span_ms: number(d, &f, b"span_ms", 0)?,
            description: description.unwrap_or_default(),
        },
    ) {
        Ok(value) => value,
        Err(why) => return Err(record_bound(why)),
    };
    let mut out = Writer::new(skein_lib::usize_of(r.measure()));
    r.encode(&mut out).expect("measured tier");
    Ok(out.finish())
}
fn pack(items: List<Box<[u8]>>) -> Result<Box<[u8]>, Error> {
    let mut size = 0_u32;
    for bytes in &items {
        size = size
            .checked_add(4)
            .ok_or(error(b"repositories", b"settings carrier bound"))?
            .checked_add(u32::try_from(bytes.len()).or(Err(error(b"repositories", b"settings carrier bound")))?)
            .ok_or(error(b"repositories", b"settings carrier bound"))?;
    }
    if size > MOST {
        return Err(error(b"repositories", b"settings carrier bound"));
    }
    let mut out = Writer::new(skein_lib::usize_of(size));
    for bytes in &items {
        skein_codec::piece::write(&mut out, bytes).expect("measured pieces");
    }
    Ok(out.finish())
}
fn checks(d: &Decoder<'_>, f: &Fields, name: &[u8], gates: bool) -> Result<Box<[u8]>, Error> {
    let mut items = List::uncapped();
    if let Some(at) = optional(f, name) {
        for at in &d.array(at, MOST, name)? {
            items.push(if gates { gate(d, *at)? } else { tier(d, *at)? }).expect("counted checks");
        }
    }
    pack(items)
}
pub(super) fn gates(d: &Decoder<'_>, at: u32) -> Result<w::ProjectGates, Error> {
    let f = d.object(at, &[b"repositories"])?;
    let mut items = List::uncapped();
    for at in &d.array(field(&f, b"repositories")?, MOST, b"repositories")? {
        let f = d.object(*at, &[b"forge", b"repository", b"gates", b"tiers"])?;
        let r = match w::RepositoryGates::new(
            &d.kinds,
            w::RepositoryGatesParts {
                forge: u16::try_from(super::number(d, &f, b"forge")?).or(Err(error(b"forge", b"u16 bound")))?,
                repository: super::number(d, &f, b"repository")?,
                gates: checks(d, &f, b"gates", true)?,
                tiers: checks(d, &f, b"tiers", false)?,
            },
        ) {
            Ok(value) => value,
            Err(why) => return Err(record_bound(why)),
        };
        let mut out = Writer::new(skein_lib::usize_of(r.measure()));
        r.encode(&mut out).expect("measured repository");
        items.push(out.finish()).expect("counted repositories");
    }
    let repositories = pack(items)?;
    if repositories.len() > skein_lib::usize_of(MOST.saturating_sub(4)) {
        return Err(error(b"repositories", b"settings carrier bound"));
    }
    Ok(match w::ProjectGates::new(&d.kinds, w::ProjectGatesParts { repositories }) {
        Ok(value) => value,
        Err(why) => return Err(record_bound(why)),
    })
}
fn string(out: &mut Encoder, key: &[u8], value: &[u8]) {
    out.key(key);
    out.string(value);
}
fn uint(out: &mut Encoder, key: &[u8], value: u64) {
    out.key(key);
    out.unsigned(value);
}
pub(super) fn render_landing(v: &w::ProjectLanding, out: &mut Encoder) {
    out.object_start();
    if *v.by() != w::SettingsLandingWay::Queue {
        string(out, b"by", b"turn");
    }
    if v.batch() != 1 {
        uint(out, b"batch", u64::from(v.batch()));
    }
    if *v.combine_main() != w::SettingsCombine::Merge || *v.combine_other() != w::SettingsCombine::Squash {
        out.key(b"combine");
        out.object_start();
        string(out, b"main", combine_word(v.combine_main()));
        string(out, b"other", combine_word(v.combine_other()));
        out.object_end();
    }
    if let Some(path) = v.merge_job() {
        string(out, b"merge_job", path);
    }
    if let Some(path) = v.close_job() {
        string(out, b"close_job", path);
    }
    if v.landing_attempts() != 3 {
        uint(out, b"landing_attempts", u64::from(v.landing_attempts()));
    }
    if let Some(ms) = v.check_timeout_ms() {
        uint(out, b"check_timeout_ms", *ms);
    }
    for (name, value, default) in [
        (b"gate_retries".as_slice(), u64::from(v.gate_retries()), 2),
        (b"gate_timeout_ms", v.gate_timeout_ms(), 3_600_000),
        (b"tier_retries", u64::from(v.tier_retries()), 2),
        (b"verification_bound_ms", v.verification_bound_ms(), 7_200_000),
    ] {
        if value != default {
            uint(out, name, value);
        }
    }
    if *v.wait() != w::SettingsLandingWait::Cleared {
        string(out, b"wait", b"landed");
    }
    if let Some(repairs) = v.repairs() {
        out.key(b"repairs");
        repairs::render(repairs, out);
    }
    out.object_end();
}
fn combine_word(v: &w::SettingsCombine) -> &'static [u8] {
    match v {
        w::SettingsCombine::Merge => b"merge",
        w::SettingsCombine::Squash => b"squash",
    }
}
fn format_word(v: &w::SettingsReportFormat) -> &'static [u8] {
    match v {
        w::SettingsReportFormat::Junit => b"junit",
        w::SettingsReportFormat::Lines => b"lines",
    }
}
fn render_check(name: &[u8], workflow: &[u8], paths: &List<Box<[u8]>>, out: &mut Encoder) {
    string(out, b"name", name);
    string(out, b"workflow", workflow);
    if !paths.is_empty() {
        out.key(b"ignored");
        out.array_start();
        for path in paths {
            out.string(path);
        }
        out.array_end();
    }
}
pub(super) fn render_gates(v: &w::ProjectGates, out: &mut Encoder) -> bool {
    let mut check = Encoder::measure(&skein_json::writer::Limits { depth: 32, length: u32::MAX });
    if render_gates_checked(v, &mut check).is_none() {
        return false;
    }
    render_gates_checked(v, out).is_some()
}
fn render_gates_checked(v: &w::ProjectGates, out: &mut Encoder) -> Option<()> {
    let mut input = Reader::new(v.repositories());
    let mut charged = 4;
    out.object_start();
    out.key(b"repositories");
    out.array_start();
    while let Some(bytes) = skein_codec::piece::split_within(&mut input, &mut charged, u64::from(MOST)).ok()? {
        let mut reader = Reader::new(&bytes);
        let v = w::RepositoryGates::decode(&w::CEILINGS, &mut reader).ok()?;
        if !reader.is_empty() {
            return None;
        }
        out.object_start();
        uint(out, b"forge", u64::from(v.forge()));
        uint(out, b"repository", u64::from(v.repository()));
        out.key(b"gates");
        out.array_start();
        let mut reader = Reader::new(v.gates());
        let mut sum = 0;
        while let Some(bytes) = skein_codec::piece::split_within(&mut reader, &mut sum, u64::from(MOST)).ok()? {
            let mut r = Reader::new(&bytes);
            let gate = w::SettingsGate::decode(&w::CEILINGS, &mut r).ok()?;
            if !r.is_empty() {
                return None;
            }
            out.object_start();
            render_check(gate.name(), gate.workflow(), gate.ignored(), out);
            out.key(b"blocking");
            out.boolean(gate.blocking());
            if gate.base() {
                out.key(b"base");
                out.boolean(true);
            }
            if let Some(report) = gate.report() {
                out.key(b"report");
                out.object_start();
                string(out, b"artifact", report.artifact());
                string(out, b"format", format_word(report.format()));
                if let Some(fuzzy) = report.fuzzy() {
                    string(out, b"fuzzy", fuzzy);
                }
                out.object_end();
            }
            out.object_end();
        }
        out.array_end();
        out.key(b"tiers");
        out.array_start();
        let mut reader = Reader::new(v.tiers());
        let mut sum = 0;
        while let Some(bytes) = skein_codec::piece::split_within(&mut reader, &mut sum, u64::from(MOST)).ok()? {
            let mut r = Reader::new(&bytes);
            let tier = w::SettingsTier::decode(&w::CEILINGS, &mut r).ok()?;
            if !r.is_empty() {
                return None;
            }
            out.object_start();
            render_check(tier.name(), tier.workflow(), tier.ignored(), out);
            string(out, b"artifact", tier.artifact());
            string(out, b"format", format_word(tier.format()));
            match tier.follows() {
                w::SettingsFollows::Landing => {}
                w::SettingsFollows::Goals => string(out, b"branch", b"goal"),
                w::SettingsFollows::Branch(b) => string(out, b"branch", b.name()),
            }
            if tier.span_ms() != 0 {
                uint(out, b"span_ms", tier.span_ms());
            }
            if !tier.description().is_empty() {
                string(out, b"description", tier.description());
            }
            out.object_end();
        }
        out.array_end();
        out.object_end();
    }
    out.array_end();
    out.object_end();
    Some(())
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "this adapter receives only the landing records, not the other wire records' paths"
)]
fn record_bound(why: w::Problem) -> Error {
    let field = match why.path {
        w::Path::ProjectLandingMergeJob => b"merge_job".as_slice(),
        w::Path::ProjectLandingCloseJob => b"close_job",
        w::Path::SettingsGateName | w::Path::SettingsTierName => b"name",
        w::Path::SettingsGateWorkflow | w::Path::SettingsTierWorkflow => b"workflow",
        w::Path::SettingsGateIgnored | w::Path::SettingsTierIgnored => b"ignored",
        w::Path::SettingsGateReportArtifact | w::Path::SettingsTierArtifact => b"artifact",
        w::Path::SettingsGateReportFuzzy => b"fuzzy",
        w::Path::SettingsTierDescription => b"description",
        w::Path::RepositoryGatesGates => b"gates",
        w::Path::RepositoryGatesTiers => b"tiers",
        _ => b"repositories",
    };
    error(field, b"wire value bound")
}
