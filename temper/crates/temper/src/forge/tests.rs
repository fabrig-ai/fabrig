use super::reconcile_settings;
use crate::tests::configuration;
use std::{fs, os::unix::ffi::OsStrExt};

#[test]
fn old_form_reconcile_adopts_no_repositories() {
    let directory = std::env::temp_dir().join(format!("temper-reconcile-settings-{}", std::process::id()));
    fs::create_dir(&directory).expect("settings scratch");
    let directories = jig_protocol_config::Directories::given(
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
    )
    .expect("fixture directories");
    for fake in [true, false] {
        let mut value = configuration::old_form_check_document(&directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        let entry = &mut value["application"]["setup"][0];
        if fake {
            entry["mode"]["fake"]["repositories"] = serde_json::json!([
                {"name":"z-last", "landing":"main"}, {"name":"a-first", "landing":"trunk"},
            ]);
        } else {
            entry["mode"] = serde_json::json!({"managed":{"admin_token":"admin_token"}});
            entry["orgs"] = serde_json::json!(["example"]);
        }
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        let settings = reconcile_settings(&deployment.named, &deployment.checked, 1).expect("settings");
        assert!(settings.given.adopted.is_empty());
        assert_eq!(settings.given.listener, None, "the box makes no hook");
    }
    fs::remove_dir_all(directory).expect("settings scratch removed");
}

#[test]
fn reconcile_refuses_records_only_owned_and_fake_without_writing_files() {
    let directory = std::env::temp_dir().join(format!("temper-records-reconcile-{}", std::process::id()));
    fs::create_dir(&directory).expect("settings scratch");
    let directories = jig_protocol_config::Directories::given(
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
    )
    .expect("fixture directories");
    for owned in [false, true] {
        let mut value = configuration::check_document(&directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        let forge = value["application"]["forges"][0].as_object_mut().expect("forge");
        for name in ["api_token", "user", "webhook_secret", "identity"] {
            forge.remove(name);
        }
        let identity = &mut value["application"]["identities"][0];
        identity.as_object_mut().expect("identity").remove("token");
        identity["forge"] = 1.into();
        if owned {
            value["application"]["setup"][0]["mode"] = serde_json::json!({"owned":{
                "forgejo":"/fixture/forgejo", "runner":null, "listen":{"socket":"127.0.0.1:9"},
                "person":{"login":"ada","email":"ada@example.test"}
            }});
        }
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        assert_eq!(
            reconcile_settings(&deployment.named, &deployment.checked, 1).err(),
            Some("forge 1 starts from its records: `temper serve` takes its handover".into())
        );
        assert!(!directory.join("forge-1").exists(), "refusal creates no directory or credential file");
    }
    fs::remove_dir_all(directory).expect("settings scratch removed");
}

#[test]
fn reconcile_directs_string_managed_to_the_client_before_files() {
    let directory = std::path::Path::new("/fixture/client-managed");
    let directories = jig_protocol_config::Directories::given(
        b"/fixture/client-managed",
        b"/fixture/client-managed",
        b"/fixture/cache",
    )
    .expect("fixture directories");
    for transport in
        [serde_json::json!("plaintext"), serde_json::json!({"tls":{"server_name":"forge.example","trust":null}})]
    {
        let mut value = configuration::check_document(directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        value["application"]["forges"][0]["transport"] = transport;
        value["application"]["setup"][0] = serde_json::json!({
            "forge":1,"mode":"managed","bot":{"login":"temper","email":"temper@example.test"}
        });
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        assert_eq!(
            reconcile_settings(&deployment.named, &deployment.checked, 1).err(),
            Some("forge 1's admin’s token is given from the client: connect it as its admin there".into())
        );
    }
}
