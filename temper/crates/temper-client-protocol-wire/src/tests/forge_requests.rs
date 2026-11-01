//! Forge choices and playground requests keep the family's bounds and tags (client/README.md, 4.6).
use crate::v1;
use alloc::{boxed::Box, vec};
use skein_lib::{Reader, Writer};

fn forgejo(api: usize, git: usize, web: Option<usize>) -> v1::ForgeChoice {
    let web = match web {
        Some(length) => Some(vec![b'w'; length].into_boxed_slice()),
        None => None,
    };
    v1::ForgeChoice::Forgejo(
        v1::ForgejoChoice::new(
            &v1::CEILINGS,
            v1::ForgejoChoiceParts {
                api: vec![b'a'; api].into_boxed_slice(),
                git: vec![b'g'; git].into_boxed_slice(),
                web,
            },
        )
        .expect("addresses within ceilings"),
    )
}

fn make(choice: v1::ForgeChoice) -> v1::Request {
    v1::Request::MakeForge(v1::MakeForge::new(&v1::CEILINGS, v1::MakeForgeParts { choice }).expect("forge choice"))
}

fn request(request: v1::Request) -> v1::Requesting {
    v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request }).expect("request within ceilings")
}

#[test]
fn make_forge_round_trips_each_choice() {
    for choice in [
        v1::ForgeChoice::Owned,
        v1::ForgeChoice::Github,
        forgejo(1, 1, None),
        forgejo(1, 1, Some(1)),
        forgejo(259, 1024, None),
        forgejo(259, 1024, Some(1024)),
    ] {
        let record = request(make(choice));
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), &v1::CEILINGS), Some(record));
    }
    for (api, git, web, path) in [
        (260, 1024, 1024, v1::Path::ForgejoChoiceApi),
        (259, 1025, 1024, v1::Path::ForgejoChoiceGit),
        (259, 1024, 1025, v1::Path::ForgejoChoiceWeb),
    ] {
        assert_eq!(
            v1::ForgejoChoice::new(
                &v1::CEILINGS,
                v1::ForgejoChoiceParts {
                    api: vec![b'a'; api].into_boxed_slice(),
                    git: vec![b'g'; git].into_boxed_slice(),
                    web: Some(vec![b'w'; web].into_boxed_slice()),
                },
            ),
            Err(v1::Problem { path, reason: skein_codec::Reason::Bound })
        );
    }
}

#[test]
fn make_forge_with_an_empty_address_is_refused_on_decode() {
    for choice in [forgejo(0, 1, None), forgejo(1, 0, None), forgejo(1, 1, Some(0))] {
        let record = request(make(choice));
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), &v1::CEILINGS), None);
    }
}

#[test]
fn remove_forge_round_trips_and_refuses_forge_zero() {
    for forge in [0, 1, u16::MAX] {
        let record = request(v1::Request::RemoveForge(
            v1::RemoveForge::new(&v1::CEILINGS, v1::RemoveForgeParts { forge }).expect("forge number"),
        ));
        assert_eq!(
            crate::decode_requesting(&crate::requesting(&record), &v1::CEILINGS),
            if forge == 0 { None } else { Some(record) }
        );
    }
}

#[test]
fn end_and_start_playground_round_trip() {
    for value in [v1::Request::StartPlayground, v1::Request::EndPlayground] {
        let record = request(value);
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), &v1::CEILINGS), Some(record));
    }
}

#[test]
fn the_new_outcomes_round_trip() {
    for outcome in [
        v1::Outcome::Made(v1::MadeForge::new(&v1::CEILINGS, v1::MadeForgeParts { forge: 1 }).expect("forge")),
        v1::Outcome::Made(v1::MadeForge::new(&v1::CEILINGS, v1::MadeForgeParts { forge: u16::MAX }).expect("forge")),
        v1::Outcome::Full,
        v1::Outcome::NotOffered,
        v1::Outcome::Removing,
        v1::Outcome::Playground,
        v1::Outcome::Reset,
    ] {
        let record = v1::Answer::new(&v1::CEILINGS, v1::AnswerParts { outcome }).expect("answer");
        let bytes = crate::answer(&record);
        let mut reader = Reader::new(&bytes);
        assert_eq!(v1::Answer::decode(&v1::CEILINGS, &mut reader), Ok(record));
        assert!(reader.is_empty());
    }
}

#[test]
fn renamed_variants_keep_their_bytes() {
    let mut writer = Writer::new(1);
    v1::Request::StartPlayground.encode(&mut writer).expect("request tag");
    assert_eq!(writer.finish(), Box::from([2]));
    let mut writer = Writer::new(1);
    v1::PlaygroundState::Resetting.encode(&mut writer).expect("state tag");
    assert_eq!(writer.finish(), Box::from([2]));
    let mut writer = Writer::new(1);
    v1::Read::Playground.encode(&mut writer).expect("read tag");
    assert_eq!(writer.finish(), Box::from([0]));
}
