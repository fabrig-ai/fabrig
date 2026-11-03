//! Cookie steps against the rules of skein/docs/design/playground.md, 4.
use super::Jar;
use skein_lib::Wall;

#[test]
fn paths_use_directories_and_segment_boundaries() {
    let mut jar = Jar::new(3, 16);
    jar.set(b"a=1", b"/app/set?x=/other", Wall::EPOCH);
    jar.set(b"b=2; Path=/app/", b"/", Wall::EPOCH);
    jar.set(b"c=3; Path=/", b"/", Wall::EPOCH);
    for (path, expected) in
        [(b"/app".as_slice(), b"a=1; c=3".as_slice()), (b"/app/x", b"a=1; b=2; c=3"), (b"/apple", b"c=3")]
    {
        assert_eq!(jar.header(path, Wall::EPOCH).as_deref(), Some(expected));
    }
}
#[test]
fn replacements_clear_and_expire_from_the_answers_wall() {
    let mut jar = Jar::new(2, 16);
    jar.set(b"a=1; Max-Age=2", b"/", Wall::from_nanos(1_000_000_000));
    jar.set(b"b=2", b"/", Wall::EPOCH);
    assert_eq!(jar.header(b"/", Wall::from_nanos(2_999_999_999)).as_deref(), Some(b"a=1; b=2".as_slice()));
    assert_eq!(jar.header(b"/", Wall::from_nanos(3_000_000_000)).as_deref(), Some(b"b=2".as_slice()));
    jar.set(b"a=3; Expires=ignored; HttpOnly; Secure; SameSite=Strict; Domain=ignored", b"/", Wall::EPOCH);
    jar.set(b"b=4; Path=/x", b"/", Wall::EPOCH);
    assert_eq!(jar.header(b"/x", Wall::EPOCH).as_deref(), Some(b"a=3; b=4".as_slice()));
    jar.set(b"a=; Max-Age=0", b"/", Wall::EPOCH);
    jar.set(b"b=; Max-Age=-1", b"/", Wall::EPOCH);
    assert_eq!(jar.header(b"/x", Wall::EPOCH), None);
}
#[test]
fn the_count_and_byte_bounds_refuse_without_replacing() {
    let mut jar = Jar::new(1, 4);
    jar.set(b"a=123", b"/", Wall::EPOCH);
    jar.set(b"b=1", b"/", Wall::EPOCH);
    jar.set(b"a=1234", b"/", Wall::EPOCH);
    assert_eq!(jar.header(b"/", Wall::EPOCH).as_deref(), Some(b"a=123".as_slice()));
    jar.set(b"a=2", b"/", Wall::EPOCH);
    assert_eq!(jar.header(b"/", Wall::EPOCH).as_deref(), Some(b"a=2".as_slice()));
}
