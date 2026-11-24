use super::super::{Credential, CredentialFiles, make_missing_from_with, run_with};
use crate::forge::{
    command_tests::{Effects, Invocation, Output, login, owned_directory},
    test_fixtures::{APP_INI, Effect, Script, setup_bytes},
};
use skein_lib::Secret;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io::{self, Write as _},
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::ExitStatus,
};
use temper_config_shell::{File, ForgeSignIn, Member, OpenId, OpenIdClient, Shape, SignIn, TestGoogle, Transport};
use temper_forge_setup::{
    machine::BoxKind,
    sign_in::{self, FirstSignIn},
};
use temper_forge_setup_shell::{prepare, process};
use temper_shell::directories::EnvironmentEntry;

const KEY: &str = "/fixture/provider.key";
const SECRET: &str = "/fixture/forgejo.secret";
const KEY_BYTES: &[u8] = b"scripted signing key";
const LOCAL_DISCOVERY: &[u8] = b"http://127.0.0.1:31000/.well-known/openid-configuration";
const PUBLIC_DISCOVERY: &[u8] = b"https://temper.example.com/.well-known/openid-configuration";
const SWITCH: &[u8] = b"TEMPER_INTERNAL_FORGE_NO_REGISTRATION";
const EXTERNAL_ONLY: &[u8] = b"ALLOW_ONLY_EXTERNAL_REGISTRATION";

pub(super) fn configuration(public: bool) -> File {
    let forge = if public { "https://git.example.com" } else { "http://127.0.0.1:3000" };
    let mut file = File {
        workers: Some(([127, 0, 0, 1], 7200).into()),
        listen: Some(([127, 0, 0, 1], 31000).into()),
        public: public.then(|| "https://temper.example.com".into()),
        owner: Some("sam@example.com".into()),
        openid: Some(OpenId {
            key: Some("provider.key".into()),
            addresses: Some(vec![1]),
            clients: vec![OpenIdClient {
                id: "forgejo".into(),
                secret: Some("forgejo.secret".into()),
                redirect: format!("{forge}/user/oauth2/temper/callback"),
            }],
        }),
        sign_in: Some(SignIn {
            client: "temper-test".into(),
            secret: "google.secret".into(),
            members: vec![Member { address: "sam@example.com".into(), login: "sam".into() }],
            forges: vec![ForgeSignIn { forge: 1, client: "forgejo".into() }],
        }),
        ..File::default()
    };
    file.test.google = Some(TestGoogle {
        issuer: "http://127.0.0.1:31999".into(),
        authorization: "http://127.0.0.1:31999/auth".into(),
        token: "http://127.0.0.1:31999/token".into(),
        keys: "http://127.0.0.1:31999/keys".into(),
        transport: Transport::Plaintext,
    });
    file
}

#[derive(Debug, PartialEq, Eq)]
enum CredentialEffect {
    Stat(PathBuf),
    Mkdir(PathBuf, u32),
    Draw,
    Open(PathBuf, u32),
    OpenSsl(u8),
    Write(PathBuf),
    Publish(PathBuf, PathBuf),
    Remove(PathBuf),
    Print,
}

#[derive(Default)]
struct Credentials {
    files: RefCell<BTreeMap<PathBuf, (u32, Vec<u8>)>>,
    effects: RefCell<Vec<CredentialEffect>>,
    draws: Cell<u8>,
    openssl: u8,
}

struct Partial<'a> {
    path: PathBuf,
    host: &'a Credentials,
}

impl io::Write for Partial<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.host.effects.borrow_mut().push(CredentialEffect::Write(self.path.clone()));
        let mut files = self.host.files.borrow_mut();
        let (mode, contents) = files.get_mut(&self.path).expect("partial exists before writing");
        assert_eq!(*mode, 0o600);
        contents.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Credentials {
    fn make_missing(&self, path: &Path, key: bool) -> Result<(), String> {
        use CredentialEffect::{Draw, Mkdir, Open, OpenSsl, Publish, Remove, Stat};
        make_missing_from_with(
            path,
            if key { Credential::Key } else { Credential::Secret },
            &CredentialFiles {
                stat: &|name| {
                    self.effects.borrow_mut().push(Stat(name.into()));
                    self.files.borrow().contains_key(name).then_some(()).ok_or(io::ErrorKind::NotFound.into())
                },
                mkdir: &|name, mode| {
                    self.effects.borrow_mut().push(Mkdir(name.into(), mode));
                    Ok(())
                },
                seed: &|| {
                    self.effects.borrow_mut().push(Draw);
                    let next = self.draws.get().checked_add(1).expect("fixture draws");
                    self.draws.set(next);
                    Ok(Secret::new(Box::new([next; 32])))
                },
                open: &|name, mode| {
                    self.effects.borrow_mut().push(Open(name.into(), mode));
                    assert!(name.extension().is_some_and(|ext| ext == "partial"));
                    assert_eq!(self.files.borrow_mut().insert(name.into(), (mode, Vec::new())), None);
                    Ok(Partial { path: name.into(), host: self })
                },
                key: &|mut file: Partial<'_>| {
                    process::sign_in_key_with(|| {
                        assert_eq!(self.files.borrow().get(&file.path), Some(&(0o600, Vec::new())));
                        self.effects.borrow_mut().push(OpenSsl(self.openssl));
                        if self.openssl == 0 {
                            file.write_all(KEY_BYTES)?;
                        }
                        Ok(ExitStatus::from_raw(i32::from(self.openssl) << 8))
                    })
                },
                publish: &|from, to| {
                    self.effects.borrow_mut().push(Publish(from.into(), to.into()));
                    let mut files = self.files.borrow_mut();
                    let file = files.remove(from).expect("publish a complete partial");
                    assert_ne!(file.1.as_slice(), b"");
                    assert_eq!(files.insert(to.into(), file), None);
                    Ok(())
                },
                remove: &|name| {
                    self.effects.borrow_mut().push(Remove(name.into()));
                    assert!(self.files.borrow_mut().remove(name).is_some());
                    Ok(())
                },
            },
        )
    }

    fn sign_in(&self, command: &Invocation<'_>, environment: &[EnvironmentEntry]) -> Output {
        let stdout = RefCell::new(Vec::new());
        command.call(
            "sign-in",
            environment,
            None,
            &Effects {
                sign_in: &|dirs, bytes, named, number, environment| {
                    run_with(
                        dirs,
                        bytes,
                        named,
                        number,
                        environment,
                        |path, key| self.make_missing(path, key),
                        |instructions| {
                            self.effects.borrow_mut().push(CredentialEffect::Print);
                            stdout.borrow_mut().extend_from_slice(instructions);
                            Ok(())
                        },
                    )
                },
                ..Effects::recording(&stdout)
            },
        )
    }
}

fn partial(file: &str, draw: u8) -> PathBuf {
    let suffix = sign_in::secret(&[draw; 32]);
    format!("{file}.{}.partial", std::str::from_utf8(suffix.expose()).expect("fixture suffix")).into()
}

fn instructions(discovery: &[u8], policy: FirstSignIn) -> Vec<u8> {
    sign_in::instructions(b"forgejo", None, b"forgejo", SECRET.as_bytes(), discovery, policy).into_vec()
}

#[test]
fn sign_in_refusal_rows() {
    use CredentialEffect::{Draw, Mkdir, Open, OpenSsl, Remove, Stat};
    let setup = setup_bytes(false, false);
    for (row, reason) in [
        ("openssl", "openssl genpkey failed; openssl must be on PATH"),
        ("unnamed", "it is not named by application.sign_in.forges"),
        ("no-provider", "temper.toml:20: sign_in.forges[0].client: must agree with engine.clients.provider"),
    ] {
        let mut file = configuration(false);
        match row {
            "unnamed" => file.sign_in.as_mut().expect("named forges").forges.clear(),
            "no-provider" => file.openid = None,
            "openssl" => {}
            _ => unreachable!(),
        }
        let config = temper_config_shell::render(&file, Shape::EngineAlone);
        let command = Invocation { number: 1, group: Some(BoxKind::Owned), setup: &setup, configuration: &config };
        let host = Credentials { openssl: 127, ..Credentials::default() };
        assert_eq!(
            host.sign_in(&command, &[]),
            Output { exit: 1, stdout: Vec::new(), stderr: format!("temper-box forge sign-in: forge 1: {reason}\n") }
        );
        assert!(host.files.into_inner().is_empty(), "neither credential nor its partial was left");
        let effects = if row == "openssl" {
            let part = partial(KEY, 1);
            vec![
                Stat(KEY.into()),
                Mkdir("/fixture".into(), 0o700),
                Draw,
                Open(part.clone(), 0o600),
                OpenSsl(127),
                Remove(part),
            ]
        } else {
            vec![]
        };
        assert_eq!(host.effects.into_inner(), effects);
    }
}

#[test]
fn sign_in_print_rows() {
    use CredentialEffect::{Draw, Mkdir, Open, OpenSsl, Print, Publish, Stat, Write};
    for public in [false, true] {
        let config = temper_config_shell::render(&configuration(public), Shape::EngineAlone);
        let setup = setup_bytes(false, false);
        let mut setup: serde_json::Value = serde_json::from_slice(&setup).expect("fixture setup");
        if public {
            setup["public"] = "https://git.example.com".into();
        }
        let setup = serde_json::to_vec(&setup).expect("setup bytes");
        let command = Invocation { number: 1, group: Some(BoxKind::Owned), setup: &setup, configuration: &config };
        let host = Credentials::default();
        let key_part = partial(KEY, 1);
        let secret_part = partial(SECRET, 2);
        let expected = Output {
            stdout: instructions(if public { PUBLIC_DISCOVERY } else { LOCAL_DISCOVERY }, FirstSignIn::MakesAccount),
            ..Output::silent()
        };
        let output = host.sign_in(&command, &[]);
        assert_eq!(output, expected);
        assert_eq!(
            host.effects.take(),
            [
                Stat(KEY.into()),
                Mkdir("/fixture".into(), 0o700),
                Draw,
                Open(key_part.clone(), 0o600),
                OpenSsl(0),
                Write(key_part.clone()),
                Publish(key_part, KEY.into()),
                Stat(SECRET.into()),
                Mkdir("/fixture".into(), 0o700),
                Draw,
                Open(secret_part.clone(), 0o600),
                Draw,
                Write(secret_part.clone()),
                Publish(secret_part, SECRET.into()),
                Print,
            ]
        );
        let files = host.files.borrow().clone();
        assert_eq!(files.len(), 2, "only the published files");
        assert_eq!(files[Path::new(KEY)], (0o600, KEY_BYTES.to_vec()));
        let (mode, secret) = &files[Path::new(SECRET)];
        assert_eq!(*mode, 0o600);
        assert_eq!(secret.len(), 43);
        assert!(secret.iter().all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(byte)));
        for bytes in [&output.stdout[..], output.stderr.as_bytes()] {
            assert!(!bytes.windows(secret.len()).any(|part| part == secret));
        }
        assert_eq!(host.sign_in(&command, &[]), output);
        assert_eq!(host.effects.take(), [Stat(KEY.into()), Stat(SECRET.into()), Print]);
        assert_eq!(host.files.into_inner(), files, "repeat keeps the same secret and key");
    }
}

fn prepare_ini(command: &Invocation<'_>, environment: &[EnvironmentEntry], host: &mut Script) -> Vec<u8> {
    let host = RefCell::new(host);
    let output = command.call(
        "prepare",
        environment,
        None,
        &Effects {
            directory: &owned_directory,
            login: &|user| Ok(login(user)),
            prepare: &|_, place, machine| prepare::run_with(&mut **host.borrow_mut(), place, machine),
            ..Effects::recording(&RefCell::new(Vec::new()))
        },
    );
    assert_eq!(output, Output::silent());
    let host = host.into_inner();
    assert!(host.source_calls.is_empty() && host.notes.is_empty());
    assert!(matches!(host.effects.first(), Some(Effect::Lock(_))));
    host.effects.clear();
    host.files[Path::new(APP_INI)].clone()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|part| part == needle)
}

#[test]
fn registration_switch_rows() {
    let mut file = configuration(false);
    let config = temper_config_shell::render(&file, Shape::EngineAlone);
    let setup = setup_bytes(false, false);
    let named = Invocation { number: 1, group: Some(BoxKind::Owned), setup: &setup, configuration: &config };
    let closed = [(SWITCH.into(), b"1".as_slice().into())];
    let empty = [(SWITCH.into(), b"".as_slice().into())];
    let mut host = Script::default();
    let initial = prepare_ini(&named, &[], &mut host);
    let external_section = b"[service]\nDISABLE_REGISTRATION = true\nALLOW_ONLY_EXTERNAL_REGISTRATION = true\n[oauth2_client]\nENABLE_AUTO_REGISTRATION = true\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[mailer]";
    assert!(contains(&initial, external_section));
    let links_section = b"[service]\nDISABLE_REGISTRATION = true\n[oauth2_client]\nENABLE_AUTO_REGISTRATION = false\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[mailer]";
    let links_only = prepare_ini(&named, &closed, &mut host);
    assert!(contains(&links_only, links_section));
    assert!(!contains(&links_only, EXTERNAL_ONLY));
    assert_eq!(prepare_ini(&named, &closed, &mut host), links_only);
    assert_eq!(prepare_ini(&named, &empty, &mut host), initial);

    let credentials = Credentials::default();
    let printed = credentials.sign_in(&named, &closed);
    assert_eq!(printed, Output { stdout: instructions(LOCAL_DISCOVERY, FirstSignIn::LinksOnly), ..Output::silent() });
    assert!(contains(&printed.stdout, sign_in::LINKS_ONLY_SETTINGS));
    assert!(!contains(&printed.stdout, EXTERNAL_ONLY));
    assert_eq!(credentials.effects.borrow().last(), Some(&CredentialEffect::Print));
    assert_eq!(credentials.files.borrow().len(), 2);

    file.sign_in.as_mut().expect("sign-in").forges.clear();
    let config = temper_config_shell::render(&file, Shape::EngineAlone);
    let unnamed = Invocation { configuration: &config, ..named };
    let initial = prepare_ini(&unnamed, &[], &mut host);
    assert_eq!(prepare_ini(&unnamed, &closed, &mut host), initial);
    assert!(!contains(&initial, b"[oauth2_client]"));
}
