use crate::{key_user, user_key};
use jig_domain_people::IdentityKey;

#[test]
fn forge_user_keys_name_one_forge_and_user() {
    assert_eq!(user_key(1, 7).provider, 0);
    assert_eq!(user_key(1, 7).subject.as_ref(), &[0, 1, 0, 0, 0, 0, 0, 0, 0, 7]);
    for (key, expected) in [
        (user_key(1, 7), Some((1, 7))),
        (user_key(3, 7), Some((3, 7))),
        (user_key(u16::MAX, u64::MAX), Some((u16::MAX, u64::MAX))),
        (IdentityKey { provider: 0, subject: Box::new([0; 8]) }, None),
        (IdentityKey { provider: 0, subject: Box::new([0; 11]) }, None),
        (IdentityKey { provider: 1, subject: user_key(1, 7).subject }, None),
    ] {
        assert_eq!(key_user(&key), expected, "key {key:?}");
    }
}
