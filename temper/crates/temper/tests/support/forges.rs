//! Runtime forge calls for process stories (temper/docs/design/draft/playground-reset.md, 8).
use super::{Ended, ctl::Call};

pub enum FixturePhase {
    Provisioning,
    Story,
}

#[must_use]
pub fn make_here() -> Call {
    Call::new(&["forge", "make", "here"])
}

#[must_use]
pub fn make_github() -> Call {
    Call::new(&["forge", "make", "github"])
}

#[must_use]
pub fn make_forgejo(api: &str, git: &str) -> Call {
    Call::new(&["forge", "make", "forgejo", api, git])
}

#[must_use]
pub fn remove(forge: u16) -> Call {
    Call::new(&["forge", "remove", &forge.to_string()])
}

#[must_use]
pub fn playground_start() -> Call {
    Call::new(&["playground", "start"])
}

#[must_use]
pub fn playground_reset() -> Call {
    Call::new(&["playground", "reset"])
}

/// Read a completed make's text or JSON answer, including a kept answer.
#[must_use]
pub fn made(call: &Ended) -> u16 {
    assert_eq!(call.exit, Some(skein_io::kernel::Exit::Code(0)), "forge make succeeded");
    let forge = if let Ok(document) = serde_json::from_slice::<serde_json::Value>(&call.stdout) {
        let outcome = document.get("outcome").unwrap_or(&document);
        u16::try_from(outcome["made"]["forge"].as_u64().expect("made forge number")).expect("u16 forge")
    } else {
        let text = std::str::from_utf8(&call.stdout).expect("forge make text");
        let text = text.trim().strip_prefix("kept: forge-make ").unwrap_or(text.trim());
        text.strip_prefix("made: forge ").expect("made forge answer").parse().expect("u16 forge")
    };
    assert_ne!(forge, 0, "forge numbers start at one");
    forge
}

/// Secret-free instructions kept beside a process deployment, never in its document.
pub fn record(directory: &std::path::Path, forge: &super::Forge<'_>) {
    let value = match forge {
        super::Forge::Unreached => serde_json::json!({"kind":"forgejo", "api":"127.0.0.1:9",
            "git":"http://127.0.0.1:9", "admin":null}),
        super::Forge::Throwaway { api, git, admin, .. } => serde_json::json!({"kind":"forgejo",
            "api":api.to_string(), "git":git, "admin":admin}),
    };
    super::write_private(&directory.join("fixture-forge.json"), &serde_json::to_vec(&value).expect("fixture forge"));
}

/// Read the runtime manifest unless this deployment explicitly seeds a compatibility forge.
#[must_use]
pub fn fixture(directory: &std::path::Path) -> Option<serde_json::Value> {
    let path = directory.join("fixture-forge.json");
    if !path.exists() {
        return None;
    }
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("temper.json")).expect("deployment document"))
            .expect("deployment JSON");
    if document["application"].get("forges").is_some() {
        return None;
    }
    Some(serde_json::from_slice(&std::fs::read(path).expect("fixture forge")).expect("fixture forge JSON"))
}

#[must_use]
pub fn fixture_steps(forge: &serde_json::Value, sign_in: Call) -> Vec<super::ctl::Step> {
    use super::ctl::Step;
    assert_eq!(forge["kind"], "forgejo");
    let api = forge["api"].as_str().expect("forge API").to_owned();
    let git = forge["git"].as_str().expect("forge git").to_owned();
    let mut steps = vec![
        Step::Run(sign_in),
        Step::Mark("fixture forge"),
        Step::Run(Call::new(&["forge", "list", "--json"])),
        Step::Then(Box::new(move |calls| {
            if super::ctl::json(&calls[1])["forges"].as_array().expect("forge rows").is_empty() {
                make_forgejo(&api, &git)
            } else {
                Call::new(&["forge", "list", "--json"])
            }
        })),
    ];
    let admin = forge["admin"].as_str().map(str::to_owned);
    steps.push(Step::Then(Box::new(move |calls| {
        let listed = super::ctl::json(&calls[1]);
        let rows = listed["forges"].as_array().expect("forge rows");
        let number = if rows.is_empty() {
            made(&calls[2])
        } else {
            u16::try_from(rows[0]["number"].as_u64().expect("forge number")).expect("u16 forge")
        };
        assert_eq!(number, 1, "fresh store's fixture forge");
        if let Some(admin) = admin
            && rows.first().is_none_or(|row| row["state"] != "connected")
        {
            Call::new(&[
                "forge",
                "connect",
                &number.to_string(),
                "forgejo-admin",
                &admin,
                "--key",
                "fixture-forge-connect",
            ])
        } else {
            Call::new(&["forge", "list", "--json"])
        }
    })));
    if !forge["admin"].is_null() {
        steps.push(Step::Until(Call::new(&["forge", "list", "--json"]), b"\"state\":\"connected\""));
    }
    steps.push(Step::FixtureReady);
    steps
}
