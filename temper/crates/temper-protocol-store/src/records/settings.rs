//! The forge's kinds (temper/docs/design/protocol/store.md, 2; temper/docs/design/domain/forge/change/gates.md, 3).
use super::{Bounds, FITS, Undecodable, w};
use skein_codec::Record;
use skein_lib::{List, Reader, Writer};
use temper_engine_domain_forge_vocabulary::{GateLast, GatePattern, JudgeFreshness, settings as d};

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
    let last = match last {
        GateLast::Exact(name) => w::PatternLast::Exact(w::Segment { name: name.clone() }),
        GateLast::Open(name) => w::PatternLast::Open(w::Segment { name: name.clone() }),
    };
    w::RulePattern { segments: writer.finish(), last }
}
fn decode_pattern(value: w::RulePattern, bounds: &Bounds) -> Result<GatePattern, Undecodable> {
    let w::RulePattern { segments, last } = value;
    let mut reader = Reader::new(&segments);
    let mut parts = List::with_capacity(bounds.forge().authority_segments);
    for _ in 0..=bounds.forge().authority_segments {
        let piece = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))?;
        let Some(piece) = piece else {
            let last = match last {
                w::PatternLast::Exact(segment) => GateLast::Exact(segment.name),
                w::PatternLast::Open(segment) => GateLast::Open(segment.name),
            };
            return Ok(GatePattern { segments: parts.into_boxed(), last });
        };
        let part = w::Segment::decode(bounds.limits(), &mut Reader::new(&piece)).or(Err(Undecodable::Malformed))?;
        parts.push(part.name).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_rule(value: &d::LandingRule) -> w::LandingRule {
    let d::LandingRule { pattern, enforce, ci, up_to_date, gates, approvals, sets } = value;
    let mut encoded_gates = List::with_capacity(u32::try_from(gates.len()).expect("bounded gates"));
    for gate in gates {
        let d::RuleGate { number, blocking, freshness } = gate;
        encoded_gates
            .push(w::RuleGate { number: *number, blocking: *blocking, freshness: encode_freshness(*freshness) })
            .expect("same count");
    }
    let mut encoded_approvals = List::with_capacity(u32::try_from(approvals.len()).expect("bounded approvals"));
    for approval in approvals {
        let d::RuleApproval { role, people, freshness } = approval;
        encoded_approvals
            .push(w::RuleApproval { role: *role, people: *people, freshness: encode_freshness(*freshness) })
            .expect("same count");
    }
    w::LandingRule {
        pattern: encode_pattern(pattern),
        enforce: *enforce,
        ci: *ci,
        up_to_date: *up_to_date,
        gates: encoded_gates,
        approvals: encoded_approvals,
        sets: List::from_boxed(sets.clone()),
    }
}
fn decode_rule(value: w::LandingRule, bounds: &Bounds) -> Result<d::LandingRule, Undecodable> {
    let w::LandingRule { pattern, enforce, ci, up_to_date, gates, approvals, sets } = value;
    let mut decoded_gates = List::with_capacity(gates.len());
    for gate in gates.into_boxed() {
        let w::RuleGate { number, blocking, freshness } = gate;
        decoded_gates
            .push(d::RuleGate { number, blocking, freshness: decode_freshness(freshness) })
            .expect("same count");
    }
    let mut decoded_approvals = List::with_capacity(approvals.len());
    for approval in approvals.into_boxed() {
        let w::RuleApproval { role, people, freshness } = approval;
        decoded_approvals
            .push(d::RuleApproval { role, people, freshness: decode_freshness(freshness) })
            .expect("same count");
    }
    Ok(d::LandingRule {
        sets: sets.into_boxed(),
        pattern: decode_pattern(pattern, bounds)?,
        enforce,
        ci,
        up_to_date,
        gates: decoded_gates.into_boxed(),
        approvals: decoded_approvals.into_boxed(),
    })
}
pub(super) fn encode(value: &d::SettingRow) -> w::SettingRow {
    let d::SettingRow { key: d::SettingKey { kind, scope, number, slot }, edit } = value;
    let slot = match slot {
        d::SettingSlot::Default => w::SettingSlot::Default,
        d::SettingSlot::Change => w::SettingSlot::Change,
        d::SettingSlot::Replaced(number) => w::SettingSlot::Replaced(w::Number { number: *number }),
        d::SettingSlot::Proposal(number) => w::SettingSlot::Proposal(w::Number { number: *number }),
    };
    let edit = match edit {
        d::Edit::Set(d::SizedValue { value, size }) => {
            let value = match value {
                d::Value::Gates(v) => {
                    let record = d::gates_record(v);
                    let mut out = Writer::new(skein_lib::usize_of(record.measure()));
                    record.encode(&mut out).expect(FITS);
                    w::SettingValue::ProjectGates(
                        w::ProjectGates::decode(&w::CEILINGS, &mut Reader::new(&out.finish()))
                            .expect("matching landing settings records"),
                    )
                }
                d::Value::Landing(v) => {
                    let record = d::landing_record(v);
                    let mut out = Writer::new(skein_lib::usize_of(record.measure()));
                    record.encode(&mut out).expect(FITS);
                    w::SettingValue::ProjectLanding(
                        w::ProjectLanding::decode(&w::CEILINGS, &mut Reader::new(&out.finish()))
                            .expect("matching landing settings records"),
                    )
                }
                d::Value::Classes(classes) => {
                    let record = temper_protocol_reaches::classes_record(classes).expect(FITS);
                    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured classes length"));
                    record.encode(&mut writer).expect("measured classes room");
                    w::SettingValue::Classes(
                        w::ProjectClasses::decode(&w::CEILINGS, &mut Reader::new(&writer.finish()))
                            .expect("matching table records"),
                    )
                }
                d::Value::Dependency(value) => w::SettingValue::Dependency(w::Dependency { project: value.project }),
                d::Value::Follow(follow) => {
                    let record = d::follow_record(follow).expect(FITS);
                    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured follow length"));
                    record.encode(&mut writer).expect("measured follow room");
                    w::SettingValue::Follow(
                        w::Follow::decode(&w::CEILINGS, &mut Reader::new(&writer.finish()))
                            .expect("matching follow records"),
                    )
                }
                d::Value::Rule(rule) => w::SettingValue::Rule(encode_rule(rule)),
                d::Value::Questions(route) => w::SettingValue::Questions(w::ProjectQuestions {
                    route: match route {
                        temper_engine_domain_forge_change::Route::None => w::Route::None,
                        temper_engine_domain_forge_change::Route::Up => w::Route::Up,
                    },
                }),
                d::Value::Ways(ways) => w::SettingValue::Ways(w::ProjectWays {
                    drafts: ways.drafts,
                    goals: ways.goals,
                    changes: ways.changes,
                    way: ways.way,
                }),
                d::Value::Limits(values) => {
                    let mut overrides = List::with_capacity(jig_domain_core_vocabulary::settings::OVERRIDES);
                    for value in values {
                        overrides
                            .push(w::LimitOverride {
                                name: value.name.clone(),
                                value: value.value,
                                reason: value.reason.clone(),
                            })
                            .expect(FITS);
                    }
                    w::SettingValue::Limits(w::LimitOverrides { overrides })
                }
                d::Value::CharterWords(line) => w::SettingValue::Line(w::CharterWords { line: line.clone() }),
                d::Value::ProjectWords(words) => w::SettingValue::Words(w::ProjectWords {
                    headline: words.headline.clone(),
                    lede: words.lede.clone(),
                }),
                d::Value::Goal(goal) => {
                    w::SettingValue::Goal(w::TemplateGoal { design: goal.design, close_out: goal.close_out })
                }
                d::Value::AgentFigures(value) => {
                    let mut writer = Writer::new(usize::try_from(value.measure()).expect("record size fits"));
                    value.encode(&mut writer).expect(FITS);
                    w::SettingValue::Agents(
                        w::AgentFigures::decode(&w::CEILINGS, &mut Reader::new(&writer.finish())).expect(FITS),
                    )
                }
                d::Value::Welcome(value) => w::SettingValue::Welcome(w::Welcome { shown: value.shown }),
                d::Value::ProjectAreas(value) => w::SettingValue::Areas(wire_areas(value)),
                d::Value::ProjectTriager(value) => w::SettingValue::Triager(wire_triager(value)),
                d::Value::Charter(number) => w::SettingValue::Charter(w::CharterValue { number: *number }),
            };
            w::SettingEdit::Set(w::SizedValue { value, size: *size })
        }
        d::Edit::Remove => w::SettingEdit::Remove,
        d::Edit::Reset => w::SettingEdit::Reset,
    };
    w::SettingRow { kind: *kind, scope: *scope, number: *number, slot, edit }
}
#[expect(clippy::too_many_lines, reason = "the setting decoder keeps each codec arm and retirement check together")]
pub(super) fn decode(value: w::SettingRow, bounds: &Bounds) -> Result<d::SettingRow, Undecodable> {
    let w::SettingRow { kind, scope, number, slot, edit } = value;
    d::Kind::of(kind).ok_or(Undecodable::Malformed)?;
    let refused = Undecodable::Setting(d::SettingKey {
        kind,
        scope,
        number,
        slot: match &slot {
            w::SettingSlot::Default => d::SettingSlot::Default,
            w::SettingSlot::Change => d::SettingSlot::Change,
            w::SettingSlot::Replaced(value) => d::SettingSlot::Replaced(value.number),
            w::SettingSlot::Proposal(value) => d::SettingSlot::Proposal(value.number),
        },
    });
    if kind == 3 && number == 3 {
        return Err(refused);
    }
    let slot = match slot {
        w::SettingSlot::Default => d::SettingSlot::Default,
        w::SettingSlot::Change => d::SettingSlot::Change,
        w::SettingSlot::Replaced(number) => d::SettingSlot::Replaced(number.number),
        w::SettingSlot::Proposal(number) => d::SettingSlot::Proposal(number.number),
    };
    let edit = match edit {
        w::SettingEdit::Set(value) => {
            let w::SizedValue { value, size } = value;
            let value = match value {
                w::SettingValue::ProjectGates(record) => {
                    use temper_client_protocol_wire::v1;
                    let bytes = record.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
                    let record = v1::ProjectGates::decode(&v1::CEILINGS, &mut Reader::new(&bytes))
                        .or(Err(Undecodable::Malformed))?;
                    d::Value::Gates(d::gates_from(record).ok_or(Undecodable::Malformed)?)
                }
                w::SettingValue::ProjectLanding(record) => {
                    use temper_client_protocol_wire::v1;
                    let bytes = record.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
                    let record = v1::ProjectLanding::decode(&v1::CEILINGS, &mut Reader::new(&bytes))
                        .or(Err(Undecodable::Malformed))?;
                    d::Value::Landing(d::landing_from(record))
                }
                w::SettingValue::Follow(record) => {
                    use temper_client_protocol_wire::v1;
                    let bytes = record.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
                    let record =
                        v1::Follow::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).or(Err(Undecodable::Malformed))?;
                    d::Value::Follow(d::follow_from(record))
                }
                w::SettingValue::Dependency(value) => d::Value::Dependency(d::Dependency { project: value.project }),
                w::SettingValue::Rule(rule) => d::Value::Rule(decode_rule(rule, bounds)?),
                w::SettingValue::Questions(value) => d::Value::Questions(match value.route {
                    w::Route::None => temper_engine_domain_forge_change::Route::None,
                    w::Route::Retired => return Err(refused),
                    w::Route::Up => temper_engine_domain_forge_change::Route::Up,
                }),
                w::SettingValue::Ways(value) => {
                    let w::ProjectWays { drafts, goals, changes, way } = value;
                    d::Value::Ways(d::ProjectWays { drafts, goals, changes, way })
                }
                w::SettingValue::Charter(number) => d::Value::Charter(number.number),
                w::SettingValue::Classes(record) => {
                    use temper_client_protocol_wire::v1;
                    let bytes = record.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
                    let record = v1::ProjectClasses::decode(&v1::CEILINGS, &mut Reader::new(&bytes))
                        .or(Err(Undecodable::Malformed))?;
                    let classes = temper_protocol_reaches::classes_from(&record);
                    classes.check().or(Err(Undecodable::Malformed))?;

                    d::Value::Classes(classes)
                }
                w::SettingValue::Limits(record) => {
                    let mut overrides = List::with_capacity(jig_domain_core_vocabulary::settings::OVERRIDES);
                    for value in record.overrides.into_boxed() {
                        let w::LimitOverride { name, value, reason } = value;
                        overrides
                            .push(jig_domain_core_vocabulary::settings::Override { name, value, reason })
                            .or(Err(Undecodable::Malformed))?;
                    }
                    jig_domain_core_vocabulary::settings::check_overrides(overrides.as_slice())
                        .or(Err(Undecodable::Malformed))?;
                    d::Value::Limits(overrides.into_boxed())
                }
                w::SettingValue::Line(value) => {
                    let line = value.line;
                    if !d::words_fit(&line, d::LINE_BYTES) {
                        return Err(Undecodable::Malformed);
                    }
                    d::Value::CharterWords(line)
                }
                w::SettingValue::Words(value) => {
                    let w::ProjectWords { headline, lede } = value;
                    if !d::words_fit(&headline, d::HEADLINE_BYTES) || !d::words_fit(&lede, d::LEDE_BYTES) {
                        return Err(Undecodable::Malformed);
                    }
                    d::Value::ProjectWords(d::ProjectWords { headline, lede })
                }
                w::SettingValue::Agents(value) => {
                    use temper_client_protocol_wire::v1;
                    let bytes = value.encode(&w::CEILINGS).or(Err(Undecodable::Malformed))?;
                    d::Value::AgentFigures(
                        v1::AgentFigures::decode(&v1::CEILINGS, &mut Reader::new(&bytes))
                            .or(Err(Undecodable::Malformed))?,
                    )
                }
                w::SettingValue::Welcome(value) => d::Value::Welcome(d::Welcome { shown: value.shown }),
                w::SettingValue::Areas(value) => d::Value::ProjectAreas(areas_from(value)),
                w::SettingValue::Triager(value) => {
                    d::Value::ProjectTriager(d::ProjectTriager { owner: owner_from(value.owner) })
                }
                w::SettingValue::Goal(value) => {
                    let w::TemplateGoal { design, close_out } = value;
                    d::Value::Goal(d::TemplateGoal { design, close_out })
                }
            };
            d::Edit::Set(d::SizedValue { value, size })
        }
        w::SettingEdit::Remove => d::Edit::Remove,
        w::SettingEdit::Reset => d::Edit::Reset,
    };
    Ok(d::SettingRow { key: d::SettingKey { kind, scope, number, slot }, edit })
}

fn owner_record(value: d::AreaOwner) -> w::AreaOwner {
    match value {
        d::AreaOwner::Task(number) => w::AreaOwner::Task(w::AreaTask { number }),
        d::AreaOwner::Party(number) => w::AreaOwner::Party(w::AreaParty { number }),
        d::AreaOwner::Role(number) => w::AreaOwner::Role(w::AreaRole { number }),
        d::AreaOwner::Charter(number) => w::AreaOwner::Charter(w::AreaCharter { number }),
    }
}
fn owner_from(value: w::AreaOwner) -> d::AreaOwner {
    match value {
        w::AreaOwner::Task(number) => d::AreaOwner::Task(number.number),
        w::AreaOwner::Party(number) => d::AreaOwner::Party(number.number),
        w::AreaOwner::Role(number) => d::AreaOwner::Role(number.number),
        w::AreaOwner::Charter(number) => d::AreaOwner::Charter(number.number),
    }
}
fn wire_areas(value: &d::ProjectAreas) -> w::ProjectAreas {
    let mut areas = List::with_capacity(u32::try_from(value.areas.len()).expect("admitted area count"));
    for area in &value.areas {
        areas
            .push(w::Area { name: area.name.clone(), words: area.words.clone(), owner: owner_record(area.owner) })
            .expect("counted areas");
    }
    w::ProjectAreas { areas }
}
fn wire_triager(value: &d::ProjectTriager) -> w::ProjectTriager {
    w::ProjectTriager { owner: owner_record(value.owner) }
}
fn areas_from(value: w::ProjectAreas) -> d::ProjectAreas {
    let mut areas = List::uncapped();
    for area in value.areas.into_boxed() {
        let w::Area { name, words, owner } = area;
        areas.push(d::Area { name, words, owner: owner_from(owner) }).expect("decoded areas");
    }
    d::ProjectAreas { areas: areas.into_boxed() }
}
