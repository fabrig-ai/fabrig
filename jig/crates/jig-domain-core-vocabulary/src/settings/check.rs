//! Whole-set settings value checks (domain/engine/settings.md, 1).
use crate::{
    Limits, Settings,
    settings::{Current, Edit, Failure, Fault, Item, Setting, Value},
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::length::len_u64;

// Both passes borrow the same two maps. The flag selects one view; no values
// or borrowed generic wrapper are materialised.
fn base_value(current: &Current, defaults: bool, setting: Setting) -> Option<&Value> {
    if defaults {
        match current.defaults.get(&setting) {
            Some(default) => Some(&default.value),
            None => None,
        }
    } else {
        current.value(setting)
    }
}
#[must_use]
pub fn edit(items: &[Item], setting: Setting) -> Option<&Edit> {
    for item in items {
        if item.setting == setting {
            return Some(&item.edit);
        }
    }
    None
}
// The shared lifetime borrows a value from either the current maps or the decision; no view is stored.
#[must_use]
pub fn overlay_value<'a>(
    current: &'a Current,
    defaults: bool,
    items: &'a [Item],
    setting: Setting,
) -> Option<&'a Value> {
    match edit(items, setting) {
        Some(Edit::Set(value)) => Some(value),
        Some(Edit::Removed) => None,
        Some(Edit::Reset) => base_value(current, true, setting),
        None => base_value(current, defaults, setting),
    }
}
#[must_use]
pub fn offered(current: &Current, defaults: bool, items: &[Item]) -> skein_lib::List<Setting> {
    let count = current
        .defaults
        .capacity()
        .checked_add(current.changes.capacity())
        .expect("settings union")
        .checked_add(u32::try_from(items.len()).expect("bounded items"))
        .expect("overlay names");
    let mut names = skein_lib::Map::with_capacity(count);
    for (&setting, _) in &current.defaults {
        names.insert(setting, ()).expect("default names");
    }
    if !defaults {
        for (&setting, _) in &current.changes {
            names.insert(setting, ()).expect("change names");
        }
        for item in items {
            if !super::application::is_application(item.setting) {
                names.insert(item.setting, ()).expect("overlay names");
            }
        }
    }
    let mut result = skein_lib::List::with_capacity(names.len());
    for (&setting, ()) in &names {
        let offered = overlay_value(current, defaults, items, setting);
        if offered.is_some() {
            result.push(setting).expect("offered names");
        }
    }
    result
}
/// Check proposed settings through the same borrowed selection as startup.
pub fn check_change(
    current: &Current,
    items: &[Item],
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Result<(), Failure> {
    super::change_shape(items, limits)?;
    for item in items {
        let number = match item.setting {
            Setting::Model(number) => number,
            Setting::Charter(_)
            | Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Rules
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. } => continue,
        };
        let failure = Failure { setting: item.setting, fault: Fault::Shape, detail: None };
        match &item.edit {
            Edit::Reset => {
                if current.default_value(item.setting).is_none() {
                    return Err(failure);
                }
            }
            Edit::Removed => {}
            Edit::Set(value) => {
                let value = value.as_model().ok_or(failure)?;
                for previous in [current.default_value(item.setting), current.value(item.setting)] {
                    if let Some(previous) = previous
                        && let Some(previous) = previous.as_model()
                        && previous.model.name != value.model.name
                    {
                        return Err(failure);
                    }
                }
                if let Err(fault) = super::catalogue::shape(value, number, settings) {
                    return Err(Failure { fault, ..failure });
                }
            }
        }
    }
    check_set(current, false, items, settings, connectors, limits)
}
#[expect(
    clippy::too_many_lines,
    reason = "the whole-set check judges catalogue, charters, templates, policies and rules"
)]
pub fn check_set(
    current: &Current,
    defaults: bool,
    items: &[Item],
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Result<(), Failure> {
    let mut counts = [0_u32; 3];
    for (&setting, _) in &current.defaults {
        count(setting, &mut counts, limits)?;
    }
    counts = [0; 3];
    for (&setting, _) in &current.changes {
        let retained = match edit(items, setting) {
            Some(Edit::Reset) => false,
            Some(Edit::Removed) => current.defaults.get(&setting).is_some(),
            Some(Edit::Set(_)) | None => true,
        };
        if !retained {
            continue;
        }
        count(setting, &mut counts, limits)?;
    }
    for item in items {
        let retained = match &item.edit {
            Edit::Set(_) => true,
            Edit::Removed => current.defaults.get(&item.setting).is_some(),
            Edit::Reset => false,
        };
        if current.changes.get(&item.setting).is_none() && retained {
            count(item.setting, &mut counts, limits)?;
        }
    }
    counts = [0; 3];
    let names = offered(current, defaults, items);
    for &setting in &names {
        count(setting, &mut counts, limits)?;
    }
    let rules = match overlay_value(current, defaults, items, Setting::Rules) {
        Some(value) => value.as_rules(),
        None => None,
    }
    .ok_or(Failure { setting: Setting::Rules, fault: Fault::Rules, detail: None })?;
    if let Some(fault) = rules_fault(rules, &limits.authority, Fault::Rules) {
        return Err(Failure { setting: Setting::Rules, fault, detail: None });
    }
    let mut catalogue = super::catalogue::overlay(current, defaults, items);
    if catalogue.is_empty() && super::catalogue::entries(current).is_empty() {
        catalogue = super::catalogue::defaults(settings);
    }
    for (at, model) in catalogue.iter().enumerate() {
        for previous in catalogue.iter().take(at) {
            if previous.model.name == model.model.name {
                return Err(Failure { setting: Setting::Model(model.number), fault: Fault::Shape, detail: None });
            }
        }
    }
    let models = super::catalogue::rows(&catalogue);
    if !authority_refs(current, defaults, items, &rules.ceiling, None, connectors, &models)
        || !requirement_refs(&rules.requirements, connectors)
    {
        return Err(Failure { setting: Setting::Rules, fault: Fault::Reference, detail: None });
    }
    for pair in rules.implies.entries() {
        if !connectors.contains(&pair.connector) {
            return Err(Failure { setting: Setting::Rules, fault: Fault::Reference, detail: None });
        }
    }
    // Hidden defaults still obey their own byte bounds; removals do not remove that cost.
    for (&setting, default) in &current.defaults {
        shape(setting, &default.value, limits)?;
    }
    for (&setting, change) in &current.changes {
        if let Some(value) = &change.value {
            shape(setting, value, limits)?;
        }
    }

    for &setting in &names {
        let stored = overlay_value(current, defaults, items, setting).expect("offered setting");
        let made = match stored {
            Value::Overlay(_) => {
                let charter = current.made_charter(setting, stored).ok_or(Failure {
                    setting,
                    fault: Fault::Reference,
                    detail: None,
                })?;
                if charter.policy.resume && !charter.parts.is_empty() {
                    return Err(Failure { setting, fault: Fault::Shape, detail: None });
                }
                let value = Value::Charter(Box::new(charter));
                shape(setting, &value, limits)?;
                Some(value)
            }
            Value::Charter(_)
            | Value::Template(_)
            | Value::Policy(_)
            | Value::Rules(_)
            | Value::Model(_)
            | Value::Application(_) => None,
        };
        let fault = match made.as_ref().unwrap_or(stored) {
            Value::Charter(charter) => {
                registered_documents(setting, charter, connectors)?;
                match named_fault(charter, &models, limits) {
                    Some(fault) => Some(fault),
                    None => match super::charters::resolved(charter, &catalogue, limits) {
                        Some(resolved) => {
                            if let Some(failure) = charter_fault(setting, &resolved, limits) {
                                return Err(failure);
                            }
                            None
                        }
                        None => Some(Fault::Reference),
                    },
                }
            }
            Value::Template(template) => {
                match template.flags.ended_by {
                    tasks::EndedBy::Result | tasks::EndedBy::Requester | tasks::EndedBy::Standing => {}
                    tasks::EndedBy::Closer(closer) => {
                        if !connectors.contains(&closer.connector) {
                            return Err(Failure { setting, fault: Fault::Reference, detail: None });
                        }
                    }
                }
                if let Some(fault) = contract_fault(&template.contract, connectors) {
                    return Err(Failure { setting, fault, detail: None });
                }
                template_fault(current, defaults, items, template, template.executor, &models, limits)
            }
            Value::Policy(policy) => match authority::policy_refusal(rules, &limits.authority, &policy.policy) {
                Some(authority::PolicyRefusal::AboveRules) => Some(Fault::Rules),
                Some(authority::PolicyRefusal::Oversized | authority::PolicyRefusal::InvalidRole) => Some(Fault::Shape),
                Some(authority::PolicyRefusal::Full) => Some(Fault::Count),
                None => policy_fault(current, defaults, items, policy, &models, connectors, limits),
            },
            Value::Model(value) => super::catalogue::shape(value, value.number, settings).err(),
            Value::Overlay(_) => unreachable!("overlay made into a charter"),
            Value::Rules(_) | Value::Application(_) => None,
        };
        if let Some(fault) = fault {
            return Err(Failure { setting, fault, detail: None });
        }
    }
    Ok(())
}
pub fn shape(setting: Setting, value: &Value, limits: &Limits) -> Result<(), Failure> {
    let valid = match value {
        Value::Overlay(_) => match setting {
            Setting::Charter(number) => number != 0,
            Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Model(_)
            | Setting::Rules
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. } => false,
        },
        Value::Charter(value) => {
            setting == Setting::Charter(value.number) && value.number != 0 && tasks::wake_valid(&value.wake)
        }
        Value::Template(value) => {
            setting == Setting::Template(value.number)
                && value.number != 0
                && tasks::valid_contract(&limits.tasks, &value.contract)
        }
        Value::Policy(_) => match setting {
            Setting::Policy(number) => number != 0,
            Setting::ProjectPolicy { project } => project != 0,
            Setting::Charter(_)
            | Setting::Template(_)
            | Setting::Model(_)
            | Setting::Rules
            | Setting::Application { .. } => false,
        },
        Value::Rules(value) => {
            if setting == Setting::Rules
                && let Some(fault) = rules_fault(value, &limits.authority, Fault::Shape)
            {
                return Err(Failure { setting, fault, detail: None });
            }
            setting == Setting::Rules
        }
        Value::Model(value) => setting == Setting::Model(value.number) && super::catalogue::fits(value, value.number),
        Value::Application(_) => super::application::is_application(setting),
    };
    if !valid {
        return Err(Failure { setting, fault: Fault::Shape, detail: None });
    }
    restore_fit(setting, value, limits)
}
/// Stored values retain their byte and count bounds without judging their writer.
pub fn restore_fit(setting: Setting, value: &Value, limits: &Limits) -> Result<(), Failure> {
    let valid = match value {
        Value::Overlay(value) => return overlay_fit(setting, value, limits),
        Value::Charter(value) => {
            if let Some(detail) = documents_fit(&value.documents, limits) {
                return Err(document_failure(setting, detail));
            }
            if let Some(failure) = parts_fault(setting, value, limits) {
                return Err(failure);
            }
            true
        }
        Value::Template(value) => {
            value.name.len() <= skein_lib::length::usize_of(limits.documents.text_bytes)
                && contract_fits(&value.contract, &limits.tasks)
        }
        Value::Policy(value) => {
            value.permissions.len() <= skein_lib::length::usize_of(limits.settings.permissions)
                && value.templates.len() <= skein_lib::length::usize_of(limits.templates)
                && match crate::admission::policy::snapshot(&value.policy, &value.permissions, &value.templates) {
                    Some(value) => match people::policy_bytes(&value) {
                        Some(bytes) => bytes <= limits.policy_bytes,
                        None => false,
                    },
                    None => false,
                }
        }
        Value::Rules(value) => {
            let bound = &limits.authority;
            if !delegation_fits(value, bound) {
                return Err(Failure { setting, fault: Fault::Count, detail: None });
            }
            value.ceiling.grants.len() <= skein_lib::length::usize_of(bound.grants)
                && value.ceiling.note_resources.len() <= skein_lib::length::usize_of(bound.grants)
                && value.requirements.len() <= skein_lib::length::usize_of(bound.requirements)
                && value.implies.len() <= bound.implications
                && rules_patterns_fit(value, bound)
        }
        Value::Model(value) => setting == Setting::Model(value.number) && super::catalogue::fits(value, value.number),
        Value::Application(_) => super::application::is_application(setting),
    };
    if valid { Ok(()) } else { Err(Failure { setting, fault: Fault::Shape, detail: None }) }
}
fn overlay_fit(setting: Setting, overlay: &super::Overlay, limits: &Limits) -> Result<(), Failure> {
    let failure = Failure { setting, fault: Fault::Shape, detail: None };
    if overlay.parts.len() > skein_lib::usize_of(limits.tasks.charter_parts) {
        return Err(Failure { fault: Fault::Count, ..failure });
    }
    for part in &overlay.parts {
        if part.on.is_empty() || !part.on.is_known() {
            return Err(failure);
        }
    }
    if let Some(documents) = &overlay.documents
        && let Some(detail) = documents_fit(documents, limits)
    {
        return Err(document_failure(setting, detail));
    }
    if let Some(alternatives) = &overlay.alternatives
        && alternatives.len() > skein_lib::usize_of(limits.settings.alternatives)
    {
        return Err(Failure { fault: Fault::Count, ..failure });
    }
    let bytes = crate::limits::overlay_policy_bytes(overlay).ok_or(failure)?;
    if bytes > u64::from(limits.run_bytes) {
        return Err(failure);
    }
    Ok(())
}
fn delegation_fits(rules: &authority::Rules, limits: &authority::Limits) -> bool {
    rules.ceiling.delegation.kinds.len() <= skein_lib::length::usize_of(limits.executors)
}
fn rules_fault(rules: &authority::Rules, limits: &authority::Limits, malformed: Fault) -> Option<Fault> {
    if !delegation_fits(rules, limits) {
        Some(Fault::Count)
    } else if !authority::valid_rules(rules, limits) {
        Some(malformed)
    } else {
        None
    }
}
fn registered_documents(setting: Setting, charter: &crate::Charter, connectors: &[u16]) -> Result<(), Failure> {
    for (at, document) in charter.documents.iter().enumerate() {
        if !connectors.contains(&document.connector) {
            return Err(document_failure(
                setting,
                people::SettingDetail {
                    member: people::SettingMember::Connector,
                    document: u32::try_from(at).expect("bounded documents"),
                    segment: 0,
                    rule: people::SettingRule::Registered,
                    supplied: u32::from(document.connector),
                    bound: 0,
                },
            ));
        }
    }
    Ok(())
}
fn document_failure(setting: Setting, detail: people::SettingDetail) -> Failure {
    Failure { setting, fault: Fault::Shape, detail: Some(detail) }
}
fn document_bound(
    member: people::SettingMember,
    document: u32,
    segment: u32,
    supplied: usize,
    minimum: u32,
    maximum: u32,
) -> Option<people::SettingDetail> {
    let (rule, bound) = if supplied < skein_lib::usize_of(minimum) {
        (people::SettingRule::Minimum, minimum)
    } else if supplied > skein_lib::usize_of(maximum) {
        (people::SettingRule::Maximum, maximum)
    } else {
        return None;
    };
    Some(people::SettingDetail {
        member,
        document,
        segment,
        rule,
        supplied: u32::try_from(supplied).expect("admitted settings bytes"),
        bound,
    })
}
fn documents_fit(documents: &[crate::Document], limits: &Limits) -> Option<people::SettingDetail> {
    use people::SettingMember as M;
    let count = document_bound(M::Documents, 0, 0, documents.len(), 0, limits.tasks.charter_documents);
    if count.is_some() {
        return count;
    }
    for (at, document) in documents.iter().enumerate() {
        let at = u32::try_from(at).expect("bounded documents");
        let name = document_name_fit(document, at, &limits.authority);
        if name.is_some() {
            return name;
        }
        let path = document_bound(M::Path, at, 0, document.path.len(), 1, crate::DOCUMENT_PATH_BYTES);
        if path.is_some() {
            return path;
        }
        if let Some(heading) = &document.heading {
            let heading = document_bound(M::Heading, at, 0, heading.len(), 1, crate::DOCUMENT_HEADING_BYTES);
            if heading.is_some() {
                return heading;
            }
        }
    }
    None
}
fn document_name_fit(document: &crate::Document, at: u32, limits: &authority::Limits) -> Option<people::SettingDetail> {
    use people::SettingMember as M;
    let count = document_bound(M::NameSegments, at, 0, document.name.segments.len(), 0, limits.segments);
    if count.is_some() {
        return count;
    }
    for (segment, bytes) in document.name.segments.iter().enumerate() {
        let segment = u32::try_from(segment).expect("admitted name segments");
        let detail = document_bound(M::NameSegmentBytes, at, segment, bytes.len(), 0, limits.segment_bytes);
        if detail.is_some() {
            return detail;
        }
    }
    None
}
fn rules_patterns_fit(value: &authority::Rules, limits: &authority::Limits) -> bool {
    for grant in &value.ceiling.grants {
        if !pattern_fits(&grant.pattern, limits) {
            return false;
        }
    }
    for scope in &value.ceiling.note_resources {
        if !pattern_fits(&scope.pattern, limits) {
            return false;
        }
    }
    for requirement in &value.requirements {
        if !pattern_fits(&requirement.pattern, limits) {
            return false;
        }
    }
    true
}
fn pattern_fits(value: &authority::Pattern, limits: &authority::Limits) -> bool {
    if value.segments.len() > skein_lib::length::usize_of(limits.segments) {
        return false;
    }
    let bound = skein_lib::length::usize_of(limits.segment_bytes);
    for segment in &value.segments {
        if segment.len() > bound {
            return false;
        }
    }
    match &value.last {
        authority::Last::Exact(bytes) | authority::Last::Open(bytes) => bytes.len() <= bound,
    }
}
fn contract_fits(value: &tasks::Contract, limits: &tasks::Limits) -> bool {
    match value {
        tasks::Contract::Job { .. } => false,
        tasks::Contract::Report { words } | tasks::Contract::Change { words, .. } => *words <= limits.result_bytes,
        tasks::Contract::Approval { state, part } => {
            if state.len() > skein_lib::length::usize_of(limits.result_bytes) {
                return false;
            }
            match &part.key {
                tasks::PartKey::Number(_) => true,
                tasks::PartKey::Path(segments) => {
                    if segments.len() > limits.hold_segments || segments.capacity() > limits.hold_segments {
                        return false;
                    }
                    let mut bytes = 0_usize;
                    for segment in segments {
                        let Some(total) = bytes.checked_add(segment.len()) else {
                            return false;
                        };
                        bytes = total;
                    }
                    bytes <= skein_lib::length::usize_of(limits.hold_bytes)
                }
            }
        }
        tasks::Contract::Verdict { choices } => {
            if choices.len() > skein_lib::length::usize_of(limits.contract_choices) {
                return false;
            }
            for choice in choices {
                if choice.fields.len() > skein_lib::length::usize_of(limits.result_bytes)
                    || choice.followups > limits.batch
                {
                    return false;
                }
                let mut bytes = u64::from(choice.words);
                for field in &choice.fields {
                    let Some(total) = bytes.checked_add(len_u64(field.name.len())) else {
                        return false;
                    };
                    let Some(total) = total.checked_add(u64::from(field.max)) else {
                        return false;
                    };
                    bytes = total;
                }
                if bytes > u64::from(limits.result_bytes) {
                    return false;
                }
            }
            true
        }
    }
}
fn named_fault(charter: &crate::Charter, models: &[crate::Model], limits: &Limits) -> Option<Fault> {
    if charter.policy.alternatives.len() > skein_lib::length::usize_of(limits.settings.alternatives) {
        return Some(Fault::Count);
    }
    for named in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
        let mut found = false;
        for model in models {
            found |= !model.is_hole() && model.name == named.name;
        }
        if !found {
            return Some(Fault::Reference);
        }
    }
    for (at, model) in charter.policy.alternatives.iter().enumerate() {
        for previous in charter.policy.alternatives.iter().take(at) {
            if model.name == previous.name {
                return Some(Fault::Shape);
            }
        }
    }
    None
}
fn charter_fault(setting: Setting, charter: &crate::Charter, limits: &Limits) -> Option<Failure> {
    let policy = &charter.policy;
    if policy.alternatives.len() > skein_lib::length::usize_of(limits.settings.alternatives.min(tasks::MODELS)) {
        return Some(Failure { setting, fault: Fault::Count, detail: None });
    }
    if policy.turns <= policy.reserve_turns || policy.time <= policy.reserve_time {
        return Some(Failure { setting, fault: Fault::Budget, detail: None });
    }
    if match crate::runs::state::reserve_floor(policy) {
        Some(floor) => charter.spend < floor,
        None => true,
    } {
        return Some(Failure { setting, fault: Fault::Budget, detail: None });
    }
    parts_fault(setting, charter, limits)
}
/// The parts' shape and stored policy bound also hold for restored values
/// (jig/docs/design/protocol/limits.md; jig/docs/design/domain/tasks/messages.md, 6).
fn parts_fault(setting: Setting, charter: &crate::Charter, limits: &Limits) -> Option<Failure> {
    if charter.parts.len() > skein_lib::length::usize_of(limits.tasks.charter_parts) {
        return Some(Failure { setting, fault: Fault::Count, detail: None });
    }
    for part in &charter.parts {
        if part.on.is_empty() || !part.on.is_known() {
            return Some(Failure { setting, fault: Fault::Shape, detail: None });
        }
    }
    let bytes = match crate::limits::policy_owned_bytes(&charter.policy) {
        Some(policy) => match crate::limits::parts_owned_bytes(&charter.parts) {
            Some(parts) => policy.checked_add(parts),
            None => None,
        },
        None => None,
    };
    let supplied = match bytes {
        Some(bytes) if bytes <= u64::from(limits.run_bytes) => return None,
        Some(bytes) => u32::try_from(bytes).unwrap_or(u32::MAX),
        None => u32::MAX,
    };
    Some(Failure {
        setting,
        fault: Fault::Shape,
        detail: Some(people::SettingDetail {
            member: people::SettingMember::StoredPolicy,
            document: 0,
            segment: 0,
            rule: people::SettingRule::Maximum,
            supplied,
            bound: limits.run_bytes,
        }),
    })
}
fn contract_fault(contract: &tasks::Contract, connectors: &[u16]) -> Option<Fault> {
    match contract {
        tasks::Contract::Job { .. } => return Some(Fault::Reference),
        tasks::Contract::Change { connector, .. } => {
            if !connectors.contains(connector) {
                return Some(Fault::Reference);
            }
        }
        tasks::Contract::Approval { part, .. } => {
            if !connectors.contains(&part.connector) {
                return Some(Fault::Reference);
            }
        }
        tasks::Contract::Report { .. } | tasks::Contract::Verdict { .. } => {}
    }
    None
}
fn template_fault(
    current: &Current,
    defaults: bool,
    items: &[Item],
    template: &crate::Template,
    executor: people::TemplateExecutor,
    models: &[crate::Model],
    limits: &Limits,
) -> Option<Fault> {
    if let Some(grace) = template.grace
        && grace > limits.tasks.grace
    {
        return Some(Fault::Grace);
    }
    if let Some(choice) = &template.llm
        && !choice_valid(models, choice)
    {
        return Some(Fault::Reference);
    }
    if !tasks::shaped(
        template.flags,
        match executor {
            people::TemplateExecutor::Charter(charter) => tasks::Executor::Agent { charter },
            // A standing template leaves the agent's charter to its creator.
            people::TemplateExecutor::Chosen if template.flags.ended_by == tasks::EndedBy::Standing => {
                tasks::Executor::Agent { charter: 1 }
            }
            people::TemplateExecutor::Chosen => tasks::Executor::Person(tasks::PersonAddress::Person(1)),
        },
    ) {
        return Some(Fault::Shape);
    }
    match executor {
        people::TemplateExecutor::Charter(number) => {
            let charter = match overlay_value(current, defaults, items, Setting::Charter(number)) {
                Some(value) => current.made_charter(Setting::Charter(number), value),
                None => None,
            };
            let Some(charter) = charter else { return Some(Fault::Reference) };
            if let Some(choice) = &template.llm {
                let catalogue = super::catalogue::overlay(current, defaults, items);
                let policy = match super::charters::resolved(&charter, &catalogue, limits) {
                    Some(resolved) => resolved.policy,
                    None => charter.policy.clone(),
                };
                if !crate::runs::llm::takes(models, &policy, choice) {
                    return Some(Fault::Reference);
                }
            }
        }
        people::TemplateExecutor::Chosen => {
            if let Some(choice) = &template.llm
                && !choice_valid(models, choice)
            {
                return Some(Fault::Reference);
            }
        }
    }
    None
}
fn choice_valid(models: &[crate::Model], choice: &tasks::LlmChoice) -> bool {
    if !agents_valid(models, choice.main, &choice.subagents) {
        return false;
    }
    match &choice.later {
        Some(later) => agents_valid(models, later.main, &later.subagents),
        None => true,
    }
}
fn agents_valid(models: &[crate::Model], main: tasks::ModelEffort, subagents: &[tasks::ModelEffort]) -> bool {
    if subagents.len() > skein_lib::length::usize_of(tasks::MODELS) || !effort_valid(models, main) {
        return false;
    }
    for (at, chosen) in subagents.iter().enumerate() {
        for earlier in subagents.iter().take(at) {
            if earlier.model == chosen.model {
                return false;
            }
        }
        if !effort_valid(models, *chosen) {
            return false;
        }
    }
    true
}
fn policy_fault(
    current: &Current,
    defaults: bool,
    items: &[Item],
    configured: &crate::ConfiguredPolicy,
    models: &[crate::Model],
    connectors: &[u16],
    limits: &Limits,
) -> Option<Fault> {
    let policy = &configured.policy;
    let mut owner = false;
    for role in &policy.roles {
        if role.number == people::Role::Owner.number() {
            owner = role.requests.allows(authority::RequestKind::Create);
        }
        if !authority_refs(current, defaults, items, &role.authority, Some(policy), connectors, models) {
            return Some(Fault::Reference);
        }
    }
    if !owner {
        return Some(Fault::Owner);
    }
    if !authority_refs(current, defaults, items, &policy.ceiling, Some(policy), connectors, models)
        || !requirement_refs(&policy.requirements, connectors)
    {
        return Some(Fault::Reference);
    }
    for grant in &policy.projections {
        if !connectors.contains(&grant.connector) {
            return Some(Fault::Reference);
        }
    }
    for (at, permission) in configured.permissions.iter().enumerate() {
        if !connectors.contains(&permission.connector) {
            return Some(Fault::Reference);
        }
        let mut found = false;
        for role in &policy.roles {
            found |= role.number == permission.role;
        }
        if !found {
            return Some(Fault::Reference);
        }
        for earlier in configured.permissions.iter().take(at) {
            if earlier.connector == permission.connector && earlier.permission == permission.permission {
                return Some(Fault::Shape);
            }
        }
    }
    for (at, entry) in configured.templates.iter().enumerate() {
        for earlier in configured.templates.iter().take(at) {
            if earlier.template == entry.template {
                return Some(Fault::Shape);
            }
        }
        let template = match overlay_value(current, defaults, items, Setting::Template(entry.template)) {
            Some(value) => value.as_template(),
            None => None,
        };
        let Some(template) = template else { return Some(Fault::Reference) };
        let executor = entry.executor.unwrap_or(template.executor);
        if let Some(fault) = contract_fault(&template.contract, connectors) {
            return Some(fault);
        }
        if let Some(fault) = template_fault(current, defaults, items, template, executor, models, limits) {
            return Some(fault);
        }
    }
    None
}
fn authority_refs(
    current: &Current,
    defaults: bool,
    items: &[Item],
    value: &authority::Authority,
    policy: Option<&authority::Policy>,
    connectors: &[u16],
    models: &[crate::Model],
) -> bool {
    let count = u32::try_from(models.len()).expect("configured model count");
    if !value.models.is_all() && value.models.bits().checked_shr(count).unwrap_or(0) != 0 {
        return false;
    }
    for grant in &value.grants {
        if !connectors.contains(&grant.connector) {
            return false;
        }
    }
    for scope in &value.note_resources {
        if !connectors.contains(&scope.connector) {
            return false;
        }
    }
    for executor in &value.delegation.kinds {
        match executor {
            authority::Executor::Charter(number) => {
                if overlay_value(current, defaults, items, Setting::Charter(*number)).is_none() {
                    return false;
                }
            }
            authority::Executor::Role(number) => {
                if let Some(policy) = policy {
                    let mut found = false;
                    for role in &policy.roles {
                        found |= role.number == *number;
                    }
                    if !found {
                        return false;
                    }
                }
            }
            authority::Executor::Procedure(_) | authority::Executor::Job(_) => {}
        }
    }
    true
}
fn requirement_refs(requirements: &[authority::Requirement], connectors: &[u16]) -> bool {
    for requirement in requirements {
        if !connectors.contains(&requirement.connector) || !connectors.contains(&requirement.judge.connector) {
            return false;
        }
    }
    true
}
pub fn count(setting: Setting, counts: &mut [u32; 3], limits: &Limits) -> Result<(), Failure> {
    let (slot, bound) = match setting {
        Setting::Charter(_) => (0, limits.tasks.charters),
        Setting::Template(_) => (1, limits.templates),
        Setting::Policy(_) => (2, limits.authority.projects),
        Setting::Model(_) | Setting::Rules | Setting::Application { .. } | Setting::ProjectPolicy { .. } => {
            return Ok(());
        }
    };
    let count = counts.get_mut(slot).expect("three numbered kinds");
    *count = count.saturating_add(1);
    if *count > bound { Err(Failure { setting, fault: Fault::Count, detail: None }) } else { Ok(()) }
}
fn effort_valid(models: &[crate::Model], chosen: tasks::ModelEffort) -> bool {
    let Some(model) = models.get(usize::from(chosen.model)) else { return false };
    if model.is_hole() {
        return false;
    }
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}
