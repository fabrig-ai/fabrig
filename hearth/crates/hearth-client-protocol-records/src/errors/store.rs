use super::{Encoder, Lines, field};
use hearth_domain_render_vocabulary::errors::{Step, StoreError, StoreFault, StoreLimit};

pub(super) fn step_name(item: Step) -> &'static [u8] {
    match item {
        Step::List => b"list",
        Step::Remove => b"remove",
        Step::Load => b"load",
        Step::Empty => b"empty",
        Step::Keep => b"keep",
        Step::Write => b"write",
    }
}

pub(super) fn store_error_text(item: &StoreError, lines: &mut Lines) {
    lines.put(b"the store ");
    lines.put(&item.store);
    match item.fault {
        StoreFault::Full { limit, most } => {
            lines.put(b" is full: ");
            lines.decimal(u64::from(most));
            lines.put(match limit {
                StoreLimit::Resolutions => b" resolutions",

                StoreLimit::CacheFiles => b" cache files",
            });
        }
        StoreFault::Failed { step } => {
            lines.put(b": its ");
            lines.put(step_name(step));
            lines.put(b" failed");
        }
    }
}
pub(super) fn store_error_json(item: &StoreError, json: &mut Encoder) {
    field(json, b"store", &item.store);
    match item.fault {
        StoreFault::Full { limit, most } => {
            field(
                json,
                b"limit",
                match limit {
                    StoreLimit::Resolutions => b"resolutions",

                    StoreLimit::CacheFiles => b"cache_files",
                },
            );
            json.key(b"most");
            json.unsigned(u64::from(most));
        }
        StoreFault::Failed { step } => field(json, b"step", step_name(step)),
    }
}
