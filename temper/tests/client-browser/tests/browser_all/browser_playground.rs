//! The built playground, its frame's figures and a downloaded log replayed natively
//! (temper/docs/design/client/web/testing.md, 6, item 12; temper/docs/design/playground.md, 9).
use skein_fake_person::{Face, Find, Found, Next, Step as PersonStep, Why};
use skein_fake_person_browser::harness::{Names, Output, Printed, Sent};
use skein_fake_person_browser::{BrowserFace, Harness, Outcome, Play, Script, Setup, Step};
use skein_lib::Time;
use skein_ui::Role;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use temper_client_browser_world::{WITHIN, chromium, playground_serve};
use temper_playground::{programs, story};

struct Walk {
    script: Script,
    figures: Vec<String>,
    download: Option<PathBuf>,
    until: Option<Time>,
}

impl Play for Walk {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        let next = self.script.next(face, now);
        if next != Next::Done {
            return next;
        }
        let until = *self.until.get_or_insert(now.saturating_add(WITHIN));
        let words = match self.figures.len() {
            0 => b"Memory high water".as_slice(),
            1 => b"Longest slice".as_slice(),
            _ => b"Download the session log".as_slice(),
        };
        if self.figures.len() < 2 {
            if let Found::Node(node) = face.find(&Find::text(words)) {
                self.figures.push(
                    String::from_utf8(face.seen(node).expect("frame text").name.to_vec()).expect("UTF-8 figures"),
                );
            }
        } else if let Some(download) = face.downloads().first() {
            assert_eq!(download.name.as_ref(), b"temper-playground.log");
            self.download = Some(download.path.clone());
            return Next::Done;
        }
        if now >= until {
            return Next::Failed { step: self.at(), why: Why::Missing { find: Find::text(words) } };
        }
        Next::Wait { until: Some(until) }
    }

    fn at(&self) -> u32 {
        self.script.at()
    }

    fn expected(&self) -> Vec<Box<[u8]>> {
        self.script.expected()
    }
}

/// Choosing the playground opens `NewChat`. Later native moves use the rail's Change menu:
/// a document navigation would start a fresh worker.
fn story_steps() -> Vec<Step> {
    let mut first_new_chat = true;
    temper_playground_world::stories::a()
        .into_iter()
        .enumerate()
        .flat_map(|(index, step)| {
            vec![Step::Person(match step {
                PersonStep::Go { address } => {
                    let path = temper_client_domain::address::print(&address);
                    if index == 0 {
                        assert_eq!(path.as_ref(), b"/welcome");
                        return Vec::new();
                    }
                    assert_eq!(path.as_ref(), b"/chats/new");
                    if first_new_chat {
                        first_new_chat = false;
                        return Vec::new();
                    }
                    return temper_client_browser_world::start_change(true);
                }
                PersonStep::Press { find } => PersonStep::Press { find },
                PersonStep::Near { find, edge } => PersonStep::Near { find, edge },
                PersonStep::Type { find, words } => PersonStep::Type { find, words },
                PersonStep::Send { find } => PersonStep::Send { find },
                PersonStep::See { find, within } => PersonStep::See { find, within },
                PersonStep::Gone { find, within } => PersonStep::Gone { find, within },
                PersonStep::Reload => PersonStep::Reload,
                PersonStep::Pause { span } => PersonStep::Pause { span },
            })]
        })
        .collect()
}

fn download_size(directory: &Path) -> (u64, u64) {
    let mut raw = 0;
    let mut compressed = 0;
    for entry in fs::read_dir(directory).expect("bundle directory") {
        let entry = entry.expect("bundle entry");
        let (bytes, gzip) = if entry.file_type().expect("file type").is_dir() {
            download_size(&entry.path())
        } else {
            let bytes = entry.metadata().expect("bundle file size").len();
            // This measures the static download; compression is a tool, outside the code under test
            // (skein/docs/foundation/testing-strategy.md, 8; temper/docs/design/playground.md, 9).
            let gzip = Command::new("gzip")
                .args(["--no-name", "--stdout"])
                .arg(entry.path())
                .output()
                .expect("gzip is required to measure the compressed download");
            assert!(gzip.status.success(), "gzip: {}", String::from_utf8_lossy(&gzip.stderr));
            (bytes, u64::try_from(gzip.stdout.len()).expect("gzip size"))
        };
        raw += bytes;
        compressed += gzip;
    }
    (raw, compressed)
}

#[test]
#[expect(clippy::result_large_err, reason = "the replay wrapper preserves the complete mismatch diagnostic")]
fn story_a_walks_on_the_built_playground() {
    let out =
        PathBuf::from(temper_test_env::value(&temper_test_env::PLAYGROUND).expect("run temper/scripts/browser.sh"))
            .canonicalize()
            .expect("built playground");
    let scratch = skein_scratch::Scratch::new("browser-playground");
    let downloads = scratch.path().join("downloads");
    fs::create_dir(&downloads).expect("downloads directory");
    let setup = Setup {
        server: playground_serve(),
        args: vec![out.clone().into_os_string()],
        restart: vec![],
        printed: Printed { sent: Sent::Page, prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url },
        chromium: chromium(),
        downloads: Some(downloads),
        expected: vec![],
        origins: vec![],
    };
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut steps = vec![
        see(Find::text("Loading the playground…".as_bytes())),
        see(Find::text(b"temper's playground: a scripted model and forge, running in your browser")),
        see(Find::exactly_named(Role::Button, b"Try the playground")),
        Step::Person(PersonStep::Gone { find: Find::text("Loading the playground…".as_bytes()), within: WITHIN }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, "10×".as_bytes()) }),
    ];
    steps.extend(story_steps());
    steps
        .push(Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Download the session log") }));
    let mut walk = Walk { script: Script::new(steps.into_boxed_slice()), figures: vec![], download: None, until: None };
    // Empty expected trouble means the harness rejects every CSP violation and exception.
    assert_eq!(Harness::new(setup).run(&mut [&mut walk]), Outcome::Done);
    let bytes = fs::read(walk.download.expect("completed session log download")).expect("downloaded log");
    let read = skein_ui_web::log::read(&bytes).expect("session log");
    assert!(matches!(read.end, skein_ui_web::log::End::At { .. }), "the complete story fits the log");
    let a = &temper_playground::stories::A;
    let world = temper_playground::world(a.start(read.seed, read.start));
    let parameters = story::parameters(&world);
    programs::with(parameters, || skein_playground::replay::replay(world, story::reach(), &read, &mut ()))
        .unwrap_or_else(|differs| panic!("{differs:#?}"));

    let memory = walk.figures[0]
        .strip_prefix("Memory high water ")
        .expect("memory label")
        .strip_suffix(" MiB")
        .expect("memory unit");
    let (whole, tenth) = memory.split_once('.').expect("tenths of MiB");
    let memory =
        (whole.parse::<u64>().expect("whole MiB") * 10 + tenth.parse::<u64>().expect("tenth MiB")) * 1024 * 1024 / 10;
    let slice =
        walk.figures[1].strip_prefix("Longest slice ").expect("slice label").strip_suffix(" ms").expect("slice unit");
    let _: f64 = slice.parse().expect("slice milliseconds");
    let (raw, gzip) = download_size(&out);
    eprintln!(
        "playground figures: download raw {raw} B gzip {gzip} B; memory high water {memory} B; longest slice {slice} ms"
    );
}
