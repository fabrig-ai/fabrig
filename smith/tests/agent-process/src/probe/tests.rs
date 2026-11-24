use super::Probe;
use std::cell::Cell;

// Directory facts are keyed by name. Like Scratch, the handle owns just its own entry.
struct Home<'a> {
    path: &'static str,
    mode: &'a Cell<Option<u32>>,
}

impl Drop for Home<'_> {
    fn drop(&mut self) {
        self.mode.set(None);
    }
}

#[test]
fn live_probes_have_distinct_private_state() {
    let directories = [("/fixture/home-a", Cell::new(None)), ("/fixture/home-b", Cell::new(None))];
    let make_directory = |label: &str, mode| {
        assert_eq!(label, "smith-live");
        let (path, entry) = directories.iter().find(|(_, entry)| entry.get().is_none()).expect("free home");
        entry.set(Some(mode));
        Home { path, mode: entry }
    };
    let first = Probe::with_home(42, 3, make_directory);
    let second = Probe::with_home(42, 4, make_directory);
    assert_ne!(first.home.path, second.home.path);
    assert_ne!(first.session, second.session);
    assert_eq!(first.session.len(), 36);
    assert_eq!(second.session.len(), 36);
    let first_entry = first.home.mode;
    let second_entry = second.home.mode;
    assert_eq!(first_entry.get(), Some(0o700), "first home is a private directory");
    assert_eq!(second_entry.get(), Some(0o700), "second home is a private directory");
    drop(first);
    assert_eq!(first_entry.get(), None, "the first home was removed");
    assert_eq!(second_entry.get(), Some(0o700), "one probe does not remove another's home");
}
