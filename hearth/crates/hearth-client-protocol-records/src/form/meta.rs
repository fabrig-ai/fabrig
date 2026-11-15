//! Meta's changed fields (hearth/docs/design/domain/render/README.md, 4.1).
use crate::form::{json::field, text::Lines};
use hearth_domain_render_vocabulary::parts::meta::{
    Change as MetaChange, MemberPair, NamePair, Placement, PlacementPair,
};
use skein_json::writer::Encoder;

enum Value<'a> {
    Bytes(Option<&'a [u8]>),
    Placement(&'a Placement),
}
struct Field<'a> {
    name: &'static [u8],
    pair: Option<ValuePair<'a>>,
}
struct ValuePair<'a> {
    before: Value<'a>,
    after: Value<'a>,
}
fn fields(this: &MetaChange) -> [Field<'_>; 4] {
    [
        Field { name: b"base", pair: bytes(this.base.as_ref()) },
        Field { name: b"member", pair: member(this.member.as_ref()) },
        Field { name: b"environment", pair: bytes(this.environment.as_ref()) },
        Field { name: b"placement", pair: placement(this.placement.as_ref()) },
    ]
}

pub(crate) fn text(this: &MetaChange, lines: &mut Lines) {
    for Field { name, pair } in fields(this) {
        if let Some(ValuePair { before, after }) = pair {
            lines.put(b"  meta     ~ ");
            lines.put(name);
            lines.put(b"  ");
            before.text(lines);
            lines.put(" → ".as_bytes());
            after.text(lines);
            lines.put(b"\n");
        }
    }
}

pub(crate) fn json(this: &MetaChange, json: &mut Encoder) {
    json.object_start();
    field(json, b"kind", b"meta");
    for Field { name, pair } in fields(this) {
        if let Some(ValuePair { before, after }) = pair {
            json.key(name);
            json.object_start();
            json.key(b"before");
            before.json(json);
            json.key(b"after");
            after.json(json);
            json.object_end();
        }
    }
    json.object_end();
}
fn bytes(pair: Option<&NamePair>) -> Option<ValuePair<'_>> {
    let pair = pair?;
    Some(ValuePair { before: Value::Bytes(Some(&pair.before)), after: Value::Bytes(Some(&pair.after)) })
}
impl Value<'_> {
    fn text(&self, lines: &mut Lines) {
        match self {
            Self::Bytes(Some(bytes)) => lines.put(bytes),
            Self::Bytes(None) => lines.put(b"none"),
            Self::Placement(placement) => placement_text(placement, lines),
        }
    }
    fn json(&self, json: &mut Encoder) {
        match self {
            Self::Bytes(Some(bytes)) => json.string(bytes),
            Self::Bytes(None) => json.null(),
            Self::Placement(placement) => placement_json(placement, json),
        }
    }
}

fn member(pair: Option<&MemberPair>) -> Option<ValuePair<'_>> {
    let p = pair?;
    Some(ValuePair { before: Value::Bytes(p.before.as_deref()), after: Value::Bytes(p.after.as_deref()) })
}
fn placement(pair: Option<&PlacementPair>) -> Option<ValuePair<'_>> {
    let p = pair?;
    Some(ValuePair { before: Value::Placement(&p.before), after: Value::Placement(&p.after) })
}

fn placement_text(p: &Placement, lines: &mut Lines) {
    use hearth_domain_render_vocabulary::revision::{Firmware, Runs};
    lines.put(match p.kind {
        Runs::Metal => b"metal",
        Runs::Vm => b"vm",
        Runs::Container => b"container",
    });
    if let Some(firmware) = p.firmware {
        lines.put(b" ");
        lines.put(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &p.hardware {
        lines.put(b" ");
        lines.put(hardware);
    }
    if let Some(on) = &p.on {
        lines.put(b" on ");
        lines.put(on);
    }
}
fn placement_json(p: &Placement, json: &mut Encoder) {
    use hearth_domain_render_vocabulary::revision::{Firmware, Runs};
    json.object_start();
    if let Some(firmware) = p.firmware {
        field(
            json,
            b"firmware",
            match firmware {
                Firmware::Efi => b"efi",
                Firmware::Bios => b"bios",
            },
        );
    }
    if let Some(hardware) = &p.hardware {
        field(json, b"hardware", hardware);
    }
    field(
        json,
        b"kind",
        match p.kind {
            Runs::Metal => b"metal",
            Runs::Vm => b"vm",
            Runs::Container => b"container",
        },
    );
    if let Some(on) = &p.on {
        field(json, b"on", on);
    }
    json.object_end();
}
