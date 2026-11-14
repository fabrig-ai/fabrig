//! Typed differences and the lowest changed layer (hearth/docs/design/domain/render/README.md, 4.1, 4.2).
pub use hearth_domain_render_vocabulary::diff::{
    Change, Hunks, Line, LineKind, Needs, PartChange, Rebuild, Side, Tally, TargetChange,
};
mod resolution;
use crate::{
    parts::{PartRecord, Spec, boot, config, data, exports, live, packages, secrets, software},
    vocabulary::Class,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
pub use resolution::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Made {
    Spec(Box<Spec>),
    Failed(Class),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Layer {
    Packages,
    Software,
    Config,
    Slot,
    Other,
}

#[must_use]
pub(crate) const fn layer(part: &PartRecord) -> Layer {
    match part {
        PartRecord::Config(_) => Layer::Config,
        PartRecord::Secrets(_) | PartRecord::Boot(_) | PartRecord::Data(_) => Layer::Slot,
        PartRecord::Software(_) => Layer::Software,
        PartRecord::Packages(_) => Layer::Packages,
        PartRecord::Live(_) | PartRecord::Exports(_) => Layer::Other,
    }
}
#[must_use]
pub(crate) fn rebuild(changed: &[Layer], resolution: Needs) -> Rebuild {
    match changed.iter().min() {
        Some(Layer::Packages) => Rebuild::FromPackages { resolution },
        Some(Layer::Software) => Rebuild::FromSoftware,
        Some(Layer::Config) => Rebuild::FromConfig,
        Some(Layer::Slot) => Rebuild::SlotOnly,
        Some(Layer::Other) | None => Rebuild::Nothing,
    }
}
#[must_use]
pub(crate) fn compare(from: Option<&Made>, to: Option<&Made>, resolution: Needs) -> TargetChange {
    match (from, to) {
        (Some(Made::Failed(_)), Some(Made::Failed(_))) => TargetChange::Failed { side: Side::Both },
        (Some(Made::Failed(_)), _) => TargetChange::Failed { side: Side::From },
        (_, Some(Made::Failed(_))) => TargetChange::Failed { side: Side::To },
        (None, Some(Made::Spec(spec))) => TargetChange::Added { spec_digest: spec.digest },
        (Some(Made::Spec(_)), None) => TargetChange::Removed,
        (Some(Made::Spec(before)), Some(Made::Spec(after))) => {
            if before.digest == after.digest {
                return TargetChange::Same;
            }
            let old = compared_parts(before);
            let new = compared_parts(after);
            let mut parts = skein_lib::List::with_capacity(9);
            if let Some(meta) = crate::parts::meta::change(&before.meta, &after.meta) {
                parts.push(PartChange::Meta(meta)).expect("one meta change");
            }
            let mut layers = skein_lib::List::with_capacity(7);
            if let (Some((from, before)), Some((to, after))) = (old.packages, new.packages)
                && from != to
            {
                parts.push(PartChange::Packages(packages::change(before, after))).expect("one packages");
                layers.push(Layer::Packages).expect("three layers");
            }
            if let (Some((from, before)), Some((to, after))) = (old.boot, new.boot)
                && from != to
            {
                parts.push(PartChange::Boot(boot::change(before, after))).expect("one boot");
                layers.push(Layer::Slot).expect("three layers");
            }
            let changed = match (old.config, new.config) {
                (Some((old, _)), Some((new, _))) => old != new,
                (None, None) => false,
                (Some(_), None) | (None, Some(_)) => true,
            };
            if changed {
                let old = config_record(old.config);
                let new = config_record(new.config);
                parts.push(PartChange::Config(config::change(old, new))).expect("one config");
                layers.push(Layer::Config).expect("one config");
            }
            let data = data::change(old.data, new.data);
            if !data.entries.is_empty() {
                parts.push(PartChange::Data(data)).expect("one data");
                layers.push(Layer::Slot).expect("four layers");
            }
            additional(before, after, &mut parts, &mut layers);
            let exports = exports::change(old.exports, new.exports);
            if !exports.entries.is_empty() {
                parts.push(PartChange::Exports(exports)).expect("one exports");
            }
            TargetChange::Changed { parts: parts.into_boxed(), rebuild: rebuild(layers.as_slice(), resolution) }
        }
        (None, None) => unreachable!("a compared target exists on a side"),
    }
}
fn config_record<'a>(part: Option<(&Digest, &'a config::Config)>) -> Option<&'a config::Config> {
    let (_, record) = part?;
    Some(record)
}

pub(crate) fn tree(change: &mut TargetChange) -> &mut [config::EntryChange] {
    let mut tree: &mut [config::EntryChange] = &mut [];
    if let TargetChange::Changed { parts, .. } = change {
        for part in parts {
            match part {
                PartChange::Config(config) => tree = &mut config.tree,
                PartChange::Data(_)
                | PartChange::Meta(_)
                | PartChange::Packages(_)
                | PartChange::Boot(_)
                | PartChange::Secrets(_)
                | PartChange::Software(_)
                | PartChange::Live(_)
                | PartChange::Exports(_) => {}
            }
        }
    }
    tree
}

struct ComparedParts<'a> {
    config: Option<(&'a Digest, &'a config::Config)>,
    data: Option<&'a data::Data>,
    exports: Option<&'a exports::Exports>,
    packages: Option<(&'a Digest, &'a packages::Packages)>,
    boot: Option<(&'a Digest, &'a boot::Boot)>,
}
fn compared_parts(spec: &Spec) -> ComparedParts<'_> {
    let mut found = ComparedParts { config: None, data: None, exports: None, packages: None, boot: None };
    for part in &spec.parts {
        match &part.record {
            PartRecord::Config(config) => found.config = Some((&part.digest, config)),
            PartRecord::Data(data) => found.data = Some(data),
            PartRecord::Exports(exports) => found.exports = Some(exports),
            PartRecord::Packages(packages) => found.packages = Some((&part.digest, packages)),
            PartRecord::Boot(boot) => found.boot = Some((&part.digest, boot)),
            PartRecord::Secrets(_) | PartRecord::Software(_) | PartRecord::Live(_) => {}
        }
    }
    found
}

fn additional(
    before: &Spec,
    after: &Spec,
    changes: &mut skein_lib::List<PartChange>,
    layers: &mut skein_lib::List<Layer>,
) {
    for old in &before.parts {
        for new in &after.parts {
            if old.digest == new.digest {
                continue;
            }
            let change = match (&old.record, &new.record) {
                (PartRecord::Secrets(before), PartRecord::Secrets(after)) => {
                    PartChange::Secrets(secrets::change(before, after))
                }
                (PartRecord::Software(before), PartRecord::Software(after)) => {
                    PartChange::Software(software::change(before, after))
                }
                (PartRecord::Live(before), PartRecord::Live(after)) => PartChange::Live(live::change(before, after)),
                (
                    PartRecord::Config(_)
                    | PartRecord::Data(_)
                    | PartRecord::Packages(_)
                    | PartRecord::Boot(_)
                    | PartRecord::Exports(_)
                    | PartRecord::Secrets(_)
                    | PartRecord::Software(_)
                    | PartRecord::Live(_),
                    PartRecord::Config(_)
                    | PartRecord::Data(_)
                    | PartRecord::Packages(_)
                    | PartRecord::Boot(_)
                    | PartRecord::Exports(_)
                    | PartRecord::Secrets(_)
                    | PartRecord::Software(_)
                    | PartRecord::Live(_),
                ) => continue,
            };
            changes.push(change).expect("one change per part");
            layers.push(layer(&new.record)).expect("one layer per part");
        }
    }
}
