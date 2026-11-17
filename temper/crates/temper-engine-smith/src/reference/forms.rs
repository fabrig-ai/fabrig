//! Repeated whole-form subtrees, largest first (protocol/documentation.md, 8).

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer};

use super::{field, properties};
use crate::json::Value;

pub(super) struct Form {
    pub(super) name: Box<[u8]>,
    pub(super) schema: Value,
    weight: u32,
    occurrences: u32,
    considered: bool,
    pub(super) order: Option<u32>,
}

pub(super) fn discover(schemas: &[Value]) -> Box<[Form]> {
    let mut capacity = 0_u32;
    for schema in schemas {
        capacity = capacity.checked_add(nodes(schema)).expect("static whole forms' nodes");
    }
    let mut forms = List::with_capacity(capacity);
    for schema in schemas {
        for (name, member) in properties(schema) {
            collect(name, member, &mut forms);
        }
    }
    let mut order = 0_u32;
    for _ in 0..forms.len() {
        let mut largest = None;
        let mut weight = 0;
        for (index, form) in forms.iter().enumerate() {
            if !form.considered && form.occurrences >= 2 && form.weight > weight {
                largest = Some(u32::try_from(index).expect("bounded form index"));
                weight = form.weight;
            }
        }
        let Some(index) = largest else { break };
        let form = forms.get(index).expect("candidate form");
        let mut uses = 0_u32;
        for schema in schemas {
            uses = uses.checked_add(count_children(schema, &form.schema, forms.as_slice())).expect("static form uses");
            if uses >= 2 {
                break;
            }
        }
        for selected in &forms {
            if uses >= 2 {
                break;
            }
            if selected.order.is_some() {
                uses = uses
                    .checked_add(count_children(&selected.schema, &form.schema, forms.as_slice()))
                    .expect("shared uses");
            }
        }
        let mut name = form.name.clone();
        if uses >= 2 {
            // A member can carry different shapes: retain its name and add a numbered suffix.
            for suffix in 1..=forms.len() {
                let mut collision = false;
                for selected in &forms {
                    collision |= selected.order.is_some() && selected.name == name;
                }
                if !collision {
                    break;
                }
                let number = Decimal::of(u64::from(suffix.checked_add(1).expect("form suffix")));
                let size = form.name.len().checked_add(1).expect("form name separator length");
                let size = size.checked_add(number.as_bytes().len()).expect("form name length");
                let mut out = Writer::new(size);
                out.put(&form.name).expect("form base name");
                out.put(b"-").expect("form name separator");
                out.put(number.as_bytes()).expect("form name suffix");
                name = out.finish();
            }
        }
        let form = forms.get_mut(index).expect("candidate form");
        form.considered = true;
        if uses >= 2 {
            form.name = name;
            form.order = Some(order);
            order = order.checked_add(1).expect("form order");
        }
    }
    forms.into_boxed()
}

fn collect(name: &[u8], schema: &Value, forms: &mut List<Form>) {
    if has_shape(schema) {
        let mut seen = false;
        for form in forms.iter_mut() {
            if same_shape(schema, &form.schema) {
                form.occurrences = form.occurrences.checked_add(1).expect("static form occurrences");
                seen = true;
                break;
            }
        }
        if !seen {
            let shape = shape(schema).expect("candidate has a shape");
            let weight = nodes(&shape);
            forms
                .push(Form { name: name.into(), schema: shape, weight, occurrences: 1, considered: false, order: None })
                .ok()
                .expect("candidate capacity counts every node");
        }
    }
    for (name, member) in properties(schema) {
        collect(name, member, forms);
    }
    if let Some(items) = field(schema, b"items") {
        collect(name, items, forms);
    }
    for alternative in [b"oneOf".as_slice(), b"anyOf", b"allOf"] {
        if let Some(Value::Array(variants)) = field(schema, alternative) {
            for variant in variants {
                collect(name, variant, forms);
            }
        }
    }
}

fn has_shape(schema: &Value) -> bool {
    field(schema, b"enum").is_some() || !properties(schema).is_empty()
}

/// Compare the carried shape without copying its subtree at each visit.
fn same_shape(left: &Value, right: &Value) -> bool {
    match (field(left, b"enum"), field(right, b"enum")) {
        (Some(left), Some(right)) => same_set(left, right),
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => {
            let (Some(left_members), Some(right_members)) =
                (field(left, b"properties"), field(right, b"properties"))
            else {
                return false;
            };
            if properties(left).is_empty() || !same(left_members, right_members) {
                return false;
            }
            let empty = Value::Array(Box::new([]));
            same_set(field(left, b"required").unwrap_or(&empty), field(right, b"required").unwrap_or(&empty))
        }
    }
}

/// Only the carried members and their requiredness, or the enum values, are shared;
/// a carrier's own description, bounds and type stay on its line.
fn shape(schema: &Value) -> Option<Value> {
    if let Some(values) = field(schema, b"enum") {
        return Some(Value::Object(Box::new([(b"enum".as_slice().into(), values.clone())])));
    }
    if let Some(Value::Object(members)) = field(schema, b"properties")
        && !members.is_empty()
    {
        let required = match field(schema, b"required") {
            Some(value) => value.clone(),
            None => Value::Array(Box::new([])),
        };
        return Some(Value::Object(Box::new([
            (b"properties".as_slice().into(), Value::Object(members.clone())),
            (b"required".as_slice().into(), required),
        ])));
    }
    None
}

#[expect(clippy::manual_find, reason = "step code uses loops rather than iterator closures")]
pub(super) fn shared<'a>(schema: &Value, forms: &'a [Form]) -> Option<&'a Form> {
    for form in forms {
        if form.order.is_some() && same_shape(schema, &form.schema) {
            return Some(form);
        }
    }
    None
}

fn count_node(schema: &Value, target: &Value, forms: &[Form]) -> u32 {
    if has_shape(schema) {
        if same_shape(schema, target) {
            return 1;
        }
        if shared(schema, forms).is_some() {
            return 0;
        }
    }
    count_children(schema, target, forms)
}

fn count_children(schema: &Value, target: &Value, forms: &[Form]) -> u32 {
    let mut count = 0_u32;
    for (_, member) in properties(schema) {
        count = count.checked_add(count_node(member, target, forms)).expect("static member count");
        if count >= 2 {
            return count;
        }
    }
    if let Some(items) = field(schema, b"items") {
        count = count.checked_add(count_node(items, target, forms)).expect("static item count");
        if count >= 2 {
            return count;
        }
    }
    for alternative in [b"oneOf".as_slice(), b"anyOf", b"allOf"] {
        if let Some(Value::Array(variants)) = field(schema, alternative) {
            for variant in variants {
                count = count.checked_add(count_node(variant, target, forms)).expect("static alternative count");
                if count >= 2 {
                    return count;
                }
            }
        }
    }
    count
}

// Object key order and enum/required set order do not change a form's facts.
fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Object(left), Value::Object(right_members)) => {
            if left.len() != right_members.len() {
                return false;
            }
            for (key, value) in left {
                let mut other = None;
                for (candidate, value) in right_members {
                    if candidate == key {
                        other = Some(value);
                        break;
                    }
                }
                let Some(other) = other else { return false };
                let equal = if key.as_ref() == b"enum" || key.as_ref() == b"required" {
                    same_set(value, other)
                } else {
                    same(value, other)
                };
                if !equal {
                    return false;
                }
            }
            true
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return false;
            }
            for (left, right) in left.iter().zip(right) {
                if !same(left, right) {
                    return false;
                }
            }
            true
        }
        (
            Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null,
            Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null,
        )
        | (Value::Object(_), Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null)
        | (Value::Array(_), Value::Object(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null) => {
            left == right
        }
    }
}

fn same_set(left: &Value, right: &Value) -> bool {
    let left = left.array(b"set").expect("enum or required array");
    let right = right.array(b"set").expect("enum or required array");
    if left.len() != right.len() {
        return false;
    }
    for item in left {
        if !right.contains(item) {
            return false;
        }
    }
    true
}

fn nodes(value: &Value) -> u32 {
    let mut count = 1_u32;
    match value {
        Value::Object(members) => {
            for (_, value) in members {
                count = count.checked_add(nodes(value)).expect("static nodes");
            }
        }
        Value::Array(values) => {
            for value in values {
                count = count.checked_add(nodes(value)).expect("static nodes");
            }
        }
        Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {}
    }
    count
}
