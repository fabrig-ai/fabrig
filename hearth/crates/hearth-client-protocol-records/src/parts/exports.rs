//! Typed public values and their compact JSON forms (hearth/docs/design/protocol/format/README.md, 6.1;
//! hearth/docs/design/domain/render/spec.md, 1.4).
use crate::form::text::Lines;
use alloc::boxed::Box;
use hearth_domain_render_vocabulary::{
    evaluate::{Entry, Value},
    parts::exports::{Change, Exports},
};
use skein_json::writer::{Encoder, Limits};
use skein_lib::Stack;

pub(crate) fn json(this: &Exports, json: &mut Encoder) {
    json.object_start();
    for task in &this.tasks {
        json.key(&task.task);
        json.object_start();
        for named in &task.values {
            if let Some(value) = &named.value {
                json.key(&named.key);
                value_json(value, json);
            }
        }
        json.object_end();
    }
    json.object_end();
}

pub(crate) fn text(this: &Exports, lines: &mut Lines) {
    let mut width = 0_usize;
    for task in &this.tasks {
        for named in &task.values {
            if named.value.is_some() {
                width = width.max(task.task.len().saturating_add(named.key.len()).saturating_add(1));
            }
        }
    }
    for task in &this.tasks {
        for named in &task.values {
            let Some(value) = &named.value else { continue };
            lines.put(b"  ");
            lines.put(&task.task);
            lines.put(b".");
            lines.put(&named.key);
            lines.pad_to(width.saturating_add(5));
            lines.put(&value_compact(value));
            lines.put(b"\n");
        }
    }
}
#[derive(Debug)]
enum Frame<'a> {
    List(core::slice::Iter<'a, Value>),
    Map(core::slice::Iter<'a, Entry>),
}
pub(crate) fn value_json(this: &Value, json: &mut Encoder) {
    let mut stack = Stack::with_capacity(crate::form::json::DEPTH);
    let mut next = Some(this);
    for _ in 0..u64::from(crate::limits::SHIPPED.record).saturating_mul(3).saturating_add(3) {
        if let Some(value) = next.take() {
            match value {
                Value::Bool(value) => json.boolean(*value),
                Value::Int(value) => json.signed(*value),
                Value::String(value) => json.string(value),
                Value::List(items) => {
                    json.array_start();
                    stack.push(Frame::List(items.iter())).expect("export depth");
                }
                Value::Map(entries) => {
                    json.object_start();
                    stack.push(Frame::Map(entries.iter())).expect("export depth");
                }
            }
        }
        let Some(mut frame) = stack.pop() else { return };
        next = match &mut frame {
            Frame::List(each) => {
                let next = each.next();
                if next.is_none() {
                    json.array_end();
                }
                next
            }
            Frame::Map(each) => {
                if let Some((name, value)) = present(each) {
                    json.key(name);
                    Some(value)
                } else {
                    json.object_end();
                    None
                }
            }
        };
        if next.is_some() {
            stack.push(frame).expect("popped frame fits");
        }
    }
    unreachable!("bounded records bound the value walk")
}

pub(crate) fn value_compact(this: &Value) -> Box<[u8]> {
    let limits = Limits { depth: crate::form::json::DEPTH, length: u32::MAX };
    let mut json = Encoder::measure(&limits);
    value_json(this, &mut json);
    let length = json.measured().expect("typed export JSON");
    let mut json = Encoder::write(length, &limits);
    value_json(this, &mut json);
    json.finish()
}

pub(crate) fn change_text(this: &Change, lines: &mut Lines) {
    let mut width = 0_usize;
    for entry in &this.entries {
        width = width.max(entry.task.len().saturating_add(entry.name.len()).saturating_add(1));
    }
    for entry in &this.entries {
        lines.put(b"  exports  ");
        lines.put(match (&entry.before, &entry.after) {
            (Some(_), Some(_)) => b"~ ",
            (None, Some(_)) => b"+ ",
            (Some(_), None) => b"- ",
            (None, None) => unreachable!("a changed export has a side"),
        });
        lines.put(&entry.task);
        lines.put(b".");
        lines.put(&entry.name);
        lines.pad_to(width.saturating_add(16));
        if let Some(before) = &entry.before {
            lines.put(&value_compact(before));
        }
        if entry.before.is_some() && entry.after.is_some() {
            lines.put(" → ".as_bytes());
        }
        if let Some(after) = &entry.after {
            lines.put(&value_compact(after));
        }
        lines.put(b"\n");
    }
}

pub(crate) fn change_json(this: &Change, json: &mut Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"exports");
    json.key(b"exports");
    json.array_start();
    for entry in &this.entries {
        json.object_start();
        if let Some(after) = &entry.after {
            json.key(b"after");
            value_json(after, json);
        }
        if let Some(before) = &entry.before {
            json.key(b"before");
            value_json(before, json);
        }
        crate::form::json::field(json, b"name", &entry.name);
        crate::form::json::field(json, b"task", &entry.task);
        json.object_end();
    }
    json.array_end();
    json.object_end();
}

fn present<'a>(each: &mut core::slice::Iter<'a, Entry>) -> Option<(&'a [u8], &'a Value)> {
    for entry in each.by_ref() {
        if let Some(value) = &entry.value {
            return Some((&entry.key, value));
        }
    }
    None
}
