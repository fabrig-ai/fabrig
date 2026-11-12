use crate::tokens;
use skein_lib::Token;

#[test]
fn no_two_component_prefixes_meet() {
    let owners = [
        tokens::WORKER_LISTENERS,
        tokens::WORKER_CONNECTIONS,
        tokens::ACCOUNTS,
        tokens::CLIENTS,
        tokens::SERVER,
        tokens::FORGE,
        tokens::GIT,
        tokens::STORE[0],
        tokens::STORE[1],
        tokens::STORE[2],
    ];
    for (index, owner) in owners.iter().enumerate() {
        for other in owners.iter().skip(index + 1) {
            assert_ne!(tokens::prefix(Token::new(*owner)), tokens::prefix(Token::new(*other)), "{owner:x}, {other:x}");
        }
    }
    assert_eq!(tokens::prefix(Token::new(tokens::SIGNALS)), 0);
    for index in [0, 1, 4094] {
        assert_eq!(tokens::worker(index), Some((3_u64 << 60_u32) + (u64::from(index + 1) << 48_u32)));
        assert_eq!(tokens::git(index), Some((6_u64 << 60_u32) + (u64::from(index + 1) << 48_u32)));
    }
    assert_ne!(tokens::worker(0), Some(tokens::ACCOUNTS));
    for index in [4095, 4096, u32::MAX] {
        assert_eq!(tokens::worker(index), None, "{index}");
        assert_eq!(tokens::git(index), None, "{index}");
    }
}
