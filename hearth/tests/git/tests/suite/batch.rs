use hearth_git_world::{
    fixtures, generate,
    world::{self, BatchOutcome, Settings},
};
use skein_lib::Rng;
#[test]
fn generated_blobs_arrive_whole_in_order_whatever_the_cuts() {
    let mut empty = false;
    let mut missing = false;
    let mut nul = false;
    for seed in 0..32 {
        let batch = generate::batch(&mut Rng::new(seed), &fixtures::SMALL);
        for blob in &batch.blobs {
            match &blob.bytes {
                Some(bytes) => {
                    empty |= bytes.is_empty();
                    nul |= bytes.contains(&0);
                }
                None => missing = true,
            }
        }
        assert_eq!(
            world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &Settings::cut(seed)),
            BatchOutcome::Done(batch.blobs),
            "seed {seed}"
        );
    }
    assert!(empty && missing && nul, "drawn empty, absent and NUL-containing bodies");
}
#[test]
fn a_close_between_pieces_answers_closed_and_withdraws_its_demand() {
    let batch = generate::batch(&mut Rng::new(5), &fixtures::SMALL);
    for close in 0..32 {
        let settings = Settings { close: Some(close), ..Settings::cut(7) };
        let outcome = world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &settings);
        match outcome {
            BatchOutcome::Done(blobs) => assert_eq!(blobs, batch.blobs, "whole before close"),
            BatchOutcome::Closed => {}
            BatchOutcome::Failed(error) => panic!("unexpected {error:?}"),
        }
    }
}

/// A piece precedes the next read (hearth/docs/design/domain/render/README.md, 5.5).
#[test]
fn the_next_read_waits_until_the_piece_before_it_is_taken() {
    let mut more_than_one_piece = false;
    for seed in 0..32 {
        let batch = generate::batch(&mut Rng::new(seed), &fixtures::SMALL);
        more_than_one_piece |= batch.blobs.iter().any(|blob| {
            blob.bytes
                .as_ref()
                .is_some_and(|bytes| bytes.len() > usize::try_from(fixtures::SMALL.chunk).expect("small chunk"))
        });
        assert_eq!(
            world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &Settings::cut(seed)),
            BatchOutcome::Done(batch.blobs),
            "seed {seed}"
        );
    }
    assert!(more_than_one_piece, "a body larger than a chunk needs more than one piece");
}

/// A held outcome answers the next request (hearth/docs/design/domain/render/README.md, 5.5).
#[test]
fn an_end_or_a_fault_behind_a_piece_answers_the_next_request() {
    use hearth_protocol_git::{
        batch::{self, Batch, Error, Event, Request},
        oid::Oid,
    };
    use skein_lib::{
        Env, Queue, Time, Wall,
        stream::{Down, Fault, Held, Up},
    };

    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::SMALL };
    let oid = Oid::parse(b"ce013625030ba8dba906f756967f9e9ca394464a").expect("literal id");
    for (place, size, whole) in [
        ("paused with bytes left", u64::from(fixtures::SMALL.chunk) + 1, false),
        ("paused with none left", 1, false),
        ("idle after whole", 1, true),
    ] {
        for (outcome, error) in [(Up::End, Error::Truncated), (Up::Failed(Fault::Reset), Error::Stream(Fault::Reset))] {
            let mut machine = Batch::START;
            let mut above = Queue::uncapped();
            let mut below = Queue::uncapped();
            let read = Request::Read { oid, size, max: size };
            let mut stdout = format!("{} blob {size}\n", core::str::from_utf8(oid.hex()).expect("hex")).into_bytes();
            stdout.extend(std::iter::repeat_n(b'x', usize::try_from(size).expect("small body")));
            stdout.push(b'\n');
            let mut held = Held::new(stdout.into_boxed_slice());
            batch::down(&mut machine, &env, read, &mut above, &mut below);
            let mut waiting = false;
            for _ in 0..5 {
                let Some(Down::Demand { read, room: 0 }) = below.pop() else { panic!("{place}: one read demand") };
                assert!(below.is_empty(), "{place}: one demand at a time");
                batch::up(&mut machine, &env, held.answer(read).expect("held delivery"), &mut above, &mut below);
                match above.pop() {
                    Some(Event::Piece(piece)) => {
                        assert_eq!(
                            piece.len(),
                            usize::try_from(size.min(u64::from(fixtures::SMALL.chunk))).expect("piece fits")
                        );
                        assert!(below.is_empty(), "{place}: piece pauses reading");
                        if !whole {
                            waiting = true;
                            break;
                        }
                        batch::down(&mut machine, &env, Request::Next, &mut above, &mut below);
                    }
                    Some(Event::Whole { oid: got, size: got_size }) => {
                        assert!(whole, "{place}: expected a piece");
                        assert_eq!((got, got_size), (oid, size));
                        waiting = true;
                        break;
                    }
                    None => {}
                    other => panic!("{place}: unexpected {other:?}"),
                }
            }
            assert!(waiting, "{place}: reached the waiting state");
            assert!(above.is_empty() && below.is_empty(), "{place}: waiting has no output");
            batch::up(&mut machine, &env, outcome, &mut above, &mut below);
            assert!(above.is_empty(), "{place}: outcome waits for the next request");
            assert!(below.is_empty(), "{place}: held outcome demands nothing");
            batch::down(&mut machine, &env, if whole { read } else { Request::Next }, &mut above, &mut below);
            assert_eq!(above.pop(), Some(Event::Failed(error)), "{place}: next request gets the held outcome");
            assert!(above.is_empty() && below.is_empty(), "{place}: one failure and no demand");
        }
    }
}
