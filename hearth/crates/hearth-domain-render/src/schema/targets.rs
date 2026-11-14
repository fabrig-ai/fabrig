//! Target roots and document patches no discovered target uses.
use super::{Against, Check, Defaults, Fault, Validator, push};
use crate::{
    compose::{self, Merge, Settings, Target},
    evaluate::{Entry, Value},
    revision::{Documents, Entry as Listed},
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Debug)]
pub(crate) struct Revision<'a> {
    pub documents: &'a Documents,
    pub defaults: &'a Defaults,
    pub listing: &'a [Listed],
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Checked {
    pub name: Box<[u8]>,
    pub settings: Value,
    pub tasks: Value,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Validated {
    pub tasks: Box<[Checked]>,
}
pub(crate) fn validate(
    settings: &Settings,
    revision: &Revision<'_>,
    check: &Check<'_>,
    validator: &mut Validator,
    left: &mut u64,
    faults: &mut List<(Box<[u8]>, Fault)>,
) -> Option<Validated> {
    validator.limited = false;
    let before = faults.len();
    let mut tasks = List::with_capacity(u32::try_from(settings.tasks.len()).expect("task count"));
    for task in &settings.tasks {
        let (_, Some(declaration)) = revision.documents.declaration(&task.name)? else { return None };
        let defaults = revision.defaults.of(&task.name)?;
        let against = Against { declaration, defaults, listing: revision.listing };
        let mut local = List::uncapped();
        let mut value = task.settings.clone();
        value.from = task.from;
        let value = validator.settings(&value, &against, check, left, &mut local);
        for fault in &local {
            push(faults, &task.name, fault.clone());
        }
        if validator.limited {
            return None;
        }
        if let Some(settings) = value {
            tasks
                .push(Checked { name: task.name.clone(), settings, tasks: Value::Map(Box::new([])) })
                .expect("task count");
        }
    }
    if tasks.len() != u32::try_from(settings.tasks.len()).expect("task count") {
        return None;
    }
    for index in 0..tasks.len() {
        let task = tasks.get(index).expect("task count");
        let (document, Some(declaration)) = revision.documents.declaration(&task.name)? else { return None };
        let mut size = 0_u64;
        for read in &declaration.reads {
            size = size.saturating_add(super::entry_size(&read.task.text));
        }
        if !reserve(size, left, validator) {
            return None;
        }
        let mut reads = List::with_capacity(u32::try_from(declaration.reads.len()).expect("read count"));
        for read in &declaration.reads {
            let mut value = None;
            let mut source = None;
            for other in &tasks {
                if other.name == read.task.text {
                    source = Some(&other.settings);
                }
            }
            if let Some(source) = source {
                let Value::Map(settings) = source else { unreachable!("settings map") };
                let mut size = 0_u64;
                for name in &read.settings {
                    size = size.saturating_add(super::entry_size(&name.text));
                }
                if !reserve(size, left, validator) {
                    return None;
                }
                let mut entries = List::with_capacity(u32::try_from(read.settings.len()).expect("read count"));
                for name in &read.settings {
                    let mut found = None;
                    for entry in settings {
                        if entry.key == name.text {
                            found = Some(entry);
                        }
                    }
                    let entry = found.expect("declared read");
                    if let Some(value) = &entry.value
                        && !crate::held::charge(crate::held::Tree::Validated(value), left, validator.stack.capacity())
                    {
                        validator.limited = true;
                        return None;
                    }
                    entries.push(entry.clone()).expect("read count");
                }
                value = Some(Value::Map(entries.into_boxed()));
            } else if !(read.optional || read.exports && read.settings.is_empty()) {
                push(
                    faults,
                    &task.name,
                    Fault {
                        schema: super::Schema::Absent { task: read.task.text.clone() },
                        document,
                        at: read.task.at,
                    },
                );
            }
            reads.push(Entry { key: read.task.text.clone(), value }).expect("read count");
        }
        let reads = Value::Map(reads.into_boxed());
        tasks.get_mut(index).expect("task count").tasks = reads;
    }
    if faults.len() == before { Some(Validated { tasks: tasks.into_boxed() }) } else { None }
}
fn reserve(size: u64, left: &mut u64, validator: &mut Validator) -> bool {
    match left.checked_sub(size) {
        Some(rest) => {
            *left = rest;
            true
        }
        None => {
            validator.limited = true;
            false
        }
    }
}
pub(crate) fn unclaimed(
    targets: &[Target],
    revision: &Revision<'_>,
    validator: &mut Validator,
    merge: &mut Merge,
    faults: &mut List<(Box<[u8]>, Fault)>,
) {
    for (index, _) in revision.documents.entries().iter().enumerate() {
        let index = u32::try_from(index).expect("document count");
        let Some(document) = revision.documents.get(index) else { continue };
        for task in &document.layer.tasks {
            let mut claimed = false;
            for target in targets {
                if [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
                    .contains(&Some(index))
                    && compose::present(target, revision.documents).contains(&task.name)
                {
                    claimed = true;
                    break;
                }
            }
            if claimed {
                continue;
            }
            let Some((_, Some(declaration))) = revision.documents.declaration(&task.name) else { continue };
            let Some(defaults) = revision.defaults.of(&task.name) else { continue };
            let Some(value) = merge.merge(None, &task.value, compose::Layer::Base, index) else { continue };
            let against = Against { declaration, defaults, listing: revision.listing };
            let mut local = List::uncapped();
            let mut left = u64::MAX;
            let _value = validator.settings(&value, &against, &Check::Patch, &mut left, &mut local);
            for fault in &local {
                push(faults, &task.name, fault.clone());
            }
        }
    }
}
