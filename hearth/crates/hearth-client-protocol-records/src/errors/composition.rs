use hearth_domain_render_vocabulary::errors::{Composition, Host};
use super::{Box, Encoder, Lines, field};



pub(super) fn composition_text(item: &Composition, lines: &mut Lines, target: Option<&[u8]>, task: Option<&[u8]>) {
    match item {
        Composition::Duplicate => {
            lines.put(b"two targets are named ");
            if let Some(target) = target {
                lines.put(target);
            }
        }
        Composition::Long => lines.put(b"a target name past 63 bytes"),
        Composition::Placement => lines.put(b"placement in fleet.json, a base or its member"),
        Composition::Members => lines.put(b"members in fleet.json or an environment"),
        Composition::Unplaced => lines.put(b"no placement once composed"),
        Composition::Null => lines.put(b"null for a task in a default"),
        Composition::Task => {
            lines.put(b"no task ");
            if let Some(task) = task {
                lines.put(task);
            }
            lines.put(b" under tasks/");
        }
        Composition::On { on, host } => {
            lines.put(b"placement's on names ");
            lines.put(on);
            match host {
                Host::Unplaced => lines.put(b", a target with no placement"),
                Host::Unknown => lines.put(b", no discovered target"),
                Host::Container => lines.put(b", a container"),
                Host::Elsewhere { environment } => {
                    lines.put(b", a target of ");
                    lines.put(environment);
                }
            }
        }
    }
}
pub(super) fn composition_json(item: &Composition, json: &mut Encoder) {
    if let Composition::On { on, host } = item {
        json.object_start();
        field(json, b"kind", b"on");
        field(json, b"on", on);
        json.key(b"host");
        match host {
            Host::Unplaced => json.string(b"unplaced"),
            Host::Unknown => json.string(b"unknown"),
            Host::Container => json.string(b"container"),
            Host::Elsewhere { environment } => {
                json.object_start();
                field(json, b"kind", b"elsewhere");
                field(json, b"environment", environment);
                json.object_end();
            }
        }
        json.object_end();
    } else {
        json.string(composition_name(item));
    }
}
pub(super) fn composition_name(item: &Composition) -> &'static [u8] {
    match item {
        Composition::Duplicate => b"duplicate",
        Composition::Long => b"long",
        Composition::Placement => b"placement",
        Composition::Members => b"members",
        Composition::Unplaced => b"unplaced",
        Composition::On { .. } => b"on",
        Composition::Null => b"null",
        Composition::Task => b"task",
    }
}

