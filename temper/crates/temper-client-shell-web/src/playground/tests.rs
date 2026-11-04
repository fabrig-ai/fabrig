use super::*;
use skein_ui_web::bridge::{Printed, Stream};

#[test]
fn the_figures_name_their_units_and_keep_fractional_parts() {
    for (now, expected) in [(0, "World time 00:00:00.000"), (3_723_456_789_000, "World time 01:02:03.456")] {
        assert_eq!(time_words(Time::from_nanos(now)), expected);
    }
    for (bytes, expected) in [(0, "Memory high water 0.0 MiB"), (13_107_200, "Memory high water 12.5 MiB")] {
        assert_eq!(memory_words(bytes), expected);
    }
    for (nanos, expected) in [(0, "Longest slice 0.000 ms"), (2_345_678, "Longest slice 2.345 ms")] {
        assert_eq!(slice_words(Duration::from_nanos(nanos)), expected);
    }
    assert!(memory_words(u64::MAX).ends_with(" MiB"));
    assert!(slice_words(Duration::from_nanos(u64::MAX)).ends_with(" ms"));
    assert!(time_words(Time::from_nanos(u64::MAX)).starts_with("World time "));
}

#[test]
fn the_speeds_are_the_frames_words() {
    for (speed, words) in [(0, "Paused"), (500, "½×"), (1000, "1×"), (10_000, "10×")] {
        assert_eq!(speed_words(Speed(speed)), words);
    }
}

#[test]
fn only_the_services_output_link_supplies_a_path_and_query() {
    let mut printed = Printed {
        process: b"temper serve".as_slice().into(),
        stream: Stream::Output,
        line: b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=example&extra=kept".as_slice().into(),
    };
    assert_eq!(printed_link(&printed), Some("/jig/v1/sign-in/link?secret=example&extra=kept"));
    printed.stream = Stream::Error;
    assert_eq!(printed_link(&printed), None);
    printed.stream = Stream::Output;
    printed.process = b"another process".as_slice().into();
    assert_eq!(printed_link(&printed), None);
    printed.process = b"temper serve".as_slice().into();
    for line in [
        "Starting temper serve",
        "/jig/v1/sign-in/link?secret=example",
        "http://host/other?secret=example",
        "http://host/jig/v1/sign-in/link?other=example",
        "http://host/jig/v1/sign-in/link?secret=",
        "http:///jig/v1/sign-in/link?secret=example",
        "http://host/jig/v1/sign-in/link?secret=example#fragment",
        "http://host/jig/v1/sign-in/link?secret=example\n",
        "http://host/jig/v1/sign-in/link?secret=two words",
        "http://host/?next=/jig/v1/sign-in/link?secret=example",
        "ftp://host/jig/v1/sign-in/link?secret=example",
    ] {
        printed.line = line.as_bytes().into();
        assert_eq!(printed_link(&printed), None, "{line:?}");
    }
    printed.line = b"https://host/jig/v1/sign-in/link?secret=example".as_slice().into();
    assert_eq!(printed_link(&printed), Some("/jig/v1/sign-in/link?secret=example"));
    printed.line = b"http://host/jig/v1/sign-in/link?secret=\xff".as_slice().into();
    assert_eq!(printed_link(&printed), None);
}

#[test]
fn each_frame_control_has_its_report() {
    for (id, expected) in [
        (PAUSE, Control::Pause),
        (HALF, Control::Speed(Speed(500))),
        (ONE, Control::Speed(Speed::ONE)),
        (TEN, Control::Speed(Speed(10_000))),
        (RESET, Control::Reset),
        (START_OVER, Control::Reset),
        (STORY, Control::Story("a")),
        (LOG, Control::Log),
    ] {
        assert_eq!(control(id, "a"), Some(expected), "{id}");
    }
    assert_eq!(control(STORY, "unknown"), None);
    assert_eq!(control(APP, "a"), None);
}

#[test]
fn story_a_has_its_own_description() {
    let (_, description) = STORIES.iter().find(|(name, _)| *name == "a").expect("story a");
    assert!(!description.trim().is_empty());
    assert!(description.contains("three suggested requests"));
}

#[test]
fn the_page_supplies_every_element_the_frame_and_failure_hook_need() {
    let page = include_str!("../../playground/index.html");
    for id in [
        APP,
        FRAME,
        LOADING,
        DESCRIPTION,
        STORY,
        PAUSE,
        HALF,
        ONE,
        TEN,
        RESET,
        LOG,
        TIME,
        MEMORY,
        SLICE,
        PACE,
        VERSION,
        FAILURE,
        FAILURE_WORDS,
        FAILURE_LOG,
        START_OVER,
    ] {
        assert_eq!(page.matches(&format!("id=\"{id}\"")).count(), 1, "{id}");
    }
    for file in ["skein-ui.css", "temper.css", "task-pages.css", "playground.css", "boot.js"] {
        assert!(page.contains(&format!("\"{{bundle}}{file}\"")), "{file}");
    }
    assert!(!page.contains("temper-sign-in"));
    assert!(!page.contains("temper-forge-sign-in"));
    assert!(page.contains("Download temper to use a real model and forge."));
}
