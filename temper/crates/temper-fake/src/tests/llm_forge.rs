//! Playground scripts use the forge assigned to their group at runtime.
use super::{api, demo, json, llm, llm_document};
use core::{mem::size_of, num::NonZeroU16};
use serde_json::Value;

fn template() -> llm::Document {
    use temper_init::{Content, Directories, Facts, Ports};
    let files = temper_init::files(
        &Facts {
            binary: b"/bin/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: Ports {
                clients: NonZeroU16::new(7100).unwrap(),
                webhooks: NonZeroU16::new(7200).unwrap(),
                forge: NonZeroU16::new(8369).unwrap(),
                owned: NonZeroU16::new(3000).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
            },
        },
        &Directories { config: b"/config", data: b"/data" },
        &[1; 32],
        &[2; 32],
    )
    .unwrap();
    let file = files.files.iter().find(|file| file.name == b"temper.json").unwrap();
    let Content::Bytes { bytes, .. } = &file.content else { panic!("document") };
    demo::read(&serde_json::from_slice(bytes).unwrap()).unwrap().unwrap().llm().clone()
}

#[test]
fn the_templates_csv_chat_delegates_on_forge_n() {
    let document = template().for_forge(5).unwrap();
    let domain = document.domain(0);
    let csv = domain.scripts().iter().find(|script| script.cue.as_ref() == b"Add CSV export to widget").unwrap();
    let api::Line::Call { arguments, .. } = &csv.turns[0].lines[1] else { panic!("delegation") };
    let arguments: Value = serde_json::from_slice(arguments).unwrap();
    assert_eq!(arguments["batch"][0]["spec"]["parameters"][0]["resource"], json!(21_474_836_481_u64));
}

#[test]
fn forge_one_leaves_the_scripts_as_they_are() {
    let document = template();
    assert_eq!(document.clone().for_forge(1).unwrap(), document);
}

fn call(arguments: Value) -> Value {
    json!({"lines":[{"call":"delegate","arguments":arguments}],"finish":"tool_calls","tokens":1})
}

#[test]
fn other_numbers_and_another_forges_tags_are_kept() {
    let mut value = llm_document();
    let before = json!({"resource":4_294_967_297_u64, "other":4_294_967_297_u64,
        "nested":[{"resource":4_294_967_298_u64}, {"resource":8_589_934_593_u64},
            {"resource":"4294967297"}, {"resource":-1}, {"resource":4_294_967_297.0_f64},
            {"resource":18_446_744_073_709_551_615_u64}, {"resource":[4_294_967_297_u64]},
            {"resource":{"resource":8_589_934_591_u64}}], "text":"resource 4294967297"});
    let mut expected = before.clone();
    expected["resource"] = json!(21_474_836_481_u64);
    expected["nested"][0]["resource"] = json!(21_474_836_482_u64);
    expected["nested"][7]["resource"]["resource"] = json!(25_769_803_775_u64);
    for group in ["cued", "asked", "otherwise"] {
        let turns = json!([call(before.clone()), call(json!({"other":4_294_967_297_u64}))]);
        value[group] = if group == "otherwise" { turns } else { json!([{"cue":group,"turns":turns}]) };
    }
    let document = llm::Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    let original = document.domain(0);
    let rewritten = document.for_forge(5).unwrap().domain(0);
    for (before, after) in original.scripts().iter().zip(rewritten.scripts()) {
        let api::Line::Call { arguments, .. } = &after.turns[0].lines[0] else { panic!("call") };
        assert_eq!(arguments.as_ref(), serde_json::to_vec(&expected).unwrap().as_slice());
        assert_eq!(before.turns[1], after.turns[1], "unchanged arguments retain their bytes");
    }
}

#[test]
fn a_rewrite_past_the_joint_bound_is_refused() {
    // Five turns stay below the answer bound, together reaching the script bound.
    for group in ["otherwise", "cued"] {
        let arguments = json!({"resource":4_294_967_297_u64});
        let argument_bytes = serde_json::to_vec(&arguments).unwrap().len();
        let wrappers = size_of::<api::Script>() + 5 * size_of::<api::Turn>() + 6 * size_of::<api::Line>();
        let rules = if group == "cued" { size_of::<Box<[api::Rule]>>() + 1 } else { 0 };
        let fallback = if group == "cued" {
            size_of::<api::Script>() + size_of::<api::Turn>() + size_of::<api::Line>() + b"fallback".len()
        } else {
            0
        };
        let padding = usize::try_from(crate::SCRIPT_BYTES).unwrap()
            - wrappers
            - rules
            - fallback
            - b"delegate".len()
            - argument_bytes;
        let mut value = llm_document();
        let mut turns = vec![call(arguments)];
        let chunk = padding / 5;
        for index in 0..5 {
            let size = if index == 4 { padding - 4 * chunk } else { chunk };
            if index == 0 {
                turns[0]["lines"].as_array_mut().unwrap().push(json!({"text":"x".repeat(size)}));
            } else {
                turns.push(json!({"lines":[{"text":"x".repeat(size)}],"finish":"stop","tokens":1}));
            }
        }
        value[group] = if group == "otherwise" { json!(turns) } else { json!([{"cue":"x","turns":turns}]) };
        let document = llm::Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        document.clone().for_forge(1).expect("unchanged scripts at the joint bound");
        assert_eq!(
            document.for_forge(5).unwrap_err().line().as_ref(),
            b"otherwise: scripts past their joint byte bound\n"
        );
    }
}
