//! Software records and their fetch hints (hearth/docs/design/domain/render/spec.md, 1.2).
use crate::form::text::{Lines, octal};
use hearth_domain_render_vocabulary::parts::software::{Change, Download, DownloadKeyed, How, Software};

pub(crate) fn text(this: &Software, lines: &mut Lines) {
    for download in &this.downloads {
        lines.put(b"  download   ");
        lines.put(&download.path);
        lines.put(b" ");
        download_text(download, lines);
        lines.put(b"\n");
    }
    for script in &this.scripts {
        lines.put(b"  scripts    ");
        super::config::script_text(script, lines);
        lines.put(b"\n");
    }
    for fetch in &this.fetch {
        lines.put(b"  fetch      ");
        lines.hex12(&fetch.sha256.0);
        lines.put(b" ");
        lines.put(&fetch.url);
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &Software, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"downloads");
    json.object_start();
    for download in &this.downloads {
        json.key(&download.path);
        download_json(download, json);
    }
    json.object_end();
    json.key(b"fetch");
    json.object_start();
    for fetch in &this.fetch {
        json.key(&crate::form::json::digest_hex(&fetch.sha256.0));
        json.string(&fetch.url);
    }
    json.object_end();
    json.key(b"scripts");
    super::config::scripts_json(&this.scripts, json);
    json.object_end();
}

pub(crate) fn download_text(this: &Download, lines: &mut Lines) {
    match this.how {
        How::File { mode } => {
            lines.put(b"file ");
            lines.put(&octal(mode));
        }
        How::Tar { strip } => {
            lines.put(b"tar ");
            lines.decimal(u64::from(strip));
        }
    }
    lines.put(b" ");
    lines.put(&this.owner);
    lines.put(b":");
    lines.put(&this.group);
    lines.put(b" ");
    lines.hex12(&this.sha256.0);
}

pub(crate) fn download_json(this: &Download, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"group");
    json.string(&this.group);
    if let How::File { mode } = this.how {
        json.key(b"mode");
        json.string(&octal(mode));
    }
    json.key(b"owner");
    json.string(&this.owner);
    json.key(b"sha256");
    crate::form::json::digest(json, &this.sha256.0);
    if let How::Tar { strip } = this.how {
        json.key(b"strip");
        json.unsigned(u64::from(strip));
    }
    json.key(b"unpack");
    json.string(match this.how {
        How::File { .. } => b"file",
        How::Tar { .. } => b"tar",
    });
    json.object_end();
}

pub(crate) fn download_sides(this: &DownloadKeyed) -> (&[u8], Option<&Download>, Option<&Download>) {
    match this {
        DownloadKeyed::Added { key, value } => (key, None, Some(value)),
        DownloadKeyed::Removed { key, value } => (key, Some(value), None),
        DownloadKeyed::Changed { key, before, after } => (key, Some(before), Some(after)),
    }
}
pub(crate) fn change_text(this: &Change, lines: &mut Lines) {
    for entry in &this.downloads {
        let (key, before, after) = download_sides(entry);
        crate::diff::placed::header(lines, b"software", key, before.is_some(), after.is_some());
        lines.pad_to(52);
        download_text(after.or(before).expect("changed row"), lines);
        lines.put(b"\n");
    }
    if let Some(scripts) = &this.scripts {
        crate::diff::directives::scripts_text(scripts, lines, b"software");
    }
}

pub(crate) fn change_json(this: &Change, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"software");
    json.key(b"software");
    json.object_start();
    json.key(b"downloads");
    json.array_start();
    for entry in &this.downloads {
        let (key, before, after) = download_sides(entry);
        crate::diff::placed::keyed(json, key, optional_download(before), optional_download(after));
    }
    json.array_end();
    if let Some(scripts) = &this.scripts {
        json.key(b"scripts");
        crate::diff::directives::scripts_json(scripts, json);
    }
    json.object_end();
    json.object_end();
}

fn optional_download(value: Option<&Download>) -> Option<crate::diff::placed::Value<'_>> {
    Some(crate::diff::placed::Value::Download(value?))
}
