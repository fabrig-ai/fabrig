//! The settings file's form, a row per shape and member (client/ctl/tools.md, 3).
mod jig;
use temper_client_protocol_wire::v1 as kinds;
pub mod render;
#[cfg(test)]
mod tests;

/// Each object the decoder reads, by the name the page gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    File,
    Item,
    Setting,
    Charter,
    CharterPart,
    Document,
    Model,
    Catalogue,
    Wake,
    WakeRule,
    WakeBatch,
    Template,
    /// A connector and its resource symbol.
    Symbol,
    Text,
    Executor,
    Contract,
    Approval,
    Part,
    Key,
    Report,
    Verdict,
    Choice,
    ChoiceField,
    Change,
    Budget,
    AsAllowed,
    Deadline,
    Flags,
    EndedBy,
    Closer,
    Llm,
    LlmLater,
    ModelEffort,
    Policy,
    Role,
    TemplateOverride,
    Permission,
    Rules,
    Implication,
    Authority,
    Models,
    Grant,
    Pattern,
    Last,
    Delegation,
    Delegate,
    Resource,
    Requirement,
    Judge,
    Guard,
    Observed,
    LandingRule,
    LandingGate,
    LandingApproval,
    LandingPattern,
    LandingLast,
    ForgeCharter,
    TemplateGoal,
    ProjectWays,
    ProjectQuestions,
    ProjectClasses,
    ClassItem,
    PathSet,
    CharterWords,
    ProjectWords,
}
/// A shape's own row.
#[derive(Debug)]
pub struct Form {
    pub shape: Shape,
    pub name: &'static [u8],
    pub choice: Choice,
    pub line: &'static [u8],
}
#[derive(Debug)]
pub enum Choice {
    Every,
    Number,
    Text,
    One { words: &'static [Word] },
}
#[derive(Debug)]
pub struct Word {
    pub word: &'static [u8],
    pub line: &'static [u8],
}
/// A member of a shape.
#[derive(Debug)]
pub struct Field {
    pub shape: Shape,
    pub name: &'static [u8],
    pub value: Value,
    pub list: Option<Bound>,
    pub null: bool,
    pub line: &'static [u8],
}
#[derive(Debug)]
pub enum Value {
    Integer(Width),
    Nanoseconds,
    Time,
    Boolean,
    Text(Bound),
    Hex(Bound),
    Words(&'static [Word]),
    Shape(Shape),
    Setting,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    U8,
    U16,
    U32,
    U64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Most(u32),
    Limit(Limit),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    CharterInstructions,
    CharterAlternatives,
    CharterDocuments,
    CharterParts,
    CharterPartInstructions,
    DocumentSegment,
    DocumentRepository,
    DocumentBranch,
    DocumentPath,
    DocumentHeading,
    CatalogueEntryShownSome,
    CatalogueEntryDescriptionSome,
    SettingModelProgramSome,
    SettingModelName,
    SettingModelEffortsItem,
    SettingModelEfforts,
    SettingModelEffortSome,
    DocumentTextBytes,
    TaskContractDocumentApprovalState,
    PartKeyPathValueItem,
    PartKeyPathValue,
    TaskContractDocumentVerdictChoices,
    VerdictChoiceFields,
    FieldRuleName,
    /// The number of resource symbols in a template.
    TemplateResources,
    LlmChoiceSubagents,
    ModelEffortModel,
    ModelEffortEffortSome,
    SettingPolicyProjections,
    SettingPolicyRoles,
    SettingPolicyRequirements,
    SettingPolicyPermissions,
    SettingPolicyTemplates,
    RulesImplies,
    RulesRequirements,
    AuthorityGrants,
    AuthorityNoteResources,
    ModelSetNamesNamesItem,
    ModelSetNamesNames,
    PatternSegmentsItem,
    PatternSegments,
    PatternEndExactValue,
    PatternEndOpenValue,
    DelegationKinds,
    LandingSegments,
    LandingSegmentBytes,
    LandingExact,
    LandingOpen,
    LandingGates,
    LandingApprovals,
    LandingSets,
    ClassesItems,
    ClassesSets,
    ClassWords,
    SetName,
    SetPatterns,
    SetPatternBytes,
}

pub use jig::{FIELDS, FORMS};
pub use render::page;
#[must_use]
pub fn most(bound: Bound, limits: &crate::Limits) -> u64 {
    u64::from(match bound {
        Bound::Most(n) => n,
        Bound::Limit(limit) => match limit {
            Limit::CharterInstructions => limits.settings.charter_instructions,
            Limit::CharterAlternatives => limits.settings.charter_alternatives,
            Limit::CharterDocuments => limits.settings.charter_documents,
            Limit::CharterParts => limits.settings.charter_parts,
            Limit::CharterPartInstructions => {
                limits.settings.charter_instructions.min(limits.settings.charter_part_instructions)
            }
            Limit::DocumentSegment => limits.settings.charter_document_name_item,
            Limit::DocumentRepository => limits.settings.charter_document_name_item.saturating_mul(2).saturating_add(1),
            Limit::DocumentBranch => limits
                .settings
                .charter_document_name
                .saturating_sub(5)
                .saturating_mul(limits.settings.charter_document_name_item.saturating_add(1))
                .saturating_sub(1),
            Limit::DocumentPath => limits.settings.charter_document_path,
            Limit::DocumentHeading => limits.settings.charter_document_heading_some,
            Limit::CatalogueEntryShownSome => limits.settings.catalogue_entry_shown_some,
            Limit::CatalogueEntryDescriptionSome => limits.settings.catalogue_entry_description_some,
            Limit::SettingModelProgramSome => limits.settings.setting_model_program_some,
            Limit::SettingModelName => limits.settings.setting_model_name,
            Limit::SettingModelEffortsItem => limits.settings.setting_model_efforts_item,
            Limit::SettingModelEfforts => limits.settings.setting_model_efforts,
            Limit::SettingModelEffortSome => limits.settings.setting_model_effort_some,
            Limit::DocumentTextBytes => limits.settings.document_text_bytes,
            Limit::TaskContractDocumentApprovalState => limits.settings.task_contract_document_approval_state,
            Limit::PartKeyPathValueItem => limits.settings.part_key_path_value_item,
            Limit::PartKeyPathValue => limits.settings.part_key_path_value,
            Limit::TaskContractDocumentVerdictChoices => limits.settings.task_contract_document_verdict_choices,
            Limit::VerdictChoiceFields => limits.settings.verdict_choice_fields,
            Limit::FieldRuleName => limits.settings.field_rule_name,
            Limit::TemplateResources => limits.settings.template_resources,
            Limit::LlmChoiceSubagents => limits.settings.llm_choice_subagents,
            Limit::ModelEffortModel => limits.settings.model_effort_model,
            Limit::ModelEffortEffortSome => limits.settings.model_effort_effort_some,
            Limit::SettingPolicyProjections => limits.settings.setting_policy_projections,
            Limit::SettingPolicyRoles => limits.settings.setting_policy_roles,
            Limit::SettingPolicyRequirements => limits.settings.setting_policy_requirements,
            Limit::SettingPolicyPermissions => limits.settings.setting_policy_permissions,
            Limit::SettingPolicyTemplates => limits.settings.setting_policy_templates,
            Limit::RulesImplies => limits.settings.rules_implies,
            Limit::RulesRequirements => limits.settings.rules_requirements,
            Limit::AuthorityGrants => limits.settings.authority_grants,
            Limit::AuthorityNoteResources => limits.settings.authority_note_resources,
            Limit::ModelSetNamesNamesItem => limits.settings.model_set_names_names_item,
            Limit::ModelSetNamesNames => limits.settings.model_set_names_names,
            Limit::PatternSegmentsItem => limits.settings.pattern_segments_item,
            Limit::PatternSegments => limits.settings.pattern_segments,
            Limit::PatternEndExactValue => limits.settings.pattern_end_exact_value,
            Limit::PatternEndOpenValue => limits.settings.pattern_end_open_value,
            Limit::DelegationKinds => limits.settings.delegation_kinds,
            Limit::ClassesItems => limits.kinds.project_classes_items.min(kinds::CEILINGS.project_classes_items),
            Limit::ClassesSets => limits.kinds.project_classes_sets.min(kinds::CEILINGS.project_classes_sets),
            Limit::ClassWords => limits.kinds.class_item_words.min(kinds::CEILINGS.class_item_words),
            Limit::SetName => limits.kinds.path_set_name.min(kinds::CEILINGS.path_set_name),
            Limit::SetPatterns => limits.kinds.path_set_patterns.min(kinds::CEILINGS.path_set_patterns),
            Limit::SetPatternBytes => limits.kinds.path_set_patterns_item.min(kinds::CEILINGS.path_set_patterns_item),
            Limit::LandingSegments
            | Limit::LandingSegmentBytes
            | Limit::LandingExact
            | Limit::LandingOpen
            | Limit::LandingGates
            | Limit::LandingApprovals
            | Limit::LandingSets => landing_limit(limit, limits),
        },
    })
}
#[expect(clippy::wildcard_enum_match_arm, reason = "the caller selects only landing limits")]
fn landing_limit(limit: Limit, limits: &crate::Limits) -> u32 {
    match limit {
        Limit::LandingSegments => limits
            .settings
            .pattern_segments
            .min(limits.kinds.landing_pattern_segments)
            .min(kinds::CEILINGS.landing_pattern_segments),
        Limit::LandingSegmentBytes => limits
            .settings
            .pattern_segments_item
            .min(limits.kinds.landing_pattern_segments_item)
            .min(kinds::CEILINGS.landing_pattern_segments_item),
        Limit::LandingExact => limits
            .settings
            .pattern_end_exact_value
            .min(limits.kinds.landing_exact_value)
            .min(kinds::CEILINGS.landing_exact_value),
        Limit::LandingOpen => limits
            .settings
            .pattern_end_open_value
            .min(limits.kinds.landing_open_value)
            .min(kinds::CEILINGS.landing_open_value),
        Limit::LandingGates => limits.kinds.landing_rule_gates.min(kinds::CEILINGS.landing_rule_gates),
        Limit::LandingApprovals => limits.kinds.landing_rule_approvals.min(kinds::CEILINGS.landing_rule_approvals),
        Limit::LandingSets => limits.kinds.landing_rule_sets.min(kinds::CEILINGS.landing_rule_sets),
        _ => unreachable!("landing limit"),
    }
}
/// A setting's kind word (`charter`, `template`, `policy`, `rules`) to its value's shape.
#[must_use]
pub fn shape_of_kind(kind: &[u8]) -> Option<Shape> {
    match kind {
        b"model" => Some(Shape::Catalogue),
        b"charter" => Some(Shape::Charter),
        b"template" => Some(Shape::Template),
        b"policy" | b"project-policy" => Some(Shape::Policy),
        b"rules" => Some(Shape::Rules),
        b"landing-rule" => Some(Shape::LandingRule),
        b"forge-charter" => Some(Shape::ForgeCharter),
        b"charter-words" => Some(Shape::CharterWords),
        b"project-words" => Some(Shape::ProjectWords),
        b"template-goal" => Some(Shape::TemplateGoal),
        b"project-ways" => Some(Shape::ProjectWays),
        b"project-questions" => Some(Shape::ProjectQuestions),
        b"project-classes" => Some(Shape::ProjectClasses),
        _ => None,
    }
}
#[must_use]
pub fn form(shape: Shape) -> &'static Form {
    for row in &FORMS {
        if row.shape == shape {
            return row;
        }
    }
    unreachable!("each shape has a form")
}
#[must_use]
pub fn members(shape: Shape) -> &'static [Field] {
    let mut start = FIELDS.len();
    let mut end = FIELDS.len();
    for (index, row) in FIELDS.iter().enumerate() {
        if row.shape == shape {
            start = index;
            break;
        }
    }
    for (index, row) in FIELDS.iter().enumerate().skip(start) {
        if row.shape != shape {
            end = index;
            break;
        }
    }
    FIELDS.get(start..end).expect("grouped fields")
}
/// `charter.model.name`, or from any shape, `model.name`: each word after
/// the first a member reached through `Value::Shape`, a list's member
/// with or without `[]` (`policy.roles[].number`); for the pages' check.
#[must_use]
pub fn resolve(path: &[u8]) -> Option<&'static Field> {
    let mut shape = None;
    let mut row = None;
    let mut start = 0;
    for end in 0..=path.len() {
        if end < path.len() && path.get(end) != Some(&b'.') {
            continue;
        }
        let word = path.get(start..end)?;
        if start == 0 {
            for form in &FORMS {
                if form.name == word {
                    shape = Some(form.shape);
                    break;
                }
            }
            shape?;
        } else {
            let (name, listed) = match word.strip_suffix(b"[]") {
                Some(name) => (name, true),
                None => (word, false),
            };
            let mut found = None;
            for member in members(shape?) {
                if member.name == name && (!listed || member.list.is_some()) {
                    found = Some(member);
                    break;
                }
            }
            let member = found?;
            row = Some(member);
            shape = match member.value {
                Value::Shape(s) => Some(s),
                Value::Integer(_)
                | Value::Nanoseconds
                | Value::Time
                | Value::Boolean
                | Value::Text(_)
                | Value::Hex(_)
                | Value::Words(_)
                | Value::Setting => None,
            };
        }
        start = end.checked_add(1)?;
    }
    row
}
