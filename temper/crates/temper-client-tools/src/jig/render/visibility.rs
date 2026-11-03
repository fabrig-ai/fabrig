//! Public paths come only from me (client/browser.md, 8.1).
use super::Text;
use crate::jig::{
    digest::Digest,
    runner::{FileEnd, Printed},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_json::writer::{Encoder, Limits};

pub(super) fn page(identifier: u64) -> [u8; 33] {
    let mut path = *b"/public/projects/0000000000000000";
    let hex = Digest::from_bytes(identifier.to_be_bytes()).hex();
    for (slot, byte) in path.get_mut(17..).expect("public path prefix").iter_mut().zip(hex) {
        *slot = byte;
    }
    path
}
pub(super) fn identifier(me: Option<&c::Me>, project: u32) -> Option<u64> {
    for held in &me?.projects {
        if held.project == project {
            return held.public;
        }
    }
    None
}
pub(crate) fn print(project: u32, identifier: Option<u64>, json: bool) -> Printed {
    let stdout = if json {
        // One answer holds a u32, a bool and the fixed public path.
        let bounds = Limits { depth: 2, length: 128 };
        let mut measure = Encoder::measure(&bounds);
        encode(project, identifier, &mut measure);
        let size = measure.measured().expect("fixed visibility answer fits");
        let mut out = Encoder::write(size, &bounds);
        encode(project, identifier, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(project, identifier, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(project, identifier, &mut out);
        out.finish()
    };
    Printed { file: FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}
fn text(project: u32, identifier: Option<u64>, out: &mut Text) {
    out.put(b"project ");
    out.number(u64::from(project));
    match identifier {
        Some(identifier) => {
            out.put(b": public\npage: ");
            out.put(&page(identifier));
            out.put(b"\n");
        }
        None => out.put(b": private\n"),
    }
}
fn encode(project: u32, identifier: Option<u64>, out: &mut Encoder) {
    out.object_start();
    out.key(b"answer");
    out.object_start();
    out.key(b"project");
    out.unsigned(u64::from(project));
    out.key(b"public");
    out.boolean(identifier.is_some());
    out.key(b"page");
    match identifier {
        Some(identifier) => out.string(&page(identifier)),
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
