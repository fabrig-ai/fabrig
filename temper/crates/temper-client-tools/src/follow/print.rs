//! Whole follow output (temper/docs/design/domain/forge/follows.md, 7).
use super::{Box, Limits, Text, follows, jig, render, w};
use skein_json::writer::Encoder;
fn repository(value: &w::FollowRepository, out: &mut Text) {
    out.number(u64::from(value.forge()));
    out.put(b"/");
    out.number(u64::from(value.repository()));
}
fn show(row: &w::FollowView, out: &mut Text) {
    out.put(&follows::line(row));
    out.put(b"\nsource: ");
    out.put(row.source());
    out.put(b" (");
    repository(row.setting().source().repository(), out);
    out.put(b")\nbranch: ");
    out.put(row.source_branch());
    out.put(b"\npaths:");
    if row.setting().source().paths().is_empty() {
        out.put(b" every path");
    }
    for path in row.setting().source().paths() {
        out.put(b" ");
        out.put(path);
    }
    out.put(b"\ntarget: ");
    out.put(row.target());
    out.put(b" (");
    repository(row.setting().target().repository(), out);
    out.put(b")\nbranch: ");
    out.put(row.target_branch());
    out.put(b"\nowner: person ");
    match row.setting().owner() {
        w::FollowOwner::Person(owner) => out.number(owner.person()),
    }
    out.put(b"\nstart: ");
    out.put(match row.setting().start() {
        w::FollowStart::Owner => b"owner",
        w::FollowStart::Code => b"code",
    });
    out.put(b"\nproducer: ");
    out.put(match row.setting().producer() {
        Some(producer) => producer,
        None => b"default",
    });
    let check = row.setting().verification();
    out.put(b"\ncheck: ");
    out.put(check.workflow());
    out.put(b" \xc2\xb7 artifact ");
    out.put(check.artifact());
    out.put(b" \xc2\xb7 ");
    out.put(match check.format() {
        w::FollowFormat::Junit => b"junit",
        w::FollowFormat::Lines => b"lines",
    });
    out.put(b"\ntask: ");
    match row.task() {
        Some(task) => out.number(*task),
        None => out.put(b"not made yet"),
    }
    if let Some(stopped) = row.stopped() {
        out.put(b"\nstopped: ");
        out.put(stopped);
    }
    out.put(b"\nlag: ");
    out.number(u64::from(row.lag()));
    out.put(b"\n");
    for entry in row.entries() {
        out.put(b"  ");
        out.put(&follows::after12(entry.after()));
        if let Some(change) = entry.change() {
            out.put(b" \xc2\xb7 change ");
            out.number(*change);
        }
        out.put(b" \xc2\xb7 ");
        out.put(&follows::state(entry));
        out.put(b"\n");
    }
    out.put(b"skips:");
    if row.skips().is_empty() {
        out.put(b" none");
    }
    out.put(b"\n");
    for skip in row.skips() {
        out.put(b"  ");
        out.put(&follows::after12(skip.after()));
        out.put(b" \xc2\xb7 ");
        out.put(skip.reason());
        out.put(b"\n");
    }
    out.put(b"parity: ");
    out.put(&follows::parity(row.parity().as_ref()));
    if let Some(parity) = row.parity() {
        out.put(b" \xc2\xb7 target ");
        out.put(&follows::after12(parity.head()));
        out.put(b" \xc2\xb7 ");
        out.put(parity.run());
    }
    out.put(b"\n");
}
fn text(panel: &w::FollowsPanel, row: Option<&w::FollowView>, out: &mut Text) {
    if let Some(row) = row {
        show(row, out);
        return;
    }
    if panel.follows().is_empty() {
        out.put(b"no follows\n");
    }
    for row in panel.follows() {
        out.put(&follows::line(row));
        out.put(b"\n");
    }
}
fn hex(bytes: &[u8; 32]) -> [u8; 64] {
    let mut digits = [0; 64];
    for (i, digit) in digits.iter_mut().enumerate() {
        let byte = *bytes.get(i / 2).expect("64 digits in 32 bytes");
        let nibble = if i % 2 == 0 { byte >> 4_u8 } else { byte & 15 };
        *digit = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex nibble");
    }
    digits
}
fn optional_number(value: Option<u64>, out: &mut Encoder) {
    match value {
        Some(value) => out.unsigned(value),
        None => out.null(),
    }
}
fn repository_json(value: &w::FollowRepository, out: &mut Encoder) {
    out.key(b"forge");
    out.unsigned(u64::from(value.forge()));
    out.key(b"repository");
    out.unsigned(u64::from(value.repository()));
}
fn setting(value: &w::Follow, out: &mut Encoder) {
    out.object_start();
    out.key(b"source");
    out.object_start();
    repository_json(value.source().repository(), out);
    out.key(b"branch");
    out.string(value.source().branch());
    out.key(b"paths");
    out.array_start();
    for path in value.source().paths() {
        out.string(path);
    }
    out.array_end();
    out.object_end();
    out.key(b"target");
    out.object_start();
    repository_json(value.target().repository(), out);
    out.key(b"branch");
    out.string(value.target().branch());
    out.object_end();
    out.key(b"owner");
    out.object_start();
    out.key(b"person");
    match value.owner() {
        w::FollowOwner::Person(person) => out.unsigned(person.person()),
    }
    out.object_end();
    out.key(b"start");
    out.string(match value.start() {
        w::FollowStart::Owner => b"owner",
        w::FollowStart::Code => b"code",
    });
    out.key(b"producer");
    match value.producer() {
        Some(producer) => out.string(producer),
        None => out.null(),
    }
    out.key(b"check");
    out.object_start();
    out.key(b"workflow");
    out.string(value.verification().workflow());
    out.key(b"artifact");
    out.string(value.verification().artifact());
    out.key(b"format");
    out.string(match value.verification().format() {
        w::FollowFormat::Junit => b"junit",
        w::FollowFormat::Lines => b"lines",
    });
    out.object_end();
    out.object_end();
}
fn row_json(row: &w::FollowView, out: &mut Encoder) {
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(row.number()));
    out.key(b"setting");
    setting(row.setting(), out);
    for (key, value) in [
        (b"source".as_slice(), row.source()),
        (b"source_branch", row.source_branch()),
        (b"target", row.target()),
        (b"target_branch", row.target_branch()),
    ] {
        out.key(key);
        out.string(value);
    }
    out.key(b"task");
    optional_number(*row.task(), out);
    out.key(b"stopped");
    match row.stopped() {
        Some(words) => out.string(words),
        None => out.null(),
    }
    out.key(b"lag");
    out.unsigned(u64::from(row.lag()));
    out.key(b"entries");
    out.array_start();
    for entry in row.entries() {
        out.object_start();
        out.key(b"after");
        out.string(&hex(entry.after()));
        out.key(b"change");
        optional_number(*entry.change(), out);
        out.key(b"state");
        out.string(&follows::state(entry));
        out.object_end();
    }
    out.array_end();
    out.key(b"skips");
    out.array_start();
    for skip in row.skips() {
        out.object_start();
        out.key(b"after");
        out.string(&hex(skip.after()));
        out.key(b"reason");
        out.string(skip.reason());
        out.object_end();
    }
    out.array_end();
    out.key(b"parity");
    match row.parity() {
        Some(parity) => {
            out.object_start();
            out.key(b"source");
            out.string(&hex(parity.source()));
            out.key(b"head");
            out.string(&hex(parity.head()));
            out.key(b"verdict");
            match parity.verdict() {
                w::FollowVerdict::Passed => out.string(b"passed"),
                w::FollowVerdict::Many => out.string(b"many"),
                w::FollowVerdict::Failing(failing) => {
                    out.object_start();
                    out.key(b"failing");
                    out.unsigned(u64::from(failing.count()));
                    out.object_end();
                }
            }
            out.key(b"run");
            out.string(parity.run());
            out.object_end();
        }
        None => out.null(),
    }
    out.object_end();
}
fn json(panel: &w::FollowsPanel, row: Option<&w::FollowView>, out: &mut Encoder) {
    if let Some(row) = row {
        row_json(row, out);
        return;
    }
    out.object_start();
    out.key(b"project");
    out.unsigned(u64::from(panel.project()));
    out.key(b"follows");
    out.array_start();
    for row in panel.follows() {
        row_json(row, out);
    }
    out.array_end();
    out.object_end();
}
pub(super) fn print(
    panel: &w::FollowsPanel,
    row: Option<&w::FollowView>,
    is_json: bool,
    limits: &Limits,
) -> jig::Printed {
    let bytes = if is_json {
        let bounds = skein_json::writer::Limits { depth: 10, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        json(panel, row, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", is_json, 1) };
        let mut out = Encoder::write(size, &bounds);
        json(panel, row, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(panel, row, &mut measure);
        if measure.written() > usize::try_from(limits.tools.text).expect("text fits") {
            return render::failure(b"result limit", is_json, 1);
        }
        let mut out = Text::writing_exact(measure);
        text(panel, row, &mut out);
        out.finish()
    };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::default() }
}
