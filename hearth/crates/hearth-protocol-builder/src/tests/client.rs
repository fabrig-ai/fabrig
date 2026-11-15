use super::fixtures::{self, CLIENT, Layer};
use crate::{
    Asked, Form,
    builder::{
        render::vocabulary::Outcome,
        vocabulary::{Answer, Request},
    },
    limits,
};
use skein_io as io;
use skein_lib::{
    ReplyTo, Token,
    stream::{Down, Fault, Up},
};

#[test]
fn a_store_error_names_the_store_in_the_answer() {
    use crate::render::errors::{Class, Error, Kind, Step, StoreError, StoreFault};
    use alloc::boxed::Box;

    for (form, expected) in [
        (Form::Text, "the store /store: its write failed\n"),
        (
            Form::JsonLines,
            concat!(
                "{\"record\":\"version\",\"client\":1}\n",
                "{\"record\":\"error\",\"kind\":\"store\",\"store\":\"/store\",\"step\":\"write\",\"class\":\"operational\"}\n"
            ),
        ),
    ] {
        let mut layer = Layer::with_form(form, limits::SHIPPED);
        layer.bind_output();
        layer.client(Request::Answer(Answer::Error(Error {
            kind: Kind::Store(StoreError { store: Box::new([]), fault: StoreFault::Failed { step: Step::Write } }),
            class: Class::Operational,
            side: None,
            at: None,
            also: None,
            target: None,
            task: None,
        })));
        let requests = layer.printed();
        let [Down::Demand { .. }, Down::Send(bytes)] = requests.as_ref() else {
            panic!("the error is printed");
        };
        assert_eq!(bytes.as_ref(), expected.as_bytes(), "{form:?}");
    }
}

#[test]
fn a_non_store_error_passes_through_unchanged() {
    use crate::render::{
        diff::Side,
        errors::{Class, Error, Guard, Kind, Place},
        revision::Position,
    };
    use skein_lib::bytes::copy_of;

    let error = Error {
        kind: Kind::Guard(Guard { name: copy_of(b"ready"), message: copy_of(b"not ready") }),
        class: Class::Configuration,
        side: Some(Side::To),
        at: Some(Place { path: copy_of(b"tasks/check.json"), position: Some(Position { line: 2, column: 3 }) }),
        also: Some(Place { path: copy_of(b"tasks/other.json"), position: None }),
        target: Some(copy_of(b"host")),
        task: Some(copy_of(b"check")),
    };
    for form in [Form::Text, Form::JsonLines] {
        let expected = crate::records::form::print(
            form,
            &crate::records::records::Record::Error(error.clone()),
            None,
            &crate::records::limits::SHIPPED,
        )
        .expect("the error fits");
        let mut layer = Layer::with_form(form, limits::SHIPPED);
        layer.bind_output();
        layer.client(Request::Answer(Answer::Error(error.clone())));
        let requests = layer.printed();
        let [Down::Demand { .. }, Down::Send(bytes)] = requests.as_ref() else {
            panic!("the error is printed");
        };
        assert_eq!(bytes.as_ref(), expected.as_ref(), "{form:?}");
    }
}

#[test]
fn the_terminal_record_follows_the_last_spec_then_the_stream_finishes() {
    let mut layer = Layer::with_form(Form::JsonLines, limits::SHIPPED);
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    drop(layer.printed());
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Done, diff: None, left_out: 0 });
    let requests = layer.printed();
    let [Down::Demand { read: _, room: _ }, Down::Send(bytes), Down::Finish] = requests.as_ref() else {
        panic!("terminal is printed before Finish");
    };
    assert_eq!(
        &**bytes, b"{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}\n",
        "terminal follows the spec without a second version"
    );
}
#[test]
fn a_broken_output_exits_75_after_the_terminal_reply_and_close() {
    let mut layer = Layer::new();
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    let Some(Asked::Io(io::Request::Stream { stream: CLIENT, down: Down::Demand { .. } })) = layer.asked() else {
        panic!("pending output room");
    };
    layer.up(io::Event::Stream { owner: CLIENT, up: Up::Failed(Fault::Other) });
    drop(layer.printed());
    assert_eq!(layer.code(), None, "output failure still waits for the call's reply");
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Usage, diff: None, left_out: 0 });
    assert_eq!(layer.code(), Some(75), "a broken standard output has code 75");
}
const TERMINALS: &[(Outcome, u8)] =
    &[(Outcome::Done, 0), (Outcome::Failed, 2), (Outcome::Usage, 64), (Outcome::TryAgain, 75)];
#[test]
fn a_terminal_record_selects_the_exit_code_after_its_stream_closes() {
    for (outcome, expected) in TERMINALS {
        let mut layer = Layer::new();
        layer.bind_output();
        layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: *outcome, diff: None, left_out: 0 });
        assert_eq!(layer.code(), None, "the stream still owns resources");
        drop(layer.printed());
        assert_eq!(layer.code(), Some(*expected), "terminal {outcome:?}");
    }
}
#[test]
fn a_refused_spec_is_left_out_and_the_call_ends_failed() {
    let mut limits = limits::SHIPPED;
    limits.printer.record = 1;
    let mut layer = Layer::with_form(Form::Text, limits);
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    assert_eq!(layer.printed().as_ref(), &[], "a refused spec prints no bytes");
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Done, diff: None, left_out: 0 });
    drop(layer.printed());
    assert_eq!(layer.code(), Some(2), "refusing the spec makes the call Failed");
}

#[test]
fn rendered_left_out_reaches_the_terminal_record() {
    let mut layer = Layer::with_form(Form::JsonLines, limits::SHIPPED);
    layer.bind_output();
    layer.client(Request::Reply {
        to: ReplyTo::new(Token::new(0)),
        outcome: Outcome::Failed,
        diff: None,
        left_out: 44,
    });
    let requests = layer.printed();
    let [Down::Demand { .. }, Down::Send(bytes), Down::Finish] = requests.as_ref() else {
        panic!("terminal then Finish");
    };
    assert_eq!(
        bytes.as_ref(),
        b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"outcome\",\"outcome\":\"failed\",\"left_out\":44}\n"
    );
}

#[test]
fn progress_prints_on_standard_error_and_answers_on_standard_output() {
    let mut layer = Layer::new();
    layer.bind_output();
    layer.bind_error();
    layer.client(Request::Progress(crate::builder::vocabulary::Progress::Waiting));
    let Some(Asked::Io(io::Request::Stream { stream, down: Down::Demand { .. } })) = layer.asked() else {
        panic!("notice demand")
    };
    assert_eq!(stream, fixtures::ERROR);
    layer.up(io::Event::Stream { owner: stream, up: Up::Room });
    assert_eq!(&*layer.printed(), &[Down::Send(b"waiting for the store's lock: /store\n".as_slice().into())]);
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    let Some(Asked::Io(io::Request::Stream { stream, down: Down::Demand { .. } })) = layer.asked() else {
        panic!("answer demand")
    };
    assert_eq!(stream, CLIENT);
}
