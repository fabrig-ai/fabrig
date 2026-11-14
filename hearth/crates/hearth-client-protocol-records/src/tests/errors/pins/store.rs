use super::assert_prints;
use super::plain;
use crate::errors::{Class, Error, Kind, Step, StoreError, StoreFault, StoreLimit};
use skein_lib::bytes::copy_of;
fn store(fault: StoreFault) -> Error {
    let mut error = plain(Kind::Store(StoreError { store: copy_of(b"/store"), fault }));
    error.class = Class::Operational;
    error
}

#[test]
fn store_bytes() {
    let cases = [
        (
            store(StoreFault::Failed { step: Step::Write }),
            "the store /store: its write failed\n",
            concat!(r#"{"record":"error","kind":"store","store":"/store","step":"write","class":"operational"}"#, "\n"),
        ),
        (
            store(StoreFault::Failed { step: Step::List }),
            "the store /store: its list failed\n",
            concat!(r#"{"record":"error","kind":"store","store":"/store","step":"list","class":"operational"}"#, "\n"),
        ),
        (
            store(StoreFault::Failed { step: Step::Remove }),
            "the store /store: its remove failed\n",
            concat!(
                r#"{"record":"error","kind":"store","store":"/store","step":"remove","class":"operational"}"#,
                "\n"
            ),
        ),
        (
            store(StoreFault::Failed { step: Step::Load }),
            "the store /store: its load failed\n",
            concat!(r#"{"record":"error","kind":"store","store":"/store","step":"load","class":"operational"}"#, "\n"),
        ),
        (
            store(StoreFault::Failed { step: Step::Empty }),
            "the store /store: its empty failed\n",
            concat!(r#"{"record":"error","kind":"store","store":"/store","step":"empty","class":"operational"}"#, "\n"),
        ),
        (
            store(StoreFault::Failed { step: Step::Keep }),
            "the store /store: its keep failed\n",
            concat!(r#"{"record":"error","kind":"store","store":"/store","step":"keep","class":"operational"}"#, "\n"),
        ),
        (
            store(StoreFault::Full { limit: StoreLimit::Resolutions, most: 3 }),
            "the store /store is full: 3 resolutions\n",
            concat!(
                r#"{"record":"error","kind":"store","store":"/store","limit":"resolutions","most":3,"class":"operational"}"#,
                "\n"
            ),
        ),
        (
            store(StoreFault::Full { limit: StoreLimit::Layers, most: 4 }),
            "the store /store is full: 4 layers\n",
            concat!(
                r#"{"record":"error","kind":"store","store":"/store","limit":"layers","most":4,"class":"operational"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
