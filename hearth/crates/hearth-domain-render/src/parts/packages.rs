//! The packages record and its four resolution inputs (hearth/docs/design/domain/render/spec.md, 1.2, 2.1).
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::parts::packages::{
    Apt, Component, Initramfs, Microcode, Packages, Preseed, PreseedKind, SeedGroup, SeedUser, Seeds, Sources, Variant,
};
mod change;
pub use change::*;
use hearth_lib::{
    encode::{Digest, Item},
    hash::H,
};
use skein_lib::{List, bytes};
pub(super) mod build;
mod declarations;
pub(crate) use build::build;
pub use build::read;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inputs {
    pub sources: Box<[Item]>,
    pub request: Box<[Item]>,
    pub variant: Box<[Item]>,
    pub apt: Box<[Item]>,
}

fn text(value: &[u8]) -> Item {
    Item::Bytes(bytes::copy_of(value))
}
fn boolean(value: bool) -> Item {
    if value { Item::True } else { Item::False }
}
fn push(items: &mut List<Item>, item: Item) {
    items.push(item).expect("record item count");
}
fn field(items: &mut List<Item>, name: &[u8], value: Item) {
    push(items, text(name));
    push(items, value);
}
fn count(length: usize) -> u64 {
    u64::try_from(length).expect("bounded record")
}
#[must_use]
pub fn inputs(packages: &Packages) -> Inputs {
    let source = &packages.sources;
    let mut sources = List::with_capacity(19);
    push(&mut sources, Item::Map(5_u64.saturating_add(u64::from(source.security.is_some()))));
    field(&mut sources, b"architecture", text(&source.architecture));
    field(&mut sources, b"archive", text(&source.archive));
    field(&mut sources, b"components", Item::List(count(source.components.len())));
    for component in &source.components {
        push(&mut sources, text(component.name()));
    }
    if let Some(security) = &source.security {
        field(&mut sources, b"security", text(security));
    }
    field(&mut sources, b"suite", text(&source.suite));
    field(&mut sources, b"updates", boolean(source.updates));
    let mut request =
        List::with_capacity(u32::try_from(packages.request.len()).expect("bounded request").saturating_add(1));
    push(&mut request, Item::List(count(packages.request.len())));
    for name in &packages.request {
        push(&mut request, text(name));
    }
    Inputs {
        sources: sources.into_boxed(),
        request: request.into_boxed(),
        variant: Box::new([text(packages.variant.name())]),
        apt: Box::new([Item::Map(1), text(b"install_recommends"), boolean(packages.apt.install_recommends)]),
    }
}
#[must_use]
pub fn digest(packages: &Packages) -> Digest {
    let inputs = inputs(packages);
    let mut hash = H::new(b"part:packages", 1);
    let record = hash.values();
    record.map(6_u64.saturating_add(u64::from(packages.initramfs.is_some())));
    record.key(b"apt");
    for item in &inputs.apt {
        record.item(item);
    }
    if let Some(initramfs) = &packages.initramfs {
        record.key(b"initramfs");
        record.map(2);
        record.key(b"microcode");
        record.bytes(initramfs.microcode.name());
        record.key(b"modules");
        record.list(count(initramfs.modules.len()));
        for module in &initramfs.modules {
            record.bytes(module);
        }
    }
    record.key(b"preseeds");
    record.map(count(packages.preseeds.len()));
    for preseed in &packages.preseeds {
        record.key(&preseed.question);
        record.map(3);
        record.key(b"owner");
        record.bytes(&preseed.owner);
        record.key(b"type");
        record.bytes(preseed.kind.name());
        record.key(b"value");
        record.bytes(&preseed.value);
    }
    record.key(b"request");
    for item in &inputs.request {
        record.item(item);
    }
    record.key(b"seeds");
    record.map(2);
    record.key(b"groups");
    record.map(count(packages.seeds.groups.len()));
    for group in &packages.seeds.groups {
        record.key(&group.name);
        record.map(1);
        record.key(b"gid");
        record.uint(u64::from(group.gid));
    }
    record.key(b"users");
    record.map(count(packages.seeds.users.len()));
    for user in &packages.seeds.users {
        record.key(&user.name);
        record.map(2);
        record.key(b"gid");
        match user.gid {
            Some(gid) => record.uint(u64::from(gid)),
            None => record.none(),
        }
        record.key(b"uid");
        record.uint(u64::from(user.uid));
    }
    for (name, items) in [(b"sources".as_slice(), &inputs.sources), (b"variant", &inputs.variant)] {
        record.key(name);
        for item in items {
            record.item(item);
        }
    }
    hash.finish()
}
