//! Offline B0 driver proofs (temper/docs/design/domain/testing.md, 2.7).
pub mod support;
use skein_io::kernel::Exit;

fn ctl_scratch(case: &str) -> support::Scratch {
    let scratch = support::Scratch::new(case);
    support::configuration(scratch.path(), &support::Forge::Unreached);
    scratch
}

#[test]
#[ignore = "offline B0 driver proof; run temper/scripts/proof-live.sh rung-b0-offline"]
fn rung_b0s_funds_document_applies_through_ctl_and_preserves_the_settings() {
    use serde_json::json;
    use support::ctl::{Call, Step, Story};
    let scratch = ctl_scratch("rung-b0-funds");
    let file = scratch.path().join("funds.json");
    let seen = support::ctl::run(Story::new(
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["settings", "show", "--json"])),
            Step::Run(Call::new(&["settings", "show", "rules", "--json"])),
            Step::Run(Call::new(&["settings", "show", "policy", "1", "--json"])),
            Step::Then(Box::new(move |calls| {
                let rules = support::ctl::json(&calls[2]);
                let policy = support::ctl::json(&calls[3]);
                let document = support::forge_land::rung::funds_document(&rules, &policy).expect("funds document");
                let bytes = serde_json::to_vec(&document).expect("funds JSON");
                temper_client_tools::jig::settings_file::decode(&bytes, &temper_client_tools::limits::tools())
                    .expect("the driver's document passes ctl's real settings parser");
                support::write_private(&file, &bytes);
                support::ctl::settings_change(&calls[1], &file, "rung-funds")
            })),
            Step::Run(Call::new(&["settings", "show", "rules", "--json"])),
            Step::Run(Call::new(&["settings", "show", "policy", "1", "--json"])),
        ],
    ));
    assert_eq!(seen.calls.len(), 7, "read, apply, read back");
    for call in &seen.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    for (before, after) in [(2, 5), (3, 6)] {
        let old = support::ctl::json(&seen.calls[before]);
        let got = support::ctl::json(&seen.calls[after]);
        assert_eq!(got["revision"].as_u64(), old["revision"].as_u64().map(|n| n + 1));
        let mut expected = old["value"].clone();
        expected["ceiling"]["spend"] = json!(207_552);
        expected["period_spend"] = json!(449_696);
        if before == 3 {
            expected["roles"][0]["authority"]["spend"] = json!(207_552);
            expected["roles"][0]["period_spend"] = json!(415_104);
        }
        assert_eq!(got["value"], expected, "only the intended spend members changed");
    }
    scratch.remove();
}

#[test]
#[ignore = "offline B0 driver proof; run temper/scripts/proof-live.sh rung-b0-offline"]
fn rung_b0s_setup_reaches_account_import_with_the_fake_model_and_no_account() {
    let scratch = support::Scratch::new("rung-b0-offline-setup");
    support::forge_land::rung::offline_setup(scratch.path());
    scratch.remove();
}

#[test]
#[ignore = "offline B0 driver proof; run temper/scripts/proof-live.sh rung-b0-offline"]
fn rung_b0s_walk_passes_offline_on_its_twin() {
    let scratch = support::Scratch::new("rung-b0-offline-walk");
    support::forge_land::rung::offline_walk(scratch.path());
    scratch.remove();
}
