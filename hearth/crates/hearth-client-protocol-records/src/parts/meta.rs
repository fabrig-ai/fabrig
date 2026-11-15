//! The meta part (hearth/docs/design/domain/render/spec.md, 1.4).
use crate::form::text::Lines;
use hearth_domain_render_vocabulary::{
    parts::meta::Meta,
    revision::{Firmware, Runs},
};

pub(crate) fn text(this: &Meta, lines: &mut Lines, column: usize) {
    lines.put(b"target");
    lines.pad_to(column);
    lines.put(&this.target);
    lines.put(b"   base ");
    lines.put(&this.base);
    if let Some(member) = &this.member {
        lines.put(b"   member ");
        lines.put(member);
    }
    lines.put(b"   environment ");
    lines.put(&this.environment);
    lines.put(b"\n");
    lines.put(b"placement");
    lines.pad_to(column);
    lines.put(match this.placement.kind {
        Runs::Metal => b"metal",
        Runs::Vm => b"vm",
        Runs::Container => b"container",
    });
    if let Some(firmware) = this.placement.firmware {
        lines.put(b" ");
        lines.put(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &this.placement.hardware {
        lines.put(b" ");
        lines.put(hardware);
    }
    if let Some(on) = &this.placement.on {
        lines.put(b" on ");
        lines.put(on);
    }
    lines.put(b"\n");
}

pub(crate) fn json(this: &Meta, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"base");
    json.string(&this.base);
    json.key(b"environment");
    json.string(&this.environment);
    if let Some(member) = &this.member {
        json.key(b"member");
        json.string(member);
    }
    json.key(b"placement");
    json.object_start();
    if let Some(firmware) = this.placement.firmware {
        json.key(b"firmware");
        json.string(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &this.placement.hardware {
        json.key(b"hardware");
        json.string(hardware);
    }
    json.key(b"kind");
    json.string(match this.placement.kind {
        Runs::Metal => b"metal",
        Runs::Vm => b"vm",
        Runs::Container => b"container",
    });
    if let Some(on) = &this.placement.on {
        json.key(b"on");
        json.string(on);
    }
    json.object_end();
    json.key(b"target");
    json.string(&this.target);
    json.object_end();
}
