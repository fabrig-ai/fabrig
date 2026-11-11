//! The conditional store's byte admission, replacement and pinned LRU policy.
use super::*;
use crate::connection::field;

fn key(target: &[u8]) -> Key {
    Key { forge: 1, origin: None, identity: Identity::Installation(7), target: Box::from(target), range: None }
}
fn head() -> http::Response {
    http::Response {
        version: skein_http::Version::Http11,
        status: 200,
        headers: Box::new([field(b"ETag", b"\"a\"")]),
        framing: http::Framing::Length(1),
    }
}
fn put(store: &mut Store, target: &[u8], body: &[u8]) {
    let head = head();
    let validators = Validators::of(&head).expect("valid ETag");
    store.keep(key(target), head, validators, Box::from(body));
}
#[derive(Clone, Copy)]
enum Action {
    Put(&'static [u8], &'static [u8]),
    Drop(&'static [u8]),
    Read(&'static [u8]),
    Pin(&'static [u8]),
    Release,
    Forget,
    Reserve,
    Unreserve,
}
type Row = (&'static str, &'static [Action], [Option<&'static [u8]>; 3]);
#[test]
fn conditional_answers_are_bounded_replaced_and_pinned_oldest_out() {
    use Action::{Drop, Forget, Pin, Put, Read, Release, Reserve, Unreserve};
    let response = head();
    let width = Entry::overhead(&key(b"a"), &response, &Validators::of(&response).expect("tag"))
        .checked_add(1)
        .expect("small entry");
    let limit = width.checked_mul(2).expect("two entries");
    let rows: [Row; 13] = [
        ("keep", &[Put(b"a", b"a")], [Some(b"a"), None, None]),
        ("replace", &[Put(b"a", b"a"), Put(b"a", b"b")], [Some(b"b"), None, None]),
        ("validator-less 200 drops", &[Put(b"a", b"a"), Drop(b"a")], [None, None, None]),
        ("oldest out", &[Put(b"a", b"a"), Put(b"b", b"b"), Put(b"c", b"c")], [None, Some(b"b"), Some(b"c")]),
        (
            "use refreshes age",
            &[Put(b"a", b"a"), Put(b"b", b"b"), Read(b"a"), Put(b"c", b"c")],
            [Some(b"a"), None, Some(b"c")],
        ),
        (
            "in flight never out",
            &[Put(b"a", b"a"), Put(b"b", b"b"), Pin(b"a"), Read(b"b"), Put(b"c", b"c")],
            [Some(b"a"), None, Some(b"c")],
        ),
        (
            "all pinned refuses a newcomer",
            &[Put(b"a", b"a"), Pin(b"a"), Put(b"b", b"b"), Pin(b"b"), Put(b"c", b"c")],
            [Some(b"a"), Some(b"b"), None],
        ),
        (
            "released oldest can go",
            &[Put(b"a", b"a"), Pin(b"a"), Put(b"b", b"b"), Release, Put(b"c", b"c")],
            [None, Some(b"b"), Some(b"c")],
        ),
        (
            "pinned replacement cannot displace the sent answer",
            &[Put(b"a", b"a"), Pin(b"a"), Put(b"a", b"b")],
            [Some(b"b"), None, None],
        ),
        (
            "retired pinned bytes are released",
            &[Put(b"a", b"a"), Pin(b"a"), Drop(b"a"), Release, Put(b"b", b"b"), Put(b"c", b"c")],
            [None, Some(b"b"), Some(b"c")],
        ),
        ("pending copy shares the byte bound", &[Put(b"a", b"a"), Reserve, Put(b"b", b"b")], [None, Some(b"b"), None]),
        (
            "pending copy and pin refuse until room returns",
            &[Put(b"a", b"a"), Pin(b"a"), Reserve, Put(b"b", b"b"), Unreserve, Put(b"c", b"c")],
            [Some(b"a"), None, Some(b"c")],
        ),
        ("forge ending drops all", &[Put(b"a", b"a"), Forget], [None, None, None]),
    ];
    for (label, actions, expected) in rows {
        let mut store = Store::new(limit);
        let mut held = None;
        let mut held_body = b"a".as_slice();
        for action in actions {
            match *action {
                Put(target, body) => put(&mut store, target, body),
                Drop(target) => store.drop_key(&key(target)),
                Read(target) => {
                    let id = store.pin(&key(target), &mut List::with_capacity(2)).expect("kept read");
                    store.release(id);
                }
                Pin(target) => {
                    held = store.pin(&key(target), &mut List::with_capacity(2));
                    held_body = target;
                }
                Release => store.release(held.take().expect("pinned entry")),
                Forget => store.forget(1),
                Reserve => assert!(store.reserve(width)),
                Unreserve => store.unreserve(width),
            }
            assert!(
                store.bytes.saturating_add(store.reserved) <= limit,
                "{label}: all keys, heads, validators and bodies fit"
            );
            if let Some(id) = held {
                assert_eq!(store.body(id), Some(held_body), "{label}: sent version stays pinned");
            }
        }
        for (target, expected) in [b"a", b"b", b"c"].into_iter().zip(expected) {
            let got = match store.pin(&key(target), &mut List::with_capacity(2)) {
                Some(id) => store.body(id),
                None => None,
            };
            assert_eq!(got, expected, "{label}: {target:?}");
        }
    }
    for bytes in [limit, limit.checked_add(1).expect("one past")] {
        let mut store = Store::new(limit);
        let head = head();
        let validators = Validators::of(&head).expect("tag");
        let body_len = bytes.checked_sub(Entry::overhead(&key(b"a"), &head, &validators)).expect("body room");
        let body = crate::fixtures::filled(usize::try_from(body_len).expect("small body"), b'x');
        store.keep(key(b"a"), head, validators, body);
        assert_eq!(
            store.pin(&key(b"a"), &mut List::with_capacity(2)).is_some(),
            bytes == limit,
            "entry bound includes overhead"
        );
    }
}
