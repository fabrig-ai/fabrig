//! Box walks of scratch journeys (draft/archive/journey-walks.md, 4.2 and 10).
use std::{
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{PermissionsExt, symlink},
    },
    path::Path,
};
use temper_client_browser_world::{bundle, chromium};
use temper_client_journeys_tool::{drive, journey_walk};
use temper_client_walk_lines::walk::Roots;

struct Journey {
    scratch: skein_scratch::Scratch,
    roots: Roots,
    options: drive::Options,
}
impl Journey {
    fn new(label: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(label);
        let root = scratch.path();
        let roots = Roots { web: root.join("web"), tui: root.join("tui") };
        let bin = root.join("bin");
        let box_scratch = root.join("box");
        for directory in [&roots.web, &roots.tui, &bin, &box_scratch] {
            fs::create_dir(directory).expect("scratch directory");
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("private directory");
        }
        for (package, binary) in
            [("temper-box", "temper-box"), ("smith-fake-claude", "fake-claude"), ("temper", "temper")]
        {
            symlink(skein_scratch::paths::built(package, binary), bin.join(binary)).expect("built binary link");
        }
        let options = drive::Options {
            bin,
            bundle: Some(bundle()),
            deployment: None,
            scratch: box_scratch,
            scripts: vec![],
            strict: false,
            ctl: false,
            journey: true,
            widths: vec![420],
            ports: None,
            files: None,
        };
        Self { scratch, roots, options }
    }
    fn file(root: &Path, name: &str, lines: &str) {
        let directory = root.join(name);
        fs::create_dir(&directory).expect("journey");
        fs::write(directory.join("walk"), lines).expect("walk");
    }
    fn walk(&self, name: &str) -> String {
        let chromium = chromium();
        let path = temper_test_env::value(&temper_shell::environment::PATH).expect("PATH");
        let environment = vec![
            (b"SKEIN_TEST_CHROMIUM".as_slice().into(), chromium.as_os_str().as_bytes().into()),
            (b"PATH".as_slice().into(), path.as_os_str().as_bytes().into()),
        ];
        let mut output = vec![];
        let exit = journey_walk::walk(&self.roots, name, self.options.clone(), &environment, &mut output);
        assert_eq!(exit.status, 0, "{exit:?}; answers: {}", String::from_utf8_lossy(&output));
        assert_eq!(exit.shown.last().expect("summary"), &format!("walked {name}"));
        assert!(self.scratch.path().join("box/session/lines").is_file());
        String::from_utf8(output).expect("UTF-8 answers")
    }
    fn looks(&self) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(self.options.scratch.join("session/looks"))
            .expect("looks")
            .map(|entry| entry.expect("look").file_name().into_string().expect("look name"))
            .collect();
        names.sort();
        names
    }
}

#[test]
fn browser_journey_answers_a_screenshot_and_the_controls_text() {
    let journey = Journey::new("browser-journey");
    Journey::file(
        &journey.roots.web,
        "welcome",
        "walk 2\nweb see \"Welcome to temper\"\nweb press link \"Set up temper\"\nweb see \"Run on this machine\"\nweb mark\n",
    );
    let output = journey.walk("welcome");
    assert!(output.contains("welcome:5 | ") && output.contains("Run on this machine"), "{output}");
    assert_eq!(journey.looks(), ["0004-420.png"]);
    let png = fs::read(journey.options.scratch.join("session/looks/0004-420.png")).expect("PNG");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(
        fs::read_to_string(journey.options.scratch.join("session/lines")).expect("kept lines"),
        "web see \"Welcome to temper\"\nweb press link \"Set up temper\"\nweb see \"Run on this machine\"\n"
    );
}

#[test]
fn tui_journey_waits_for_explicit_approval_then_marks_the_launchpad() {
    let journey = Journey::new("tui-journey");
    Journey::file(
        &journey.roots.tui,
        "terminal",
        "walk 2\ntui see \"then press Approve.\"\ntui approve\ntui see \"Setting temper up makes your first project\"\ntui mark\n",
    );
    let output = journey.walk("terminal");
    assert!(output.contains("terminal:3 ok"), "{output}");
    assert!(
        output.lines().any(
            |line| line.starts_with("terminal:5 | ") && line.contains("Setting temper up makes your first project")
        ),
        "{output}"
    );
    assert_eq!(journey.looks(), Vec::<String>::new());
    let kept = fs::read_to_string(journey.options.scratch.join("session/lines")).expect("kept");
    assert!(kept.contains("tui approve\n") && !kept.contains("tui mark"), "{kept}");
}
