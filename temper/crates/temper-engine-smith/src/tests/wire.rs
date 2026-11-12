//! The host tools' kept contract (protocol/wires.md, 6).

use std::{fs, path::Path};

use smith_domain_run::HostEffect;

use crate::{
    calls::{Offer, SPECS, Spec},
    wire::{self, render},
};

fn check_current(rendered: &[u8], kept: &[u8]) -> Result<(), &'static str> {
    if rendered == kept {
        Ok(())
    } else {
        Err("the host tools' layout moved: move `wire::VERSION` to N+1 and write its text with `SKEIN_CODEGEN_WRITE=1`")
    }
}

#[test]
fn host_tools_contract_is_current() {
    let rendered = render(&SPECS);
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("contract").join(format!("host-tools-v{}.txt", wire::VERSION));
    if std::env::var_os("SKEIN_CODEGEN_WRITE").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("contract directory")).expect("create contract directory");
        fs::write(&path, &rendered).expect("write current host-tool contract");
    }
    let kept = fs::read(&path).unwrap_or_default();
    if let Err(why) = check_current(&rendered, &kept) {
        panic!("{why}");
    }
    assert_eq!(wire::FAMILY, "temper-host-tools");
    assert_eq!(wire::layout(), skein_lib::sha256::digest(&kept));
}

#[test]
fn schema_changes_require_a_version_but_descriptions_do_not() {
    let kept = render(&SPECS);
    let mut specs = SPECS;
    specs.first_mut().expect("a declaration").description = b"A different description.";
    assert_eq!(check_current(&render(&specs), &kept), Ok(()));
    specs.first_mut().expect("a declaration").schema = br#"{"type":"object","properties":{"new":{"type":"boolean"}}}"#;
    assert_eq!(
        check_current(&render(&specs), &kept),
        Err(
            "the host tools' layout moved: move `wire::VERSION` to N+1 and write its text with `SKEIN_CODEGEN_WRITE=1`"
        )
    );
}

#[test]
fn contract_lines_sort_names_keep_ties_and_escape_schema_bytes() {
    let specs = [
        Spec {
            name: b"z",
            description: b"not in the contract",
            schema: b"{}",
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
        Spec {
            name: b"a",
            description: b"",
            schema: b"{\n\"x\":\"\\\"\t\r\x01\"}",
            effect: HostEffect::Write,
            offer: Offer::Every,
        },
        Spec { name: b"a", description: b"", schema: b"{}", effect: HostEffect::Read, offer: Offer::Every },
        Spec {
            name: b"a",
            description: b"duplicate offering",
            schema: b"{}",
            effect: HostEffect::Read,
            offer: Offer::Every,
        },
    ];
    let expected = concat!(
        "{\"name\":\"a\",\"effect\":\"write\",\"input\":\"{\\n\\\"x\\\":\\\"\\\\\\\"\\t\\r\\u0001\\\"}\"}\n",
        "{\"name\":\"a\",\"effect\":\"read\",\"input\":\"{}\"}\n",
        "{\"name\":\"a\",\"effect\":\"read\",\"input\":\"{}\"}\n",
        "{\"name\":\"z\",\"effect\":\"read\",\"input\":\"{}\"}\n",
    );
    assert_eq!(render(&specs).as_ref(), expected.as_bytes());
}
