//! Structured edits count the matching fields in recorded JSON lines.

pub fn edit_lines(
    source: &str,
    pointer: &str,
    before: &serde_json::Value,
    after: Option<&serde_json::Value>,
    expected: usize,
) -> String {
    let mut changed = 0;
    let mut output = String::new();
    for line in source.lines() {
        let mut record: serde_json::Value = serde_json::from_str(line).expect("recorded JSON line");
        if record.pointer(pointer) == Some(before) {
            if let Some(after) = after {
                *record.pointer_mut(pointer).expect("matched field") = after.clone();
            } else {
                let (parent, key) = pointer.rsplit_once('/').expect("object field pointer");
                record.pointer_mut(parent).expect("matched parent").as_object_mut().expect("object field")
                    .remove(key).expect("matched field");
            }
            changed += 1;
        }
        output.push_str(&serde_json::to_string(&record).expect("recorded JSON"));
        output.push('\n');
    }
    assert_eq!(changed, expected, "recorded fields matching {pointer}");
    output
}
