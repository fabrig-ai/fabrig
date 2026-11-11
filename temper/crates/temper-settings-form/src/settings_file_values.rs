//! Concrete whole-value decoders; the schema has fixed depth, with no recursive calls.
use super::{Decoder, Error, bound, error, field, optional};
use crate::view as c;
use alloc::boxed::Box;
use skein_json::Kind;
use skein_lib::{Duration, List, Wall, bytes};
pub(super) fn charter(d: &Decoder<'_>, at: u32) -> Result<c::Charter, Error> {
    let fields = d.object(
        at,
        &[
            b"rest",
            b"number",
            b"instructions",
            b"waiting",
            b"resume",
            b"turns",
            b"time",
            b"reserve_turns",
            b"reserve_time",
            b"model",
            b"alternatives",
            b"inspect",
            b"modify",
            b"shell",
            b"agents",
            b"call_timeout",
            b"tools",
            b"hosts",
            b"spend",
            b"wake",
            b"documents",
            b"parts",
        ],
    )?;
    let resume = d.boolean(field(&fields, b"resume")?)?;
    let parts = charter_parts(d, optional(&fields, b"parts"))?;
    if resume && !parts.is_empty() {
        return Err(error(b"charter.parts", b"a charter that resumes takes no parts"));
    }
    Ok(c::Charter {
        parts,
        rest: match optional(&fields, b"rest") {
            Some(at) => d.boolean(at)?,
            None => false,
        },
        documents: documents(d, field(&fields, b"documents")?)?,
        switches: [
            resume,
            d.boolean(field(&fields, b"inspect")?)?,
            d.boolean(field(&fields, b"modify")?)?,
            d.boolean(field(&fields, b"shell")?)?,
            d.boolean(field(&fields, b"agents")?)?,
        ],
        number: u32::try_from(d.number(field(&fields, b"number")?)?)
            .or(Err(error(b"charter_number", b"integer bound")))?,
        instructions: d.words(
            field(&fields, b"instructions")?,
            d.limits.charter_instructions,
            b"charter_instructions",
        )?,
        waiting: Duration::from_nanos(d.number(field(&fields, b"waiting")?)?),
        turns: u32::try_from(d.number(field(&fields, b"turns")?)?)
            .or(Err(error(b"charter_turns", b"integer bound")))?,
        time: Duration::from_nanos(d.number(field(&fields, b"time")?)?),
        reserve_turns: u32::try_from(d.number(field(&fields, b"reserve_turns")?)?)
            .or(Err(error(b"charter_reserve_turns", b"integer bound")))?,
        reserve_time: Duration::from_nanos(d.number(field(&fields, b"reserve_time")?)?),
        model: setting_model(d, field(&fields, b"model")?)?,
        alternatives: {
            let positions =
                d.array(field(&fields, b"alternatives")?, d.limits.charter_alternatives, b"charter_alternatives")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(setting_model(d, *at)?).expect("checked list length");
            }
            values
        },
        call_timeout: Duration::from_nanos(d.number(field(&fields, b"call_timeout")?)?),
        tools: match optional(&fields, b"tools") {
            Some(at) => d.number(at)?,
            None => crate::CHARTER_TOOLS_DEFAULT,
        },
        hosts: setting_hosts(d, field(&fields, b"hosts")?)?,
        spend: d.number(field(&fields, b"spend")?)?,
        wake: match optional(&fields, b"wake") {
            Some(at) if d.kind(at)? != Kind::Null => wake_policy(d, at)?,
            Some(_) | None => c::WakePolicy::DEFAULT,
        },
    })
}
fn charter_parts(d: &Decoder<'_>, at: Option<u32>) -> Result<List<c::CharterPart>, Error> {
    let Some(at) = at else { return Ok(List::with_capacity(0)) };
    let positions = match d.array(at, d.limits.charter_parts, b"charter.parts") {
        Ok(positions) => positions,
        Err(mut why) => {
            if why.bound.is_some() {
                why.rule = b"tasks.charter_parts bound".as_slice().into();
            }
            return Err(why);
        }
    };
    let mut parts = List::with_capacity(positions.len());
    for (index, at) in positions.iter().enumerate() {
        match charter_part(d, *at) {
            Ok(part) => parts.push(part).expect("checked part count"),
            Err(why) => return Err(indexed_error(b"charter.parts", index, why)),
        }
    }
    Ok(parts)
}
fn indexed_error(path: &[u8], index: usize, why: Error) -> Error {
    let digits = skein_lib::Decimal::of(u64::try_from(index).expect("bounded index"));
    let mut measured = crate::Text::measure();
    for item in [path, b"[", digits.as_bytes(), b"]"] {
        measured.put(item);
    }
    let mut out = crate::Text::writing_exact(measured);
    for item in [path, b"[", digits.as_bytes(), b"]"] {
        out.put(item);
    }
    why.at(&out.finish())
}
fn charter_part(d: &Decoder<'_>, at: u32) -> Result<c::CharterPart, Error> {
    let fields = d.object(at, &[b"on", b"instructions"])?;
    let on = part_reasons(d, field(&fields, b"on")?)?;
    let instructions = match d.words(
        field(&fields, b"instructions")?,
        d.limits.charter_instructions.min(d.limits.charter_part_instructions),
        b"instructions",
    ) {
        Ok(text) => text,
        Err(why) => return Err(why.named(b"instructions")),
    };
    Ok(c::CharterPart { on, instructions })
}
fn part_reasons(d: &Decoder<'_>, at: u32) -> Result<u32, Error> {
    // Each name takes a distinct bit; the JSON token bound limits malformed lists.
    let positions = d.array(at, d.doc.len(), b"on")?;
    if positions.is_empty() {
        return Err(error(b"on", b"names no kind"));
    }
    let mut on = 0_u32;
    for (index, at) in positions.iter().enumerate() {
        let name = match d.string(*at) {
            Ok(name) => name,
            Err(why) => return Err(indexed_error(b"on", index, why)),
        };
        let mut bit = None;
        for (index, candidate) in c::REASON_NAMES.iter().enumerate() {
            if name == *candidate {
                bit = Some(1_u32.checked_shl(u32::try_from(index).expect("reason index")).expect("reason bit"));
                break;
            }
        }
        let why = match bit {
            Some(bit) if on & bit == 0 => {
                on |= bit;
                continue;
            }
            Some(_) => b" twice".as_slice(),
            None if c::ALWAYS_OFFERED_NAMES.contains(&name) => b" goes to every run and chooses no part",
            None => b" is no reason",
        };
        return Err(indexed_error(b"on", index, reason_error(name, why)));
    }
    Ok(on)
}
fn reason_error(name: &[u8], why: &[u8]) -> Error {
    let mut measured = crate::Text::measure();
    for item in [b"\"".as_slice(), name, b"\"", why] {
        measured.put(item);
    }
    let mut out = crate::Text::writing_exact(measured);
    for item in [b"\"".as_slice(), name, b"\"", why] {
        out.put(item);
    }
    Error { member: b"".as_slice().into(), rule: out.finish(), bound: None }
}
fn wake_policy(d: &Decoder<'_>, at: u32) -> Result<c::WakePolicy, Error> {
    match wake_policy_value(d, at) {
        Ok(value) => Ok(value),
        Err(why) => Err(wake_error(b"charter.wake", why)),
    }
}
fn wake_policy_value(d: &Decoder<'_>, at: u32) -> Result<c::WakePolicy, Error> {
    let fields = d.object(at, &[b"words", b"notices", b"news", b"results", b"questions", b"answers", b"timers"])?;
    Ok(c::WakePolicy {
        words: wake_rule(d, field(&fields, b"words")?, b"words")?,
        notices: wake_rule(d, field(&fields, b"notices")?, b"notices")?,
        news: wake_rule(d, field(&fields, b"news")?, b"news")?,
        results: match d.string(field(&fields, b"results")?)? {
            b"never" => c::ResultsWake::Never,
            b"each" => c::ResultsWake::Each,
            b"last_or_failure" => c::ResultsWake::LastOrFailure,
            _ => return Err(error(b"results", b"unknown results rule")),
        },
        questions: d.boolean(field(&fields, b"questions")?)?,
        answers: d.boolean(field(&fields, b"answers")?)?,
        timers: d.boolean(field(&fields, b"timers")?)?,
    })
}
fn wake_rule(d: &Decoder<'_>, at: u32, path: &[u8]) -> Result<c::WakeRule, Error> {
    match wake_rule_value(d, at) {
        Ok(value) => Ok(value),
        Err(why) => Err(wake_error(path, why)),
    }
}
fn wake_rule_value(d: &Decoder<'_>, at: u32) -> Result<c::WakeRule, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"never" => Ok(c::WakeRule::Never),
            b"immediate" => Ok(c::WakeRule::Immediate),
            _ => Err(error(b"", b"unknown wake rule")),
        };
    }
    let fields = d.object(at, &[b"batch", b"paced"])?;
    if fields.len() != 1 {
        return Err(error(b"", b"one wake rule member required"));
    }
    if let Some(at) = optional(&fields, b"paced") {
        return Ok(c::WakeRule::Paced { span: Duration::from_nanos(d.number(at)?) });
    }
    match wake_batch(d, field(&fields, b"batch")?) {
        Ok(value) => Ok(value),
        Err(why) => Err(wake_error(b"batch", why)),
    }
}
fn wake_batch(d: &Decoder<'_>, at: u32) -> Result<c::WakeRule, Error> {
    let fields = d.object(at, &[b"count", b"age"])?;
    let count = u32::try_from(d.number(field(&fields, b"count")?)?).or(Err(error(b"count", b"integer bound")))?;
    Ok(c::WakeRule::Batch { count, age: Duration::from_nanos(d.number(field(&fields, b"age")?)?) })
}
fn wake_error(path: &[u8], why: Error) -> Error {
    why.at(path)
}

fn documents(d: &Decoder<'_>, at: u32) -> Result<List<c::CharterDocument>, Error> {
    let positions = d.array(at, d.limits.charter_documents, b"charter.documents")?;
    let mut values = List::with_capacity(positions.len());
    for (index, at) in positions.iter().enumerate() {
        match document(d, *at) {
            Ok(value) => values.push(value).expect("checked list length"),
            Err(why) => {
                let mut measured = crate::Text::measure();
                document_member(index, &why.member, &mut measured);
                let mut out = crate::Text::writing_exact(measured);
                document_member(index, &why.member, &mut out);
                return Err(Error { member: out.finish(), ..why });
            }
        }
    }
    Ok(values)
}
fn document_member(index: usize, member: &[u8], out: &mut crate::Text) {
    out.put(b"charter.documents[");
    out.number(u64::try_from(index).expect("document index"));
    out.put(b"].");
    out.put(member);
}
fn document(d: &Decoder<'_>, at: u32) -> Result<c::CharterDocument, Error> {
    let fields = d.object(at, &[b"forge", b"repository", b"branch", b"path", b"heading", b"connector", b"name"])?;
    for member in [b"name".as_slice(), b"connector"] {
        if optional(&fields, member).is_some() {
            return Err(error(member, b"shape: forge branch required"));
        }
    }
    let forge = match optional(&fields, b"forge") {
        Some(at) => Some(d.document_string(at, b"forge")?),
        None if d.repositories.is_some() => None,
        None => return Err(error(b"forge", b"missing member")),
    };
    let repository = d.document_string(field(&fields, b"repository")?, b"repository")?;
    let branch = d.document_string(field(&fields, b"branch")?, b"branch")?;
    let slash = bytes::find(repository, b"/").ok_or(error(b"repository", b"OWNER/NAME required"))?;
    let owner = repository.get(..slash).expect("repository prefix");
    let repo = repository.get(slash.checked_add(1).expect("slash position")..).expect("repository suffix");
    let (forge, owner, repo) = match forge {
        Some(forge) => (forge, owner, repo),
        None => {
            let repositories = d.repositories.expect("host proposal resolver");
            let mut found = None;
            for identity in repositories {
                if owner.eq_ignore_ascii_case(identity.owner) && repo.eq_ignore_ascii_case(identity.name) {
                    found = Some(identity);
                    break;
                }
            }
            match found {
                Some(identity) => (identity.host, identity.owner, identity.name),
                None => return Err(repository_missing(repository, repositories)),
            }
        }
    };
    let segment_bound = d.limits.charter_document_name_item;
    document_segment(forge, segment_bound, b"forge")?;
    document_segment(owner, segment_bound, b"repository")?;
    document_segment(repo, segment_bound, b"repository")?;
    document_segment(b"forge", segment_bound, b"forge")?;
    document_segment(b"branch", segment_bound, b"branch")?;
    let count = bytes::count(branch, b"/", u32::MAX).saturating_add(6);
    if count > d.limits.charter_document_name {
        return Err(bound(b"branch", d.limits.charter_document_name));
    }
    let mut name = List::with_capacity(count);
    for segment in [b"forge".as_slice(), forge, owner, repo, b"branch"] {
        name.push(Box::from(segment)).expect("measured name");
    }
    let mut start = 0;
    for _ in 5..count {
        let end = bytes::find_from(branch, b"/", start).unwrap_or(branch.len());
        let segment = branch.get(start..end).expect("branch segment");
        document_segment(segment, segment_bound, b"branch")?;
        name.push(Box::from(segment)).expect("measured name");
        start = end.checked_add(1).expect("file bound");
    }
    let path_at = field(&fields, b"path")?;
    d.document_string(path_at, b"path")?;
    let path = d.words(path_at, d.limits.charter_document_path, b"path")?.into_bytes();
    if path.is_empty() {
        return Err(error(b"path", b"nonempty text required"));
    }
    let heading = match optional(&fields, b"heading") {
        Some(at) if d.kind(at)? != Kind::Null => {
            d.document_string(at, b"heading")?;
            let heading = d.words(at, d.limits.charter_document_heading_some, b"heading")?.into_bytes();
            if heading.is_empty() {
                return Err(error(b"heading", b"nonempty text required"));
            }
            Some(heading)
        }
        Some(_) | None => None,
    };
    Ok(c::CharterDocument { connector: 1, name, path, heading })
}

fn repository_missing(name: &[u8], repositories: &[super::Repository<'_>]) -> Error {
    let mut measure = crate::Text::measure();
    repository_refusal(&mut measure, name, repositories);
    let mut out = crate::Text::writing_exact(measure);
    repository_refusal(&mut out, name, repositories);
    Error { member: b"repository".as_slice().into(), rule: out.finish(), bound: None }
}
fn repository_refusal(out: &mut crate::Text, name: &[u8], repositories: &[super::Repository<'_>]) {
    out.put(b"no repository ");
    out.put(name);
    out.put(b" in this project");
    if repositories.is_empty() {
        out.put(b"; it has none");
    } else {
        out.put(b"; its repositories: ");
        let mut separator = b"".as_slice();
        for repository in repositories {
            out.put(separator);
            out.put(repository.owner);
            out.put(b"/");
            out.put(repository.name);
            separator = " · ".as_bytes();
        }
    }
}
impl Decoder<'_> {
    fn document_string(&self, at: u32, member: &[u8]) -> Result<&[u8], Error> {
        match self.string(at) {
            Ok(bytes) => Ok(bytes),
            Err(why) => Err(Error { member: member.into(), ..why }),
        }
    }
}
fn document_segment(bytes: &[u8], most: u32, member: &[u8]) -> Result<(), Error> {
    if bytes.len() > usize::try_from(most).expect("segment bound") {
        return Err(bound(member, most));
    }
    if bytes.is_empty() || bytes.contains(&b'/') {
        return Err(error(member, b"nonempty name segment required"));
    }
    Ok(())
}
pub(super) fn template(d: &Decoder<'_>, at: u32) -> Result<c::Template, Error> {
    let fields = d.object(
        at,
        &[
            b"resources",
            b"number",
            b"name",
            b"executor",
            b"contract",
            b"shallower",
            b"budget",
            b"deadline",
            b"tracked",
            b"flags",
            b"label",
            b"grace",
            b"llm",
        ],
    )?;
    Ok(c::Template {
        resources: template_resources(d, optional(&fields, b"resources"))?,
        number: u32::try_from(d.number(field(&fields, b"number")?)?)
            .or(Err(error(b"template_number", b"integer bound")))?,
        name: d.document_text(field(&fields, b"name")?)?,
        executor: template_executor(d, field(&fields, b"executor")?)?,
        contract: task_contract_document(d, field(&fields, b"contract")?)?,
        shallower: u32::try_from(d.number(field(&fields, b"shallower")?)?)
            .or(Err(error(b"template_shallower", b"integer bound")))?,
        budget: template_budget(d, field(&fields, b"budget")?)?,
        deadline: template_deadline(d, field(&fields, b"deadline")?)?,
        tracked: if d.kind(field(&fields, b"tracked")?)? == Kind::Null {
            None
        } else {
            Some(
                u32::try_from(d.number(field(&fields, b"tracked")?)?)
                    .or(Err(error(b"template_tracked_some", b"integer bound")))?,
            )
        },
        flags: task_flags(d, field(&fields, b"flags")?)?,
        label: u32::try_from(d.number(field(&fields, b"label")?)?)
            .or(Err(error(b"template_label", b"integer bound")))?,
        grace: if d.kind(field(&fields, b"grace")?)? == Kind::Null {
            None
        } else {
            Some(Duration::from_nanos(d.number(field(&fields, b"grace")?)?))
        },
        llm: if d.kind(field(&fields, b"llm")?)? == Kind::Null {
            None
        } else {
            Some(llm_choice(d, field(&fields, b"llm")?)?)
        },
    })
}
fn template_resources(d: &Decoder<'_>, at: Option<u32>) -> Result<List<c::Symbol>, Error> {
    let Some(at) = at else { return Ok(List::with_capacity(0)) };
    let positions = d.array(at, d.limits.template_resources, b"template.resources")?;
    let mut values = List::with_capacity(positions.len());
    for at in &positions {
        let value = template_symbol(d, *at)?;
        if values.as_slice().contains(&value) {
            return Err(error(b"template.resources", b"duplicate pair"));
        }
        values.push(value).expect("checked list length");
    }
    Ok(values)
}
fn template_symbol(d: &Decoder<'_>, at: u32) -> Result<c::Symbol, Error> {
    let fields = match d.object(at, &[b"connector", b"symbol"]) {
        Ok(fields) => fields,
        Err(why) => return Err(why.at(b"template.resources")),
    };
    Ok(c::Symbol {
        connector: symbol_number(d, field(&fields, b"connector"), b"connector")?,
        symbol: symbol_number(d, field(&fields, b"symbol"), b"symbol")?,
    })
}
fn symbol_number(d: &Decoder<'_>, at: Result<u32, Error>, member: &[u8]) -> Result<u16, Error> {
    let number = match at {
        Ok(at) => d.number(at),
        Err(why) => return Err(why.at(b"template.resources")),
    };
    let number = match number {
        Ok(number) => number,
        Err(why) => return Err(why.named(member).at(b"template.resources")),
    };
    u16::try_from(number).or(Err(error(member, b"integer bound").at(b"template.resources")))
}
pub(super) fn setting_policy(d: &Decoder<'_>, at: u32) -> Result<c::SettingPolicy, Error> {
    let fields = d.object(
        at,
        &[
            b"escalation_role",
            b"ceiling",
            b"projections",
            b"period_spend",
            b"roles",
            b"requirements",
            b"permissions",
            b"templates",
        ],
    )?;
    Ok(c::SettingPolicy {
        escalation_role: if d.kind(field(&fields, b"escalation_role")?)? == Kind::Null {
            None
        } else {
            Some(
                u32::try_from(d.number(field(&fields, b"escalation_role")?)?)
                    .or(Err(error(b"setting_policy_escalation_role_some", b"integer bound")))?,
            )
        },
        ceiling: owner_authority(d, field(&fields, b"ceiling")?)?,
        projections: {
            let positions = d.array(
                field(&fields, b"projections")?,
                d.limits.setting_policy_projections,
                b"setting_policy_projections",
            )?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(owner_grant(d, *at)?).expect("checked list length");
            }
            values
        },
        period_spend: d.number(field(&fields, b"period_spend")?)?,
        roles: {
            let positions =
                d.array(field(&fields, b"roles")?, d.limits.setting_policy_roles, b"setting_policy_roles")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(policy_edit_role(d, *at)?).expect("checked list length");
            }
            values
        },
        requirements: {
            let positions = d.array(
                field(&fields, b"requirements")?,
                d.limits.setting_policy_requirements,
                b"setting_policy_requirements",
            )?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(policy_requirement(d, *at)?).expect("checked list length");
            }
            values
        },
        permissions: {
            let positions = d.array(
                field(&fields, b"permissions")?,
                d.limits.setting_policy_permissions,
                b"setting_policy_permissions",
            )?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(policy_permission(d, *at)?).expect("checked list length");
            }
            values
        },
        templates: {
            let positions =
                d.array(field(&fields, b"templates")?, d.limits.setting_policy_templates, b"setting_policy_templates")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(template_override(d, *at)?).expect("checked list length");
            }
            values
        },
    })
}
pub(super) fn rules(d: &Decoder<'_>, at: u32) -> Result<c::Rules, Error> {
    let fields = d.object(
        at,
        &[b"ceiling", b"period_spend", b"minimum_run_spend", b"maximum_run_spend", b"implies", b"requirements"],
    )?;
    Ok(c::Rules {
        ceiling: owner_authority(d, field(&fields, b"ceiling")?)?,
        period_spend: d.number(field(&fields, b"period_spend")?)?,
        minimum_run_spend: d.number(field(&fields, b"minimum_run_spend")?)?,
        maximum_run_spend: d.number(field(&fields, b"maximum_run_spend")?)?,
        implies: {
            let positions = d.array(field(&fields, b"implies")?, d.limits.rules_implies, b"rules_implies")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(owner_kind_implication(d, *at)?).expect("checked list length");
            }
            values
        },
        requirements: {
            let positions =
                d.array(field(&fields, b"requirements")?, d.limits.rules_requirements, b"rules_requirements")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(policy_requirement(d, *at)?).expect("checked list length");
            }
            values
        },
    })
}
fn setting_model(d: &Decoder<'_>, at: u32) -> Result<c::SettingModel, Error> {
    let fields = d.object(
        at,
        &[
            b"provider",
            b"dialect",
            b"endpoint",
            b"program",
            b"name",
            b"window",
            b"output",
            b"input_price",
            b"cached_price",
            b"cache_write_price",
            b"output_price",
            b"price_unit",
            b"efforts",
            b"effort",
        ],
    )?;
    Ok(c::SettingModel {
        provider: match optional(&fields, b"provider") {
            Some(at) => u32::try_from(d.number(at)?).or(Err(error(b"provider", b"integer bound")))?,
            None => 1,
        },
        dialect: u32::try_from(d.number(field(&fields, b"dialect")?)?)
            .or(Err(error(b"setting_model_dialect", b"integer bound")))?,
        endpoint: u32::try_from(d.number(field(&fields, b"endpoint")?)?)
            .or(Err(error(b"setting_model_endpoint", b"integer bound")))?,
        program: if d.kind(field(&fields, b"program")?)? == Kind::Null {
            None
        } else {
            Some(d.words(
                field(&fields, b"program")?,
                d.limits.setting_model_program_some,
                b"setting_model_program_some",
            )?)
        },
        name: d.words(field(&fields, b"name")?, d.limits.setting_model_name, b"setting_model_name")?,
        window: u32::try_from(d.number(field(&fields, b"window")?)?)
            .or(Err(error(b"setting_model_window", b"integer bound")))?,
        output: u32::try_from(d.number(field(&fields, b"output")?)?)
            .or(Err(error(b"setting_model_output", b"integer bound")))?,
        input_price: d.number(field(&fields, b"input_price")?)?,
        cached_price: d.number(field(&fields, b"cached_price")?)?,
        cache_write_price: if d.kind(field(&fields, b"cache_write_price")?)? == Kind::Null {
            None
        } else {
            Some(d.number(field(&fields, b"cache_write_price")?)?)
        },
        output_price: d.number(field(&fields, b"output_price")?)?,
        price_unit: u32::try_from(d.number(field(&fields, b"price_unit")?)?)
            .or(Err(error(b"setting_model_price_unit", b"integer bound")))?,
        efforts: {
            let positions =
                d.array(field(&fields, b"efforts")?, d.limits.setting_model_efforts, b"setting_model_efforts")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values
                    .push(d.words(*at, d.limits.setting_model_efforts_item, b"setting_model_efforts_item")?)
                    .expect("checked list length");
            }
            values
        },
        effort: if d.kind(field(&fields, b"effort")?)? == Kind::Null {
            None
        } else {
            Some(d.words(
                field(&fields, b"effort")?,
                d.limits.setting_model_effort_some,
                b"setting_model_effort_some",
            )?)
        },
    })
}
fn setting_hosts(d: &Decoder<'_>, at: u32) -> Result<c::SettingHosts, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"workers" => Ok(c::SettingHosts::Workers),
            b"engine" => Ok(c::SettingHosts::Engine),
            b"both" => Ok(c::SettingHosts::Both),
            _ => Err(error(b"setting_hosts", b"unknown word")),
        };
    }
    Err(error(b"setting_hosts", b"expected word"))
}
fn template_executor(d: &Decoder<'_>, at: u32) -> Result<c::TemplateExecutor, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"chosen" => Ok(c::TemplateExecutor::Chosen),
            _ => Err(error(b"template_executor", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"charter"])?;
    if fields.len() != 1 {
        return Err(error(b"template_executor", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"charter") {
        return Ok(c::TemplateExecutor::Charter(
            u32::try_from(d.number(at)?).or(Err(error(b"template_executor_charter", b"integer bound")))?,
        ));
    }
    Err(error(b"template_executor", b"unknown variant"))
}
fn task_contract_document(d: &Decoder<'_>, at: u32) -> Result<c::TaskContractDocument, Error> {
    let fields = d.object(at, &[b"approval", b"report", b"verdict", b"change", b"job"])?;
    if fields.len() != 1 {
        return Err(error(b"task_contract_document", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"job") {
        let fields = d.object(at, &[b"bytes"])?;
        return Ok(c::TaskContractDocument::Job {
            bytes: u32::try_from(d.number(field(&fields, b"bytes")?)?)
                .or(Err(error(b"task_contract_document_job_bytes", b"integer bound")))?,
        });
    }
    if let Some(at) = optional(&fields, b"approval") {
        let fields = d.object(at, &[b"state", b"part"])?;
        return Ok(c::TaskContractDocument::Approval {
            state: d.hex(
                field(&fields, b"state")?,
                d.limits.task_contract_document_approval_state,
                b"task_contract_document_approval_state",
            )?,
            part: owner_part(d, field(&fields, b"part")?)?,
        });
    }
    if let Some(at) = optional(&fields, b"report") {
        let fields = d.object(at, &[b"words"])?;
        return Ok(c::TaskContractDocument::Report {
            words: u32::try_from(d.number(field(&fields, b"words")?)?)
                .or(Err(error(b"task_contract_document_report_words", b"integer bound")))?,
        });
    }
    if let Some(at) = optional(&fields, b"verdict") {
        let fields = d.object(at, &[b"choices", b"cut"])?;
        return Ok(c::TaskContractDocument::Verdict {
            choices: {
                let positions = d.array(
                    field(&fields, b"choices")?,
                    d.limits.task_contract_document_verdict_choices,
                    b"task_contract_document_verdict_choices",
                )?;
                let mut values = List::with_capacity(positions.len());
                for at in &positions {
                    values.push(verdict_choice(d, *at)?).expect("checked list length");
                }
                super::verdict_bound(&values, &d.limits)?;
                values
            },
            cut: {
                if d.boolean(field(&fields, b"cut")?)? {
                    return Err(error(b"cut", b"whole value required"));
                }
                false
            },
        });
    }
    if let Some(at) = optional(&fields, b"change") {
        let fields = d.object(at, &[b"connector", b"kind", b"words"])?;
        return Ok(c::TaskContractDocument::Change {
            connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
                .or(Err(error(b"task_contract_document_change_connector", b"integer bound")))?,
            kind: u16::try_from(d.number(field(&fields, b"kind")?)?)
                .or(Err(error(b"task_contract_document_change_kind", b"integer bound")))?,
            words: u32::try_from(d.number(field(&fields, b"words")?)?)
                .or(Err(error(b"task_contract_document_change_words", b"integer bound")))?,
        });
    }
    Err(error(b"task_contract_document", b"unknown variant"))
}
fn template_budget(d: &Decoder<'_>, at: u32) -> Result<c::TemplateBudget, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"given" => Ok(c::TemplateBudget::Given),
            _ => Err(error(b"template_budget", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"amount", b"as_allowed"])?;
    if fields.len() != 1 {
        return Err(error(b"template_budget", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"amount") {
        return Ok(c::TemplateBudget::Amount(d.number(at)?));
    }
    if let Some(at) = optional(&fields, b"as_allowed") {
        let fields = d.object(at, &[b"cap"])?;
        return Ok(c::TemplateBudget::AsAllowed {
            cap: if d.kind(field(&fields, b"cap")?)? == Kind::Null {
                None
            } else {
                Some(d.number(field(&fields, b"cap")?)?)
            },
        });
    }
    Err(error(b"template_budget", b"unknown variant"))
}
fn template_deadline(d: &Decoder<'_>, at: u32) -> Result<c::TemplateDeadline, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"none" => Ok(c::TemplateDeadline::None),
            b"given" => Ok(c::TemplateDeadline::Given),
            _ => Err(error(b"template_deadline", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"span"])?;
    if fields.len() != 1 {
        return Err(error(b"template_deadline", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"span") {
        return Ok(c::TemplateDeadline::Span(Duration::from_nanos(d.number(at)?)));
    }
    Err(error(b"template_deadline", b"unknown variant"))
}
fn task_flags(d: &Decoder<'_>, at: u32) -> Result<c::TaskFlags, Error> {
    let fields = d.object(at, &[b"conversation", b"listed", b"retention", b"ended_by"])?;
    Ok(c::TaskFlags {
        conversation: d.boolean(field(&fields, b"conversation")?)?,
        listed: d.boolean(field(&fields, b"listed")?)?,
        retention: retention(d, field(&fields, b"retention")?)?,
        ended_by: ended_by(d, field(&fields, b"ended_by")?)?,
    })
}
fn llm_choice(d: &Decoder<'_>, at: u32) -> Result<c::LlmChoice, Error> {
    let fields = d.object(at, &[b"account", b"main", b"subagents", b"later"])?;
    Ok(c::LlmChoice {
        later: match optional(&fields, b"later") {
            Some(at) => match llm_later(d, at) {
                Ok(later) => Some(later),
                Err(why) => return Err(why.at(b"llm.later")),
            },
            None => None,
        },
        account: if d.kind(field(&fields, b"account")?)? == Kind::Null {
            None
        } else {
            Some(
                u32::try_from(d.number(field(&fields, b"account")?)?)
                    .or(Err(error(b"llm_choice_account_some", b"integer bound")))?,
            )
        },
        main: model_effort(d, field(&fields, b"main")?)?,
        subagents: {
            let positions =
                d.array(field(&fields, b"subagents")?, d.limits.llm_choice_subagents, b"llm_choice_subagents")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(model_effort(d, *at)?).expect("checked list length");
            }
            values
        },
    })
}
fn llm_later(d: &Decoder<'_>, at: u32) -> Result<c::LlmLater, Error> {
    let fields = d.object(at, &[b"main", b"subagents"])?;
    let main = match later_model_effort(d, field(&fields, b"main")?) {
        Ok(main) => main,
        Err(why) => return Err(why.at(b"main")),
    };
    let mut subagents = List::with_capacity(0);
    if let Some(at) = optional(&fields, b"subagents") {
        let positions = d.array(at, d.limits.llm_choice_subagents, b"subagents")?;
        subagents = List::with_capacity(positions.len());
        for (index, at) in positions.iter().enumerate() {
            match later_model_effort(d, *at) {
                Ok(value) => subagents.push(value).expect("checked subagent count"),
                Err(why) => return Err(indexed_error(b"subagents", index, why)),
            }
        }
    }
    Ok(c::LlmLater { main, subagents })
}
fn later_model_effort(d: &Decoder<'_>, at: u32) -> Result<c::ModelEffort, Error> {
    let fields = d.object(at, &[b"model", b"effort"])?;
    let model = match d.words(field(&fields, b"model")?, d.limits.model_effort_model, b"model") {
        Ok(model) => model,
        Err(why) => return Err(why.named(b"model")),
    };
    let effort = match field(&fields, b"effort")? {
        at if d.kind(at)? == Kind::Null => None,
        at => match d.words(at, d.limits.model_effort_effort_some, b"effort") {
            Ok(effort) => Some(effort),
            Err(why) => return Err(why.named(b"effort")),
        },
    };
    Ok(c::ModelEffort { model, effort })
}
fn owner_authority(d: &Decoder<'_>, at: u32) -> Result<c::Authority, Error> {
    let fields = d.object(
        at,
        &[b"models", b"tools", b"grants", b"delegation", b"spend", b"deadline", b"notes", b"note_resources"],
    )?;
    Ok(c::Authority {
        models: if d.kind(field(&fields, b"models")?)? == Kind::Null {
            None
        } else {
            Some(authority_models(d, field(&fields, b"models")?)?)
        },
        tools: d.number(field(&fields, b"tools")?)?,
        grants: {
            let positions = d.array(field(&fields, b"grants")?, d.limits.authority_grants, b"authority_grants")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(owner_grant(d, *at)?).expect("checked list length");
            }
            values
        },
        delegation: owner_delegation(d, field(&fields, b"delegation")?)?,
        spend: d.number(field(&fields, b"spend")?)?,
        deadline: if d.kind(field(&fields, b"deadline")?)? == Kind::Null {
            None
        } else {
            Some(Wall::from_nanos(d.number(field(&fields, b"deadline")?)?))
        },
        notes: u8::try_from(d.number(field(&fields, b"notes")?)?)
            .or(Err(error(b"owner_authority_notes", b"integer bound")))?,
        note_resources: {
            let positions = d.array(
                field(&fields, b"note_resources")?,
                d.limits.authority_note_resources,
                b"authority_note_resources",
            )?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(owner_resource_scope(d, *at)?).expect("checked list length");
            }
            values
        },
    })
}
fn owner_grant(d: &Decoder<'_>, at: u32) -> Result<c::Grant, Error> {
    let fields = d.object(at, &[b"connector", b"kind", b"pattern"])?;
    Ok(c::Grant {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"owner_grant_connector", b"integer bound")))?,
        kind: u16::try_from(d.number(field(&fields, b"kind")?)?)
            .or(Err(error(b"owner_grant_kind", b"integer bound")))?,
        pattern: owner_pattern(d, field(&fields, b"pattern")?)?,
    })
}
fn policy_edit_role(d: &Decoder<'_>, at: u32) -> Result<c::PolicyEditRole, Error> {
    let fields = d.object(at, &[b"number", b"authority", b"period_spend", b"requests", b"decides"])?;
    Ok(c::PolicyEditRole {
        number: u32::try_from(d.number(field(&fields, b"number")?)?)
            .or(Err(error(b"policy_edit_role_number", b"integer bound")))?,
        authority: owner_authority(d, field(&fields, b"authority")?)?,
        period_spend: d.number(field(&fields, b"period_spend")?)?,
        requests: u16::try_from(d.number(field(&fields, b"requests")?)?)
            .or(Err(error(b"policy_edit_role_requests", b"integer bound")))?,
        decides: u8::try_from(d.number(field(&fields, b"decides")?)?)
            .or(Err(error(b"policy_edit_role_decides", b"integer bound")))?,
    })
}
fn policy_requirement(d: &Decoder<'_>, at: u32) -> Result<c::PolicyRequirement, Error> {
    let fields = d.object(at, &[b"connector", b"kind", b"pattern", b"judge", b"guard", b"must_be_guarded"])?;
    Ok(c::PolicyRequirement {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"policy_requirement_connector", b"integer bound")))?,
        kind: u16::try_from(d.number(field(&fields, b"kind")?)?)
            .or(Err(error(b"policy_requirement_kind", b"integer bound")))?,
        pattern: owner_pattern(d, field(&fields, b"pattern")?)?,
        judge: policy_judge(d, field(&fields, b"judge")?)?,
        guard: policy_guard(d, field(&fields, b"guard")?)?,
        must_be_guarded: d.boolean(field(&fields, b"must_be_guarded")?)?,
    })
}
fn policy_permission(d: &Decoder<'_>, at: u32) -> Result<c::PolicyPermission, Error> {
    let fields = d.object(at, &[b"connector", b"permission", b"role"])?;
    Ok(c::PolicyPermission {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"policy_permission_connector", b"integer bound")))?,
        permission: u16::try_from(d.number(field(&fields, b"permission")?)?)
            .or(Err(error(b"policy_permission_permission", b"integer bound")))?,
        role: u32::try_from(d.number(field(&fields, b"role")?)?)
            .or(Err(error(b"policy_permission_role", b"integer bound")))?,
    })
}
fn template_override(d: &Decoder<'_>, at: u32) -> Result<c::TemplateOverride, Error> {
    let fields = d.object(at, &[b"template", b"executor", b"budget", b"deadline", b"priority"])?;
    Ok(c::TemplateOverride {
        template: u32::try_from(d.number(field(&fields, b"template")?)?)
            .or(Err(error(b"template_override_template", b"integer bound")))?,
        executor: if d.kind(field(&fields, b"executor")?)? == Kind::Null {
            None
        } else {
            Some(template_executor(d, field(&fields, b"executor")?)?)
        },
        budget: if d.kind(field(&fields, b"budget")?)? == Kind::Null {
            None
        } else {
            Some(template_budget(d, field(&fields, b"budget")?)?)
        },
        deadline: if d.kind(field(&fields, b"deadline")?)? == Kind::Null {
            None
        } else {
            Some(template_deadline(d, field(&fields, b"deadline")?)?)
        },
        priority: if d.kind(field(&fields, b"priority")?)? == Kind::Null {
            None
        } else {
            Some(
                u32::try_from(d.number(field(&fields, b"priority")?)?)
                    .or(Err(error(b"template_override_priority_some", b"integer bound")))?,
            )
        },
    })
}
fn owner_kind_implication(d: &Decoder<'_>, at: u32) -> Result<c::KindImplication, Error> {
    let fields = d.object(at, &[b"connector", b"kind", b"implies"])?;
    Ok(c::KindImplication {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"owner_kind_implication_connector", b"integer bound")))?,
        kind: u16::try_from(d.number(field(&fields, b"kind")?)?)
            .or(Err(error(b"owner_kind_implication_kind", b"integer bound")))?,
        implies: u16::try_from(d.number(field(&fields, b"implies")?)?)
            .or(Err(error(b"owner_kind_implication_implies", b"integer bound")))?,
    })
}
fn owner_part(d: &Decoder<'_>, at: u32) -> Result<c::Part, Error> {
    let fields = d.object(at, &[b"connector", b"kind", b"key"])?;
    Ok(c::Part {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"owner_part_connector", b"integer bound")))?,
        kind: owner_part_kind(d, field(&fields, b"kind")?)?,
        key: owner_part_key(d, field(&fields, b"key")?)?,
    })
}
fn verdict_choice(d: &Decoder<'_>, at: u32) -> Result<c::VerdictChoice, Error> {
    let fields = d.object(at, &[b"code", b"words", b"followups", b"fields"])?;
    Ok(c::VerdictChoice {
        code: u32::try_from(d.number(field(&fields, b"code")?)?)
            .or(Err(error(b"verdict_choice_code", b"integer bound")))?,
        words: u32::try_from(d.number(field(&fields, b"words")?)?)
            .or(Err(error(b"verdict_choice_words", b"integer bound")))?,
        followups: u32::try_from(d.number(field(&fields, b"followups")?)?)
            .or(Err(error(b"verdict_choice_followups", b"integer bound")))?,
        fields: {
            let positions =
                d.array(field(&fields, b"fields")?, d.limits.verdict_choice_fields, b"verdict_choice_fields")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(field_rule(d, *at)?).expect("checked list length");
            }
            values
        },
    })
}
fn retention(d: &Decoder<'_>, at: u32) -> Result<c::Retention, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"kept" => Ok(c::Retention::Kept),
            b"forgotten_once_read" => Ok(c::Retention::ForgottenOnceRead),
            _ => Err(error(b"retention", b"unknown word")),
        };
    }
    Err(error(b"retention", b"expected word"))
}
fn ended_by(d: &Decoder<'_>, at: u32) -> Result<c::EndedBy, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"result" => Ok(c::EndedBy::Result),
            b"requester" => Ok(c::EndedBy::Requester),
            b"standing" => Ok(c::EndedBy::Standing),
            _ => Err(error(b"ended_by", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"closer"])?;
    let closer = d.object(field(&fields, b"closer")?, &[b"connector", b"code"])?;
    Ok(c::EndedBy::Closer(c::Closer {
        connector: u16::try_from(d.number(field(&closer, b"connector")?)?)
            .or(Err(error(b"ended_by.closer.connector", b"integer bound")))?,
        code: u32::try_from(d.number(field(&closer, b"code")?)?)
            .or(Err(error(b"ended_by.closer.code", b"integer bound")))?,
    }))
}
fn model_effort(d: &Decoder<'_>, at: u32) -> Result<c::ModelEffort, Error> {
    let fields = d.object(at, &[b"model", b"effort"])?;
    Ok(c::ModelEffort {
        model: d.words(field(&fields, b"model")?, d.limits.model_effort_model, b"model_effort_model")?,
        effort: if d.kind(field(&fields, b"effort")?)? == Kind::Null {
            None
        } else {
            Some(d.words(field(&fields, b"effort")?, d.limits.model_effort_effort_some, b"model_effort_effort_some")?)
        },
    })
}
fn authority_models(d: &Decoder<'_>, at: u32) -> Result<c::AuthorityModels, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"all" => Ok(c::AuthorityModels::All),
            _ => Err(error(b"authority_models", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"named"])?;
    if fields.len() != 1 {
        return Err(error(b"authority_models", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"named") {
        return Ok(c::AuthorityModels::Named({
            let positions = d.array(at, d.limits.model_set_names_names, b"model_set_names_names")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values
                    .push(d.hex(*at, d.limits.model_set_names_names_item, b"model_set_names_names_item")?)
                    .expect("checked list length");
            }
            values
        }));
    }
    Err(error(b"authority_models", b"unknown variant"))
}
fn owner_delegation(d: &Decoder<'_>, at: u32) -> Result<c::Delegation, Error> {
    let fields = d.object(at, &[b"kinds", b"tasks", b"depth"])?;
    Ok(c::Delegation {
        kinds: {
            let positions = d.array(field(&fields, b"kinds")?, d.limits.delegation_kinds, b"delegation_kinds")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values.push(owner_authority_executor(d, *at)?).expect("checked list length");
            }
            values
        },
        tasks: u32::try_from(d.number(field(&fields, b"tasks")?)?)
            .or(Err(error(b"owner_delegation_tasks", b"integer bound")))?,
        depth: u32::try_from(d.number(field(&fields, b"depth")?)?)
            .or(Err(error(b"owner_delegation_depth", b"integer bound")))?,
    })
}
fn owner_resource_scope(d: &Decoder<'_>, at: u32) -> Result<c::ResourceScope, Error> {
    let fields = d.object(at, &[b"connector", b"pattern"])?;
    Ok(c::ResourceScope {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"owner_resource_scope_connector", b"integer bound")))?,
        pattern: owner_pattern(d, field(&fields, b"pattern")?)?,
    })
}
fn owner_pattern(d: &Decoder<'_>, at: u32) -> Result<c::Pattern, Error> {
    let fields = d.object(at, &[b"segments", b"last"])?;
    Ok(c::Pattern {
        segments: {
            let positions = d.array(field(&fields, b"segments")?, d.limits.pattern_segments, b"pattern_segments")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values
                    .push(d.hex(*at, d.limits.pattern_segments_item, b"pattern_segments_item")?)
                    .expect("checked list length");
            }
            values
        },
        last: owner_pattern_end(d, field(&fields, b"last")?)?,
    })
}
fn policy_judge(d: &Decoder<'_>, at: u32) -> Result<c::PolicyJudge, Error> {
    let fields = d.object(at, &[b"connector", b"requirement", b"parameters"])?;
    Ok(c::PolicyJudge {
        connector: u16::try_from(d.number(field(&fields, b"connector")?)?)
            .or(Err(error(b"policy_judge_connector", b"integer bound")))?,
        requirement: u16::try_from(d.number(field(&fields, b"requirement")?)?)
            .or(Err(error(b"policy_judge_requirement", b"integer bound")))?,
        parameters: u32::try_from(d.number(field(&fields, b"parameters")?)?)
            .or(Err(error(b"policy_judge_parameters", b"integer bound")))?,
    })
}
fn policy_guard(d: &Decoder<'_>, at: u32) -> Result<c::PolicyGuard, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"guarded" => Ok(c::PolicyGuard::Guarded),
            _ => Err(error(b"policy_guard", b"unknown word")),
        };
    }
    let fields = d.object(at, &[b"observed"])?;
    if fields.len() != 1 {
        return Err(error(b"policy_guard", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"observed") {
        let fields = d.object(at, &[b"freshness"])?;
        return Ok(c::PolicyGuard::Observed {
            freshness: Duration::from_nanos(d.number(field(&fields, b"freshness")?)?),
        });
    }
    Err(error(b"policy_guard", b"unknown variant"))
}
fn owner_part_kind(d: &Decoder<'_>, at: u32) -> Result<c::PartKind, Error> {
    if d.kind(at)? == Kind::String {
        return match d.string(at)? {
            b"resource" => Ok(c::PartKind::Resource),
            b"pool" => Ok(c::PartKind::Pool),
            b"result" => Ok(c::PartKind::Result),
            b"procedure" => Ok(c::PartKind::Procedure),
            b"effect" => Ok(c::PartKind::Effect),
            b"outbox" => Ok(c::PartKind::Outbox),
            b"hold" => Ok(c::PartKind::Hold),
            _ => Err(error(b"owner_part_kind", b"unknown word")),
        };
    }
    Err(error(b"owner_part_kind", b"expected word"))
}
fn owner_part_key(d: &Decoder<'_>, at: u32) -> Result<c::PartKey, Error> {
    let fields = d.object(at, &[b"path", b"number"])?;
    if fields.len() != 1 {
        return Err(error(b"owner_part_key", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"path") {
        return Ok(c::PartKey::Path({
            let positions = d.array(at, d.limits.part_key_path_value, b"part_key_path_value")?;
            let mut values = List::with_capacity(positions.len());
            for at in &positions {
                values
                    .push(d.hex(*at, d.limits.part_key_path_value_item, b"part_key_path_value_item")?)
                    .expect("checked list length");
            }
            values
        }));
    }
    if let Some(at) = optional(&fields, b"number") {
        return Ok(c::PartKey::Number(d.number(at)?));
    }
    Err(error(b"owner_part_key", b"unknown variant"))
}
fn field_rule(d: &Decoder<'_>, at: u32) -> Result<c::FieldRule, Error> {
    let fields = d.object(at, &[b"name", b"max"])?;
    Ok(c::FieldRule {
        name: d.hex(field(&fields, b"name")?, d.limits.field_rule_name, b"field_rule_name")?,
        max: u32::try_from(d.number(field(&fields, b"max")?)?).or(Err(error(b"field_rule_max", b"integer bound")))?,
    })
}
fn owner_authority_executor(d: &Decoder<'_>, at: u32) -> Result<c::AuthorityExecutor, Error> {
    let fields = d.object(at, &[b"charter", b"procedure", b"role", b"job"])?;
    if fields.len() != 1 {
        return Err(error(b"owner_authority_executor", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"job") {
        return Ok(c::AuthorityExecutor::Job(
            u32::try_from(d.number(at)?).or(Err(error(b"owner_authority_executor_job", b"integer bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"charter") {
        return Ok(c::AuthorityExecutor::Charter(
            u32::try_from(d.number(at)?).or(Err(error(b"owner_authority_executor_charter", b"integer bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"procedure") {
        return Ok(c::AuthorityExecutor::Procedure(
            u32::try_from(d.number(at)?).or(Err(error(b"owner_authority_executor_procedure", b"integer bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"role") {
        return Ok(c::AuthorityExecutor::Role(
            u32::try_from(d.number(at)?).or(Err(error(b"owner_authority_executor_role", b"integer bound")))?,
        ));
    }
    Err(error(b"owner_authority_executor", b"unknown variant"))
}
fn owner_pattern_end(d: &Decoder<'_>, at: u32) -> Result<c::PatternEnd, Error> {
    let fields = d.object(at, &[b"exact", b"open"])?;
    if fields.len() != 1 {
        return Err(error(b"owner_pattern_end", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"exact") {
        return Ok(c::PatternEnd::Exact(d.hex(at, d.limits.pattern_end_exact_value, b"pattern_end_exact_value")?));
    }
    if let Some(at) = optional(&fields, b"open") {
        return Ok(c::PatternEnd::Open(d.hex(at, d.limits.pattern_end_open_value, b"pattern_end_open_value")?));
    }
    Err(error(b"owner_pattern_end", b"unknown variant"))
}

pub(super) fn catalogue(d: &Decoder<'_>, at: u32) -> Result<c::CatalogueEntry, Error> {
    let fields = d.object(
        at,
        &[b"number", b"model", b"shown", b"description", b"picker", b"withdrawn", b"source", b"estimated"],
    )?;
    let source = match d.string(field(&fields, b"source")?)? {
        b"stock" => c::ModelSource::Stock,
        b"list" => c::ModelSource::List,
        b"person" => c::ModelSource::Person,
        _ => return Err(error(b"source", b"unknown word")),
    };
    Ok(c::CatalogueEntry {
        number: u32::try_from(d.number(field(&fields, b"number")?)?).or(Err(error(b"number", b"integer bound")))?,
        model: setting_model(d, field(&fields, b"model")?)?,
        shown: catalogue_text(d, &fields, b"shown", d.limits.catalogue_entry_shown_some)?,
        description: catalogue_text(d, &fields, b"description", d.limits.catalogue_entry_description_some)?,
        picker: d.boolean(field(&fields, b"picker")?)?,
        withdrawn: d.boolean(field(&fields, b"withdrawn")?)?,
        source,
        estimated: d.boolean(field(&fields, b"estimated")?)?,
    })
}
fn catalogue_text(
    d: &Decoder<'_>,
    fields: &super::Fields,
    member: &[u8],
    bound: u32,
) -> Result<Option<c::Text>, Error> {
    let at = field(fields, member)?;
    if d.kind(at)? == Kind::Null { Ok(None) } else { Ok(Some(d.words(at, bound, member)?)) }
}
