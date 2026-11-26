use super::*;
use crate::directories::jig_directories;
use crate::directories::tests::scratch;
use std::fs;

#[test]
fn toml_fake_check_refuses_the_former_file_alone_or_beside_the_current_file() {
    for present in [false, true] {
        let scratch = skein_scratch::Scratch::new("fake-check-both-files");
        let directories = crate::tests::configuration::directories(scratch.path());
        std::fs::write(scratch.path().join(temper_config_shell::FILE), "listen = \"127.0.0.1:7100\"\n")
            .expect("box file");
        std::fs::create_dir(scratch.path().join(temper_config_shell::FORMER)).expect("former directory");
        if !present {
            std::fs::remove_file(scratch.path().join(temper_config_shell::FILE)).expect("current absent");
        }
        let check = super::super::Check {
            forge: 1,
            observations: scratch.path().join("absent-observations"),
            store: scratch.path().join("absent-store"),
        };
        let actual = super::outcome(&directories, &check);
        assert_eq!(actual.status, 1);
        assert!(actual.stdout.is_empty());
        assert_eq!(actual.stderr.as_ref(), format!(
        "temper-box fake forge check: {}: moved: the configuration is {}; write it from temper-box help config, then remove {}\n",
        temper_config_shell::FORMER, temper_config_shell::FILE, temper_config_shell::FORMER
    ).as_bytes());
    }
}

fn store_limits() -> jig_protocol_store::Limits {
    let mut limits = jig_protocol_config_fixtures::small().store;
    // Skein's real KV fixture, with chunk >= key + value + 20, and room
    // for the format and the eight rows of the tag-filter case.
    limits.store = skein_kv::Limits {
        key: 64,
        value: 64,
        ops: 8,
        commit: 1024,
        queued: 8,
        queued_bytes: 8192,
        budget: 8192,
        segment: 4096,
        snapshot_after: 4096,
        chunk: 256,
        page: skein_kv::Page { rows: 16, bytes: 1024 },
        record: 1,
        log: 1 << 40,
        logs: 16,
        deadline: skein_lib::Duration::from_secs(5),
    };
    limits
}
fn rows(keys: &[u8], tag: Option<u8>) -> Rows {
    keys.iter()
        .map(|key| {
            let key = match tag {
                Some(tag) => Box::from([tag, *key]),
                None => Box::from([*key]),
            };
            (key, Box::from(b"value".as_slice()))
        })
        .collect()
}
fn seed(path: &Path, limits: &jig_protocol_store::Limits, format: Option<Format>, rows: Rows) {
    fs::create_dir(path).expect("store directory");
    let mut rows = rows.into_vec();
    if let Some(format) = format {
        rows.push((
            keys::encode_own(&keys::Own::Format, limits.store.key).expect("format key"),
            format.encode().expect("format"),
        ));
    }
    let mut host = StoreHost::new(limits.store);
    host.seed = Some(rows.into_boxed_slice());
    run_store(path, host).expect("seed through the real store");
}
enum Copy {
    Store { format: Option<Format>, rows: Rows },
    Absent,
    File,
}
#[test]
fn store() {
    // The row-count page bound forces three pages for five temper rows.
    const SMALL_PAGE: u32 = 2;
    let root = scratch("fake-check-store");
    let current = format::current(temper_protocol_store::STORE_VERSION, [0; 16]);
    let mut application = current;
    application.application += 1;
    let mut component = current;
    component.component += 1;
    let temper = &[9, 10, 12];
    let many = &[9, 10, 11, 12, 16];
    let mixed = [(0, 9), (1, 10), (2, 9), (2, 10), (2, 12), (3, 12)]
        .into_iter()
        .map(|(tag, key)| (Box::from([tag, key]), Box::from(b"value".as_slice())))
        .collect();
    let limits = store_limits();
    let page_rows = limits.store.page.rows;
    for (name, copy, page, expected) in [
        (
            "good",
            Copy::Store { format: Some(current), rows: rows(temper, Some(keys::APPLICATION)) },
            page_rows,
            Ok(rows(temper, None)),
        ),
        ("absent", Copy::Absent, page_rows, Err(format!("opening {}: errno 2", root.join("absent").display()))),
        ("file", Copy::File, page_rows, Err(format!("opening {}: errno 20", root.join("file").display()))),
        ("empty", Copy::Store { format: None, rows: Box::new([]) }, page_rows, Err("format: Got None".to_owned())),
        (
            "application",
            Copy::Store { format: Some(application), rows: Box::new([]) },
            page_rows,
            Err(format!(
                "format check: Format {{ family: Application, found: {}, wanted: {} }}",
                temper_protocol_store::STORE_VERSION.checked_add(1).expect("application version increment fits u32"),
                temper_protocol_store::STORE_VERSION,
            )),
        ),
        (
            "component",
            Copy::Store { format: Some(component), rows: Box::new([]) },
            page_rows,
            Err(format!(
                "format check: Format {{ family: Component, found: {}, wanted: {} }}",
                component.component, current.component,
            )),
        ),
        ("tags", Copy::Store { format: Some(current), rows: mixed }, page_rows, Ok(rows(temper, None))),
        (
            "pages",
            Copy::Store { format: Some(current), rows: rows(many, Some(keys::APPLICATION)) },
            SMALL_PAGE,
            Ok(rows(many, None)),
        ),
    ] {
        let path = root.join(name);
        let mut limits = limits;
        limits.store.page.rows = page;
        match copy {
            Copy::Store { format, rows } => seed(&path, &limits, format, rows),
            Copy::Absent => {}
            Copy::File => fs::write(&path, b"file").expect("file"),
        }
        assert_eq!(read_store(&path, &limits), expected, "{name}");
    }
    fs::remove_dir_all(root).expect("scratch removed");
}

const START: &[u8] = b"{\"observations_lost\":0}\n";
const LOST: &[u8] = b"{\"observations_lost\":7}\n";
const DELETED: &[u8] =
    b"{\"observation\":\"deleted\",\"repository\":\"org/repo\",\"branch\":\"work\",\"at\":7,\"by\":2}\n";
const MISSING: &str = "observations: no checkpoint";
const REFUSED: &str = "line 1: decode: Refusal { variant: None, member: [111, 98, 115, 101, 114, 118, 97, 116, 105, 111, 110], reason: Json }";
fn encoded(records: Records<File>) -> Result<Vec<Vec<u8>>, String> {
    records
        .map(|record| {
            let mut bytes = skein_lib::List::with_capacity(u32::try_from(DELETED.len()).expect("fixture line length"));
            skein_fake_forge_forgejo::observations::encode(&record?, &mut bytes).expect("fixture record fits");
            Ok(bytes.as_slice().to_vec())
        })
        .collect()
}
// A fixed 64 KiB write buffer, independent of the observation line bound.
const OBSERVATION_CHUNK: usize = 64 * 1024;
fn write_overlong(path: &Path, before: &[u8], after: &[u8]) {
    use std::io::Write;
    let mut file = fs::File::create(path).expect("overlong observations fixture");
    file.write_all(before).expect("before overlong line");
    let chunk = vec![b'x'; OBSERVATION_CHUNK];
    let bound = skein_lib::usize_of(temper_fake::limits::OBSERVATION_BYTES);
    for _ in 0..bound / OBSERVATION_CHUNK {
        file.write_all(&chunk).expect("overlong line chunk");
    }
    file.write_all(&chunk[..bound % OBSERVATION_CHUNK]).expect("overlong line remainder");
    // Exactly one byte beyond the line bound (including LF).
    file.write_all(b"\n").expect("overlong line LF");
    file.write_all(after).expect("after overlong line");
}

#[test]
fn observations() {
    let root = scratch("fake-check-observations");
    write_overlong(&root.join("overlong-prefix"), b"", START);
    write_overlong(&root.join("overlong-tail"), START, b"");
    let bound = skein_lib::usize_of(temper_fake::limits::OBSERVATION_BYTES);
    let overlong_error = format!("line 1: line past {bound} bytes");
    let missing_file =
        format!("observations {}: {}", root.join("absent").display(), std::io::Error::from_raw_os_error(2));
    for (name, bytes, expected) in [
        ("start", Some(START.to_vec()), Ok((1, 0, vec![START.to_vec()]))),
        (
            "github",
            Some(
                [
                    START,
                    b"{\"observation\":\"git_call\",\"installation\":1,\"mint\":2,\"writes\":true,\"status\":200}\n",
                    START,
                ]
                .concat(),
            ),
            Ok((3, 0, vec![START.to_vec(), START.to_vec()])),
        ),
        (
            "github-full",
            Some([START, b"{\"observation\":\"git_calls_full\",\"capacity\":128}\n"].concat()),
            Err(GIT_FULL),
        ),
        ("partial", Some([DELETED, START, b"{partial"].concat()), Ok((2, 0, vec![DELETED.to_vec(), START.to_vec()]))),
        ("refused-tail", Some([DELETED, START, b"{\n"].concat()), Ok((2, 0, vec![DELETED.to_vec(), START.to_vec()]))),
        ("refused-prefix", Some([b"{\n", START].concat()), Err(REFUSED)),
        ("no-checkpoint", Some(DELETED.to_vec()), Err(MISSING)),
        ("empty", Some(vec![]), Err(MISSING)),
        ("absent", None, Err(missing_file.as_str())),
        ("overlong-prefix", None, Err(overlong_error.as_str())),
        ("overlong-tail", None, Ok((1, 0, vec![START.to_vec()]))),
        (
            "checkpoints",
            Some([START, DELETED, LOST, b"partial"].concat()),
            Ok((3, 7, vec![START.to_vec(), DELETED.to_vec(), LOST.to_vec()])),
        ),
        (
            "checkpoint-tail",
            Some([START, DELETED, b"{\"observations_lost\":7}"].concat()),
            Ok((1, 0, vec![START.to_vec()])),
        ),
    ] {
        let path = root.join(name);
        if let Some(bytes) = bytes {
            fs::write(&path, bytes).expect("observations fixture");
        }
        let found = Observations::open(&path).and_then(|observations| {
            Ok((observations.lines(), observations.lost(), encoded(observations.records()?)?))
        });
        assert_eq!(found.as_ref().map_err(String::as_str), expected.as_ref().map_err(|why| *why), "{name}");
    }
    // The second pass detects a changed prefix, without reading beyond its
    // saved boundary. Appends are the running fake's ordinary case.
    for (name, replacement, expected) in [
        (
            "shortened",
            b"{\"observations_lost\":".as_slice(),
            Err("observations: file shorter than checkpoint boundary"),
        ),
        ("changed", b"xxxxxxxxxxxxxxxxxxxxxxx\n", Err("decode:")),
        ("grown", [START, b"ignored\n"].concat().as_slice(), Ok(vec![START.to_vec()])),
    ] {
        let path = root.join(name);
        fs::write(&path, START).expect("initial prefix");
        let observations = Observations::open(&path).expect("boundary");
        fs::write(&path, replacement).expect("changed file");
        match expected {
            Ok(expected) => assert_eq!(encoded(observations.records().expect(name)).expect(name), expected, "{name}"),
            Err(expected) => {
                assert!(encoded(observations.records().expect(name)).expect_err(name).starts_with(expected), "{name}");
            }
        }
    }
    fs::remove_dir_all(root).expect("scratch removed");
}

use skein_forge::api::{Repository, Write as ForgeWrite};
use skein_forge_client::{Condition, Effect, Entry, Writer};
use temper_engine_domain_forge as d;
use temper_protocol_store::{keys as temper_keys, records};

const REPO: Repository = Repository { forge: 1, repository: 2 };
fn repository() -> d::Stored {
    d::Stored::Repository(d::Repository {
        contexts: Box::new([]),
        landing: b"main".as_slice().into(),
        refusals: Box::default(),
        workflows: Box::default(),
        project: 1,
        home: true,
        provider: REPO,
        host: b"forge".as_slice().into(),
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: d::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: d::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: false,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: d::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: true,
            rebase: true,
        },
    })
}
fn entry() -> d::Stored {
    d::Stored::Entry {
        entry: Entry {
            number: 1,
            task: 4,
            repository: REPO,
            effect: Effect {
                write: ForgeWrite::Merge {
                    number: 1,
                    head: skein_fake_forge_forgejo::routes::object(2).bytes,
                    how: skein_forge::api::MergeHow::Style,
                },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
        held: false,
    }
}
fn deployment(merge: bool) -> Rows {
    let mut rows = vec![d::Stored::Writer(Writer { forge: 1, author: 7 }), repository()];
    if merge {
        rows.push(entry());
    }
    stored_rows(&rows)
}
fn stored_rows(rows: &[d::Stored]) -> Rows {
    rows.iter()
        .map(|row| {
            let key = temper_keys::encode(&d::stored_key(row), u32::MAX).expect("fixture key");
            ([&[keys::APPLICATION], key.as_ref()].concat().into_boxed_slice(), records::encode(row))
        })
        .collect()
}
const BOUNDARY: &[u8] = b"{\"boundary\":{\"lines\":1,\"lost\":0}}\n";
const TWO_LINES: &[u8] = b"{\"boundary\":{\"lines\":2,\"lost\":0}}\n";
const PROFILE: &[u8] = b"{\"profile\":{\"client\":7,\"marker\":[\"<!-- temper:key \",\" -->\\n\"],\"owned_branches\":[\"temper/\"],\"allowed\":[\"comment\",\"merge\",\"close\"],\"bounds\":{\"objects\":256,\"decisions\":0}},\"decisions_complete\":true}\n";
const DECIDED_PROFILE: &[u8] = b"{\"profile\":{\"client\":7,\"marker\":[\"<!-- temper:key \",\" -->\\n\"],\"owned_branches\":[\"temper/\"],\"allowed\":[\"comment\",\"merge\",\"close\"],\"bounds\":{\"objects\":256,\"decisions\":1}},\"decisions_complete\":true}\n";
const DECISION: &[u8] = b"{\"decision\":{\"repository\":\"playground/widget\",\"number\":1,\"head\":2}}\n";
const MERGE: &[u8] = b"{\"observation\":\"merged\",\"repository\":\"playground/widget\",\"number\":1,\"base\":\"main\",\"head\":2,\"commit\":3,\"by\":7}\n";
const ORDER: &[u8] =
    b"{\"fail\":\"promise/order/playground%2Fwidget/1/2\",\"summary\":\"a merge with no decision at its head\"}\n";
fn expected(status: u8, stdout: &[u8], stderr: &[u8]) -> Outcome {
    Outcome { status, stdout: stdout.into(), stderr: stderr.into() }
}
fn diagnostic(why: &str) -> Vec<u8> {
    format!("temper-box fake forge check: {why}\n").into_bytes()
}
fn refused(why: &str) -> Outcome {
    expected(1, b"", &diagnostic(why))
}
fn undecided(word: &[u8], stderr: &[u8]) -> Outcome {
    expected(2, &[b"{\"undecided\":\"", word, b"\"}\n"].concat(), stderr)
}
fn missing_observations(why: &str) -> Outcome {
    undecided(b"observations missing", &diagnostic(why))
}
fn unreadable_store(path: &Path, errno: u8) -> Outcome {
    undecided(
        b"store unreadable",
        &[BOUNDARY, &diagnostic(&format!("opening {}: errno {errno}", path.display()))].concat(),
    )
}
#[test]
fn check_run() {
    use crate::serve::configured_agent_with;
    use std::{cell::RefCell, collections::BTreeMap, io::Cursor};
    use temper_config_shell::{Facts, Shape};
    use temper_service::stock;

    let root = Path::new("/fixture");
    let directories = crate::directories::tests::given(root);
    let data = crate::path(&directories.data);
    let config = crate::path(&directories.config).join(temper_config_shell::FILE);
    let agents = data.join("agents.setting");
    let local = data.join("local.setting");
    let check = Check { forge: 1, observations: root.join("observations"), store: root.join("copy") };
    let copy = temper_agent_fixtures::Setting { figures: stock::agents(), providers: Box::new([]) }.bytes();
    let inventory = BTreeMap::from([
        (config.clone(), temper_config_shell::render(&crate::tests::configuration::file(), Shape::Standalone)),
        (agents.clone(), copy),
        (check.observations.clone(), START.into()),
    ]);
    let before = inventory.clone();
    let reads = RefCell::new(Vec::new());
    let mut log = Vec::new();
    let actual = outcome_with(
        &check,
        || {
            let read = temper_config_shell::read_with(
                Shape::Standalone,
                || {
                    reads.borrow_mut().push(crate::path(&directories.config).join(temper_config_shell::FORMER));
                    Err(std::io::Error::from_raw_os_error(2))
                },
                |bound| {
                    reads.borrow_mut().push(config.clone());
                    assert!(skein_lib::len_u64(inventory[&config].len()) <= u64::from(bound));
                    Ok(inventory[&config].clone())
                },
                || Ok(Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false }),
            )
            .map_err(|why| why.to_string())?;
            configuration_with(
                &directories,
                &read.into(),
                |path| panic!("no configured executable: {}", path.display()),
                |bytes, _| {
                    configured_agent_with(
                        bytes,
                        &directories,
                        &[],
                        &mut log,
                        |path, bound| {
                            reads.borrow_mut().push(path.to_path_buf());
                            let copy = inventory.get(path).cloned();
                            assert!(copy.as_ref().is_none_or(|bytes| skein_lib::len_u64(bytes.len()) <= bound));
                            Ok(copy)
                        },
                        || {
                            Ok(stock::Given::Stock
                                .with_providers(Box::new([]), Box::new([stock::scripted(crate::scripted::PREFERRED)])))
                        },
                        |part| {
                            let bytes = crate::agent::figures::configuration(
                                part.figures.as_ref().expect("box figures"),
                                None,
                                data,
                                &part.environment,
                            )?;
                            crate::agent::prepare_bytes_with(bytes, &mut std::io::sink(), |_| Ok(()))
                        },
                    )
                },
            )
        },
        |path| {
            reads.borrow_mut().push(path.to_path_buf());
            Ok(Cursor::new(inventory[path].as_ref()))
        },
        |path, limits| {
            run_store_with(path, StoreHost::new(limits.store), |root| {
                reads.borrow_mut().push(root.to_path_buf());
                Err(2)
            })
        },
    );
    assert_eq!(actual, unreadable_store(&check.store, 2));
    assert!(log.is_empty(), "setting reads emit no fallback warnings: {log:?}");
    assert_eq!(
        reads.into_inner(),
        [
            crate::path(&directories.config).join(temper_config_shell::FORMER),
            config,
            agents,
            local,
            check.observations,
            check.store,
        ]
    );
    assert_eq!(inventory, before, "the check creates no data or cache entries and keeps agents.setting");
}

fn agent_metadata_fixture(root: &Path) -> temper_service::settings::Agent {
    let (agent, _) = crate::tests::configuration::documents(root, "/fixture/temper");
    fs::write(root.join("root.der"), skein_tls_kit::pki::ROOT).expect("fixture trust");
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
    for endpoint in agent["endpoints"].as_array_mut().expect("fixture endpoints") {
        endpoint.as_object_mut().expect("fixture endpoint").remove("program");
    }
    let bytes = serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice();
    crate::agent::prepare_bytes(bytes).expect("agent metadata").agent
}
#[test]
#[expect(clippy::too_many_lines, reason = "the verb's outcome matrix shares its store and file setup")]
fn verb() {
    let root = scratch("fake-check-verb");
    let agent = agent_metadata_fixture(&root);
    let document = crate::tests::configuration::check_document(&root);
    let current = Some(format::current(temper_protocol_store::STORE_VERSION, [0; 16]));
    let good = || Copy::Store { format: current, rows: deployment(false) };
    let doc = || Some(document.clone());
    let checkpoint = || Some(START.to_vec());
    let path = |name: &str, file: &str| root.join(name).join(file);
    for (name, document, observations, copy, expected) in [
        ("pass", doc(), checkpoint(), good(), expected(0, b"", &[BOUNDARY, PROFILE].concat())),
        ("fail", doc(), Some([MERGE, START].concat()), good(), expected(1, ORDER, &[TWO_LINES, PROFILE].concat())),
        (
            "decision",
            doc(),
            Some([MERGE, START].concat()),
            Copy::Store { format: current, rows: deployment(true) },
            expected(0, b"", &[TWO_LINES, DECIDED_PROFILE, DECISION].concat()),
        ),
        (
            "absent-observations",
            doc(),
            None,
            good(),
            missing_observations(&format!(
                "observations {}: {}",
                path("absent-observations", "observations").display(),
                std::io::Error::from_raw_os_error(2)
            )),
        ),
        ("no-checkpoint", doc(), Some(MERGE.to_vec()), good(), missing_observations(MISSING)),
        ("refused-observations", doc(), Some([b"{\n", START].concat()), good(), missing_observations(REFUSED)),
        ("absent-store", doc(), checkpoint(), Copy::Absent, unreadable_store(&path("absent-store", "copy"), 2)),
        ("not-store", doc(), checkpoint(), Copy::File, unreadable_store(&path("not-store", "copy"), 20)),
        (
            "client-unknown",
            doc(),
            checkpoint(),
            Copy::Store { format: current, rows: stored_rows(&[repository()]) },
            undecided(b"client unknown", BOUNDARY),
        ),
        (
            "absent-document",
            None,
            checkpoint(),
            Copy::Absent,
            refused(&format!("{}: absent; temper-box init writes one", temper_config_shell::FILE)),
        ),
        (
            "invalid-document",
            Some(serde_json::json!({})),
            None,
            Copy::Absent,
            refused(&format!(
                "{}:1: listen: a socket address on loopback, such as \"127.0.0.1:7100\"",
                temper_config_shell::FILE
            )),
        ),
    ] {
        let directory = root.join(name);
        fs::create_dir(&directory).expect("row directory");
        let directories = crate::directories::tests::given(&directory);
        let data = crate::path(&directories.data);
        fs::create_dir_all(data).expect("fixture data");
        let agent_value: serde_json::Value =
            serde_json::from_slice(&crate::tests::configuration::documents(&root, "/fixture/temper").0)
                .expect("agent JSON");
        let mut fixture = crate::tests::configuration::check_document(&root);
        temper_agent_fixtures::apply(&mut fixture, data, &agent_value);
        let copy_before = fs::read(data.join("agents.setting")).expect("fixture copy");
        let bytes = document.map(|document| serde_json::to_vec(&document).expect("fixture document"));
        if let Some(bytes) = &bytes {
            let text = if bytes.as_slice() == b"{}" {
                b"listen = 42\n".as_slice().into()
            } else {
                temper_config_shell::render(
                    &crate::tests::configuration::file(),
                    temper_config_shell::Shape::Standalone,
                )
            };
            fs::write(crate::path(&directories.config).join(temper_config_shell::FILE), text)
                .expect("configuration file");
        }
        let check = Check { forge: 1, observations: directory.join("observations"), store: directory.join("copy") };
        if let Some(observations) = observations {
            fs::write(&check.observations, observations).expect("observations file");
        }
        match copy {
            Copy::Store { format, rows } => {
                let limits = temper_service::startup::store_limits(
                    bytes.as_ref().expect("seed document"),
                    &jig_directories(&directories),
                    Some(&agent.declaration()),
                )
                .expect("store limits");
                seed(&check.store, &limits.store, format, rows);
            }
            Copy::Absent => {}
            Copy::File => fs::write(&check.store, b"not a store").expect("non-store"),
        }
        let entries_before = fs::read_dir(data).expect("fixture entries").count();
        assert_eq!(outcome(&directories, &check), expected, "{name}");
        assert_eq!(fs::read(data.join("agents.setting")).expect("copy remains"), copy_before, "{name}: read-only copy");
        assert_eq!(
            fs::read_dir(data).expect("data entries").count(),
            entries_before,
            "{name}: creates no data resources"
        );
        assert!(!directory.join("cache").exists(), "{name}: creates no cache directory");
    }
    fs::remove_dir_all(root).expect("scratch removed");
}

#[test]
fn installation_records_are_validated_and_full_is_terminal_past_a_checkpoint() {
    let root = scratch("fake-check-installations");
    let path = root.join("observations");
    let created = b"{\"observation\":\"installation_created\",\"installation\":1}\n";
    let deleted = b"{\"observation\":\"installation_deleted\",\"installation\":1}\n";
    fs::write(&path, [created.as_slice(), deleted, START].concat()).expect("installation observation fixture");
    let observations = Observations::open(&path).expect("installation observation fixture");
    assert_eq!(
        observations.records().expect("installation observation fixture").count(),
        1,
        "only checkpoint goes to domain judge"
    );
    fs::write(&path, [START, b"{\"observation\":\"installations_full\",\"capacity\":2}\n"].concat())
        .expect("installation observation fixture");
    assert_eq!(Observations::open(&path).err().as_deref(), Some(INSTALLATIONS_FULL));
    fs::write(
        &path,
        [b"{\"observation\":\"installation_created\",\"installation\":1,\"token\":\"leak\"}\n", START].concat(),
    )
    .expect("installation observation fixture");
    assert!(Observations::open(&path).is_err(), "unknown fields cannot be skipped as observations");
    fs::remove_dir_all(root).expect("installation observation fixture");
}
