use hearth_domain_render_vocabulary::errors::{Ids, Owns, Why};
//! Missing fixed ids name the concrete owned entry (hearth/docs/design/domain/render/errors.md, 2).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;




pub(super) const fn owns_name(item: Owns) -> &'static [u8] {
    match item {
        Owns::Data => b"data",
        Owns::Secret => b"secret",
        Owns::Identity => b"identity",
        Owns::Download => b"download",
        Owns::User => b"user",
    }
}


pub(super) const fn why_name(item: Why) -> &'static [u8] {
    match item {
        Why::NoUser => b"no_user",
        Why::NoGroup => b"no_group",
        Why::NoUid => b"no_uid",
        Why::NoGid => b"no_gid",
        Why::OneId => b"one_id",
        Why::NotPackageUser => b"not_package_user",
        Why::NotPackageGroup => b"not_package_group",
    }
}


pub(super) fn ids_text(item: &Ids, lines: &mut Lines) {
    lines.put(&item.owner);
    lines.put(b" owns ");
    lines.put(&item.entry);
    lines.put(b" and has no fixed id: ");
    match item.why {
        Why::NoUser | Why::NoGroup => {
            lines.put(if item.why == Why::NoUser { b"no user " } else { b"no group " });
            lines.put(&item.owner);
            lines.put(b" is declared");
        }
        Why::NoUid => lines.put(b"the user has no uid"),
        Why::NoGid => lines.put(b"the group has no gid"),
        Why::OneId => lines.put(b"a package user needs both uid and gid or neither"),
        Why::NotPackageUser => lines.put(b"a download needs a package user"),
        Why::NotPackageGroup => lines.put(b"a download needs a package group"),
    }
}
pub(super) fn ids_json(item: &Ids, json: &mut Encoder) {
    json.object_start();
    field(json, b"entry", &item.entry);
    field(json, b"of", owns_name(item.of));
    field(json, b"owner", &item.owner);
    field(json, b"why", why_name(item.why));
    json.object_end();
}

