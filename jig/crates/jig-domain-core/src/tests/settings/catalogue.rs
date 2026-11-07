//! List patterns match whole names and drops win (jig/docs/design/protocol/configuration.md, 3).
use crate::settings::catalogue::{kept, matches};
use jig_domain_accounts::ProviderList;

#[test]
fn catalogue_patterns_match_whole_names_with_backtracking() {
    for (pattern, name, expected) in [
        (b"gpt-*".as_slice(), b"gpt-6-luna".as_slice(), true),
        (b"gpt-*", b"xgpt-6", false),
        (b"*audio*", b"audio-model", true),
        (b"*audio*", b"gpt-audio-model", true),
        (b"*audio*", b"gpt-audio", true),
        (b"*audio*", b"gpt-6", false),
        (b"*-20??-??-??", b"gpt-4o-2024-08-06", true),
        (b"*-20??-??-??", b"gpt-4o-audio", false),
        (b"?", b"a", true),
        (b"?", b"", false),
        (b"?", b"ab", false),
        (b"?", "é".as_bytes(), false),
        (b"??", "é".as_bytes(), true),
        (b"gpt-*", b"gpt-", true),
        (b"*", b"", true),
        (b"", b"", true),
        (b"", b"a", false),
        (b"**a**b*c?d", b"xaaybzzcxd", true),
        (b"*ab*ab?c", b"xabababxc", true),
        (b"*ab*ab?c", b"xabababxd", false),
        (b"a*b", b"abx", false),
        (b"**", b"abc", true),
    ] {
        assert_eq!(matches(pattern, name), expected, "{pattern:?} on {name:?}");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded table fixtures (programming-model.md, 10.2)")]
fn catalogue_drop_patterns_win_over_keeps() {
    for (keep, drop, name, expected) in [
        (&[b"other*".as_slice(), b"gpt-*"][..], &[b"*audio*".as_slice()][..], b"gpt-6".as_slice(), true),
        (&[b"gpt-*".as_slice()][..], &[b"*audio*".as_slice(), b"*preview*".as_slice()][..], b"gpt-audio", false),
        (&[b"gpt-*".as_slice()][..], &[][..], b"other", false),
        (&[][..], &[][..], b"gpt-6", false),
        (&[b"*".as_slice()][..], &[][..], b"gpt-6", true),
        (&[b"gpt-*".as_slice()][..], &[b"*audio*".as_slice(), b"*preview*".as_slice()][..], b"gpt-preview", false),
    ] {
        let list = ProviderList {
            keep: keep.iter().map(|pattern| Box::from(*pattern)).collect(),
            drop: drop.iter().map(|pattern| Box::from(*pattern)).collect(),
        };
        assert_eq!(kept(name, &list), expected, "{list:?} on {name:?}");
    }
}
