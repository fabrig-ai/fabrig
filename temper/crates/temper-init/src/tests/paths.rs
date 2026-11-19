use super::fixtures::{self, DIRECTORIES};
use crate::{Directories, Fact, Facts, Reason, Refusal, files};
use alloc::boxed::Box;
use skein_lib::bytes;

fn absolute(length: usize) -> Box<[u8]> {
    let mut path = bytes::zeroed(length);
    path.fill(b'x');
    *path.first_mut().expect("a nonempty test path") = b'/';
    path
}

fn assign<'a>(facts: &mut Facts<'a>, directories: &mut Directories<'a>, fact: Fact, path: &'a [u8]) {
    match fact {
        Fact::Binary => facts.binary = path,
        Fact::Git => facts.git = Some(path),
        Fact::Claude => facts.claude = Some(path),
        Fact::Config => directories.config = path,
        Fact::Data => directories.data = path,
    }
}

#[test]
fn a_fact_is_refused_naming_it() {
    let long = absolute(4097);
    let mut nontext_long = long.clone();
    *nontext_long.last_mut().expect("the path has 4097 bytes") = 0xff;
    let facts_in_order = [Fact::Binary, Fact::Git, Fact::Claude, Fact::Config, Fact::Data];
    for fact in facts_in_order {
        for (row, reason, path) in [
            ("relative", Reason::Relative, b"relative".as_slice()),
            ("empty", Reason::Relative, b"".as_slice()),
            ("text", Reason::Text, b"/\xff".as_slice()),
            ("bound", Reason::Bound, long.as_ref()),
            ("relative before text", Reason::Relative, b"\xff".as_slice()),
            ("text before bound", Reason::Text, nontext_long.as_ref()),
        ] {
            let mut facts = fixtures::facts();
            let mut directories = DIRECTORIES;
            assign(&mut facts, &mut directories, fact, path);
            assert_eq!(
                crate::bare(&fixtures::bare_facts(&facts), &directories, &[0; 32]),
                Err(Refusal { fact, reason }),
                "{row}: {fact:?}"
            );
            assert_eq!(files(&facts, &directories, &[0; 32]), Err(Refusal { fact, reason }), "{row}: {fact:?}");
        }
    }
    let data = absolute(4083);
    {
        let (row, fact, path) = ("secret joined past bound", Fact::Data, data.as_ref());
        let mut facts = fixtures::facts();
        let mut directories = DIRECTORIES;
        assign(&mut facts, &mut directories, fact, path);
        assert_eq!(
            crate::bare(&fixtures::bare_facts(&facts), &directories, &[0; 32]),
            Err(Refusal { fact, reason: Reason::Bound }),
            "{row}"
        );
        assert_eq!(files(&facts, &directories, &[0; 32]), Err(Refusal { fact, reason: Reason::Bound }), "{row}");
    }
    for (index, first) in facts_in_order.iter().enumerate() {
        for second in facts_in_order.iter().skip(index.checked_add(1).expect("there are six facts")) {
            let mut facts = fixtures::facts();
            let mut directories = DIRECTORIES;
            assign(&mut facts, &mut directories, *first, b"/\xff");
            assign(&mut facts, &mut directories, *second, b"relative");
            assert_eq!(
                files(&facts, &directories, &[0; 32]),
                Err(Refusal { fact: *first, reason: Reason::Text }),
                "{first:?} before {second:?}"
            );
        }
    }
}

#[test]
fn paths_join_once_and_accept_the_bound() {
    let long = absolute(4096);
    for fact in [Fact::Binary, Fact::Git, Fact::Claude] {
        let mut facts = fixtures::facts();
        let mut directories = DIRECTORIES;
        assign(&mut facts, &mut directories, fact, &long);
        files(&facts, &directories, &[0; 32]).expect("4096 bytes is within the bound");
    }
    for (row, config_length, data_length, slash) in
        [("joined bound", 4096, 4082, false), ("joined bound, trailing slash", 4096, 4083, true)]
    {
        let mut config = absolute(config_length);
        let mut data = absolute(data_length);
        if slash {
            *config.last_mut().expect("the directory is nonempty") = b'/';
            *data.last_mut().expect("the directory is nonempty") = b'/';
        }
        files(&fixtures::facts(), &Directories { config: &config, data: &data }, &[0; 32]).expect(row);
    }
    // Text paths retain their bytes, including control bytes.
    let mut controls = long;
    controls.get_mut(1..).expect("the path has 4096 bytes").fill(1);
    let facts = Facts { binary: &controls, git: Some(&controls), claude: Some(&controls), ..fixtures::facts() };
    let mut config = absolute(4084);
    config.get_mut(1..).expect("the configuration directory has 4085 bytes").fill(1);
    let mut data = absolute(4082);
    data.get_mut(1..).expect("the data directory has 4081 bytes").fill(1);
    let directories = Directories { config: &config, data: &data };
    let answered = files(&facts, &directories, &[0; 32]).expect("bounded text paths are accepted");
    let crate::Content::Link { target } = &fixtures::file(&answered.files, b"demo/bin/git").content else {
        panic!("fake git launcher")
    };
    assert_eq!(target.as_ref(), controls.as_ref());
}
