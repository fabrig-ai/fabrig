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
    fn file(root: &Path, name: &str, lines: &str, story: &str) {
        let directory = root.join(name);
        fs::create_dir(&directory).unwrap_or_else(|why| panic!("{story}: create journey: {why}"));
        fs::write(directory.join("walk"), lines).unwrap_or_else(|why| panic!("{story}: write walk: {why}"));
    }
    fn walk(&self, name: &str, story: &str) -> String {
        let chromium = chromium();
        let path =
            temper_test_env::value(&temper_shell::environment::PATH).unwrap_or_else(|| panic!("{story}: missing PATH"));
        let environment = vec![
            (b"SKEIN_TEST_CHROMIUM".as_slice().into(), chromium.as_os_str().as_bytes().into()),
            (b"PATH".as_slice().into(), path.as_os_str().as_bytes().into()),
        ];
        let mut output = vec![];
        let exit = journey_walk::walk(&self.roots, name, self.options.clone(), &environment, &mut output);
        assert_eq!(exit.status, 0, "{story}: {exit:?}; answers: {}", String::from_utf8_lossy(&output));
        assert_eq!(exit.shown.last(), Some(&format!("walked {name}")), "{story}: walk summary");
        assert!(self.scratch.path().join("box/session/lines").is_file(), "{story}: missing session/lines");
        String::from_utf8(output).unwrap_or_else(|why| panic!("{story}: UTF-8 answers: {why}"))
    }
    fn looks(&self, story: &str) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(self.options.scratch.join("session/looks"))
            .unwrap_or_else(|why| panic!("{story}: read looks: {why}"))
            .map(|entry| {
                entry
                    .unwrap_or_else(|why| panic!("{story}: read look: {why}"))
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|name| panic!("{story}: non-UTF-8 look name: {name:?}"))
            })
            .collect();
        names.sort();
        names
    }
}

#[test]
fn a_browser_journey_looks_and_a_tui_journey_is_approved_in_chromium() {
    let web_story = "17, a browser journey's look";
    let tui_story = "18, a TUI journey approved in Chromium";
    // The fake box leaves data/ behind, and a journey walk requires scratch without it.
    let web = Journey::new("browser-journey");
    let tui = Journey::new("tui-journey");
    Journey::file(
        &web.roots.web,
        "welcome",
        "walk 2\nweb see \"Welcome to temper\"\nweb press link \"Set up temper\"\nweb see \"Run on this machine\"\nweb mark\n",
        web_story,
    );
    Journey::file(
        &tui.roots.tui,
        "terminal",
        "walk 2\ntui see \"then press Approve.\"\ntui approve\ntui see \"Setting temper up makes your first project\"\ntui mark\n",
        tui_story,
    );

    let output = web.walk("welcome", web_story);
    assert!(output.contains("welcome:5 | ") && output.contains("Run on this machine"), "{web_story}: {output}");
    assert_eq!(web.looks(web_story), ["0004-420.png"], "{web_story}: looks");
    let png = fs::read(web.options.scratch.join("session/looks/0004-420.png"))
        .unwrap_or_else(|why| panic!("{web_story}: read PNG: {why}"));
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "{web_story}: PNG signature");
    assert_eq!(
        fs::read_to_string(web.options.scratch.join("session/lines"))
            .unwrap_or_else(|why| panic!("{web_story}: read kept lines: {why}")),
        "web see \"Welcome to temper\"\nweb press link \"Set up temper\"\nweb see \"Run on this machine\"\n",
        "{web_story}: kept lines"
    );

    let output = tui.walk("terminal", tui_story);
    assert!(output.contains("terminal:3 ok"), "{tui_story}: {output}");
    assert!(
        output.lines().any(
            |line| line.starts_with("terminal:5 | ") && line.contains("Setting temper up makes your first project")
        ),
        "{tui_story}: {output}"
    );
    assert_eq!(tui.looks(tui_story), Vec::<String>::new(), "{tui_story}: looks");
    let kept = fs::read_to_string(tui.options.scratch.join("session/lines"))
        .unwrap_or_else(|why| panic!("{tui_story}: read kept lines: {why}"));
    assert!(kept.contains("tui approve\n") && !kept.contains("tui mark"), "{tui_story}: kept lines: {kept}");
}
