//! A reference-case observation follows every supplied activity cursor.
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Pages {
    first: Option<Value>,
    rows: Vec<Value>,
    cursors: BTreeSet<String>,
    cut: bool,
}
impl Pages {
    pub(super) fn push(&mut self, bytes: &[u8]) -> Option<String> {
        let value: Value = serde_json::from_slice(bytes).expect("activity response JSON");
        let page = value.get("activity").unwrap_or(&value);
        let rows = page.get("rows").and_then(Value::as_array)?;
        self.rows.extend(rows.iter().cloned());
        self.cut |= page["cut"] == true;
        let next = page["before"]["value"].as_str().map(str::to_owned);
        if let Some(cursor) = &next {
            assert!(self.cursors.insert(cursor.clone()), "activity cursor must make progress");
        }
        if self.first.is_none() {
            self.first = Some(value);
        }
        next
    }
    pub(super) fn finish(self) -> Option<Vec<u8>> {
        let mut value = self.first?;
        let page = if value.get("activity").is_some() { &mut value["activity"] } else { &mut value };
        page["rows"] = self.rows.into();
        page["before"] = Value::Null;
        page["cut"] = self.cut.into();
        Some(serde_json::to_vec(&value).expect("collected activity JSON"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_whole_observation_keeps_every_page_and_stops_at_the_absent_cursor() {
        let mut pages = Pages::default();
        assert_eq!(pages.push(br#"{"task":{"number":7},"activity":{"rows":[{"position":2}],"before":{"value":"older"},"after":null,"cut":false}}"#), Some("older".into()));
        assert_eq!(
            pages.push(br#"{"rows":[{"position":1}],"before":null,"after":{"value":"newer"},"cut":false}"#),
            None
        );
        let value: Value = serde_json::from_slice(&pages.finish().unwrap()).unwrap();
        assert_eq!(value["task"]["number"], 7);
        assert_eq!(value["activity"]["rows"], serde_json::json!([{"position":2},{"position":1}]));
        assert!(value["activity"]["before"].is_null());
        assert!(value["activity"]["after"].is_null());
        assert_eq!(value["activity"]["cut"], false);
    }
}
