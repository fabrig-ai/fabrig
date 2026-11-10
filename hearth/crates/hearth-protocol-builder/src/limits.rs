//! Each child machine retains its own configured bounds (hearth/docs/design/domain/render/README.md, 2.3).
use crate::{config, git_machine, records};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub mmdebstrap: hearth_protocol_mmdebstrap::limits::Limits,
    pub walk: Walk,
    pub git: git_machine::limits::Limits,
    pub config: config::limits::Limits,
    pub template: hearth_protocol_template::limits::Limits,
    pub tree: crate::tree::limits::Limits,
    pub printer: records::limits::Limits,
}
/// The bounded staging walk (hearth/docs/design/domain/store/README.md, 4.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Walk {
    pub depth: u32,
    pub page: u32,
    pub entries: u32,
}
pub const SHIPPED: Limits = Limits {
    mmdebstrap: hearth_protocol_mmdebstrap::limits::SHIPPED,
    walk: Walk { depth: 16, page: 64, entries: 65_536 },
    tree: crate::tree::limits::SHIPPED,
    git: git_machine::limits::SHIPPED,
    config: config::limits::SHIPPED,
    template: hearth_protocol_template::limits::SHIPPED,
    printer: records::limits::SHIPPED,
};

/// Whether these limits are representable: every edge bound they size is
/// finite (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    use git_machine::limits::Decoder as Git;
    use hearth_protocol_template::limits::Decoder as Template;
    use hearth_protocol_tree::frame::Kind;

    for decoder in [Git::Tail, Git::Listing, Git::Batch, Git::Diff] {
        if git_machine::limits::decoder_bound(decoder, &limits.git).is_none() {
            return false;
        }
    }
    for decoder in [Template::Expression, Template::Template, Template::Interpolated { length: limits.config.document }]
    {
        if hearth_protocol_template::limits::decoder_bound(decoder, &limits.template).is_none() {
            return false;
        }
    }
    for kind in [Kind::Store, Kind::Layer, Kind::Tree, Kind::Resolution, Kind::CacheFile] {
        if crate::tree::limits::decoder_bound(kind, &limits.tree).is_none()
            || crate::tree::limits::decoded_bound(kind, &limits.tree).is_none()
        {
            return false;
        }
    }
    // Declaration decoding includes the decoded JSON and schema bounds.
    config::json::decoder_bound(&limits.config).is_some()
        && config::declaration::decoder_bound(&limits.config).is_some()
        && hearth_protocol_mmdebstrap::output::decoder_bound(&limits.mmdebstrap).is_some()
        && hearth_protocol_mmdebstrap::packages::decoder_bound(&limits.mmdebstrap).is_some()
        && hearth_protocol_mmdebstrap::tail::Tail::decoder_bound(&limits.mmdebstrap).is_some()
        && records::printer::valid(&limits.printer)
}
