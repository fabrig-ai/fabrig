//! Resolution and layer keys (hearth/docs/design/domain/render/spec.md, 1.5, 2.1).
use crate::parts::{Spec, packages::Inputs};
pub use hearth_domain_render_vocabulary::keys::{Keys, Resolution};
use hearth_lib::{encode::Digest, hash::H};

#[must_use]
pub(crate) fn resolution(inputs: &Inputs) -> Digest {
    let mut hash = H::new(b"resolution", 4);
    for items in [&inputs.sources, &inputs.request, &inputs.variant, &inputs.apt] {
        for item in items {
            hash.values().item(item);
        }
    }
    hash.finish()
}
#[must_use]
pub(crate) fn layers(resolution: &Digest, packages: &Digest, software: &Digest, config: &Digest) -> Keys {
    let packages = layer(b"packages", resolution, packages);
    let software = layer(b"software", &packages, software);
    Keys { packages, software, config: layer(b"config", &software, config) }
}
fn layer(tag: &[u8], parent: &Digest, part: &Digest) -> Digest {
    let mut hash = H::new(tag, 2);
    hash.values().digest(parent);
    hash.values().digest(part);
    hash.finish()
}
#[must_use]
pub fn part(spec: &Spec, name: &[u8]) -> Digest {
    for part in &spec.parts {
        if part.record.name() == name {
            return part.digest;
        }
    }
    unreachable!("every output part is assembled")
}

#[must_use]
pub(crate) fn closure(packages: &[crate::resolution::Package]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(u64::try_from(packages.len()).expect("bounded closure"));
    for package in packages {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&package.sha256);
    }
    hash.finish()
}
