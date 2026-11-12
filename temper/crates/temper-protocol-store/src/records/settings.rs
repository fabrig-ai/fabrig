//! The forge's kinds (temper/docs/design/protocol/store.md, 2; temper/docs/design/domain/forge/change/gates.md, 3).
use super::{Bounds, FITS, Undecodable, w};
use skein_lib::{List, Reader, Writer};
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness, settings as d};

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
        let item = w::Segment::new(&w::CEILINGS, w::SegmentParts { name: segment.clone() }).expect(FITS);
        let mut encoded = Writer::new(usize::try_from(item.measure()).expect("segment length"));
        item.encode(&mut encoded).expect("measured segment room");
        skein_codec::piece::write(&mut writer, &encoded.finish()).expect("measured piece room");
    }
    let last = match last {
        GateLast::Exact(name) => {
            w::PatternLast::Exact(w::Segment::new(&w::CEILINGS, w::SegmentParts { name: name.clone() }).expect(FITS))
        }
        GateLast::Open(name) => {
            w::PatternLast::Open(w::Segment::new(&w::CEILINGS, w::SegmentParts { name: name.clone() }).expect(FITS))
        }
    };
    w::RulePattern::new(&w::CEILINGS, w::RulePatternParts { segments: writer.finish(), last }).expect(FITS)
}
fn decode_pattern(value: w::RulePattern, bounds: &Bounds) -> Result<GatePattern, Undecodable> {
    let w::RulePatternParts { segments, last } = value.into_parts();
    let mut reader = Reader::new(&segments);
    let mut parts = List::with_capacity(bounds.forge().authority_segments);
    for _ in 0..=bounds.forge().authority_segments {
        let piece = skein_codec::piece::split(&mut reader).or(Err(Undecodable))?;
        let Some(piece) = piece else {
            let last = match last {
                w::PatternLast::Exact(segment) => GateLast::Exact(segment.into_parts().name),
                w::PatternLast::Open(segment) => GateLast::Open(segment.into_parts().name),
            };
            return Ok(GatePattern { segments: parts.into_boxed(), last });
        };
        let part = w::Segment::decode(bounds.limits(), &mut Reader::new(&piece)).or(Err(Undecodable))?;
        parts.push(part.into_parts().name).or(Err(Undecodable))?;
    }
    Err(Undecodable)
}
fn encode_rule(value: &d::LandingRule) -> w::LandingRule {
    let d::LandingRule { pattern, enforce, ci, up_to_date, gates, approvals } = value;
    let mut encoded_gates = List::with_capacity(u32::try_from(gates.len()).expect("bounded gates"));
    for gate in gates {
        let d::RuleGate { number, blocking, freshness } = gate;
        encoded_gates
            .push(
                w::RuleGate::new(
                    &w::CEILINGS,
                    w::RuleGateParts { number: *number, blocking: *blocking, freshness: encode_freshness(*freshness) },
                )
                .expect(FITS),
            )
            .expect("same count");
    }
    let mut encoded_approvals = List::with_capacity(u32::try_from(approvals.len()).expect("bounded approvals"));
    for approval in approvals {
        let d::RuleApproval { role, people, freshness } = approval;
        encoded_approvals
            .push(
                w::RuleApproval::new(
                    &w::CEILINGS,
                    w::RuleApprovalParts { role: *role, people: *people, freshness: encode_freshness(*freshness) },
                )
                .expect(FITS),
            )
            .expect("same count");
    }
    w::LandingRule::new(
        &w::CEILINGS,
        w::LandingRuleParts {
            pattern: encode_pattern(pattern),
            enforce: *enforce,
            ci: *ci,
            up_to_date: *up_to_date,
            gates: encoded_gates,
            approvals: encoded_approvals,
        },
    )
    .expect(FITS)
}
fn decode_rule(value: w::LandingRule, bounds: &Bounds) -> Result<d::LandingRule, Undecodable> {
    let w::LandingRuleParts { pattern, enforce, ci, up_to_date, gates, approvals } = value.into_parts();
    let mut decoded_gates = List::with_capacity(gates.len());
    for gate in gates.into_boxed() {
        let w::RuleGateParts { number, blocking, freshness } = gate.into_parts();
        decoded_gates
            .push(d::RuleGate { number, blocking, freshness: decode_freshness(freshness) })
            .expect("same count");
    }
    let mut decoded_approvals = List::with_capacity(approvals.len());
    for approval in approvals.into_boxed() {
        let w::RuleApprovalParts { role, people, freshness } = approval.into_parts();
        decoded_approvals
            .push(d::RuleApproval { role, people, freshness: decode_freshness(freshness) })
            .expect("same count");
    }
    Ok(d::LandingRule {
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
        d::SettingSlot::Replaced(number) => {
            w::SettingSlot::Replaced(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::SettingSlot::Proposal(number) => {
            w::SettingSlot::Proposal(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
    };
    let edit = match edit {
        d::Edit::Set(d::SizedValue { value, size }) => {
            let value = match value {
                d::Value::Classes(classes) => {
                    let record = temper_protocol_reaches::classes_record(classes).expect(FITS);
                    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured classes length"));
                    record.encode(&mut writer).expect("measured classes room");
                    w::SettingValue::Classes(
                        w::ProjectClasses::decode(&w::CEILINGS, &mut Reader::new(&writer.finish()))
                            .expect("matching table records"),
                    )
                }
                d::Value::Rule(rule) => w::SettingValue::Rule(encode_rule(rule)),
                d::Value::Questions(route) => w::SettingValue::Questions(
                    w::ProjectQuestions::new(
                        &w::CEILINGS,
                        w::ProjectQuestionsParts {
                            route: match route {
                                temper_engine_domain_forge_change::Route::None => w::Route::None,
                                temper_engine_domain_forge_change::Route::Foreman => w::Route::Foreman,
                                temper_engine_domain_forge_change::Route::Up => w::Route::Up,
                            },
                        },
                    )
                    .expect(FITS),
                ),
                d::Value::Ways(ways) => w::SettingValue::Ways(
                    w::ProjectWays::new(
                        &w::CEILINGS,
                        w::ProjectWaysParts {
                            drafts: ways.drafts,
                            goals: ways.goals,
                            changes: ways.changes,
                            way: ways.way,
                        },
                    )
                    .expect(FITS),
                ),
                d::Value::CharterWords(line) => w::SettingValue::Line(
                    w::CharterWords::new(&w::CEILINGS, w::CharterWordsParts { line: line.clone() }).expect(FITS),
                ),
                d::Value::ProjectWords(words) => w::SettingValue::Words(
                    w::ProjectWords::new(
                        &w::CEILINGS,
                        w::ProjectWordsParts { headline: words.headline.clone(), lede: words.lede.clone() },
                    )
                    .expect(FITS),
                ),
                d::Value::Goal(goal) => w::SettingValue::Goal(
                    w::TemplateGoal::new(
                        &w::CEILINGS,
                        w::TemplateGoalParts { design: goal.design, close_out: goal.close_out },
                    )
                    .expect(FITS),
                ),
                d::Value::ProjectAreas(value) => w::SettingValue::Areas(wire_areas(value)),
                d::Value::ProjectTriager(value) => w::SettingValue::Triager(wire_triager(value)),
                d::Value::Charter(number) => w::SettingValue::Charter(
                    w::CharterValue::new(&w::CEILINGS, w::CharterValueParts { number: *number }).expect(FITS),
                ),
            };
            w::SettingEdit::Set(
                w::SizedValue::new(&w::CEILINGS, w::SizedValueParts { value, size: *size }).expect(FITS),
            )
        }
        d::Edit::Remove => w::SettingEdit::Remove,
        d::Edit::Reset => w::SettingEdit::Reset,
    };
    w::SettingRow::new(&w::CEILINGS, w::SettingRowParts { kind: *kind, scope: *scope, number: *number, slot, edit })
        .expect(FITS)
}
pub(super) fn decode(value: w::SettingRow, bounds: &Bounds) -> Result<d::SettingRow, Undecodable> {
    let w::SettingRowParts { kind, scope, number, slot, edit } = value.into_parts();
    d::Kind::of(kind).ok_or(Undecodable)?;
    let slot = match slot {
        w::SettingSlot::Default => d::SettingSlot::Default,
        w::SettingSlot::Change => d::SettingSlot::Change,
        w::SettingSlot::Replaced(number) => d::SettingSlot::Replaced(number.into_parts().number),
        w::SettingSlot::Proposal(number) => d::SettingSlot::Proposal(number.into_parts().number),
    };
    let edit = match edit {
        w::SettingEdit::Set(value) => {
            let w::SizedValueParts { value, size } = value.into_parts();
            let value = match value {
                w::SettingValue::Rule(rule) => d::Value::Rule(decode_rule(rule, bounds)?),
                w::SettingValue::Questions(value) => d::Value::Questions(match value.into_parts().route {
                    w::Route::None => temper_engine_domain_forge_change::Route::None,
                    w::Route::Foreman => temper_engine_domain_forge_change::Route::Foreman,
                    w::Route::Up => temper_engine_domain_forge_change::Route::Up,
                }),
                w::SettingValue::Ways(value) => {
                    let w::ProjectWaysParts { drafts, goals, changes, way } = value.into_parts();
                    d::Value::Ways(d::ProjectWays { drafts, goals, changes, way })
                }
                w::SettingValue::Charter(number) => d::Value::Charter(number.into_parts().number),
                w::SettingValue::Classes(record) => {
                    use temper_client_protocol_wire::v1;
                    let mut writer = Writer::new(usize::try_from(record.measure()).or(Err(Undecodable))?);
                    record.encode(&mut writer).or(Err(Undecodable))?;
                    let record = v1::ProjectClasses::decode(&v1::CEILINGS, &mut Reader::new(&writer.finish()))
                        .or(Err(Undecodable))?;
                    let classes = temper_protocol_reaches::classes_from(&record);
                    classes.check().or(Err(Undecodable))?;

                    d::Value::Classes(classes)
                }
                w::SettingValue::Line(value) => {
                    let line = value.into_parts().line;
                    if !d::words_fit(&line, d::LINE_BYTES) {
                        return Err(Undecodable);
                    }
                    d::Value::CharterWords(line)
                }
                w::SettingValue::Words(value) => {
                    let w::ProjectWordsParts { headline, lede } = value.into_parts();
                    if !d::words_fit(&headline, d::HEADLINE_BYTES) || !d::words_fit(&lede, d::LEDE_BYTES) {
                        return Err(Undecodable);
                    }
                    d::Value::ProjectWords(d::ProjectWords { headline, lede })
                }
                w::SettingValue::Areas(value) => d::Value::ProjectAreas(areas_from(value)),
                w::SettingValue::Triager(value) => d::Value::ProjectTriager(d::ProjectTriager { owner: owner_from(value.into_parts().owner) }),
                w::SettingValue::Goal(value) => {
                    let w::TemplateGoalParts { design, close_out } = value.into_parts();
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
        d::AreaOwner::Task(number) => w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).expect("fixed owner number")),
        d::AreaOwner::Party(number) => w::AreaOwner::Party(w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number }).expect("fixed owner number")),
        d::AreaOwner::Role(number) => w::AreaOwner::Role(w::AreaRole::new(&w::CEILINGS, w::AreaRoleParts { number }).expect("fixed owner number")),
        d::AreaOwner::Charter(number) => w::AreaOwner::Charter(w::AreaCharter::new(&w::CEILINGS, w::AreaCharterParts { number }).expect("fixed owner number")),
    }
}
fn owner_from(value: w::AreaOwner) -> d::AreaOwner {
    match value {
        w::AreaOwner::Task(number) => d::AreaOwner::Task(number.number()),
        w::AreaOwner::Party(number) => d::AreaOwner::Party(number.number()),
        w::AreaOwner::Role(number) => d::AreaOwner::Role(number.number()),
        w::AreaOwner::Charter(number) => d::AreaOwner::Charter(number.number()),
    }
}
fn wire_areas(value: &d::ProjectAreas) -> w::ProjectAreas {
    let mut areas = List::with_capacity(u32::try_from(value.areas.len()).expect("admitted area count"));
    for area in &value.areas {
        areas.push(w::Area::new(&w::CEILINGS, w::AreaParts { name: area.name.clone(), words: area.words.clone(), owner: owner_record(area.owner) }).expect("admitted area fits")).expect("counted areas");
    }
    w::ProjectAreas::new(&w::CEILINGS, w::ProjectAreasParts { areas }).expect("admitted areas fit")
}
fn wire_triager(value: &d::ProjectTriager) -> w::ProjectTriager {
    w::ProjectTriager::new(&w::CEILINGS, w::ProjectTriagerParts { owner: owner_record(value.owner) }).expect("fixed triager fields")
}
fn areas_from(value: w::ProjectAreas) -> d::ProjectAreas {
    let mut areas = List::uncapped();
    for area in value.into_parts().areas.into_boxed() {
        let w::AreaParts { name, words, owner } = area.into_parts();
        areas.push(d::Area { name, words, owner: owner_from(owner) }).expect("decoded areas");
    }
    d::ProjectAreas { areas: areas.into_boxed() }
}
