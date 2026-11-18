//! The page's internal settings and recording meta (client/web/README.md, 2.1).
//! Fixed starts follow temper/docs/design/client/journey-clock.md, 4.2.
use skein_lib::{FixedClock, FixedStart};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Internal {
    pub record: bool,
    pub small_window: bool,
    pub loopback_navigation: bool,
    pub fixed_start: Option<FixedStart>,
}

impl Internal {
    /// Substitute the start's facts, retaining the platform's native reading for its clock.
    pub(crate) fn start(self, facts: &mut skein_ui_web::platform::Facts) -> Option<FixedClock> {
        let start = self.fixed_start?;
        let clock = FixedClock::new(&start, facts.now);
        facts.seed = start.seed;
        facts.now = FixedStart::NOW;
        facts.wall = start.wall;
        facts.offset = 0;
        Some(clock)
    }

    pub(crate) fn recording(self, meta_content: Option<&str>) -> bool {
        self.record || record_meta(meta_content)
    }
}

/// Literal query names and values; the last occurrence of each setting wins.
/// Unrelated parameters and the fragment retain their bytes and order.
pub(crate) fn internal(host: &str, address: &str) -> (Internal, String) {
    let mut settings = Internal::default();
    let (before_fragment, fragment) = address.split_once('#').unwrap_or((address, ""));
    let Some((path, query)) = before_fragment.split_once('?') else { return (settings, address.to_owned()) };
    let loopback = host == "127.0.0.1" || host == "[::1]";
    let mut removed = false;
    let mut kept = Vec::new();
    for parameter in query.split('&') {
        let (name, value) = parameter.split_once('=').unwrap_or((parameter, ""));
        let (setting, permitted) = match name {
            "internal-fixed-start" => {
                settings.fixed_start = if loopback { FixedStart::read(value.as_bytes()) } else { None };
                removed = true;
                continue;
            }
            "internal-record" => (&mut settings.record, true),
            "internal-small-window" => (&mut settings.small_window, loopback),
            "internal-loopback-navigation" => (&mut settings.loopback_navigation, loopback),
            _ => {
                kept.push(parameter);
                continue;
            }
        };
        removed = true;
        *setting = permitted && value == "1";
    }
    if !removed {
        return (settings, address.to_owned());
    }
    let mut cleaned = path.to_owned();
    let query = kept.join("&");
    if !query.is_empty() {
        cleaned.push('?');
        cleaned.push_str(&query);
    }
    if address.contains('#') {
        cleaned.push('#');
        cleaned.push_str(fragment);
    }
    (settings, cleaned)
}

/// Only the page's literal `on` content enables recording.
pub(crate) fn record_meta(content: Option<&str>) -> bool {
    content == Some("on")
}

#[cfg(test)]
mod tests {
    use super::{Internal, internal, record_meta};

    #[test]
    fn fixed_start_is_loopback_only_and_always_removed() {
        let start = skein_lib::FixedStart { wall: skein_lib::Wall::from_nanos(1_767_690_000_000_000_000), seed: 7 };
        for (host, query, expected) in [
            ("127.0.0.1", "internal-fixed-start=1767690000,7", Some(start)),
            ("[::1]", "internal-fixed-start=1767690000,7", Some(start)),
            ("example.com", "internal-fixed-start=1767690000,7", None),
            ("localhost", "internal-fixed-start=1767690000,7", None),
            ("127.0.0.1.evil", "internal-fixed-start=1767690000,7", None),
            ("127.0.0.1", "internal-fixed-start=bad", None),
            ("127.0.0.1", "internal-fixed-start=01767690000,7", None),
            ("127.0.0.1", "internal-fixed-start=18446744073709551615,7", None),
            ("127.0.0.1", "internal-fixed-start=", None),
            ("127.0.0.1", "internal-fixed-start", None),
            ("127.0.0.1", "internal-fixed-start=bad&internal-fixed-start=1767690000,7", Some(start)),
            ("127.0.0.1", "internal-fixed-start=1767690000,7&internal-fixed-start=bad", None),
        ] {
            let address = format!("/welcome?z=%20&{query}&internal-record=1&a=2&a=1#part");
            let (settings, cleaned) = internal(host, &address);
            assert_eq!(settings.fixed_start, expected, "{host}: {query}");
            assert!(settings.record);
            assert_eq!(cleaned, "/welcome?z=%20&a=2&a=1#part");
        }
    }

    #[test]
    fn only_on_meta_enables_recording() {
        let rows = [
            (Some("on"), true),
            (Some(""), false),
            (Some("ON"), false),
            (Some("1"), false),
            (Some("off"), false),
            (None, false),
        ];
        for (content, expected) in rows {
            assert_eq!(record_meta(content), expected, "{content:?}");
        }
    }

    #[test]
    fn address_or_meta_enables_recording() {
        let rows = [
            (false, None, false),
            (true, None, true),
            (false, Some("on"), true),
            (true, Some("on"), true),
            (true, Some("off"), true),
        ];
        for (record, content, expected) in rows {
            let settings = Internal { record, ..Internal::default() };
            assert_eq!(settings.recording(content), expected, "record={record}, meta={content:?}");
        }
    }

    #[test]
    fn settings_are_taken_once_with_only_record_on_any_host_and_removed_everywhere() {
        let rows = [
            ("127.0.0.1", "/welcome?at=model&internal-small-window=1", [false, true, false], "/welcome?at=model"),
            ("example.com", "/?internal-record=1&internal-loopback-navigation=1", [true, false, false], "/"),
            (
                "example.com",
                "/?internal-record=1&internal-small-window=1&internal-loopback-navigation=1",
                [true, false, false],
                "/",
            ),
            (
                "example.com",
                "/?internal-record=1&internal-record=0&internal-small-window=1&internal-loopback-navigation=1",
                [false; 3],
                "/",
            ),
            ("[::1]", "/?internal-record=1&internal-small-window=1&internal-loopback-navigation=1", [true; 3], "/"),
            ("localhost", "/?internal-record=1", [true, false, false], "/"),
            ("127.0.0.1.evil", "/?internal-small-window=1", [false; 3], "/"),
            (
                "127.0.0.1",
                "/?internal-record=true&internal-small-window=&internal-loopback-navigation=01",
                [false; 3],
                "/",
            ),
            ("127.0.0.1", "/?internal-record", [false; 3], "/"),
            (
                "127.0.0.1",
                "/?z=%20&internal-record=1&a=2&a=1#part?internal-small-window=1",
                [true, false, false],
                "/?z=%20&a=2&a=1#part?internal-small-window=1",
            ),
            ("127.0.0.1", "/?internal-record=1#", [true, false, false], "/#"),
            ("127.0.0.1", "/?internal-record=1&internal-record=0", [false; 3], "/"),
            ("127.0.0.1", "/?internal-record=0&internal-record=1", [true, false, false], "/"),
            ("127.0.0.1", "/welcome?at=model#part", [false; 3], "/welcome?at=model#part"),
            ("127.0.0.1", "/welcome?#", [false; 3], "/welcome?#"),
            ("[::1]", "/welcome#part?internal-record=1", [false; 3], "/welcome#part?internal-record=1"),
        ];
        for (host, address, [record, small_window, loopback_navigation], cleaned) in rows {
            let expected = Internal { record, small_window, loopback_navigation, ..Internal::default() };
            assert_eq!(internal(host, address), (expected, cleaned.to_owned()), "{host}: {address}");
            assert_eq!(internal(host, cleaned), (Internal::default(), cleaned.to_owned()), "cleaned: {cleaned}");
        }
    }
}
