//! The browser's bundle as temper's build made it (client/web/README.md, 2.6).
//! `FILES` holds each file's path from the bundle's root (`index.html`, then `<digest>/...`) and its bytes,
//! in a directory read's order; `DIGEST` names the digest directory. Built with `TEMPER_INTERNAL_BUNDLE=none`, `FILES`
//! is empty and `DIGEST` is `None`.
#![no_std]

/// The bundle's build variable (temper/docs/design/cli/README.md, 8).
pub const INTERNAL_BUNDLE: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_BUNDLE",
    set: skein_shell::Set::Internal,
    form: "embed|none",
    way: skein_shell::Way::Reads,
    line: "Select whether the build embeds the browser's bundle.",
    absent: "Embed the browser's bundle.",
    notes: "Read by temper-client-bundle's build script; none skips the nested browser build. Not a runtime override.",
};

include!(concat!(env!("OUT_DIR"), "/bundle.rs"));
