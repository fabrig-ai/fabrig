//! The displayed cells of skein's terminal output, not its changed byte runs.
//! Only skein-ui-screen-terminal's output vocabulary is interpreted here
//! (skein/docs/design/ui/screen.md, 7 and 8).
use skein_ui_screen::width;

pub struct Screen {
    columns: usize,
    rows: usize,
    cells: Vec<char>,
    row: usize,
    column: usize,
    pending: Vec<u8>,
    watched: Vec<(Vec<u8>, bool)>,
}
impl Screen {
    #[must_use]
    pub fn new(columns: u16, rows: u16) -> Self {
        let columns = usize::from(columns);
        let rows = usize::from(rows);
        assert!(columns > 0 && rows > 0);
        Self {
            columns,
            rows,
            cells: vec![' '; columns * rows],
            row: 0,
            column: 0,
            pending: Vec::new(),
            watched: Vec::new(),
        }
    }

    #[must_use]
    pub fn has(&self, words: &[u8]) -> bool {
        skein_lib::bytes::find(self.text().as_bytes(), words).is_some()
    }

    pub fn watch(&mut self, words: &[u8]) {
        self.watched.push((words.to_vec(), false));
    }

    #[must_use]
    pub fn seen(&self, words: &[u8]) -> bool {
        self.watched.iter().any(|(watched, seen)| watched == words && *seen)
    }

    fn observe(&mut self) {
        if self.watched.iter().all(|(_, seen)| *seen) {
            return;
        }
        let text = self.text();
        for (words, seen) in &mut self.watched {
            *seen |= skein_lib::bytes::find(text.as_bytes(), words).is_some();
        }
    }

    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        for row in self.cells.chunks(self.columns) {
            text.extend(row.iter().filter(|glyph| **glyph != '\0'));
            text.push('\n');
        }
        text
    }

    pub fn read(&mut self, bytes: &[u8]) {
        let mut pending = std::mem::take(&mut self.pending);
        pending.extend_from_slice(bytes);
        let mut at = 0;
        while let Some(byte) = pending.get(at) {
            let consumed = match byte {
                0x1b => self.escape(&pending[at..]),
                b'\r' => {
                    self.column = 0;
                    Some(1)
                }
                b'\n' => {
                    self.line();
                    Some(1)
                }
                _ => self.glyph(&pending[at..]),
            };
            let Some(consumed) = consumed else { break };
            at += consumed;
        }
        self.pending.extend_from_slice(&pending[at..]);
    }

    fn glyph(&mut self, bytes: &[u8]) -> Option<usize> {
        let length = match bytes[0] {
            0..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            byte => panic!("invalid terminal UTF-8 leading byte: {byte}"),
        };
        let bytes = bytes.get(..length)?;
        let glyph = std::str::from_utf8(bytes).expect("terminal UTF-8").chars().next().expect("one glyph");
        let width = usize::from(width(glyph));
        if width > 0 && self.column < self.columns {
            let index = self.row * self.columns + self.column;
            self.cells[index] = glyph;
            if width == 2 && self.column + 1 < self.columns {
                self.cells[index + 1] = '\0';
            }
            self.column += width;
        }
        Some(length)
    }

    fn line(&mut self) {
        if self.row + 1 < self.rows {
            self.row += 1;
        } else {
            self.cells.copy_within(self.columns.., 0);
            let last = (self.rows - 1) * self.columns;
            self.cells[last..].fill(' ');
        }
    }

    fn escape(&mut self, bytes: &[u8]) -> Option<usize> {
        match bytes.get(1)? {
            b'[' => {
                let end = bytes.iter().enumerate().skip(2).find(|(_, byte)| (0x40..=0x7e).contains(*byte))?.0;
                self.control(&bytes[2..end], bytes[end]);
                Some(end + 1)
            }
            b']' => {
                // The background query is OSC 11 followed by ST; BEL also terminates OSC.
                let mut at = 2;
                while let Some(byte) = bytes.get(at) {
                    if *byte == 7 {
                        return Some(at + 1);
                    }
                    if *byte == 0x1b && bytes.get(at + 1) == Some(&b'\\') {
                        return Some(at + 2);
                    }
                    at += 1;
                }
                None
            }
            byte => panic!("unexpected terminal escape: {byte}"),
        }
    }

    fn control(&mut self, arguments: &[u8], command: u8) {
        match command {
            b'H' => {
                let mut coordinates = arguments.split(|byte| *byte == b';');
                self.row = coordinate(coordinates.next()).min(self.rows - 1);
                self.column = coordinate(coordinates.next()).min(self.columns - 1);
            }
            b'J' => {
                assert_eq!(arguments, b"2", "the renderer clears the whole screen");
                self.cells.fill(' ');
            }
            // Paint, cursor visibility, wrapping, bracketed paste and alternate screen modes.
            b'h' | b'l' if arguments == b"?25" => {
                // Every draw ends with the cursor's visibility, even two draws in one read.
                self.observe();
            }
            b'm' | b'h' | b'l' => {}
            byte => panic!("unexpected terminal control: {byte}"),
        }
    }
}

fn coordinate(value: Option<&[u8]>) -> usize {
    match value {
        None | Some(b"") => 0,
        Some(value) => std::str::from_utf8(value)
            .expect("coordinate digits")
            .parse::<usize>()
            .expect("coordinate")
            .saturating_sub(1),
    }
}
