//! Catalogue changes preserve names and drop departed choices (jig/docs/design/protocol/store/records.md, 1).

use super::{ModelName, Value};
use jig_domain_tasks::ModelEffort;

/// The bit of today's model of this name, or 0.
#[must_use]
pub fn model_bit(name: &[u8], names: &[ModelName]) -> u64 {
    for (at, today) in names.iter().enumerate() {
        if !name.is_empty() && today.name.as_ref() == name {
            return 1_u64.checked_shl(u32::try_from(at).expect("configured model index")).unwrap_or(0);
        }
    }
    0
}

/// The bit of today's model for each entry in the stored catalogue.
#[must_use]
pub fn moves(kept: &[ModelName], names: &[ModelName]) -> [u64; 64] {
    let mut moved = [0; 64];
    for (at, name) in kept.iter().enumerate() {
        *moved.get_mut(at).expect("bounded stored catalogue") = model_bit(&name.name, names);
    }
    moved
}

/// `value`, read under the catalogue `kept`, under today's `names` using their `moved` bits.
pub fn remap(value: &mut Value, moved: &[u64; 64], kept: &[ModelName], names: &[ModelName]) {
    match value {
        Value::Policy(value) => {
            value.policy.ceiling.models = value.policy.ceiling.models.remap(moved);
            for role in &mut value.policy.roles {
                role.authority.models = role.authority.models.remap(moved);
            }
        }
        Value::Rules(value) => value.ceiling.models = value.ceiling.models.remap(moved),
        Value::Template(value) => {
            if let Some(choice) = &mut value.llm {
                remap_choice(&mut choice.main, moved, kept, names);
                for model in &mut choice.subagents {
                    remap_choice(model, moved, kept, names);
                }
            }
        }
        Value::Overlay(_) | Value::Charter(_) | Value::Model(_) | Value::Application(_) => {}
    }
}

fn remap_choice(value: &mut ModelEffort, moved: &[u64; 64], kept: &[ModelName], names: &[ModelName]) {
    let old_model = value.model;
    let bit = moved.get(usize::from(old_model)).copied().unwrap_or(0);
    value.model = if bit == 0 { ModelEffort::GONE } else { u8::try_from(bit.trailing_zeros()).expect("model index") };
    let Some(effort) = value.effort else { return };
    let Some(old) = kept.get(usize::from(old_model)) else {
        value.model = ModelEffort::GONE;
        return;
    };
    let Some(name) = old.efforts.get(usize::from(effort)) else {
        value.model = ModelEffort::GONE;
        return;
    };
    let Some(today) = names.get(usize::from(value.model)) else { return };
    for (at, configured) in today.efforts.iter().enumerate() {
        if configured == name {
            value.effort = Some(u8::try_from(at).expect("effort count"));
            return;
        }
    }
    value.model = ModelEffort::GONE;
}
