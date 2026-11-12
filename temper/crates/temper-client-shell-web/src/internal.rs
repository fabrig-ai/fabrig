//! The page's internal settings (client/web/README.md, 2.1).

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Internal {
    pub record: bool,
    pub small_window: bool,
    pub loopback_navigation: bool,
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
        let setting = match name {
            "internal-record" => &mut settings.record,
            "internal-small-window" => &mut settings.small_window,
            "internal-loopback-navigation" => &mut settings.loopback_navigation,
            _ => {
                kept.push(parameter);
                continue;
            }
        };
        removed = true;
        *setting = loopback && value == "1";
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

#[cfg(test)]
mod tests {
    use super::{Internal, internal};

    #[test]
    fn settings_are_taken_once_only_on_loopback_and_removed_everywhere() {
        let rows = [
            ("127.0.0.1", "/welcome?at=model&internal-small-window=1", [false, true, false], "/welcome?at=model"),
            ("example.com", "/?internal-record=1&internal-loopback-navigation=1", [false; 3], "/"),
            ("[::1]", "/?internal-record=1&internal-small-window=1&internal-loopback-navigation=1", [true; 3], "/"),
            ("localhost", "/?internal-record=1", [false; 3], "/"),
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
            let expected = Internal { record, small_window, loopback_navigation };
            assert_eq!(internal(host, address), (expected, cleaned.to_owned()), "{host}: {address}");
            assert_eq!(internal(host, cleaned), (Internal::default(), cleaned.to_owned()), "cleaned: {cleaned}");
        }
    }
}
