//! The packages record in client forms (hearth/docs/design/domain/render/spec.md, 1.2).
use crate::form::text::Lines;
use alloc::boxed::Box;
use hearth_domain_render_vocabulary::parts::packages::{Apt, Initramfs, Packages, Preseed, Seeds, Sources, Variant};

pub(crate) mod change;
mod seeds;
pub(crate) use change::value_text;

pub(crate) fn text(this: &Packages, lines: &mut Lines) {
    lines.put(b"  sources    ");
    lines.put(&this.sources.suite);
    lines.put(b" ");
    lines.put(&this.sources.architecture);
    for component in &this.sources.components {
        lines.put(b" ");
        lines.put(component.name());
    }
    lines.put(b" ");
    lines.put(&this.sources.archive);
    if this.sources.updates {
        lines.put(b" updates");
    }
    if let Some(security) = &this.sources.security {
        lines.put(b" security ");
        lines.put(security);
    }
    lines.put(b"\n");
    if !this.request.is_empty() {
        lines.put(b"  request   ");
        for name in &this.request {
            lines.put(b" ");
            lines.put(name);
        }
        lines.put(b"\n");
    }
    lines.put(b"  variant    ");
    lines.put(this.variant.name());
    lines.put(b"\n");
    for preseed in &this.preseeds {
        lines.put(b"  preseed    ");
        lines.put(&preseed.question);
        lines.put(b" ");
        lines.put(preseed.kind.name());
        lines.put(b" ");
        lines.put(&preseed.value);
        if !preseed.question.starts_with(&preseed.owner) || preseed.question.get(preseed.owner.len()) != Some(&b'/') {
            lines.put(b" owner ");
            lines.put(&preseed.owner);
        }
        lines.put(b"\n");
    }
    if let Some(initramfs) = &this.initramfs {
        if !initramfs.modules.is_empty() {
            lines.put(b"  initramfs ");
            for module in &initramfs.modules {
                lines.put(b" ");
                lines.put(module);
            }
            lines.put(b"\n");
        }
        lines.put(b"  microcode  ");
        lines.put(initramfs.microcode.name());
        lines.put(b"\n");
    }
    lines.put(b"  apt        install_recommends ");
    lines.put(if this.apt.install_recommends { b"true" } else { b"false" });
    lines.put(b"\n");
    seeds::text(&this.seeds, lines);
}

pub(crate) fn json(this: &Packages, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"apt");
    apt(json, &this.apt);
    if this.initramfs.is_some() {
        json.key(b"initramfs");
        initramfs(json, this.initramfs.as_ref());
    }
    json.key(b"preseeds");
    preseeds(json, &this.preseeds);
    json.key(b"request");
    names(json, &this.request);
    json.key(b"seeds");
    seeds::json(&this.seeds, json);
    json.key(b"sources");
    sources(json, &this.sources);
    json.key(b"variant");
    variant(json, this.variant);
    json.object_end();
}
fn names(json: &mut skein_json::writer::Encoder, names: &[Box<[u8]>]) {
    json.array_start();
    for name in names {
        json.string(name);
    }
    json.array_end();
}

fn apt(json: &mut skein_json::writer::Encoder, value: &Apt) {
    json.object_start();
    json.key(b"install_recommends");
    json.boolean(value.install_recommends);
    json.object_end();
}

fn initramfs(json: &mut skein_json::writer::Encoder, value: Option<&Initramfs>) {
    if let Some(initramfs) = value {
        json.object_start();
        json.key(b"microcode");
        json.string(initramfs.microcode.name());
        json.key(b"modules");
        names(json, &initramfs.modules);
        json.object_end();
    } else {
        json.null();
    }
}

fn preseeds(json: &mut skein_json::writer::Encoder, value: &[Preseed]) {
    json.object_start();
    for preseed in value {
        json.key(&preseed.question);
        json.object_start();
        json.key(b"owner");
        json.string(&preseed.owner);
        json.key(b"type");
        json.string(preseed.kind.name());
        json.key(b"value");
        json.string(&preseed.value);
        json.object_end();
    }
    json.object_end();
}

fn sources(json: &mut skein_json::writer::Encoder, value: &Sources) {
    json.object_start();
    json.key(b"architecture");
    json.string(&value.architecture);
    json.key(b"archive");
    json.string(&value.archive);
    json.key(b"components");
    json.array_start();
    for component in &value.components {
        json.string(component.name());
    }
    json.array_end();
    if let Some(security) = &value.security {
        json.key(b"security");
        json.string(security);
    }
    json.key(b"suite");
    json.string(&value.suite);
    json.key(b"updates");
    json.boolean(value.updates);
    json.object_end();
}

fn variant(json: &mut skein_json::writer::Encoder, value: Variant) {
    json.string(value.name());
}
