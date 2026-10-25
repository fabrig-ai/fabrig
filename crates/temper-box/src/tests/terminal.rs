use crate::terminal::{Fact, Formatter, Item, Target};
use skein_lib::{Wall, bytes};
fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap())
}
fn item(target: Target, words: &[u8]) -> Item {
    Item { target, bytes: bytes::copy_of(words) }
}
#[test]
fn the_first_block_has_exact_words_and_three_targets() {
    let formatter = Formatter::new();
    assert_eq!(formatter.block(bytes::copy_of(b"http://link\n"), wall(50)).as_slice(), &[
        item(Target::Stderr, b"\nOpen this link in a browser on this machine. It signs you in as this\ntemper's owner, once and for a short while, so keep it private.\nOpened again once spent, it prints a fresh one here.\n\n"),
        item(Target::Stdout, b"http://link\n"),
        item(Target::Stderr, b"\nCtrl-C stops temper.\n\n"),
    ]);
}
#[test]
fn fresh_blocks_keep_state_across_restart_and_use_supplied_utc() {
    let mut formatter = Formatter::new();
    formatter.admitted_block();
    for (seconds, expected) in [
        (86_399, b"23:59:59Z A fresh sign-in link:\n".as_slice()),
        (86_400, b"00:00:00Z A fresh sign-in link:\n"),
        (60, b"00:01:00Z A fresh sign-in link:\n"),
        (172_861, b"00:01:01Z A fresh sign-in link:\n"),
    ] {
        assert_eq!(
            formatter.block(bytes::copy_of(b"http://fresh\n"), wall(seconds)).as_slice(),
            &[item(Target::Stderr, expected), item(Target::Stdout, b"http://fresh\n"),]
        );
    }
}
#[test]
fn lifecycle_facts_are_untimed_until_a_complete_block_is_admitted() {
    let mut formatter = Formatter::new();
    for (fact, words, timed) in [
        (Fact::Starting { step: 3 }, b"Starting serve\n".as_slice(), b"01:02:03Z Starting serve\n".as_slice()),
        (Fact::Stopping, b"Stopping\n", b"01:02:03Z Stopping\n"),
        (Fact::Stopped, b"Stopped\n", b"01:02:03Z Stopped\n"),
    ] {
        assert_eq!(formatter.fact(fact, b"serve", wall(3_723)), item(Target::Stderr, words));
        let mut after = Formatter::new();
        after.admitted_block();
        assert_eq!(after.fact(fact, b"serve", wall(3_723)), item(Target::Stderr, timed));
    }
    let refused = formatter.notice(crate::terminal::REFUSAL, wall(3_723));
    assert_eq!(refused, item(Target::Stderr, b"box: serve: sign-in link too long\n"));
    // Neither formatting a refusal nor constructing a block marks admission.
    drop(formatter.block(bytes::copy_of(b"held\n"), wall(3_723)));
    assert_eq!(formatter.block_entries(), 3);
    formatter.admitted_block();
    assert_eq!(formatter.notice(b"box: failure\n", wall(3_723)), item(Target::Stderr, b"01:02:03Z box: failure\n"));
}
