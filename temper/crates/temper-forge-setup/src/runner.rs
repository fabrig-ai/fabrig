//! Runner connection configuration (protocol/setup/owned.md, 3); the token stays in its private file.
use crate::{
    check::joined,
    ini::{flatten, socket},
    place::{File, Place},
    setup::{Owned, Runner},
};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};
use skein_lib::{Decimal, List, Secret, bytes};
const UUID_LINE: &[u8] = b"\n      uuid: ";
fn quoted(bytes: &[u8]) -> Box<[u8]> {
    let limits = Limits { depth: 1, length: 32_768 };
    let mut measure = Encoder::measure(&limits);
    measure.string(bytes);
    let mut write = Encoder::write(measure.measured().expect("bounded YAML string"), &limits);
    write.string(bytes);
    write.finish()
}
/// The runner's fixed policy and file-backed Forgejo connection.
#[must_use]
pub fn configuration(runner: &Runner, owned: &Owned, place: &Place, uuid: &[u8]) -> Box<[u8]> {
    let capacity = Decimal::of(u64::from(runner.capacity));
    let mut labels = List::with_capacity(
        u32::try_from(runner.labels.len()).expect("eight labels").checked_mul(2).expect("label separators"),
    );
    for (i, label) in runner.labels.iter().enumerate() {
        if i > 0 {
            labels.push(b", ".as_slice().into()).expect("label separators");
        }
        labels.push(quoted(label)).expect("label capacity");
    }
    joined(&[
        b"runner:\n  capacity: ",
        capacity.as_bytes(),
        b"\n  timeout: 3h\n  shutdown_timeout: 0s\ncache:\n  enabled: false\nhost:\n  workdir_parent: ",
        &quoted(&joined(&[&place.directory, b"/runner/jobs"])),
        b"\nserver:\n  connections:\n    forgejo:\n      url: ",
        &quoted(&joined(&[b"http://", &socket(owned.listen)])),
        UUID_LINE,
        &quoted(uuid),
        b"\n      token_url: ",
        &quoted(&joined(&[b"file:", &place.path(File::RunnerSecret)])),
        b"\n      labels: [",
        &flatten(labels.as_slice()),
        b"]\n",
    ])
}
/// The `uuid` a configuration of `configuration` names, as `register` printed it;
/// `None` for any other text, which the preparation registers again.
#[must_use]
pub fn registered(configuration: &[u8]) -> Option<Secret> {
    let at = bytes::find(configuration, UUID_LINE)?.checked_add(UUID_LINE.len())?;
    let rest = configuration.get(at..)?;
    let line = rest.get(..bytes::find(rest, b"\n")?)?;
    let document = jig_protocol_config::document(
        line,
        &jig_protocol_config::DocumentLimits { bytes: 32_768, tokens: 1, depth: 1 },
    )
    .ok()?;
    let uuid = jig_protocol_config::read::text(&document, 0, 32_768).ok()?;
    if uuid.is_empty() { None } else { Some(Secret::new(uuid)) }
}
