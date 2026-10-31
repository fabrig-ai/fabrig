use crate::harness::{BROWSER, PROCESS, SERVER, closing::Closing};
use skein_browser::boundary::Request;
use skein_io::{
    self as io,
    kernel::{Exit, Signal, Target},
};
use skein_lib::{Queue, Token};

const BROWSER_CHILD: Token = Token::new(1000);
const SERVER_CHILD: Token = Token::new(1001);

fn spawned(closing: &mut Closing, owner: Token, child: Token, requests: &mut Queue<io::Request>) {
    closing.io_event(&io::Event::Spawned { owner, child, pipes: Box::new([]) }, requests);
}
fn running() -> Closing {
    let mut closing = Closing::new();
    let mut requests = Queue::uncapped();
    spawned(&mut closing, SERVER, SERVER_CHILD, &mut requests);
    spawned(&mut closing, PROCESS, BROWSER_CHILD, &mut requests);
    assert!(requests.is_empty());
    closing
}
fn terminate(child: Token) -> io::Request {
    io::Request::Signal { child, signal: Signal::Terminate, to: Target::Child }
}
fn exited(closing: &mut Closing, requests: &mut Queue<io::Request>) {
    closing.io_event(&io::Event::Exited { owner: PROCESS, exit: Exit::Code(0) }, requests);
}

#[test]
fn closing_ends_the_browser_before_its_server() {
    let mut closing = running();
    let mut requests = Queue::uncapped();
    assert_eq!(closing.begin(false), Some(Request::Close { entity: BROWSER }));
    closing.advance(&mut requests);
    assert!(requests.is_empty(), "asking Browser.close does not signal either child");

    closing.browser_closed(Token::new(100), &mut requests);
    assert!(requests.is_empty(), "a page's Closed does not end the browser child");
    closing.browser_closed(BROWSER, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(BROWSER_CHILD)));
    closing.browser_closed(BROWSER, &mut requests);
    closing.advance(&mut requests);
    assert!(requests.is_empty(), "the server stays up while the browser child is exiting");

    exited(&mut closing, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(SERVER_CHILD)));
    closing.io_event(&io::Event::Closed { owner: PROCESS }, &mut requests);
    closing.advance(&mut requests);
    assert!(requests.is_empty(), "the server is ended once");
}

#[test]
fn a_browser_that_never_closes_is_ended_before_its_server() {
    let mut closing = running();
    let mut requests = Queue::uncapped();
    assert_eq!(closing.begin(false), Some(Request::Close { entity: BROWSER }));
    assert!(requests.is_empty());

    closing.force(&mut requests);
    assert_eq!(requests.pop(), Some(terminate(BROWSER_CHILD)));
    assert_eq!(requests.pop(), Some(io::Request::Close { entity: BROWSER_CHILD }));
    closing.force(&mut requests);
    closing.browser_closed(BROWSER, &mut requests);
    assert!(requests.is_empty(), "forced closing still awaits the browser child before ending the server");

    exited(&mut closing, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(SERVER_CHILD)));
    assert!(requests.is_empty());
}

#[test]
fn a_browser_exit_can_arrive_before_its_closed_notification() {
    let mut closing = running();
    let mut requests = Queue::uncapped();
    assert_eq!(closing.begin(false), Some(Request::Close { entity: BROWSER }));
    exited(&mut closing, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(SERVER_CHILD)));
    closing.browser_closed(BROWSER, &mut requests);
    assert!(requests.is_empty(), "an exited browser does not need another signal");
}

#[test]
fn forced_closing_waits_for_a_browser_still_spawning() {
    let mut closing = Closing::new();
    let mut requests = Queue::uncapped();
    assert_eq!(closing.begin(false), Some(Request::Close { entity: BROWSER }));
    closing.force(&mut requests);
    spawned(&mut closing, SERVER, SERVER_CHILD, &mut requests);
    assert!(requests.is_empty(), "the server stays up until the pending browser spawn exits");
    spawned(&mut closing, PROCESS, BROWSER_CHILD, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(BROWSER_CHILD)));
    assert_eq!(requests.pop(), Some(io::Request::Close { entity: BROWSER_CHILD }));
    assert!(requests.is_empty());
    exited(&mut closing, &mut requests);
    assert_eq!(requests.pop(), Some(terminate(SERVER_CHILD)));
    assert!(requests.is_empty());
}
