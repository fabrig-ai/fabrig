//! One whole-set check, also used after restoring stored changes (engine.md, 17.1).
use crate::{
    Limits, Settings,
    settings::{Current, Edit, Failure, Fault, Item, Setting, Value},
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

/// Check the maps in place: no third materialised set of values is constructed.
pub fn check(current: &Current, settings: &Settings, connectors: &[u16], limits: &Limits) -> Result<(), Failure> {
    check_set(current, true, &[], settings, connectors, limits)?;
    check_set(current, false, &[], settings, connectors, limits)
}

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
pub(crate) fn edit(items: &[Item], setting: Setting) -> Option<&Edit> {
    for item in items {
        if item.setting == setting {
            return Some(&item.edit);
        }
    }
    None
}
// The shared lifetime borrows a value from either the current maps or the decision; no view is stored.
fn overlay_value<'a>(current: &'a Current, defaults: bool, items: &'a [Item], setting: Setting) -> Option<&'a Value> {
    match edit(items, setting) {
        Some(Edit::Set(value)) => Some(value),
        Some(Edit::Removed) => None,
        Some(Edit::Reset) => base_value(current, true, setting),
        None => base_value(current, defaults, setting),
    }
}

/// Read templates from the same overlay that checks the decision, without publishing it.
pub(crate) fn templates(current: &Current, items: &[Item]) -> Box<[crate::Template]> {
    let names = offered(current, false, items);
    let mut result = skein_lib::List::with_capacity(names.len());
    for &setting in &names {
        if let Some(value) = overlay_value(current, false, items, setting)
            && let Some(template) = value.as_template()
        {
            result.push(template.clone()).expect("one template per offered name");
        }
    }
    result.into_boxed()
}

fn offered(current: &Current, defaults: bool, items: &[Item]) -> skein_lib::List<Setting> {
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
    check_set(current, false, items, settings, connectors, limits)
}
fn check_set(
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
    if !authority_refs(current, defaults, items, &rules.ceiling, None, connectors, settings)
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
        let fault = match overlay_value(current, defaults, items, setting).expect("offered setting") {
            Value::Charter(charter) => {
                registered_documents(setting, charter, connectors)?;
                charter_fault(charter, settings, limits)
            }
            Value::Template(template) => {
                if let Some(fault) = contract_fault(&template.contract, connectors) {
                    return Err(Failure { setting, fault, detail: None });
                }
                template_fault(current, defaults, items, template, template.executor, settings, limits)
            }
            Value::Policy(policy) => match authority::policy_refusal(rules, &limits.authority, &policy.policy) {
                Some(authority::PolicyRefusal::AboveRules) => Some(Fault::Rules),
                Some(authority::PolicyRefusal::Oversized | authority::PolicyRefusal::InvalidRole) => Some(Fault::Shape),
                Some(authority::PolicyRefusal::Full) => Some(Fault::Count),
                None => policy_fault(current, defaults, items, policy, settings, connectors, limits),
            },
            Value::Rules(_) | Value::Application(_) => None,
        };
        if let Some(fault) = fault {
            return Err(Failure { setting, fault, detail: None });
        }
    }
    Ok(())
}
pub(crate) fn shape(setting: Setting, value: &Value, limits: &Limits) -> Result<(), Failure> {
    let valid = match value {
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
            Setting::Charter(_) | Setting::Template(_) | Setting::Rules | Setting::Application { .. } => false,
        },
        Value::Rules(value) => {
            if setting == Setting::Rules
                && let Some(fault) = rules_fault(value, &limits.authority, Fault::Shape)
            {
                return Err(Failure { setting, fault, detail: None });
            }
            setting == Setting::Rules
        }
        Value::Application(_) => super::application::is_application(setting),
    };
    if !valid {
        return Err(Failure { setting, fault: Fault::Shape, detail: None });
    }
    restore_fit(setting, value, limits)
}

/// Stored values retain their byte and count bounds without judging their writer.
pub(crate) fn restore_fit(setting: Setting, value: &Value, limits: &Limits) -> Result<(), Failure> {
    let valid = match value {
        Value::Charter(value) => {
            if let Some(detail) = documents_fit(value, limits) {
                return Err(document_failure(setting, detail));
            }
            match crate::limits::policy_owned_bytes(&value.policy) {
                Some(bytes) => bytes <= u64::from(limits.run_bytes),
                None => false,
            }
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
        Value::Application(_) => super::application::is_application(setting),
    };
    if valid { Ok(()) } else { Err(Failure { setting, fault: Fault::Shape, detail: None }) }
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
fn documents_fit(charter: &crate::Charter, limits: &Limits) -> Option<people::SettingDetail> {
    use people::SettingMember as M;
    let count = document_bound(M::Documents, 0, 0, charter.documents.len(), 0, limits.tasks.charter_documents);
    if count.is_some() {
        return count;
    }
    for (at, document) in charter.documents.iter().enumerate() {
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
                    let Some(total) = bytes.checked_add(u64::try_from(field.name.len()).expect("field bytes")) else {
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
fn charter_fault(charter: &crate::Charter, settings: &Settings, limits: &Limits) -> Option<Fault> {
    let policy = &charter.policy;
    if !model_valid(&policy.model, settings) {
        return Some(Fault::Reference);
    }
    for (at, model) in policy.alternatives.iter().enumerate() {
        if !model_valid(model, settings) {
            return Some(Fault::Reference);
        }
        for earlier in policy.alternatives.iter().take(at) {
            if earlier.name == model.name {
                return Some(Fault::Shape);
            }
        }
    }
    if policy.alternatives.len() > skein_lib::length::usize_of(limits.settings.alternatives.min(tasks::MODELS)) {
        return Some(Fault::Count);
    }
    if policy.turns <= policy.reserve_turns || policy.time <= policy.reserve_time {
        return Some(Fault::Budget);
    }
    if match crate::runs::state::reserve_floor(policy) {
        Some(floor) => charter.spend < floor,
        None => true,
    } {
        return Some(Fault::Budget);
    }
    if crate::limits::policy_owned_bytes(policy)? > u64::from(limits.run_bytes) {
        return Some(Fault::Shape);
    }
    None
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
    settings: &Settings,
    limits: &Limits,
) -> Option<Fault> {
    if let Some(grace) = template.grace
        && grace > limits.tasks.grace
    {
        return Some(Fault::Grace);
    }
    if let Some(choice) = &template.llm
        && !choice_valid(&settings.models, choice)
    {
        return Some(Fault::Reference);
    }
    if template.flags.ended_by == tasks::EndedBy::Requester
        && (!template.flags.conversation || executor == people::TemplateExecutor::Chosen)
    {
        return Some(Fault::Shape);
    }
    match executor {
        people::TemplateExecutor::Charter(number) => {
            let charter = match overlay_value(current, defaults, items, Setting::Charter(number)) {
                Some(value) => value.as_charter(),
                None => None,
            };
            let Some(charter) = charter else { return Some(Fault::Reference) };
            if let Some(choice) = &template.llm
                && !crate::runs::llm::takes(&settings.models, &charter.policy, choice)
            {
                return Some(Fault::Reference);
            }
        }
        people::TemplateExecutor::Chosen => {
            if let Some(choice) = &template.llm
                && !choice_valid(&settings.models, choice)
            {
                return Some(Fault::Reference);
            }
        }
    }
    None
}
fn choice_valid(models: &[crate::Model], choice: &tasks::LlmChoice) -> bool {
    if choice.subagents.len() > skein_lib::length::usize_of(tasks::MODELS) {
        return false;
    }
    if !effort_valid(models, choice.main) {
        return false;
    }
    for (at, chosen) in choice.subagents.iter().enumerate() {
        for earlier in choice.subagents.iter().take(at) {
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
    settings: &Settings,
    connectors: &[u16],
    limits: &Limits,
) -> Option<Fault> {
    let policy = &configured.policy;
    let mut owner = false;
    for role in &policy.roles {
        if role.number == people::Role::Owner.number() {
            owner = role.requests.allows(authority::RequestKind::Create);
        }
        if !authority_refs(current, defaults, items, &role.authority, Some(policy), connectors, settings) {
            return Some(Fault::Reference);
        }
    }
    if !owner {
        return Some(Fault::Owner);
    }
    if !authority_refs(current, defaults, items, &policy.ceiling, Some(policy), connectors, settings)
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
        if let Some(fault) = template_fault(current, defaults, items, template, executor, settings, limits) {
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
    settings: &Settings,
) -> bool {
    let count = u32::try_from(settings.models.len()).expect("configured model count");
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

pub(super) fn count(setting: Setting, counts: &mut [u32; 3], limits: &Limits) -> Result<(), Failure> {
    let (slot, bound) = match setting {
        Setting::Charter(_) => (0, limits.tasks.charters),
        Setting::Template(_) => (1, limits.templates),
        Setting::Policy(_) => (2, limits.authority.projects),
        Setting::Rules | Setting::Application { .. } | Setting::ProjectPolicy { .. } => return Ok(()),
    };
    let count = counts.get_mut(slot).expect("three numbered kinds");
    *count = count.saturating_add(1);
    if *count > bound { Err(Failure { setting, fault: Fault::Count, detail: None }) } else { Ok(()) }
}

fn model_valid(model: &crate::Model, settings: &Settings) -> bool {
    match super::charters::configured(settings, model) {
        Some(index) => {
            let configured = settings.models.get(usize::from(index)).expect("configured model");
            model.dialect == configured.dialect
                && model.endpoint == configured.endpoint
                && model.program == configured.program
                && model.name == configured.name
                && model.window == configured.window
                && model.output == configured.output
                && model.input_price == configured.input_price
                && model.cached_price == configured.cached_price
                && model.cache_write_price == configured.cache_write_price
                && model.output_price == configured.output_price
                && model.price_unit == configured.price_unit
                && model.efforts == configured.efforts
                && match &model.effort {
                    Some(effort) => configured.efforts.contains(effort),
                    None => true,
                }
        }
        None => false,
    }
}
fn effort_valid(models: &[crate::Model], chosen: tasks::ModelEffort) -> bool {
    let Some(model) = models.get(usize::from(chosen.model)) else { return false };
    match chosen.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}

/// Check a streamed change before retaining its value under a per-kind heap bound.
pub(super) fn admits_change(current: &Current, setting: Setting, limits: &Limits) -> Result<(), Failure> {
    let mut counts = [0_u32; 3];
    for (&held, _) in &current.changes {
        count(held, &mut counts, limits)?;
    }
    count(setting, &mut counts, limits)
}

/// Only the tasks child's bounded model table is materialised for the live-task check.
pub(crate) fn changed_charters(
    current: &Current,
    items: &[Item],
    settings: &Settings,
    limits: &Limits,
) -> Box<[tasks::CharterModels]> {
    let mut result = skein_lib::List::with_capacity(limits.tasks.charters);
    for &setting in &offered(current, false, items) {
        let value = overlay_value(current, false, items, setting);
        if let Some(value) = value
            && let Some(charter) = value.as_charter()
        {
            result
                .push(super::charters::one_models(charter, current.newest(), settings))
                .expect("checked charter count");
        }
    }
    result.into_boxed()
}
