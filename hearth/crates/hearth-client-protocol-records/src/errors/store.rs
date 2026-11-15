use super::{Encoder, Lines, field};
use crate::progress::StoreLimit;
use hearth_domain_render_vocabulary::errors::{Step, StoreError, StoreFault, StoreLimit as RenderLimit};

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

fn full_limit(limit: RenderLimit) -> StoreLimit {
    match limit {
        RenderLimit::Resolutions => StoreLimit::Resolutions,
        RenderLimit::CacheFiles => StoreLimit::CacheFiles,
    }
}

pub(super) fn full_text(store: &[u8], limit: StoreLimit, most: u32, lines: &mut Lines) {
    lines.put(b"the store ");
    lines.put(store);
    lines.put(b" is full: ");
    lines.decimal(u64::from(most));
    lines.put(match limit {
        StoreLimit::Resolutions => b" resolutions",
        StoreLimit::Layers => b" layers",
        StoreLimit::CacheFiles => b" cache files",
    });
}

pub(super) fn full_json(store: &[u8], limit: StoreLimit, most: u32, json: &mut Encoder) {
    field(json, b"store", store);
    field(
        json,
        b"limit",
        match limit {
            StoreLimit::Resolutions => b"resolutions",
            StoreLimit::Layers => b"layers",
            StoreLimit::CacheFiles => b"cache_files",
        },
    );
    json.key(b"most");
    json.unsigned(u64::from(most));
}

pub(super) fn store_error_text(item: &StoreError, lines: &mut Lines) {
    match item.fault {
        StoreFault::Full { limit, most } => full_text(&item.store, full_limit(limit), most, lines),
        StoreFault::Failed { step } => {
            lines.put(b"the store ");
            lines.put(&item.store);
            lines.put(b": its ");
            lines.put(step_name(step));
            lines.put(b" failed");
        }
    }
}
pub(super) fn store_error_json(item: &StoreError, json: &mut Encoder) {
    match item.fault {
        StoreFault::Full { limit, most } => full_json(&item.store, full_limit(limit), most, json),
        StoreFault::Failed { step } => {
            field(json, b"store", &item.store);
            field(json, b"step", step_name(step));
        }
    }
}
