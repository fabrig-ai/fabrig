//! The forge's kinds (temper/docs/design/protocol/store.md, 2; temper/docs/design/domain/forge/change/gates.md, 3).
use super::{Bounds, FITS, Undecodable, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_lib::{List, Reader, Writer};
use temper_engine_domain_forge_vocabulary::{GatePattern, JudgeFreshness, settings as d};

fn encode_pattern(value: &GatePattern) -> w::RulePattern {
    let GatePattern { segments, last } = value;
    let mut size = 0_usize;
    for segment in segments {
        size = size.checked_add(segment.len()).expect("bounded segment").checked_add(8).expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for segment in segments {
        let item = w::Segment { name: segment.clone() };
        let encoded = item.encode(&w::CEILINGS).expect(FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    let last = crate::generated::store::pattern_last_to_record(last, &super::Context::UNBOUNDED).expect(FITS);
    w::RulePattern { segments: writer.finish(), last }
}
fn decode_pattern(value: w::RulePattern, bounds: &Bounds) -> Result<GatePattern, Undecodable> {
    let w::RulePattern { segments, last } = value;
    let mut reader = Reader::new(&segments);
    let mut parts = List::with_capacity(bounds.forge().authority_segments);
    for _ in 0..=bounds.forge().authority_segments {
        let piece = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))?;
        let Some(piece) = piece else {
            let last = crate::generated::store::pattern_last_to_domain(last, &super::Context::UNBOUNDED)
                .or(Err(Undecodable::Malformed))?;
            return Ok(GatePattern { segments: parts.into_boxed(), last });
        };
        let part = w::Segment::decode(bounds.limits(), &mut Reader::new(&piece)).or(Err(Undecodable::Malformed))?;
        parts.push(part.name).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

pub(super) fn encode(value: &d::SettingRow) -> w::SettingRow {
    let d::SettingRow { key: d::SettingKey { kind, scope, number, slot }, edit } = value;
    let context = super::Context::UNBOUNDED;
    w::SettingRow {
        kind: *kind,
        scope: *scope,
        number: *number,
        slot: crate::generated::store::setting_slot_to_record(slot, &context).expect(FITS),
        edit: crate::generated::store::setting_edit_to_record(edit, &context).expect(FITS),
    }
}
pub(super) fn decode(value: w::SettingRow, bounds: &Bounds) -> Result<d::SettingRow, Undecodable> {
    let w::SettingRow { kind, scope, number, slot, edit } = value;
    d::Kind::of(kind).ok_or(Undecodable::Malformed)?;
    let context = super::Context::bounded(bounds);
    let slot = crate::generated::store::setting_slot_to_domain(slot, &context).or(Err(Undecodable::Malformed))?;
    let key = d::SettingKey { kind, scope, number, slot };
    if kind == 3 && number == 3 {
        return Err(Undecodable::Setting(key));
    }
    if let w::SettingEdit::Set(value) = &edit
        && let w::SettingValue::Questions(value) = &value.value
        && let w::Route::Retired = value.route
    {
        return Err(Undecodable::Setting(key));
    }
    let edit = crate::generated::store::setting_edit_to_domain(edit, &context).or(Err(Undecodable::Malformed))?;
    Ok(d::SettingRow { key, edit })
}

/// Owner-shaped `RulePattern` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn rule_pattern_to_record(value: &GatePattern, _context: &super::Context) -> Result<w::RulePattern, w::Problem> {
    Ok(encode_pattern(value))
}
pub fn rule_pattern_to_domain(value: w::RulePattern, context: &super::Context) -> Result<GatePattern, w::Problem> {
    decode_pattern(value, context.bounds())
        .or(Err(w::paths::RULE_PATTERN_SEGMENTS.problem(skein_codec::Reason::Malformed)))
}

fn encode_freshness(value: JudgeFreshness) -> w::Freshness {
    match value {
        JudgeFreshness::Exact => w::Freshness::Exact,
        JudgeFreshness::Clean => w::Freshness::Clean,
    }
}
fn decode_freshness(value: w::Freshness) -> JudgeFreshness {
    match value {
        w::Freshness::Exact => JudgeFreshness::Exact,
        w::Freshness::Clean => JudgeFreshness::Clean,
    }
}

/// Owner-shaped `RuleGate` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn rule_gate_to_record(value: &d::RuleGate, _context: &super::Context) -> Result<w::RuleGate, w::Problem> {
    Ok(w::RuleGate { number: value.number, blocking: value.blocking, freshness: encode_freshness(value.freshness) })
}
pub fn rule_gate_to_domain(value: w::RuleGate, _context: &super::Context) -> Result<d::RuleGate, w::Problem> {
    Ok(d::RuleGate { number: value.number, blocking: value.blocking, freshness: decode_freshness(value.freshness) })
}

/// Owner-shaped `RuleApproval` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn rule_approval_to_record(
    value: &d::RuleApproval,
    _context: &super::Context,
) -> Result<w::RuleApproval, w::Problem> {
    Ok(w::RuleApproval { role: value.role, people: value.people, freshness: encode_freshness(value.freshness) })
}
pub fn rule_approval_to_domain(
    value: w::RuleApproval,
    _context: &super::Context,
) -> Result<d::RuleApproval, w::Problem> {
    Ok(d::RuleApproval { role: value.role, people: value.people, freshness: decode_freshness(value.freshness) })
}

fn encode_value(value: &d::Value) -> w::SettingValue {
    match value {
        d::Value::Gates(value) => {
            w::SettingValue::ProjectGates(project_gates_to_record(value, &super::Context::UNBOUNDED).expect(FITS))
        }
        d::Value::Landing(value) => w::SettingValue::ProjectLanding(
            crate::generated::store::project_landing_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Classes(value) => {
            w::SettingValue::Classes(project_classes_to_record(value, &super::Context::UNBOUNDED).expect(FITS))
        }
        d::Value::Dependency(value) => w::SettingValue::Dependency(
            crate::generated::store::dependency_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Follow(value) => w::SettingValue::Follow(
            crate::generated::store::follow_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Rule(value) => w::SettingValue::Rule(
            crate::generated::store::landing_rule_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Ways(value) => w::SettingValue::Ways(
            crate::generated::store::project_ways_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Goal(value) => w::SettingValue::Goal(
            crate::generated::store::template_goal_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::AgentFigures(value) => w::SettingValue::Agents(
            crate::generated::store::agent_figures_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Welcome(value) => w::SettingValue::Welcome(
            crate::generated::store::welcome_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::ProjectAreas(value) => w::SettingValue::Areas(
            crate::generated::store::project_areas_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::ProjectTriager(value) => w::SettingValue::Triager(
            crate::generated::store::project_triager_to_record(value, &super::Context::UNBOUNDED).expect(FITS),
        ),
        d::Value::Questions(route) => w::SettingValue::Questions(w::ProjectQuestions {
            route: super::by_hand::route_to_record(route, &super::Context::UNBOUNDED).expect(FITS),
        }),
        d::Value::Charter(number) => w::SettingValue::Charter(w::CharterValue { number: *number }),
        d::Value::Limits(values) => w::SettingValue::Limits(encode_limits(values)),
        d::Value::CharterWords(line) => w::SettingValue::Line(w::CharterWords { line: line.clone() }),
        d::Value::ProjectWords(words) => {
            w::SettingValue::Words(w::ProjectWords { headline: words.headline.clone(), lede: words.lede.clone() })
        }
    }
}
fn decode_value(value: w::SettingValue, bounds: &Bounds) -> Result<d::Value, Undecodable> {
    Ok(match value {
        w::SettingValue::ProjectGates(value) => d::Value::Gates(
            project_gates_to_domain(value, &super::Context::bounded(bounds)).or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::ProjectLanding(value) => d::Value::Landing(
            crate::generated::store::project_landing_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Classes(value) => d::Value::Classes(
            project_classes_to_domain(value, &super::Context::bounded(bounds)).or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Dependency(value) => d::Value::Dependency(
            crate::generated::store::dependency_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Follow(value) => d::Value::Follow(
            crate::generated::store::follow_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Rule(value) => d::Value::Rule(
            crate::generated::store::landing_rule_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Ways(value) => d::Value::Ways(
            crate::generated::store::project_ways_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Goal(value) => d::Value::Goal(
            crate::generated::store::template_goal_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Agents(value) => d::Value::AgentFigures(
            crate::generated::store::agent_figures_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Welcome(value) => d::Value::Welcome(
            crate::generated::store::welcome_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Areas(value) => d::Value::ProjectAreas(
            crate::generated::store::project_areas_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Triager(value) => d::Value::ProjectTriager(
            crate::generated::store::project_triager_to_domain(value, &super::Context::bounded(bounds))
                .or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Questions(value) => d::Value::Questions(
            super::by_hand::route_to_domain(value.route, &super::Context::UNBOUNDED).or(Err(Undecodable::Malformed))?,
        ),
        w::SettingValue::Charter(value) => d::Value::Charter(value.number),
        w::SettingValue::Limits(value) => d::Value::Limits(decode_limits(value)?),
        w::SettingValue::Line(value) => {
            if !d::words_fit(&value.line, d::LINE_BYTES) {
                return Err(Undecodable::Malformed);
            }
            d::Value::CharterWords(value.line)
        }
        w::SettingValue::Words(value) => {
            if !d::words_fit(&value.headline, d::HEADLINE_BYTES) || !d::words_fit(&value.lede, d::LEDE_BYTES) {
                return Err(Undecodable::Malformed);
            }
            d::Value::ProjectWords(d::ProjectWords { headline: value.headline, lede: value.lede })
        }
    })
}

/// Owner-shaped `SettingValue` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn setting_value_to_record(value: &d::Value, _context: &super::Context) -> Result<w::SettingValue, w::Problem> {
    Ok(encode_value(value))
}
pub fn setting_value_to_domain(value: w::SettingValue, context: &super::Context) -> Result<d::Value, w::Problem> {
    decode_value(value, context.bounds()).or(Err(w::paths::SETTING_VALUE_TAG.problem(skein_codec::Reason::Malformed)))
}

fn encode_limits(values: &[jig_domain_core_vocabulary::settings::Override]) -> w::LimitOverrides {
    let mut overrides = List::with_capacity(jig_domain_core_vocabulary::settings::OVERRIDES);
    for value in values {
        overrides
            .push(crate::generated::store::limit_override_to_record(value, &super::Context::UNBOUNDED).expect(FITS))
            .expect(FITS);
    }
    w::LimitOverrides { overrides }
}
fn decode_limits(
    value: w::LimitOverrides,
) -> Result<Box<[jig_domain_core_vocabulary::settings::Override]>, Undecodable> {
    let mut overrides = List::with_capacity(jig_domain_core_vocabulary::settings::OVERRIDES);
    for value in value.overrides.into_boxed() {
        overrides
            .push(
                crate::generated::store::limit_override_to_domain(value, &super::Context::UNBOUNDED)
                    .or(Err(Undecodable::Malformed))?,
            )
            .or(Err(Undecodable::Malformed))?;
    }
    jig_domain_core_vocabulary::settings::check_overrides(overrides.as_slice()).or(Err(Undecodable::Malformed))?;
    Ok(overrides.into_boxed())
}
/// Class tables require the owner's cross-field checks.
pub fn project_classes_to_record(
    value: &temper_protocol_reaches::Classes,
    context: &super::Context,
) -> Result<w::ProjectClasses, w::Problem> {
    let mut items = List::with_capacity(u32::try_from(value.items.len()).expect(FITS));
    for item in &value.items {
        items.push(crate::generated::store::class_item_to_record(item, context)?).expect(FITS);
    }
    let mut sets = List::with_capacity(u32::try_from(value.sets.len()).expect(FITS));
    for set in &value.sets {
        sets.push(crate::generated::store::path_set_to_record(set, context)?).expect(FITS);
    }
    Ok(w::ProjectClasses { items, sets })
}
pub fn project_classes_to_domain(
    value: w::ProjectClasses,
    context: &super::Context,
) -> Result<temper_protocol_reaches::Classes, w::Problem> {
    let mut items = List::with_capacity(value.items.len());
    for item in value.items.into_boxed() {
        items.push(crate::generated::store::class_item_to_domain(item, context)?).expect(FITS);
    }
    let mut sets = List::with_capacity(value.sets.len());
    for set in value.sets.into_boxed() {
        sets.push(crate::generated::store::path_set_to_domain(set, context)?).expect(FITS);
    }
    let classes = temper_protocol_reaches::Classes { items: items.into_boxed(), sets: sets.into_boxed() };
    classes.check().or(Err(w::paths::PROJECT_CLASSES_ITEMS.problem(skein_codec::Reason::Malformed)))?;
    Ok(classes)
}

/// Gates are framed as pieces under the client carrier budget.
pub fn project_gates_to_record(
    value: &d::ProjectGates,
    context: &super::Context,
) -> Result<w::ProjectGates, w::Problem> {
    let mut pieces = List::uncapped();
    let mut size = 0_usize;
    for repository in &value.repositories {
        let record = repository_gates_to_record(repository, context)?;
        let bytes = record.encode(&w::CEILINGS)?;
        size = size.checked_add(bytes.len()).expect(FITS).checked_add(4).expect(FITS);
        pieces.push(bytes).expect(FITS);
    }
    let mut writer = Writer::new(size);
    for bytes in pieces.into_boxed() {
        skein_codec::piece::write(&mut writer, &bytes).expect(FITS);
    }
    Ok(w::ProjectGates { repositories: writer.finish() })
}
pub fn project_gates_to_domain(
    value: w::ProjectGates,
    context: &super::Context,
) -> Result<d::ProjectGates, w::Problem> {
    let mut reader = Reader::new(&value.repositories);
    let mut charged = 4;
    let mut repositories = List::uncapped();
    while let Some(bytes) = skein_codec::piece::split_within(&mut reader, &mut charged, u64::from(d::CARRIER))
        .or(Err(w::paths::PROJECT_GATES_REPOSITORIES.problem(skein_codec::Reason::Malformed)))?
    {
        let mut reader = Reader::new(&bytes);
        let record = w::RepositoryGates::decode(&w::CEILINGS, &mut reader)?;
        if !reader.is_empty() {
            return Err(w::paths::PROJECT_GATES_REPOSITORIES.problem(skein_codec::Reason::Malformed));
        }
        repositories.push(repository_gates_to_domain(record, context)?).expect(FITS);
    }
    Ok(d::ProjectGates { repositories: repositories.into_boxed() })
}
fn repository_gates_to_record(
    value: &temper_engine_domain_forge_vocabulary::RepositoryGates,
    context: &super::Context,
) -> Result<w::RepositoryGates, w::Problem> {
    let mut pieces = List::uncapped();
    let mut size = 0_usize;
    for gate in &value.gates {
        let bytes = crate::generated::store::settings_gate_to_record(gate, context)?.encode(&w::CEILINGS)?;
        size = size.checked_add(bytes.len()).expect(FITS).checked_add(4).expect(FITS);
        pieces.push(bytes).expect(FITS);
    }
    let mut gates = Writer::new(size);
    for bytes in pieces.into_boxed() {
        skein_codec::piece::write(&mut gates, &bytes).expect(FITS);
    }
    let mut pieces = List::uncapped();
    let mut size = 0_usize;
    for tier in &value.tiers {
        let bytes = settings_tier_to_record(tier, context)?.encode(&w::CEILINGS)?;
        size = size.checked_add(bytes.len()).expect(FITS).checked_add(4).expect(FITS);
        pieces.push(bytes).expect(FITS);
    }
    let mut tiers = Writer::new(size);
    for bytes in pieces.into_boxed() {
        skein_codec::piece::write(&mut tiers, &bytes).expect(FITS);
    }
    Ok(w::RepositoryGates {
        forge: value.repository.forge,
        repository: value.repository.repository,
        gates: gates.finish(),
        tiers: tiers.finish(),
    })
}
fn repository_gates_to_domain(
    value: w::RepositoryGates,
    context: &super::Context,
) -> Result<temper_engine_domain_forge_vocabulary::RepositoryGates, w::Problem> {
    let mut reader = Reader::new(&value.gates);
    let mut charged = 0;
    let mut gates = List::uncapped();
    while let Some(bytes) = skein_codec::piece::split_within(&mut reader, &mut charged, u64::from(d::CARRIER))
        .or(Err(w::paths::REPOSITORY_GATES_GATES.problem(skein_codec::Reason::Malformed)))?
    {
        let mut reader = Reader::new(&bytes);
        let record = w::SettingsGate::decode(&w::CEILINGS, &mut reader)?;
        if !reader.is_empty() {
            return Err(w::paths::REPOSITORY_GATES_GATES.problem(skein_codec::Reason::Malformed));
        }
        gates.push(crate::generated::store::settings_gate_to_domain(record, context)?).expect(FITS);
    }
    let mut reader = Reader::new(&value.tiers);
    let mut charged = 0;
    let mut tiers = List::uncapped();
    while let Some(bytes) = skein_codec::piece::split_within(&mut reader, &mut charged, u64::from(d::CARRIER))
        .or(Err(w::paths::REPOSITORY_GATES_TIERS.problem(skein_codec::Reason::Malformed)))?
    {
        let mut reader = Reader::new(&bytes);
        let record = w::SettingsTier::decode(&w::CEILINGS, &mut reader)?;
        if !reader.is_empty() {
            return Err(w::paths::REPOSITORY_GATES_TIERS.problem(skein_codec::Reason::Malformed));
        }
        tiers.push(settings_tier_to_domain(record, context)?).expect(FITS);
    }
    Ok(temper_engine_domain_forge_vocabulary::RepositoryGates {
        repository: skein_forge::api::Repository { forge: value.forge, repository: value.repository },
        gates: gates.into_boxed(),
        tiers: tiers.into_boxed(),
    })
}
fn settings_tier_to_record(
    value: &temper_engine_domain_forge_vocabulary::Tier,
    context: &super::Context,
) -> Result<w::SettingsTier, w::Problem> {
    Ok(w::SettingsTier {
        name: value.name.clone(),
        workflow: value.workflow.clone(),
        ignored: List::from_boxed(value.ignored.clone()),
        artifact: value.artifact.clone(),
        format: crate::generated::store::settings_report_format_to_record(&value.format, context)?,
        follows: crate::generated::store::settings_follows_to_record(&value.follows, context)?,
        span_ms: match value.span {
            Some(span) => span.as_millis(),
            None => 0,
        },
        description: value.description.clone(),
    })
}
fn settings_tier_to_domain(
    value: w::SettingsTier,
    context: &super::Context,
) -> Result<temper_engine_domain_forge_vocabulary::Tier, w::Problem> {
    Ok(temper_engine_domain_forge_vocabulary::Tier {
        name: value.name,
        workflow: value.workflow,
        ignored: value.ignored.into_boxed(),
        artifact: value.artifact,
        format: crate::generated::store::settings_report_format_to_domain(value.format, context)?,
        follows: crate::generated::store::settings_follows_to_domain(value.follows, context)?,
        span: match value.span_ms {
            0 => None,
            ms => Some(skein_lib::Duration::from_millis(ms)),
        },
        description: value.description,
    })
}
