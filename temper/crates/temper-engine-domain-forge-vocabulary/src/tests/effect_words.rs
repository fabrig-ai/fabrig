use super::{REPO, adopted};
use crate::effect_words;
use skein_forge::api::{Error, UpdateStyle, Write};
use skein_forge_client::{Condition, Effect, Entry, Outcome};

#[test]
fn an_update_account_names_a_moved_base_apart_from_a_changed_head() {
    let entry = Entry {
        number: 1,
        task: 2,
        repository: REPO,
        effect: Effect {
            write: Write::Update { number: 9, style: UpdateStyle::Merge },
            condition: Condition::Update { head: [1; 32], base: [2; 32] },
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    for (error, expected) in [
        (Error::BaseMoved { base: Some([4; 32]) }, b"update pull request: base moved; resource org/repo #9".as_slice()),
        (Error::BaseMoved { base: None }, b"update pull request: base moved; resource org/repo #9".as_slice()),
        (Error::Stale, b"update pull request: head changed; resource org/repo #9".as_slice()),
    ] {
        let words = effect_words::account(&entry, Some(&adopted()), Outcome::Failed(error)).expect("failure account");
        assert_eq!(words.as_ref(), expected, "{error:?}");
    }
}
