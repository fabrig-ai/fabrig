//! Complete field text crosses the process boundary only after the last heap check.
//! The reachable vocabulary uses derived Debug/Eq, ordered collections and value
//! wrappers. `PolicySnapshot` prints its policy, just as its Eq compares it. List
//! alone prints state Eq ignores: its cap; `field_text` removes only that member.
use super::OUTSIDE_FIELDS;
use std::io::{self, Read, Write};

/// Complete outside fields carried between independent metered processes.
/// This is test-local text, not a stored product format.
#[derive(Debug, PartialEq, Eq)]
pub struct OutsideText {
    fields: Vec<String>,
}

impl OutsideText {
    pub(super) fn new(fields: Vec<String>) -> Self {
        assert_eq!(fields.len(), OUTSIDE_FIELDS.len());
        Self { fields }
    }

    /// Write every field, length delimited so that data cannot become framing.
    pub fn write(&self, mut to: impl Write) -> io::Result<()> {
        for field in &self.fields {
            to.write_all(&u64::try_from(field.len()).expect("field length fits u64").to_le_bytes())?;
            to.write_all(field.as_bytes())?;
        }
        Ok(())
    }

    /// Decode only after the receiving run has finished its heap checks.
    pub fn read(mut from: impl Read) -> io::Result<Self> {
        let mut fields = Vec::with_capacity(OUTSIDE_FIELDS.len());
        for _ in OUTSIDE_FIELDS {
            let mut length = [0; 8];
            from.read_exact(&mut length)?;
            let length = usize::try_from(u64::from_le_bytes(length))
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            let mut bytes = vec![0; length];
            from.read_exact(&mut bytes)?;
            fields.push(String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?);
        }
        let mut extra = [0];
        if from.read(&mut extra)? != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "outside fields have trailing bytes"));
        }
        Ok(Self::new(fields))
    }

    /// The same field and first differing line as `Outside::first_difference`.
    #[must_use]
    pub fn first_difference(&self, other: &Self) -> Option<String> {
        for ((name, left), right) in OUTSIDE_FIELDS.iter().zip(&self.fields).zip(&other.fields) {
            if left != right {
                let mut left = left.lines().map(Some).chain(std::iter::repeat(None));
                let (b, a) = right
                    .lines()
                    .map(Some)
                    .chain([None])
                    .zip(&mut left)
                    .find(|(b, a)| a != b)
                    .expect("different full fields have a differing line");
                return Some(format!("{name}: {a:?} != {b:?}"));
            }
        }
        None
    }
}

pub(super) fn field_text(value: &impl std::fmt::Debug) -> String {
    let mut text = format!("{value:#?}");
    if !text.contains("List {") {
        return text;
    }
    let mut lists = Vec::new();
    let mut ignored = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let line = line.trim_end_matches('\n');
        let start = offset;
        offset += line.len() + 1;
        // Derived pretty Debug puts each structural member on its own line;
        // string/character data is escaped, so cannot supply a member line.
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if lists.last() == Some(&indent) && (trimmed == "}," || trimmed == "}") {
            lists.pop();
        }
        if lists.last().is_some_and(|at| indent == at + 4) && trimmed.starts_with("capacity: ") {
            ignored.push(start..offset);
        }
        if trimmed == "List {" || trimmed.ends_with(": List {") {
            // The name may follow a field label or a tuple/enum opening.
            lists.push(indent);
        }
    }
    for range in ignored.into_iter().rev() {
        text.replace_range(range, "");
    }
    text
}
