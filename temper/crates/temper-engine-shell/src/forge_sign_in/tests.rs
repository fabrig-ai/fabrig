use super::{EnvironmentEntry, Named};
use crate::configuration_tests::read_scripted;
use crate::test_configuration as configuration;
use temper_service_shell::jig_directories;

#[test]
fn a_public_client_refuses_before_creating_any_file() {
    use crate::forge;
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        path::{Path, PathBuf},
    };
    use temper_config_shell::{
        File, ForgeSignIn, Member, OpenId, OpenIdClient, Shape, SignIn, Test, TestGoogle, Transport,
    };

    let directory = Path::new("/fixture");
    let directories = configuration::directories(directory);
    let inventory = RefCell::new(BTreeMap::<PathBuf, Box<[u8]>>::new());
    let sign_in =
        |directories: &crate::Directories, bytes: &[u8], named: &Named, number, environment: &[EnvironmentEntry]| {
            super::run_with(
                directories,
                bytes,
                named,
                number,
                environment,
                |name, _| {
                    inventory.borrow_mut().insert(name.into(), Box::new([]));
                    Ok(())
                },
                |_| Ok(()),
            )
        };
    let file = File {
        workers: Some("127.0.0.1:7200".parse().expect("workers address")),
        owner: Some("sam@example.com".into()),
        openid: Some(OpenId {
            key: Some("provider.key".into()),
            addresses: Some(vec![1]),
            clients: vec![OpenIdClient {
                id: "forgejo".into(),
                secret: None,
                redirect: "http://127.0.0.1:3000/user/oauth2/temper/callback".into(),
            }],
        }),
        sign_in: Some(SignIn {
            client: "temper-test".into(),
            secret: "google.secret".into(),
            members: vec![Member { address: "sam@example.com".into(), login: "sam".into() }],
            forges: vec![ForgeSignIn { forge: 1, client: "forgejo".into() }],
        }),
        test: Test {
            google: Some(TestGoogle {
                issuer: "http://127.0.0.1:31999".into(),
                authorization: "http://127.0.0.1:31999/auth".into(),
                token: "http://127.0.0.1:31999/token".into(),
                keys: "http://127.0.0.1:31999/keys".into(),
                transport: Transport::Plaintext,
            }),
            ..Test::default()
        },
        ..File::default()
    };
    let text = temper_config_shell::render(&file, Shape::EngineAlone);
    let bytes = read_scripted(&text, Shape::EngineAlone).expect("engine file").document;
    let jig = jig_directories(&directories);
    let named = temper_engine_service::startup::named(&bytes, &jig).expect("named public client");
    let refusal = sign_in(&directories, &bytes, &named, 1, &[]);
    let files = inventory.borrow().len();

    let configuration = directory.join(temper_config_shell::FILE);
    inventory.borrow_mut().insert(configuration.clone(), text);
    let arguments = ["sign-in".into(), "1".into()];
    let result = forge::start_with_inputs(
        "temper",
        Shape::EngineAlone,
        &directories,
        &arguments,
        None,
        &[],
        &forge::Inputs {
            reader: crate::setup::check,
            setup: &|_, _, _, _| {
                temper_forge_setup::setup_file::chosen(
                    temper_forge_setup::machine::BoxKind::Fake,
                    3000,
                    b"sam",
                    false,
                    &jig,
                )
                .map_err(|why| format!("{why:?}"))
            },
            configuration: &|_, shape| read_scripted(&inventory.borrow()[&configuration], shape),
            sign_in: &sign_in,
        },
    );
    let exit = forge::report_exit("temper", &arguments, result, drop, drop);
    let command_files = inventory.borrow().len();
    assert_eq!(refusal, Err("its client has no secret; a box adds its own source".into()));
    assert_eq!(files, 0, "refusal makes no files");
    assert_eq!(exit, 1, "the command uses its refusal exit");
    assert_eq!(command_files, 1, "only the supplied configuration remains");
}

#[test]
fn first_sign_in_uses_the_given_registration_switch() {
    use temper_forge_setup::sign_in::FirstSignIn;
    let environment = [(Box::from(b"TEMPER_INTERNAL_FORGE_NO_REGISTRATION".as_slice()), Box::from(b"1".as_slice()))];
    assert_eq!(super::first_sign_in(&environment), FirstSignIn::LinksOnly);
    assert_eq!(super::first_sign_in(&[]), FirstSignIn::MakesAccount);
    let empty = [(Box::from(b"TEMPER_INTERNAL_FORGE_NO_REGISTRATION".as_slice()), Box::from(b"".as_slice()))];
    assert_eq!(super::first_sign_in(&empty), FirstSignIn::MakesAccount);
}
