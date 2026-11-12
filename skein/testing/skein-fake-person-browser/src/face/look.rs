//! Snapshot and temporary-width screenshot reads (ui/web/testing.md, 2).
use super::{BrowserFace, Event, Request, Token};
pub(super) enum Capture {
    Snapshot { op: Token }, Resize { op: Token, restore: (u32, u32) },
    Image { op: Token, restore: Option<(u32, u32)> }, Restore { op: Token, result: Result<Box<[u8]>, String> },
}
impl Capture { fn op(&self) -> Token { match self { Self::Snapshot { op } | Self::Resize { op, .. } | Self::Image { op, .. } | Self::Restore { op, .. } => *op } } }
impl BrowserFace<'_> {
    #[must_use]
    pub fn viewport_size(&self) -> (u32, u32) { self.viewport_size }
    pub fn snapshot(&mut self) {
        self.snapshot = None; self.refusal = None; let op = self.token(); self.acting = Some(op);
        self.capture = Some(Capture::Snapshot { op }); self.requests.push(Request::Snapshot { page: self.page, op });
    }
    pub fn take_snapshot(&mut self) -> Option<Box<[u8]>> { self.snapshot.take() }
    /// Read a PNG at width and the current height; restore the last successful
    /// viewport before completing, including after a refused image.
    pub fn screenshot_at(&mut self, width: u32) {
        self.screenshot = None; self.refusal = None; let restore = self.viewport_size;
        if width == restore.0 { self.image(None); } else {
            let op = self.token(); self.acting = Some(op); self.capture = Some(Capture::Resize { op, restore });
            self.requests.push(Request::Viewport { page: self.page, op, width, height: restore.1 });
        }
    }
    pub fn take_screenshot(&mut self) -> Option<Box<[u8]>> { self.screenshot.take() }
    /// Discard a failed line's state without reusing its operation identities.
    /// A temporary-width look still restores; its caller pumps until it settles.
    pub fn cancel(&mut self) {
        self.search = None; self.answer = None; self.acting = None; self.press = None;
        self.input = None; self.choice = None; self.keep = None; self.look_again = None;
        self.failure = None; self.refusal = None;
        // Keep already queued requests ahead of the restore: a resize may not
        // have reached the browser yet. Its late answer cannot finish a new op.
        if let Some(capture) = self.capture.take() {
            match capture {
                Capture::Resize { restore, .. } | Capture::Image { restore: Some(restore), .. } => self.restore_image(restore, Err("screenshot cancelled".into())),
                Capture::Restore { op, result } => { self.acting = Some(op); self.capture = Some(Capture::Restore { op, result }); }
                Capture::Snapshot { .. } | Capture::Image { restore: None, .. } => {},
            }
        }
    }
    fn image(&mut self, restore: Option<(u32, u32)>) {
        let op = self.token(); self.acting = Some(op); self.capture = Some(Capture::Image { op, restore });
        self.requests.push(Request::Screenshot { page: self.page, op });
    }
    fn restore_image(&mut self, size: (u32, u32), result: Result<Box<[u8]>, String>) {
        let op = self.token(); self.acting = Some(op); self.capture = Some(Capture::Restore { op, result });
        self.requests.push(Request::Viewport { page: self.page, op, width: size.0, height: size.1 });
    }
    fn image_done(&mut self, result: Result<Box<[u8]>, String>) {
        self.acting = None;
        match result { Ok(png) => self.screenshot = Some(png), Err(why) => self.fail(why) }
    }
    pub(super) fn look_event(&mut self, event: &Event) -> bool {
        let op = match event { Event::Done { op } | Event::Snapshot { op, .. } | Event::Screenshot { op, .. } | Event::Refused { op, .. } => *op, _ => return false };
        if self.capture.as_ref().map(Capture::op) != Some(op) { return false; }
        let capture = self.capture.take().expect("matching capture");
        let failure = if let Event::Refused { why, detail, .. } = event { Some(format!("browser refused look: {why:?}: {}", String::from_utf8_lossy(detail))) } else { None };
        match capture {
            Capture::Snapshot { .. } => { self.acting = None; if let Some(why) = failure { self.fail(why); } else if let Event::Snapshot { text, .. } = event { self.snapshot = Some(text.clone()); } }
            Capture::Resize { restore, .. } => { if let Some(why) = failure { self.restore_image(restore, Err(why)); } else { self.image(Some(restore)); } }
            Capture::Image { restore, .. } => {
                let result = if let Some(why) = failure { Err(why) } else if let Event::Screenshot { png, .. } = event { Ok(png.clone()) } else { Err("screenshot answered without PNG".into()) };
                if let Some(restore) = restore { self.restore_image(restore, result); } else { self.image_done(result); }
            }
            Capture::Restore { result, .. } => self.image_done(if let Some(why) = failure { Err(format!("restoring viewport: {why}")) } else { result }),
        }
        true
    }
}
#[cfg(test)]
mod tests;
