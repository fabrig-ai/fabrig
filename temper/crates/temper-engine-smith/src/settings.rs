//! Shared ctl settings input and text, bounded for host tools (domain/agent/tools.md).
use crate::{Decoded, Problem, json::Value};
use alloc::boxed::Box;
use jig_domain_core::{SettingsAnswer, SettingsRead, settings as s};
use jig_protocol_engine_clients::{self as protocol, translate::settings as wire};
use settings_form::view as c;
use settings_form::wire as client;
use skein_lib::List;
use smith_domain_run as run;
use temper_engine_domain as engine;

fn only(value: &Value, fields: &[&[u8]], name: &'static [u8]) -> Result<(), Problem> {
    let Value::Object(items) = value else { return Err(Problem::Type(name)) };
    for (key, _) in items {
        if !fields.contains(&key.as_ref()) {
            return Err(Problem::Range(name));
        }
    }
    Ok(())
}
fn positive(value: &Value, name: &'static [u8]) -> Result<u32, Problem> {
    let n = value.small(name)?;
    if n == 0 { Err(Problem::Range(name)) } else { Ok(n) }
}
pub(crate) fn read(value: &Value, bytes: &[u8]) -> Result<engine::Tool, Problem> {
    only(value, &[b"setting", b"revision", b"history", b"from"], b"settings")?;
    let from = match value.field(b"from")? {
        Some(v) => v.small(b"from")?,
        None => 0,
    };
    let revision = match value.field(b"revision")? {
        Some(v) => Some(positive(v, b"revision")?),
        None => None,
    };
    let setting = value.field(b"setting")?;
    let history = value.field(b"history")?;
    let read = if let Some(history) = history {
        if setting.is_some() || revision.is_some() {
            return Err(Problem::Range(b"history"));
        }
        only(history, &[b"before"], b"history")?;
        let before = match history.field(b"before")? {
            Some(v) => Some(positive(v, b"before")?),
            None => None,
        };
        SettingsRead::Revisions { before }
    } else if let Some(setting) = setting {
        only(setting, &[b"kind", b"number", b"project"], b"setting")?;
        let kind = setting.required(b"kind")?.text(b"kind")?;
        let number = setting.field(b"number")?;
        let project = setting.field(b"project")?;
        let mut numbered = None;
        for name in [
            b"charter".as_slice(),
            b"template",
            b"policy",
            b"model",
            b"landing-rule",
            b"forge-charter",
            b"charter-words",
            b"template-goal",
        ] {
            if kind.as_ref() == name {
                numbered = Some(name);
            }
        }
        let mut by_project = None;
        for name in [
            b"project-policy".as_slice(),
            b"project-ways",
            b"project-questions",
            b"project-classes",
            b"project-words",
            b"project-areas",
            b"project-triager",
        ] {
            if kind.as_ref() == name {
                by_project = Some(name);
            }
        }
        if project.is_some() && kind.as_ref() != b"landing-rule" && by_project.is_none() {
            return Err(Problem::Range(b"project"));
        }
        if kind.as_ref() == b"rules" || by_project.is_some() {
            if number.is_some() {
                return Err(Problem::Range(b"number"));
            }
        } else if let Some(kind) = numbered {
            positive(number.ok_or(Problem::Needs { kind, member: b"number" })?, b"number")?;
        } else if let Some(number) = number {
            positive(number, b"number")?;
        }
        if let Some(project) = project {
            positive(project, b"project")?;
        } else if let Some(kind) = by_project {
            return Err(Problem::Needs { kind, member: b"project" });
        }
        let setting = settings_form::file::read_setting(bytes, &settings_form::Limits::standard())
            .or(Err(Problem::Range(b"kind")))?;
        let setting = match setting {
            c::Setting::Charter(value) => s::Setting::Charter(value.number),
            c::Setting::Template(value) => s::Setting::Template(value.number),
            c::Setting::Policy(value) => s::Setting::Policy(value.number),
            c::Setting::Model(value) => s::Setting::Model(value.number),
            c::Setting::Rules => s::Setting::Rules,
            c::Setting::Application { kind, scope, number } => s::Setting::Application { kind, scope, number },
            c::Setting::ProjectPolicy { project } => s::Setting::ProjectPolicy { project },
        };
        SettingsRead::Setting { setting, revision: revision.unwrap_or(0) }
    } else if let Some(number) = revision {
        SettingsRead::Revision { number }
    } else {
        SettingsRead::Inventory
    };
    Ok(engine::Tool::Settings { read, from })
}

fn models(values: &[s::CatalogueModel]) -> Box<[protocol::Model]> {
    protocol::translate::llm::catalogue(values)
}
pub(crate) fn proposal(
    name: run::CallName,
    bytes: &[u8],
    configured: &[s::CatalogueModel],
    repositories: &[engine::RepositoryIdentity],
) -> Result<Decoded, Problem> {
    let value = crate::json::parse(bytes)?;
    only(&value, &[b"against", b"items", b"note", b"reason", b"more"], b"propose_settings")?;
    let more = match value.field(b"more")? {
        Some(more) => more.boolean(b"more")?,
        None => false,
    };
    let against = positive(value.required(b"against")?, b"against")?;
    let reason = value.required(b"reason")?.text(b"reason")?;
    if let Some(note) = value.field(b"note")? {
        if value.field(b"items")?.is_some() {
            return Err(Problem::Range(b"items"));
        }
        if value.field(b"more")?.is_some() {
            return Err(Problem::Range(b"more"));
        }
        only(note, &[b"name", b"revision"], b"note")?;
        let note_name = note.required(b"name")?.number(b"name")?;
        if note_name == 0 {
            return Err(Problem::Range(b"name"));
        }
        return Ok(Decoded::Call(engine::Call {
            completion: name.completion,
            position: name.position,
            tool: engine::Tool::ProposeSettingsNote {
                against,
                reason,
                note: note_name,
                revision: positive(note.required(b"revision")?, b"revision")?,
            },
        }));
    }
    value.required(b"items")?;
    let limits = settings_form::Limits::standard();
    let mut identities = List::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
    for repository in repositories {
        identities
            .push(settings_form::file::Repository {
                host: &repository.host,
                owner: &repository.owner,
                name: &repository.name,
            })
            .expect("one resolver per repository");
    }
    let items = settings_form::file::proposal_items_in(bytes, &limits, Some(identities.as_slice()));
    proposal_items(name.completion, name.position, against, reason, items, configured, more)
}

/// Decode an authorized, revision-checked note's whole settings file through
/// the same parser and validation as inline proposals (domain/agent/agents.md, 1).
pub fn settings_note(call: engine::Call, bytes: &[u8], configured: &[s::CatalogueModel]) -> Result<Decoded, Problem> {
    let engine::Tool::ProposeSettingsNote { against, reason, .. } = call.tool else {
        return Err(Problem::Range(b"note"));
    };
    let items = settings_form::file::decode(bytes, &settings_form::Limits::standard());
    proposal_items(call.completion, call.position, against, reason, items, configured, false)
}

fn proposal_items(
    completion: u32,
    position: u32,
    against: u32,
    reason: Box<[u8]>,
    items: Result<List<c::SettingItem>, Box<[u8]>>,
    configured: &[s::CatalogueModel],
    more: bool,
) -> Result<Decoded, Problem> {
    let limits = settings_form::Limits::standard();
    let items = match items {
        Ok(items) => items,
        Err(why) => return Ok(Decoded::Answered(run::HostAnswer::new(why, true).ok_or(Problem::TooLarge)?)),
    };
    let wire = client::wire_change_settings(
        &c::ChangeSettings { against, items, running: c::Running::Stays },
        &limits.settings,
    )
    .or(Err(Problem::Range(b"items")))?;
    let items = match wire::items(wire.into_parts().items, &models(configured), &limits.settings) {
        Ok(items) => items,
        Err(why) => {
            if why.field == jig_client_protocol_wire::v19::Path::TemplateLlm {
                return Err(Problem::Range(b"items.template.llm"));
            }
            return Err(Problem::Range(b"items"));
        }
    };
    let kinds = kind_values(&items)?;
    Ok(Decoded::Call(engine::Call {
        completion,
        position,
        tool: engine::Tool::ProposeSettings { against, items, reason, kinds, more },
    }))
}

/// Type application values at the schema ceilings; the forge judges deployment bounds
/// (temper/docs/design/domain/routing.md, 6).
pub(super) fn kind_values(items: &[s::Item]) -> Result<Box<[engine::KindValue]>, Problem> {
    let ceilings = temper_client_protocol_wire::v1::CEILINGS;
    let bounds = crate::kinds::Bounds {
        segments: ceilings.landing_pattern_segments,
        segment_bytes: ceilings.landing_pattern_segments_item,
        gates: ceilings.landing_rule_gates,
        approvals: ceilings.landing_rule_approvals,
    };
    let mut values = List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let s::Setting::Application { kind, scope, number } = item.setting else { continue };
        if !temper_client_protocol_wire::settings_kind(kind) {
            continue;
        }
        let bytes = match &item.edit {
            s::Edit::Set(s::Value::Application(bytes)) => bytes,
            s::Edit::Set(_) => return Err(Problem::Range(b"items")),
            s::Edit::Removed | s::Edit::Reset => continue,
        };
        let value = crate::kinds::decode(kind, bytes, &bounds).ok_or(Problem::Range(b"items"))?;
        values.push(engine::KindValue { kind, scope, number, value }).expect("one value per set item");
    }
    Ok(values.into_boxed())
}

/// Render the same settings document through the same text layer as ctl.
pub fn settings_text(
    answer: &SettingsAnswer,
    requested: Option<u64>,
    configured: &[s::CatalogueModel],
) -> Result<Box<[u8]>, Problem> {
    let limits = settings_form::Limits::standard();
    let model = models(configured);
    let value = match answer {
        SettingsAnswer::Inventory(value) => c::ReadValue::Settings(Box::new(
            client::restore_settings_document(
                &wire::document(value, &limits.settings).or(Err(Problem::Range(b"items")))?,
                &limits,
            )
            .or(Err(Problem::Range(b"items")))?,
        )),
        SettingsAnswer::Setting(value) => c::ReadValue::Setting(Box::new(
            client::restore_setting_document(
                &wire::setting_document(value, &model, &limits.settings).or(Err(Problem::Range(b"items")))?,
                &limits,
            )
            .or(Err(Problem::Range(b"items")))?,
        )),
        SettingsAnswer::Revision(value) => c::ReadValue::Revision(Box::new(
            client::restore_revision_answer(
                &wire::revision(value, &model, &limits.settings).or(Err(Problem::Range(b"items")))?,
                &limits,
            )
            .or(Err(Problem::Range(b"items")))?,
        )),
        SettingsAnswer::Revisions(value) => c::ReadValue::Revisions(Box::new(
            client::restore_revisions_page(
                &wire::revisions(value, &limits.settings).or(Err(Problem::Range(b"items")))?,
                &limits,
            )
            .or(Err(Problem::Range(b"items")))?,
        )),
        SettingsAnswer::Unknown => return Ok(Box::from(b"not read: no such setting at that revision. This says nothing about the settings: read the inventory; project-ways, project-questions, project-classes, project-words: project, no number; charter-words, template-goal: number, no project.\n".as_slice())),
    };
    Ok(settings_form::settings_text(&value, requested, &limits))
}

/// Render a settings page with the default documentation-sized host allowance.
#[must_use]
pub fn settings_answer(
    answer: &SettingsAnswer,
    from: u32,
    requested: Option<u64>,
    models: &[s::CatalogueModel],
    application: Option<&temper_engine_domain_forge::settings::SizedValue>,
) -> run::HostAnswer {
    settings_answer_bounded(answer, from, requested, models, application, crate::DOC_ANSWER)
}

/// Render settings within the supplied host body allowance, reserving escaped
/// framing and continuation markers. Offsets count original UTF-8 text bytes.
#[must_use]
pub fn settings_answer_bounded(
    answer: &SettingsAnswer,
    from: u32,
    requested: Option<u64>,
    models: &[s::CatalogueModel],
    application: Option<&temper_engine_domain_forge::settings::SizedValue>,
    max_bytes: u32,
) -> run::HostAnswer {
    settings_repository_answer(answer, from, requested, models, application, &[], &[], &[], &[], max_bytes)
}

/// Render an authorized inventory's repository and people context within the
/// host allowance and the same byte-offset pagination as its settings.
#[must_use]
#[expect(clippy::too_many_arguments, reason = "inventory context travels beside the settings read and host allowance")]
pub fn settings_repository_answer(
    answer: &SettingsAnswer,
    from: u32,
    requested: Option<u64>,
    models: &[s::CatalogueModel],
    application: Option<&temper_engine_domain_forge::settings::SizedValue>,
    repositories: &[engine::RepositoryIdentity],
    people: &[engine::ProjectPerson],
    forge_people: &[engine::ForgePerson],
    offer: &[engine::OfferedModel],
    max_bytes: u32,
) -> run::HostAnswer {
    let maximum = skein_lib::usize_of(max_bytes).min(run::HostAnswer::CAPACITY);
    let result = settings_page(
        answer,
        from,
        requested,
        models,
        application,
        repositories,
        people,
        forge_people,
        offer,
        u32::try_from(maximum).expect("host bound"),
    );
    match result {
        Ok(answer) => answer,
        Err(message) => run::HostAnswer::new(
            skein_lib::bytes::copy_of(message.get(..maximum.min(message.len())).expect("bounded refusal")),
            true,
        )
        .expect("host feedback fits"),
    }
}

#[expect(clippy::too_many_arguments, reason = "inventory context travels beside the settings read and host allowance")]
fn settings_page(
    answer: &SettingsAnswer,
    from: u32,
    requested: Option<u64>,
    models: &[s::CatalogueModel],
    application: Option<&temper_engine_domain_forge::settings::SizedValue>,
    repositories: &[engine::RepositoryIdentity],
    people: &[engine::ProjectPerson],
    forge_people: &[engine::ForgePerson],
    offer: &[engine::OfferedModel],
    maximum: u32,
) -> Result<run::HostAnswer, &'static [u8]> {
    match settings_whole(answer, requested, models, application, repositories, people, forge_people, offer) {
        Ok(text) => match settings_form::page::page_bounded(&text, from, maximum) {
            Ok(body) => {
                let error = match answer {
                    SettingsAnswer::Unknown => true,
                    SettingsAnswer::Inventory(_)
                    | SettingsAnswer::Setting(_)
                    | SettingsAnswer::Revision(_)
                    | SettingsAnswer::Revisions(_) => false,
                };
                Ok(run::HostAnswer::new(body, error).expect("settings page bound"))
            }
            Err(settings_form::page::Error::Offset) => Err(b"from: not a UTF-8 text offset"),
            Err(settings_form::page::Error::TooLarge) => Err(b"settings: host answer limit cannot hold a page"),
        },
        Err(message) => Err(message),
    }
}

/// The whole settings text, including an inventory's read context, before its page.
#[expect(clippy::too_many_arguments, reason = "inventory context travels beside the settings read")]
pub(crate) fn settings_whole(
    answer: &SettingsAnswer,
    requested: Option<u64>,
    models: &[s::CatalogueModel],
    application: Option<&temper_engine_domain_forge::settings::SizedValue>,
    repositories: &[engine::RepositoryIdentity],
    people: &[engine::ProjectPerson],
    forge_people: &[engine::ForgePerson],
    offer: &[engine::OfferedModel],
) -> Result<Box<[u8]>, &'static [u8]> {
    let enriched;
    let answer = if let (SettingsAnswer::Setting(document), Some(value)) = (answer, application) {
        let mut document = document.clone();
        document.value = Some(s::Value::Application(crate::kinds::encode(&value.value)));
        enriched = SettingsAnswer::Setting(document);
        &enriched
    } else {
        answer
    };
    match settings_text(answer, requested, models) {
        Ok(text) => Ok(repository_text(answer, text, repositories, people, forge_people, offer)),
        Err(_) => Err(b"settings: document limit"),
    }
}

fn repository_text(
    answer: &SettingsAnswer,
    text: Box<[u8]>,
    repositories: &[engine::RepositoryIdentity],
    people: &[engine::ProjectPerson],
    forge_people: &[engine::ForgePerson],
    offer: &[engine::OfferedModel],
) -> Box<[u8]> {
    if let SettingsAnswer::Inventory(_) = answer {
        let mut measure = settings_form::Text::measure();
        repository_rows(&mut measure, &text, repositories, people, forge_people, offer);
        let mut out = settings_form::Text::writing_exact(measure);
        repository_rows(&mut out, &text, repositories, people, forge_people, offer);
        out.finish()
    } else {
        text
    }
}

fn repository_rows(
    out: &mut settings_form::Text,
    text: &[u8],
    repositories: &[engine::RepositoryIdentity],
    people: &[engine::ProjectPerson],
    forge_people: &[engine::ForgePerson],
    offer: &[engine::OfferedModel],
) {
    out.put(text);
    if !repositories.is_empty() {
        out.put(b"project repositories:\n");
    }
    for repository in repositories {
        out.put(b"  forge ");
        out.put(&repository.host);
        out.put(b" repository ");
        out.put(&repository.owner);
        out.put(b"/");
        out.put(&repository.name);
        out.put(b"\n");
    }
    if !people.is_empty() {
        out.put(b"project people:\n");
    }
    project_people_rows(out, people);
    if !forge_people.is_empty() {
        out.put(b"forge people:\n");
    }
    for person in forge_people {
        out.put(b"  forge ");
        out.put(&person.host);
        out.put(b" configured owned-forge account ");
        out.put(&person.login);
        out.put(b"; project membership unverified by this row\n");
    }
    if offer.is_empty() {
        out.put(b"models you can offer: none; no account can run a model yet\n");
    } else {
        out.put(b"models you can offer:\n");
    }
    for model in offer {
        out.put(b"  ");
        out.put(&model.name);
        out.put(b": ");
        out.put(&model.provider);
        out.put(b"; efforts ");
        let mut separator = b"".as_slice();
        for effort in &model.efforts {
            out.put(separator);
            out.put(effort);
            separator = b", ";
        }
        if model.efforts.is_empty() {
            out.put(b"none");
        }
        out.put(b"; default ");
        match &model.effort {
            Some(effort) => out.put(effort),
            None => out.put(b"none"),
        }
        out.put(b"\n");
    }
}

fn project_people_rows(out: &mut settings_form::Text, people: &[engine::ProjectPerson]) {
    for person in people {
        out.put(b"  person ");
        out.number(person.person);
        if !person.login.is_empty() {
            out.put(b" ");
            out.put(&person.login);
        }
        out.put(b": role ");
        out.number(u64::from(person.role));
        if person.you {
            out.put(b", requester");
        }
        let mut known = false;
        for forge in &person.forges {
            known |= forge.user.is_some();
        }
        if known {
            for forge in &person.forges {
                out.put(b", forge ");
                out.put(&forge.host);
                match forge.user {
                    Some(user) => {
                        out.put(b" user ");
                        out.number(user);
                        if forge.writer {
                            out.put(b", temper's own forge writer: never counts as a person");
                        }
                    }
                    None => out.put(b" unknown: cannot count for approvals there"),
                }
            }
        } else {
            out.put(b", forge identity unknown: cannot count for approvals");
        }
        out.put(b"\n");
    }
}

pub(crate) fn refusal(why: &jig_domain_people::Refusal) -> Box<[u8]> {
    use jig_domain_people::{Malformed, Refusal};
    match why {
        Refusal::Forgotten { oldest } => {
            let mut measure = settings_form::Text::measure();
            measure.put(b"forgotten; oldest kept revision ");
            measure.number(u64::from(*oldest));
            let mut out = settings_form::Text::writing_exact(measure);
            out.put(b"forgotten; oldest kept revision ");
            out.number(u64::from(*oldest));
            return out.finish();
        }
        Refusal::Judged { why, .. } => {
            let mut out = settings_form::Text::measure();
            out.put(b"refused by its owner: ");
            out.put(why);
            let mut rendered = settings_form::Text::writing_exact(out);
            rendered.put(b"refused by its owner: ");
            rendered.put(why);
            return rendered.finish();
        }
        Refusal::SignInRedirect { .. }
        | Refusal::Stale { .. }
        | Refusal::Full { .. }
        | Refusal::NameTaken
        | Refusal::AccountMove { .. }
        | Refusal::Circle { .. }
        | Refusal::Dependency { .. }
        | Refusal::Kept { .. }
        | Refusal::Account { .. }
        | Refusal::NotWellFormed { .. }
        | Refusal::Role { .. }
        | Refusal::Authority { .. }
        | Refusal::Limit { .. }
        | Refusal::Moved { .. }
        | Refusal::Ended { .. }
        | Refusal::Taken { .. }
        | Refusal::NotWaiting
        | Refusal::NoFurther
        | Refusal::Unknown
        | Refusal::ProgramNameReserved
        | Refusal::NotOffered
        | Refusal::Provider
        | Refusal::KeyConflict
        | Refusal::NoEscalationHolder
        | Refusal::Llm { .. } => {}
    }
    let view = if let Refusal::Stale { newest } = why {
        settings_form::Refusal::Stale(*newest)
    } else if let Refusal::NotWellFormed { at, why: Malformed::Settings { setting, fault, detail } } = why {
        malformed_setting(*at, setting, *fault, *detail)
    } else if let Refusal::NotWellFormed { why: Malformed::SettingsUnchanged, .. } = why {
        settings_form::Refusal::Unchanged
    } else if let Refusal::Authority { lacking: jig_domain_people::Lack::Approver, .. } = why {
        settings_form::Refusal::Authority
    } else if let Refusal::Limit { .. } = why {
        settings_form::Refusal::Limit
    } else {
        settings_form::Refusal::NotOffered
    };
    settings_form::refusal_text(&view)
}

/// Translate the document refusal facts without checking the settings again.
fn setting_detail(value: jig_domain_people::SettingDetail) -> c::SettingDetail {
    c::SettingDetail {
        member: match value.member {
            jig_domain_people::SettingMember::Documents => c::SettingMember::Documents,
            jig_domain_people::SettingMember::NameSegments => c::SettingMember::NameSegments,
            jig_domain_people::SettingMember::NameSegmentBytes => c::SettingMember::NameSegmentBytes,
            jig_domain_people::SettingMember::Path => c::SettingMember::Path,
            jig_domain_people::SettingMember::Heading => c::SettingMember::Heading,
            jig_domain_people::SettingMember::Connector => c::SettingMember::Connector,
            jig_domain_people::SettingMember::StoredPolicy => c::SettingMember::StoredPolicy,
        },
        rule: match value.rule {
            jig_domain_people::SettingRule::Maximum => c::SettingRule::Maximum,
            jig_domain_people::SettingRule::Minimum => c::SettingRule::Minimum,
            jig_domain_people::SettingRule::Registered => c::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
fn malformed_setting(
    at: u32,
    setting: &s::Setting,
    fault: jig_domain_people::SettingFault,
    detail: Option<jig_domain_people::SettingDetail>,
) -> settings_form::Refusal {
    let setting = match setting {
        s::Setting::ProjectPolicy { project } => c::Setting::ProjectPolicy { project: *project },
        s::Setting::Charter(number) => c::Setting::Charter(c::SettingNumber { number: *number }),
        s::Setting::Template(number) => c::Setting::Template(c::SettingNumber { number: *number }),
        s::Setting::Policy(number) => c::Setting::Policy(c::SettingNumber { number: *number }),
        s::Setting::Model(number) => c::Setting::Model(c::SettingNumber { number: *number }),
        s::Setting::Rules => c::Setting::Rules,
        s::Setting::Application { kind, scope, number } => {
            c::Setting::Application { kind: *kind, scope: *scope, number: *number }
        }
    };
    let fault = match fault {
        jig_domain_people::SettingFault::Shape => c::SettingFault::Shape,
        jig_domain_people::SettingFault::Count => c::SettingFault::Count,
        jig_domain_people::SettingFault::Reference => c::SettingFault::Reference,
        jig_domain_people::SettingFault::Rules => c::SettingFault::Rules,
        jig_domain_people::SettingFault::Budget => c::SettingFault::Budget,
        jig_domain_people::SettingFault::Grace => c::SettingFault::Grace,
        jig_domain_people::SettingFault::Owner => c::SettingFault::Owner,
        jig_domain_people::SettingFault::Judged => c::SettingFault::Judged,
    };
    settings_form::Refusal::Malformed {
        at,
        setting,
        fault,
        detail: match detail {
            Some(value) => Some(setting_detail(value)),
            None => None,
        },
    }
}

/// Render unrecorded parts in the contract's words (domain/agent/ask-temper.md, 2).
pub(crate) fn parts(text: &mut crate::answers::Text, parts: &engine::PartsAnswer) -> Result<bool, Problem> {
    use engine::PartsAnswer as P;
    text.add(b"propose_settings: ")?;
    match parts {
        P::Holding { count, total, against, dropped } => {
            text.add(b"holding ")?;
            part_items(text, *count)?;
            text.add(b" for revision ")?;
            text.number(u64::from(*against))?;
            text.add(b", ")?;
            text.number(u64::from(*total))?;
            text.add(b" in all; ")?;
            match dropped {
                Some((count, revision)) => {
                    text.add(b"the ")?;
                    text.number(u64::from(*count))?;
                    text.add(b" held for revision ")?;
                    text.number(u64::from(*revision))?;
                    text.add(b" were dropped. Nothing is proposed until a call without more.")?;
                }
                None => text.add(b"nothing is proposed until a call without more.")?,
            }
            return Ok(false);
        }
        P::Held { setting } => {
            let setting = match setting {
                s::Setting::Charter(number) => c::Setting::Charter(c::SettingNumber { number: *number }),
                s::Setting::Template(number) => c::Setting::Template(c::SettingNumber { number: *number }),
                s::Setting::Policy(number) => c::Setting::Policy(c::SettingNumber { number: *number }),
                s::Setting::Model(number) => c::Setting::Model(c::SettingNumber { number: *number }),
                s::Setting::ProjectPolicy { project } => c::Setting::ProjectPolicy { project: *project },
                s::Setting::Rules => c::Setting::Rules,
                s::Setting::Application { kind, scope, number } => c::Setting::Application { kind: *kind, scope: *scope, number: *number },
            };
            let mut measure = settings_form::Text::measure();
            settings_form::render::setting_text(setting, &mut measure);
            let mut out = settings_form::Text::writing_exact(measure);
            settings_form::render::setting_text(setting, &mut out);
            text.add(&out.finish())?;
            text.add(b" is held already, from an earlier part; nothing was added.")?;
        }
        P::PastBound { most, would } => {
            text.add(b"a proposal holds at most ")?;
            part_items(text, *most)?;
            text.add(b", and this call would make ")?;
            text.number(u64::from(*would))?;
            text.add(b"; nothing was added.")?;
        }
        P::Revision { held, against, named } => {
            text.add(b"the ")?;
            part_items(text, *held)?;
            text.add(b" held are for revision ")?;
            text.number(u64::from(*against))?;
            text.add(b", and this call names ")?;
            text.number(u64::from(*named))?;
            text.add(b"; nothing was proposed, and they were dropped. Give the whole set again against ")?;
            text.number(u64::from(*named))?;
            text.add(b".")?;
        }
        P::Restarted => text.add(b"temper restarted during this run, so parts it gave may be lost; nothing was proposed. Give the whole set again, in parts if it is large.")?,
        P::NotOffered => text.add(b"more is not offered to this charter; give the whole set in one call, or from a note.")?,
        P::Approvals { why } => {
            text.add(b"not proposed, as acceptance would refuse it: ")?;
            text.add(why)?;
        }
    }
    Ok(true)
}
fn part_items(text: &mut crate::answers::Text, count: u32) -> Result<(), Problem> {
    text.number(u64::from(count))?;
    text.add(if count == 1 { b" item" } else { b" items" })
}

#[cfg(test)]
mod catalogue_tests {
    use super::*;
    #[test]
    fn settings_models_keep_the_catalogues_metadata() {
        let mut model = engine::Model::hole();
        model.provider = 2;
        model.name = Box::from(b"gpt-6.1-terra".as_slice());
        let entry = s::CatalogueModel {
            first_read: None,
            number: 4,
            model,
            shown: Some(Box::from(b"GPT-6.1 Terra".as_slice())),
            description: Some(Box::from(b"For careful work".as_slice())),
            picker: false,
            withdrawn: false,
            source: s::ModelSource::Person,
            estimated: true,
        };
        let models = models(core::slice::from_ref(&entry));
        let model = models.get(3).expect("catalogue row");
        assert_eq!(models.len(), 4);
        assert!(models.first().expect("hole").name.is_empty());
        assert_eq!(model.number, 4);
        assert_eq!(model.provider, Some(2));
        assert_eq!(model.shown, entry.shown);
        assert_eq!(model.description, entry.description);
        assert!(!model.picker);
        assert_eq!(model.source, s::ModelSource::Person);
        assert!(model.estimated);
    }
}
