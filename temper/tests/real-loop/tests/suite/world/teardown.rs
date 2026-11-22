//! Failed startup settles the caller before reporting its failure.
use std::{net::TcpListener, rc::Rc};
use temper_real_loop_world::testing::{Receipt, startup};

#[test]
fn a_service_exit_before_its_link_fails_after_the_caller_settles() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("occupied listener");
    let scratch = skein_scratch::Scratch::new("failed-start");
    let receipt = Rc::new(Receipt::default());
    // The chooser's block claim outlives the story and belongs in its descriptor baseline.
    drop(temper_fixtures::ports::reserve::<1>(&[]));
    let descriptors = std::fs::read_dir("/dev/fd").expect("own descriptors").count();
    let Err(message) = startup(&scratch, Some(&listener), receipt.clone()) else {
        panic!("a service exit before its link must fail the story")
    };
    assert!(message.contains("service 1 exited before caller release: Code(1)"), "{message}");
    assert!(
        message.contains(&format!(
            "stderr tail:\ntemper serve: client listener {} failed: other\n",
            listener.local_addr().expect("held address")
        )),
        "{message}"
    );
    assert!(!receipt.passed.get(), "failed startup never passes the referee");
    assert!(receipt.settled.get(), "the failure is reported after every host empties");
    assert_eq!(receipt.calls.get(), 0, "no CLI command ran before release");
    assert_eq!(
        std::fs::read_dir("/dev/fd").expect("own descriptors").count(),
        descriptors,
        "failed startup releases every descriptor"
    );
}
