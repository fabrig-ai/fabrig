//! A tiny inherited-listener fake, reexecuted as an exact startup test.
use skein_io::{
    Event, Io, Request,
    kernel::{Complete, Fd, ServiceSignal, Submit},
};
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Read, Up},
};
use skein_shell::{Clock, Config, Kernel, Wait};

#[expect(clippy::too_many_lines, reason = "one small fake listener loop, reexecuted by the startup tests")]
pub fn serve() -> ! {
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).io;
    let operations = skein_io::operations(&limits).expect("io operations");
    let mut kernel = Kernel::open(Config { operations }).expect("fake kernel");
    let clock = Clock::new();
    let mut io = Io::new(&limits);
    let address = skein_shell::listener_at(Fd::new(3)).expect("inherited listener");
    io.adopt_listener(Fd::new(3), address).expect("adopt listener");
    let signals = io.adopt_signals(skein_shell::open_termination_signals().expect("signals")).expect("adopt signals");
    let mut complete = Queue::<Complete>::with_capacity(operations);
    let mut subs = Queue::<Submit>::with_capacity(operations);
    let mut events = Queue::<Event>::with_capacity(16);
    let now = clock.now();
    skein_io::down(
        &mut io,
        &Env { now: now.now, wall: now.wall, limits },
        Request::Listen { owner: Token::new(1), addr: skein_io::Addr::Ip(address) },
        &mut subs,
    );
    let mut stopping = false;
    let mut listener = None;
    eprintln!("fake ready on fd 3");
    loop {
        kernel.reap(&mut complete);
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        loop {
            if io.is_ready() {
                skein_io::resume(&mut io, &env, &mut events, &mut subs);
            } else if let Some(done) = complete.pop() {
                skein_io::up(&mut io, &env, done, &mut events, &mut subs);
            } else if io.is_due(now.now) {
                skein_io::fire(&mut io, &env, &mut events, &mut subs);
            } else {
                break;
            }
            while let Some(event) = events.pop() {
                match event {
                    Event::Listening { listener: token, .. } => listener = Some(token),
                    Event::Accepted { socket, .. } => {
                        skein_io::down(&mut io, &env, Request::Bind { socket, owner: socket }, &mut subs);
                        skein_io::down(
                            &mut io,
                            &env,
                            Request::Stream {
                                stream: socket,
                                down: Down::Demand { read: Read::Arrived { max: 1024 }, room: 0 },
                            },
                            &mut subs,
                        );
                    }
                    Event::Shutdown { signal: ServiceSignal::Terminate, .. } => {
                        stopping = true;
                        skein_io::down(&mut io, &env, Request::Close { entity: signals }, &mut subs);
                        if let Some(listener) = listener.take() {
                            skein_io::down(&mut io, &env, Request::Close { entity: listener }, &mut subs);
                        }
                    }
                    Event::Stream { owner, up: Up::Bytes(_) } => {
                        skein_io::down(
                            &mut io,
                            &env,
                            Request::Stream { stream: owner, down: Down::Demand { read: Read::Nothing, room: 64 } },
                            &mut subs,
                        );
                    }
                    Event::Stream { owner, up: Up::Room } => {
                        println!("fake connected");
                        skein_io::down(
                            &mut io,
                            &env,
                            Request::Stream {
                                stream: owner,
                                down: Down::Send(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".as_slice().into()),
                            },
                            &mut subs,
                        );
                        skein_io::down(&mut io, &env, Request::Close { entity: owner }, &mut subs);
                    }
                    Event::Stream { up: Up::Failed(why), .. } => panic!("fake stream failed: {why:?}"),
                    Event::Failed { error, .. } => panic!("fake io failed: {error:?}"),
                    Event::Stream { up: Up::End, .. }
                    | Event::Shutdown { signal: ServiceSignal::Interrupt, .. }
                    | Event::Closed { .. } => {}
                    Event::Connecting { .. }
                    | Event::Connected { .. }
                    | Event::Output { .. }
                    | Event::Spawned { .. }
                    | Event::Exited { .. }
                    | Event::Usage { .. }
                    | Event::Resized
                    | Event::Window { .. } => panic!("unexpected fake event: {event:?}"),
                }
            }
        }
        io.reclaim();
        if stopping && io.is_empty() && subs.is_empty() && kernel.in_flight() == 0 {
            std::process::exit(37);
        }
        let wait = if io.is_ready() || !complete.is_empty() {
            Wait::No
        } else {
            io.next_deadline().map_or(Wait::Forever, Wait::Until)
        };
        kernel.submit(&mut subs, wait);
    }
}
