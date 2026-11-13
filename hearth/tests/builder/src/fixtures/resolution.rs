use super::File;
use hearth_protocol_tree::resolution::{Package, Resolution};
pub use hearth_test_repository::{
    TEST_MIN_DIGEST, TEST_MIN_KEY, TEST_MIN_KEYS_LINE, TEST_MIN_RESOLUTION, TEST_REPOSITORY, hex_digest,
    record_test_min, recorded_test_min, test_min_recorded,
};
#[must_use]
pub fn spec_packages_digests(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| line.contains("\"record\":\"spec\"") || line.contains("\"name\":\"packages\""))
        .map(|line| {
            line.split_once("\"digest\":\"").expect("digest field").1.split_once('"').expect("digest text").0.to_owned()
        })
        .collect()
}

#[must_use]
pub fn dead_proxy() -> Box<[u8]> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("loopback port");
    format!("http://{}", listener.local_addr().expect("bound address")).into_bytes().into()
}

use super::{Call, Form, Release, Rendered, Repository};
use std::os::unix::ffi::OsStrExt;
pub(super) fn call_at(repository: &Repository, at: &str, targets: &[&[u8]], resolve: bool) -> Call {
    Call::Render {
        resolve,
        repository: repository.work_tree().join(".git").as_os_str().as_bytes().into(),
        revision: at.as_bytes().into(),
        targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
    }
}
#[must_use]
pub fn resolve(repository: &Repository, targets: &[&[u8]], form: Form, proxy: &[u8]) -> Rendered {
    super::run_holding(
        repository,
        call_at(repository, "HEAD", targets, true),
        form,
        None,
        &hearth::limits(),
        Release::After(skein_lib::Duration::ZERO),
        Some(proxy.into()),
    )
    .rendered
}

#[must_use]
pub fn unknown_package_repository() -> Repository {
    let repository = Repository::make("unknown-package", TEST_REPOSITORY);
    drop(repository.next(
        &[
            File { path: "tasks/absent/task.json", body: br#"{"packages":{"hearth-absent":{}}}"#, mode: 0o644 },
            File { path: "hosts/test/test-min.json", body: br#"{"absent":{}}"#, mode: 0o644 },
        ],
        &[],
    ));
    repository
}

#[must_use]
pub fn mirror() -> hearth_test_mirror::Mirror {
    hearth_test_mirror::Mirror::find(&hearth_test_mirror::Pin::committed()).expect("fetched pinned mirror")
}
#[must_use]
pub fn resolved_test_min(label: &str) -> (Repository, Rendered) {
    let repository = Repository::make(label, TEST_REPOSITORY);
    let proxy = hearth_test_mirror::Proxy::serve(&mirror()).expect("loopback proxy");
    let rendered = resolve(&repository, &[b"test-min"], Form::Text, proxy.url().as_bytes());
    (repository, rendered)
}
#[must_use]
pub fn read_test_min(repository: &Repository) -> Resolution {
    let bytes =
        std::fs::read(repository.scratch().path().join("store/resolutions").join(format!("{TEST_MIN_KEY}.record")))
            .expect("recorded resolution");
    hearth_protocol_tree::resolution::read(&bytes, &hearth_protocol_tree::limits::SHIPPED).expect("resolution record")
}
#[must_use]
pub fn recorded_closure() -> Box<[Package]> {
    let recording = hearth_recordings::all("mmdebstrap")
        .into_iter()
        .find(|(name, _)| name.as_ref() == "test-min")
        .expect("test-min recording")
        .1;
    let bytes = recording.stanzas.expect("recorded installed stanzas");
    let stanzas = std::str::from_utf8(&bytes).expect("stanza UTF-8");
    let mut packages = Vec::new();
    for stanza in stanzas.split("\n\n").filter(|stanza| !stanza.is_empty()) {
        let field =
            |name: &str| stanza.lines().find_map(|line| line.strip_prefix(name)).expect("recorded package field");
        let mut sha256 = [0; 32];
        for (byte, pair) in sha256.iter_mut().zip(field("SHA256: ").as_bytes().as_chunks::<2>().0) {
            *byte = u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16).expect("digest byte");
        }
        packages.push(Package {
            name: field("Package: ").as_bytes().into(),
            architecture: field("Architecture: ").as_bytes().into(),
            version: field("Version: ").as_bytes().into(),
            filename: field("Filename: ").as_bytes().into(),
            size: field("Size: ").parse().expect("package size"),
            sha256: hearth_lib::encode::Digest(sha256),
            source: 0,
        });
    }
    packages.sort_by(|left, right| (&left.name, &left.architecture).cmp(&(&right.name, &right.architecture)));
    packages.into_boxed_slice()
}

#[must_use]
pub fn failed_test_min(label: &str) -> (Repository, Rendered) {
    let repository = Repository::make(label, TEST_REPOSITORY);
    let rendered = resolve(&repository, &[b"test-min"], Form::Text, &dead_proxy());
    (repository, rendered)
}
#[must_use]
pub fn mirror_indices() -> [(&'static [u8], [u8; 32]); 2] {
    let mirror = mirror();
    let mut release = [0; 32];
    for (byte, pair) in release.iter_mut().zip(mirror.pin().digest.as_bytes().as_chunks::<2>().0) {
        *byte = u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16).expect("digest");
    }
    [
        (b"InRelease", release),
        (
            b"main/binary-amd64/Packages",
            skein_lib::sha256::digest(&std::fs::read(mirror.index("main")).expect("mirror index")),
        ),
    ]
}
#[must_use]
pub fn cached_index(
    repository: &Repository,
    digest: [u8; 32],
) -> (std::path::PathBuf, hearth_protocol_tree::cache_file::CacheFile) {
    let name = hex_digest(digest);
    let cache = repository.scratch().path().join("store/cache");
    let bytes = std::fs::read(cache.join(format!("{name}.record"))).expect("cache record");
    (
        cache.join(name),
        hearth_protocol_tree::cache_file::read(&bytes, &hearth_protocol_tree::limits::SHIPPED).expect("kind 5"),
    )
}
