//! The fixed head of Opened precedes any layout-dependent bytes (protocol/client/versions.md, 1).
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
        // The fixed envelope includes the range fields, which do not enter a judgement.
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
    pub(super) served: bool,
}
impl Judgement {
    pub(super) fn report(self, out: &mut skein_lib::Queue<crate::Request>) {
        out.push(crate::Request::Hello { family: self.family, served: self.served });
    }
    pub(super) fn at(layouts: &[w::Layout], family: Option<u16>, layout: [u8; 32]) -> Self {
        Self { family, served: matches_layout(layouts, family, layout) }
    }
}

fn matches_layout(layouts: &[w::Layout], family: Option<u16>, expected: [u8; 32]) -> bool {
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
