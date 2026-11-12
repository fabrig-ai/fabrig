//! Deep owned bytes of jig's settings rows; no history is retained by the root.
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::settings::{Edit, Item, Record, Value};
use jig_domain_tasks as tasks;

pub(super) fn record(row: &Record) -> Option<u64> {
    match row {
        Record::Application { state, .. } => u64::try_from(size_of::<jig_domain_core::settings::application::State>())
            .ok()?
            .checked_add(skein_lib::len_u64(state.charters.len()).checked_mul(4)?),
        Record::Default { default, .. } => value(&default.value),
        Record::Changed { changed, .. } => match &changed.value {
            Some(changed) => value(changed),
            None => Some(0),
        },
        Record::Revision { revision, .. } => {
            let mut bytes = skein_lib::len_u64(revision.items.len())
                .checked_mul(u64::try_from(size_of::<jig_domain_core::settings::RevisionItem>()).ok()?)?;
            for item in &revision.items {
                if let Edit::Set(set) = &item.edit {
                    bytes = bytes.checked_add(value(set)?)?;
                }
                if let jig_domain_core::settings::Replaced::Value(replaced) = &item.replaced {
                    bytes = bytes.checked_add(value(replaced)?)?;
                }
            }
            Some(bytes)
        }
        Record::Proposal { value, .. } => items(&value.items),
        Record::Charter { value: version, .. } => {
            let mut bytes = 0_u64;
            if let Some(charter) = &version.charter {
                bytes = bytes
                    .checked_add(u64::try_from(size_of::<jig_domain_core::Charter>()).ok()?)?
                    .checked_add(jig_domain_core::charter_owned_bytes(charter)?)?;
            }
            if let Some(overlay) = &version.overlay {
                bytes = bytes
                    .checked_add(u64::try_from(size_of::<jig_domain_core::settings::Overlay>()).ok()?)?
                    .checked_add(jig_domain_core::overlay_owned_bytes(overlay)?)?;
            }
            Some(bytes)
        }
        Record::Extent { .. } => Some(0),
    }
}
fn items(items: &[Item]) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(items.len()).checked_mul(u64::try_from(size_of::<Item>()).ok()?)?;
    for item in items {
        if let Edit::Set(set) = &item.edit {
            bytes = bytes.checked_add(value(set)?)?;
        }
    }
    Some(bytes)
}
fn value(value: &Value) -> Option<u64> {
    match value {
        Value::Model(value) => {
            let mut bytes = u64::try_from(size_of::<jig_domain_core::settings::CatalogueModel>()).ok()?;
            bytes = bytes.checked_add(skein_lib::len_u64(value.model.name.len()))?;
            for effort in &value.model.efforts {
                bytes = bytes
                    .checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?
                    .checked_add(skein_lib::len_u64(effort.len()))?;
            }
            for text in
                [&value.shown, &value.description, &value.model.program, &value.model.effort].into_iter().flatten()
            {
                bytes = bytes.checked_add(skein_lib::len_u64(text.len()))?;
            }
            Some(bytes)
        }
        Value::Charter(charter) => u64::try_from(size_of::<jig_domain_core::Charter>())
            .ok()?
            .checked_add(jig_domain_core::charter_owned_bytes(charter)?),
        Value::Template(template) => {
            let mut bytes = u64::try_from(size_of::<jig_domain_core::Template>())
                .ok()?
                .checked_add(skein_lib::len_u64(template.name.len()))?
                .checked_add(contract(&template.contract)?)?;
            if let Some(choice) = &template.llm {
                bytes = bytes.checked_add(
                    skein_lib::len_u64(choice.subagents.len())
                        .checked_mul(u64::try_from(size_of::<tasks::ModelEffort>()).ok()?)?,
                )?;
            }
            Some(bytes)
        }
        Value::Policy(configured) => {
            let policy = &configured.policy;
            let mut bytes = u64::try_from(size_of::<jig_domain_core::ConfiguredPolicy>())
                .ok()?
                .checked_add(authority(&policy.ceiling)?)?
                .checked_add(grants(&policy.projections)?)?
                .checked_add(requirements(&policy.requirements)?)?
                .checked_add(
                    skein_lib::len_u64(policy.roles.len())
                        .checked_mul(u64::try_from(size_of::<authority::Role>()).ok()?)?,
                )?
                .checked_add(
                    skein_lib::len_u64(configured.permissions.len())
                        .checked_mul(u64::try_from(size_of::<jig_domain_people::PermissionRole>()).ok()?)?,
                )?
                .checked_add(
                    skein_lib::len_u64(configured.templates.len())
                        .checked_mul(u64::try_from(size_of::<jig_domain_people::TemplateOverride>()).ok()?)?,
                )?;
            for role in &policy.roles {
                bytes = bytes.checked_add(authority(&role.authority)?)?;
            }
            Some(bytes)
        }
        Value::Rules(rules) => u64::try_from(size_of::<authority::Rules>())
            .ok()?
            .checked_add(authority(&rules.ceiling)?)?
            .checked_add(requirements(&rules.requirements)?)?
            .checked_add(
                u64::from(rules.implies.len()).checked_mul(u64::try_from(size_of::<authority::Implication>()).ok()?)?,
            ),
        Value::Overlay(overlay) => u64::try_from(size_of::<jig_domain_core::settings::Overlay>())
            .ok()?
            .checked_add(jig_domain_core::overlay_owned_bytes(overlay)?),
        Value::Application(bytes) => Some(skein_lib::len_u64(bytes.len())),
    }
}
fn authority(value: &authority::Authority) -> Option<u64> {
    let mut bytes = grants(&value.grants)?
        .checked_add(
            skein_lib::len_u64(value.delegation.kinds.len())
                .checked_mul(u64::try_from(size_of::<authority::Executor>()).ok()?)?,
        )?
        .checked_add(
            skein_lib::len_u64(value.note_resources.len())
                .checked_mul(u64::try_from(size_of::<authority::ResourceScope>()).ok()?)?,
        )?;
    for scope in &value.note_resources {
        bytes = bytes.checked_add(pattern(&scope.pattern)?)?;
    }
    Some(bytes)
}
fn grants(grants: &[authority::Grant]) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(grants.len()).checked_mul(u64::try_from(size_of::<authority::Grant>()).ok()?)?;
    for grant in grants {
        bytes = bytes.checked_add(pattern(&grant.pattern)?)?;
    }
    Some(bytes)
}
fn requirements(requirements: &[authority::Requirement]) -> Option<u64> {
    let mut bytes =
        skein_lib::len_u64(requirements.len()).checked_mul(u64::try_from(size_of::<authority::Requirement>()).ok()?)?;
    for requirement in requirements {
        bytes = bytes.checked_add(pattern(&requirement.pattern)?)?;
    }
    Some(bytes)
}
fn pattern(pattern: &authority::Pattern) -> Option<u64> {
    let mut bytes =
        skein_lib::len_u64(pattern.segments.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for segment in &pattern.segments {
        bytes = bytes.checked_add(skein_lib::len_u64(segment.len()))?;
    }
    match &pattern.last {
        authority::Last::Exact(last) | authority::Last::Open(last) => bytes.checked_add(skein_lib::len_u64(last.len())),
    }
}
fn contract(contract: &tasks::Contract) -> Option<u64> {
    match contract {
        tasks::Contract::Report { .. } | tasks::Contract::Change { .. } | tasks::Contract::Job { .. } => Some(0),
        tasks::Contract::Approval { state, part } => {
            let mut bytes = skein_lib::len_u64(state.len());
            if let tasks::PartKey::Path(segments) = &part.key {
                bytes = bytes.checked_add(
                    u64::from(segments.capacity()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                )?;
                for segment in segments {
                    bytes = bytes.checked_add(skein_lib::len_u64(segment.len()))?;
                }
            }
            Some(bytes)
        }
        tasks::Contract::Verdict { choices } => {
            // Each verdict owns its field rules and their names.
            let mut bytes =
                skein_lib::len_u64(choices.len()).checked_mul(u64::try_from(size_of::<tasks::Verdict>()).ok()?)?;
            for choice in choices {
                for field in &choice.fields {
                    bytes = bytes.checked_add(skein_lib::len_u64(field.name.len()))?;
                }
                bytes = bytes.checked_add(
                    skein_lib::len_u64(choice.fields.len())
                        .checked_mul(u64::try_from(size_of::<tasks::FieldRule>()).ok()?)?,
                )?;
            }
            Some(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_approval_settings_contract_counts_its_reserved_path_and_segments() {
        let mut segments = skein_lib::List::with_capacity(3);
        segments.push(Box::from(b"forge".as_slice())).expect("first segment");
        segments.push(Box::from(b"main".as_slice())).expect("second segment");
        let approval = tasks::Contract::Approval {
            state: Box::from(b"approved".as_slice()),
            part: tasks::Part { connector: 1, kind: tasks::PartKind::Resource, key: tasks::PartKey::Path(segments) },
        };
        assert_eq!(contract(&approval), Some(u64::try_from(3 * size_of::<Box<[u8]>>() + 17).expect("small path")));
    }
    #[test]
    fn overlay_bytes_include_its_owned_text_and_parts() {
        let mut overlay = jig_domain_core::settings::Overlay::empty();
        overlay.instructions = b"x".as_slice().into();
        overlay.parts =
            Box::new([jig_domain_core::CharterPart { on: tasks::Reasons(1), instructions: b"part".as_slice().into() }]);
        let first = value(&Value::Overlay(Box::new(overlay.clone()))).expect("overlay has a checked byte projection");
        let fixed = u64::try_from(size_of::<jig_domain_core::settings::Overlay>()).expect("struct bytes");
        assert_eq!(
            first,
            fixed + 1 + u64::try_from(size_of::<jig_domain_core::CharterPart>()).expect("part bytes") + 4 + 2
        );
        overlay.instructions = b"longer".as_slice().into();
        assert_eq!(value(&Value::Overlay(Box::new(overlay))), Some(first + 5));
    }
}
