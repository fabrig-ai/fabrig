//! The fixed head of Opened precedes any layout-dependent bytes (protocol/client/versions.md, 1).
use jig_client_domain::contract::VersionRange;
use jig_client_protocol_wire::{self as w, Record as _};
use skein_codec::field;
use skein_lib::{List, Reader};

pub(super) const OPENED: u8 = 4;

pub(super) struct Head {
    pub(super) hello: w::Hello,
    pub(super) layouts: List<w::Layout>,
}
impl Head {
    pub(super) fn read(limits: &w::Limits, reader: &mut Reader<'_>) -> Result<Self, w::Problem> {
        let hello = w::Hello::read(limits, reader)?;
        let layouts = field::read(
            field::Sequence {
                limit: limits.opened_layouts.min(w::CEILINGS.opened_layouts),
                minimum: 33,
                item: field::Nested,
            },
            limits,
            reader,
            w::paths::OPENED_LAYOUTS,
        )?;
        Ok(Self { hello, layouts })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Judgement {
    pub(super) family: Option<u16>,
    pub(super) versions: Option<VersionRange>,
    pub(super) served: bool,
}
impl Judgement {
    pub(super) fn report(self, out: &mut skein_lib::Queue<crate::Request>) {
        out.push(crate::Request::Hello { family: self.family, versions: self.versions, served: self.served });
    }
    pub(super) fn at(
        hello: &w::Hello,
        layouts: Option<&List<w::Layout>>,
        family: Option<u16>,
        version: u32,
        layout: [u8; 32],
    ) -> Self {
        let mut versions = None;
        for range in &hello.served {
            if super::translate::envelope::family(&range.owner) == family {
                versions = Some(VersionRange { first: range.first, last: range.last });
                break;
            }
        }
        let in_range = match versions {
            Some(range) => range.contains(version),
            None => false,
        };
        let layout_matches = match layouts {
            Some(layouts) => matches_layout(layouts, family, layout),
            None => true,
        };
        let served = in_range && layout_matches;
        Self { family, versions, served }
    }
}

fn matches_layout(layouts: &List<w::Layout>, family: Option<u16>, expected: [u8; 32]) -> bool {
    let mut named = false;
    for layout in layouts {
        if super::translate::envelope::family(&layout.owner) == family {
            if layout.layout != expected {
                return false;
            }
            named = true;
        }
    }
    named
}
