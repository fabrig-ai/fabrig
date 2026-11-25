//! The viewport story continues with a comparison of local renders and a blocked image.
use std::collections::VecDeque;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use skein_browser::boundary::{Element, Event, Property, Rect, Request, Trouble};
use skein_lib::{List, Token};

use crate::{PERSON, Rig, Stage};

const FILES: [(Token, &str); 3] = [(Token::new(40), "a.html"), (Token::new(41), "b.html"), (Token::new(42), "c.html")];
const IMAGE: Token = Token::new(43);
const MEASURE: Token = Token::new(44);
const WIDTHS: [u32; 3] = [1440, 1000, 420];
const HEIGHT: u32 = 900;
const IMAGE_PATH: &str = "/rendering-blocked.png";

pub(crate) struct Rendering {
    directory: PathBuf,
    width: usize,
    page: usize,
    pending: VecDeque<Element>,
    style: Option<List<Property>>,
    current: Vec<Rendered>,
    baseline: Vec<Rendered>,
    image_url: Option<String>,
    image_page_opened: bool,
    blocked_image_reported: bool,
}

struct Rendered {
    element: Element,
    style: List<Property>,
    rect: Option<Rect>,
}

impl Rendering {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            width: 0,
            page: 0,
            pending: VecDeque::new(),
            style: None,
            current: Vec::new(),
            baseline: Vec::new(),
            image_url: None,
            image_page_opened: false,
            blocked_image_reported: false,
        }
    }

    fn reading_page(&self) -> Token {
        FILES.get(self.page).expect("the fixture being read").0
    }
}

impl Rig {
    pub(super) fn begin_file_reads(&mut self) {
        std::fs::create_dir(&self.rendering.directory).expect("fresh directory for file-backed renders");
        for (index, (_, name)) in FILES.iter().enumerate() {
            let cursor = if index == 1 { "wait" } else { "crosshair" };
            // The leaf has no element children that could inherit its cursor.
            let html = format!("<!doctype html><main style='cursor:{cursor}'>A rendered leaf</main>");
            std::fs::write(self.rendering.directory.join(name), html).expect("write a fixed render fixture");
        }
        self.stage = Stage::Capturing;
        self.open_render_file(FILES[0]);
    }

    fn open_render_file(&mut self, (page, name): (Token, &str)) {
        let url = file_url(&self.rendering.directory.join(name));
        self.ask(Request::Page { person: PERSON, page, url, requests: false, files: true });
    }

    pub(super) fn file_read_event(&mut self, event: Event) -> Result<(), Event> {
        match event {
            Event::Opened { page } if FILES.iter().any(|(token, _)| *token == page) => {
                let index = FILES.iter().position(|(token, _)| *token == page).expect("one of the render files");
                if let Some(&next) = FILES.get(index + 1) {
                    self.open_render_file(next);
                } else {
                    self.measure_render();
                }
            }
            Event::Done { op: MEASURE } => {
                self.ask(Request::Elements { page: self.rendering.reading_page(), op: MEASURE, selector: None });
            }
            Event::Elements { op: MEASURE, elements, more } => {
                assert_eq!(more, 0, "the fixed document fits in one elements read");
                assert!(!elements.is_empty(), "the fixed document contains elements");
                self.rendering.pending = elements.into_boxed().into_vec().into();
                self.next_style();
            }
            Event::Style { op: MEASURE, style } => {
                let node = self.rendering.pending.front().expect("the element being read").node;
                self.rendering.style = Some(style);
                self.ask(Request::Rect { page: self.rendering.reading_page(), op: MEASURE, node });
            }
            Event::Rect { op: MEASURE, rect } => {
                let element = self.rendering.pending.pop_front().expect("the element being read");
                let style = self.rendering.style.take().expect("the element's computed style");
                self.rendering.current.push(Rendered { element, style, rect });
                if self.rendering.pending.is_empty() {
                    self.finish_render();
                } else {
                    self.next_style();
                }
            }
            Event::Opened { page: IMAGE } => {
                assert_eq!(self.pages.watched_calls(), 0, "the image must not reach the origin");
                self.rendering.image_page_opened = true;
                self.finish_image();
            }
            Event::Trouble { page: IMAGE, trouble: Trouble::Log, url, .. } => {
                let expected = self.rendering.image_url.as_deref().expect("the blocked image's URL");
                assert_eq!(url.as_ref(), expected.as_bytes(), "the log must name the blocked image");
                self.rendering.blocked_image_reported = true;
                self.finish_image();
            }
            Event::Closed { owner }
                if self.stage == Stage::Closing && (owner == IMAGE || FILES.iter().any(|(page, _)| *page == owner)) => {
            }
            event => return Err(event),
        }
        Ok(())
    }

    fn measure_render(&mut self) {
        let width = WIDTHS[self.rendering.width];
        self.ask(Request::Viewport { page: self.rendering.reading_page(), op: MEASURE, width, height: HEIGHT });
    }

    fn next_style(&mut self) {
        let node = self.rendering.pending.front().expect("the next element").node;
        self.ask(Request::Style { page: self.rendering.reading_page(), op: MEASURE, node });
    }

    fn finish_render(&mut self) {
        let current = std::mem::take(&mut self.rendering.current);
        let width = WIDTHS[self.rendering.width];
        if self.rendering.page == 0 {
            self.rendering.baseline = current;
        } else {
            let expected: &[Difference<'_>] =
                if self.rendering.page == 1 { &[(b"main", b"cursor", b"crosshair", b"wait")] } else { &[] };
            assert_eq!(differences(&self.rendering.baseline, &current, width), expected, "at {width} CSS pixels");
        }
        self.rendering.page += 1;
        if self.rendering.page == FILES.len() {
            self.rendering.page = 0;
            self.rendering.width += 1;
        }
        if self.rendering.width == WIDTHS.len() {
            self.open_image();
        } else {
            self.measure_render();
        }
    }

    fn open_image(&mut self) {
        let addr = self.pages.addr.expect("the fixture origin is listening");
        let url = format!("http://{addr}{IMAGE_PATH}");
        self.pages.watch_path(IMAGE_PATH.as_bytes().into());
        let name = "d.html";
        let html = format!("<!doctype html><img src='{url}' alt='Blocked image'>");
        std::fs::write(self.rendering.directory.join(name), html).expect("write the blocked-image fixture");
        self.rendering.image_url = Some(url);
        self.open_render_file((IMAGE, name));
    }

    fn finish_image(&mut self) {
        if self.rendering.image_page_opened && self.rendering.blocked_image_reported {
            assert_eq!(self.pages.watched_calls(), 0, "the image must not reach the origin");
            self.close_browser();
        }
    }
}

type Difference<'a> = (&'a [u8], &'a [u8], &'a [u8], &'a [u8]);

fn differences<'a>(baseline: &'a [Rendered], other: &'a [Rendered], width: u32) -> Vec<Difference<'a>> {
    assert_eq!(baseline.len(), other.len(), "element count at {width} CSS pixels");
    let mut changes = Vec::new();
    for (a, b) in baseline.iter().zip(other) {
        let label = String::from_utf8_lossy(&a.element.name);
        assert_eq!(a.element.name, b.element.name, "element order at {width} CSS pixels");
        assert_eq!(a.element.depth, b.element.depth, "{label}'s depth at {width} CSS pixels");
        assert_eq!(a.rect, b.rect, "{label}'s rect at {width} CSS pixels");
        assert_eq!(a.style.len(), b.style.len(), "{label}'s property count at {width} CSS pixels");
        for (left, right) in a.style.iter().zip(b.style.iter()) {
            assert_eq!(left.name, right.name, "{label}'s property order at {width} CSS pixels");
            if left.value != right.value {
                changes.push((a.element.name.as_ref(), left.name.as_ref(), left.value.as_ref(), right.value.as_ref()));
            }
        }
    }
    changes
}

fn file_url(path: &Path) -> Box<[u8]> {
    use std::fmt::Write;
    assert!(path.is_absolute(), "the scratch directory must be absolute");
    let mut url = String::from("file://");
    for &byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            url.push(char::from(byte));
        } else {
            write!(&mut url, "%{byte:02X}").expect("writing a string is infallible");
        }
    }
    url.into_bytes().into_boxed_slice()
}
