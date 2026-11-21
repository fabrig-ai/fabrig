//! The embedded and directory bundle readers share these bytes.
pub const PAGE: &[u8] = b"<!doctype html><title>t</title>";
pub const BUNDLE_FILES: &[(&str, &[u8])] = &[
    ("index.html", PAGE),
    ("0123456789abcdef/boot.js", b"boot();"),
    ("0123456789abcdef/files/robots.txt", b"User-agent: *\nDisallow:\n"),
];
// The page, boot.js and robots.txt: three files, 31 + 7 + 24 bytes.
pub const BUNDLE_COUNT: u32 = 3;
pub const BUNDLE_BYTES: u32 = 62;
// A .js path at the service's 1024-byte bound.
pub const PATH_STEM_BOUND: usize = temper_service::clients::bundle::NAME_BYTES as usize - b".js".len();
pub const BUNDLE_LIMITS: temper_service::limits::Clients =
    temper_service::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 };
