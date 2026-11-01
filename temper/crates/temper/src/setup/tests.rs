//! Machine templates are checked beside the seed's setup entries (contracts, 6).
use super::*;
use serde_json::Value;

fn document() -> Value {
    use core::num::NonZeroU16;
    use temper_init::{Content, Directories, Facts, Ports};
    let answer = temper_init::files(
        &Facts {
            binary: b"/bin/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: Ports {
                clients: NonZeroU16::new(7100).expect("nonzero clients port"),
                webhooks: NonZeroU16::new(7200).expect("nonzero webhooks port"),
                forge: NonZeroU16::new(8369).expect("nonzero fake forge port"),
                owned: NonZeroU16::new(3000).expect("nonzero owned forge port"),
                llm: NonZeroU16::new(8370).expect("nonzero LLM port"),
            },
        },
        &Directories { config: b"/config", data: b"/data" },
        &[1; 32],
        &[2; 32],
    )
    .expect("valid fixture value");
    let file = answer.files.iter().find(|file| file.name == b"temper.json").expect("valid fixture value");
    let Content::Bytes { bytes, .. } = &file.content else { panic!("document") };
    serde_json::from_slice(bytes).expect("valid fixture value")
}
fn template(entry: &Value) -> Value {
    let mut template = entry.clone();
    template.as_object_mut().expect("valid fixture value").remove("forge");
    template.as_object_mut().expect("valid fixture value").remove("start");
    template
}
fn machine_document() -> Value {
    let mut d = document();
    d["application"]["owned"] = template(&d["application"]["setup"][1]);
    d["application"]["demo"]["fake"] = template(&d["application"]["setup"][0]);
    d
}
fn read(d: &Value) -> Result<Deployment, String> {
    check(
        &serde_json::to_vec(d).expect("valid fixture value"),
        &config::Directories::given(b"/config", b"/data", b"/cache").expect("valid fixture value"),
    )
}
#[test]
fn the_setup_check_reads_the_owned_member() {
    let deployment = read(&machine_document()).expect("valid fixture value");
    let mut expected = deployment.checked.setup.entry(3).expect("valid fixture value").clone();
    expected.forge = 7;
    assert_eq!(deployment.owned.expect("valid fixture value").entry(7), expected);
    let demo = deployment.demo.expect("valid fixture value");
    let mut expected = deployment.checked.setup.entry(1).expect("valid fixture value").clone();
    expected.forge = 9;
    assert_eq!(demo.fake().expect("valid fixture value").entry(9), expected);
}
#[test]
fn the_init_template_reads_no_machine_member() {
    for null in [false, true] {
        let mut d = document();
        if null {
            d["application"]["owned"] = Value::Null;
            d["application"]["demo"]["fake"] = Value::Null;
        }
        let deployment = read(&d).expect("valid fixture value");
        assert!(deployment.owned.is_none());
        assert!(deployment.demo.expect("valid fixture value").fake().is_none());
    }
}
#[test]
fn the_machine_listeners_are_unique_among_other_kinds() {
    let source = machine_document();
    for pointer in [
        "/application/clients/listener/socket",
        "/application/webhooks/listener/socket",
        "/application/demo/llm/listen/socket",
        "/application/demo/fake/mode/fake/listen/socket",
    ] {
        let mut d = source.clone();
        d["application"]["owned"]["mode"]["owned"]["listen"]["socket"] =
            d.pointer(pointer).expect("valid fixture value").clone();
        assert_eq!(
            read(&d).expect_err("listener collision"),
            "application.owned.mode.owned.listen: must be unique\n",
            "{pointer}"
        );
    }
    let mut d = source;
    d["application"]["demo"]["fake"]["mode"]["fake"]["listen"]["socket"] =
        d["application"]["setup"][1]["mode"]["owned"]["listen"]["socket"].clone();
    assert_eq!(read(&d).expect_err("listener collision"), "application.demo.fake.mode.fake.listen: must be unique\n");
}
