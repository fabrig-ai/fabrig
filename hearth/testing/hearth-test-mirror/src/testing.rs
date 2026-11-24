//! Shared mirror fixtures (hearth/docs/design/testing/real-builds.md, 2).
use crate::{CACHE, Mirror, Pin};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) const PIN: &[u8] = b"# a pin\narchive 20261016T091523Z\nsuite forky\narchitecture amd64\ncomponents main\n\
digest ad093de033f89b590459503f94d25ea4a36fd044f7ce6d674f43e0d8412161f9\ntarget test-min apt\ntarget test-boot apt efibootmgr git\n";
pub(crate) const XZ_SHA256: &str = "b44f7d6b5283a44ee5f2bd98f84087a04810092122d75e8fbf8ad85f8f2981f1";
pub(crate) const SNAPSHOT: &str = "archive/debian/20261016T091523Z/dists/forky";
pub(crate) const FILE: &str = "archive/debian/20261016T091523Z/dists/forky/InRelease";
pub(crate) const URL: &str = "http://snapshot.debian.org/archive/debian/20261016T091523Z/dists/forky/InRelease";
const IN_RELEASE: &[u8] = b"signed\n";
const PACKAGES_XZ: &[u8] = b"xz\n";
const INDEX: &str = "main/binary-amd64";

fn by_hash() -> PathBuf {
    Path::new(SNAPSHOT).join(INDEX).join("by-hash/SHA256").join(XZ_SHA256)
}

#[must_use]
pub fn laid(dir: &Path) -> Mirror {
    let pin = Pin::read(PIN).expect("mirror pin");
    let root = dir.join(CACHE).join(&*pin.digest);
    let index = root.join(SNAPSHOT).join(INDEX);
    fs::create_dir_all(index.join("by-hash/SHA256")).expect("mirror index directory");
    fs::write(root.join(FILE), IN_RELEASE).expect("InRelease fixture");
    fs::write(index.join("Packages.xz"), PACKAGES_XZ).expect("compressed index fixture");
    fs::hard_link(index.join("Packages.xz"), root.join(by_hash())).expect("by-hash name for the compressed index");
    let checked = root.join("checked").join(INDEX);
    fs::create_dir_all(&checked).expect("checked index directory");
    fs::write(checked.join("Packages"), b"index\n").expect("checked index fixture");
    Mirror::find_in(dir, &pin).expect("laid mirror")
}

pub(crate) fn file(path: &Path) -> Option<(&'static [u8], u64)> {
    let path = path.as_os_str();
    let body = if path == FILE {
        IN_RELEASE
    } else if path == by_hash().as_os_str() || path == Path::new(SNAPSHOT).join(INDEX).join("Packages.xz").as_os_str() {
        PACKAGES_XZ
    } else {
        return None;
    };
    Some((body, u64::try_from(body.len()).expect("mirror fixture length")))
}

pub(crate) fn get_head(url: &str) -> String {
    format!("GET {url} HTTP/1.1\r\nHost: snapshot.debian.org\r\n\r\n")
}
