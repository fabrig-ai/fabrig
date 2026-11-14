//! A part's record and its forms (hearth/docs/design/domain/render/spec.md, 1).
pub(crate) mod boot;
pub(crate) mod config;
pub(crate) mod data;
pub(crate) mod exports;
pub(crate) mod live;
pub(crate) mod meta;
pub(crate) mod packages;
pub(crate) mod secrets;
pub(crate) mod software;

use crate::form::text::Lines;
use hearth_domain_render_vocabulary::parts::PartRecord;
pub(crate) fn part_json(this: &PartRecord, json: &mut skein_json::writer::Encoder) {
    match this {
        PartRecord::Config(config) => config::json(config, json),
        PartRecord::Data(data) => data::json(data, json),
        PartRecord::Secrets(secrets) => secrets::json(secrets, json),
        PartRecord::Packages(packages) => packages::json(packages, json),
        PartRecord::Live(live) => live::json(live, json),
        PartRecord::Software(software) => software::json(software, json),
        PartRecord::Boot(boot) => boot::json(boot, json),
        PartRecord::Exports(exports) => exports::json(exports, json),
    }
}

pub(crate) fn part_text(this: &PartRecord, lines: &mut Lines) {
    match this {
        PartRecord::Config(config) => config::text(config, lines),
        PartRecord::Data(data) => data::text(data, lines),
        PartRecord::Secrets(secrets) => secrets::text(secrets, lines),
        PartRecord::Packages(packages) => packages::text(packages, lines),
        PartRecord::Live(live) => live::text(live, lines),
        PartRecord::Software(software) => software::text(software, lines),
        PartRecord::Boot(boot) => boot::text(boot, lines),
        PartRecord::Exports(exports) => exports::text(exports, lines),
    }
}
