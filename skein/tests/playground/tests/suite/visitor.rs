//! Browser fetch against the site's independent HTTP process (skein/docs/design/playground.md, 8).
use skein_lib::{Duration, Time};
use skein_playground_world::site::{self, Body, Route, Script, Stage};
use skein_ui_web::{
    bridge::Input,
    fetch::{Exchange, Report},
    transport::StreamDuty,
};
use skein_world::Memory;

const SEEDS: [u64; 3] = [7, 19, 43];
/// Small answers and request heads fit the site's HTTP read and head bounds.
const ANSWER: u32 = 1024;
fn exchange(id: u32, method: &str, path: &[u8], body: &[u8], bound: u32) -> Input {
    Input::Exchange(Exchange {
        id,
        method: method.into(),
        path: path.into(),
        headers: Vec::new(),
        body: body.into(),
        bound,
    })
}
fn get(id: u32, path: &[u8]) -> Input {
    exchange(id, "GET", path, b"", ANSWER)
}
fn answered(id: u32, status: u16, body: &[u8]) -> Report {
    Report::Answered { exchange: id, status, body: body.into() }
}
fn fixed(path: &[u8], status: u16, body: &[u8]) -> Route {
    Route::new(path, status, Body::Fixed(body.into()))
}
fn run(script: &Script) {
    for seed in SEEDS {
        let outcome = site::run(seed, script, Memory::Checked);
        assert!(outcome.iterations > 0, "seed {seed}");
    }
}
#[test]
fn an_exchange_is_answered() {
    run(&Script::new(
        vec![fixed(b"/get", 201, b"answer"), Route::new(b"/post", 202, Body::Echo)],
        vec![
            Stage::new(0, vec![get(1, b"/get")]),
            Stage::new(1, vec![exchange(2, "POST", b"/post", b"posted body", ANSWER)]),
        ],
        vec![answered(1, 201, b"answer"), answered(2, 202, b"posted body")],
    ));
}

#[test]
fn an_answer_past_its_bound_fails_the_exchange() {
    const BOUND: u32 = 3; // The accepted answer's literal length.
    run(&Script::new(
        vec![fixed(b"/at", 200, b"abc"), fixed(b"/past", 200, b"abcd")],
        vec![
            Stage::new(0, vec![exchange(1, "GET", b"/at", b"", BOUND)]),
            Stage::new(1, vec![exchange(2, "GET", b"/past", b"", BOUND)]),
        ],
        vec![answered(1, 200, b"abc"), Report::Failed { exchange: 2 }],
    ));
}
fn never(path: &[u8]) -> Route {
    Route::new(path, 200, Body::Pieces { pieces: Vec::new(), finish: false })
}
#[test]
fn an_exchange_past_the_bound_in_flight_fails() {
    let mut script = Script::new(
        vec![never(b"/wait")],
        vec![
            Stage::new(0, vec![get(1, b"/wait"), get(2, b"/wait"), get(3, b"/wait")]),
            Stage {
                requests: 2,
                ..Stage::new(1, vec![Input::Abandon { exchange: 1 }, Input::Abandon { exchange: 2 }])
            },
        ],
        vec![Report::Failed { exchange: 3 }],
    );
    script.closed = 2;
    run(&script);
}
#[test]
fn a_refused_connection_fails_the_exchange() {
    let mut script =
        Script::new(Vec::new(), vec![Stage::new(0, vec![get(1, b"/")])], vec![Report::Failed { exchange: 1 }]);
    script.listener = false;
    run(&script);
}
fn open(path: &[u8], bound: u32) -> Input {
    Input::Stream(StreamDuty::Open { path: path.into(), headers: Vec::new(), bound })
}
#[test]
fn a_refused_connection_ends_the_stream() {
    let mut script = Script::new(Vec::new(), vec![Stage::new(0, vec![open(b"/", ANSWER)])], vec![Report::StreamEnded]);
    script.listener = false;
    run(&script);
}
#[test]
fn an_abandoned_exchange_reports_nothing() {
    let mut script = Script::new(
        vec![never(b"/wait")],
        vec![
            Stage::new(0, vec![get(1, b"/wait")]),
            Stage { requests: 1, ..Stage::new(0, vec![Input::Abandon { exchange: 1 }]) },
        ],
        Vec::new(),
    );
    script.closed = 1;
    run(&script);
}

fn piece(bytes: &[u8]) -> Report {
    Report::StreamPiece { bytes: bytes.into() }
}
fn at(millis: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_millis(millis))
}
#[test]
fn the_stream_reads_one_piece_per_read_on() {
    const PIECE: u32 = 3; // Each expected delivery is three literal bytes.
    let route = Route::new(
        b"/stream",
        200,
        Body::Pieces {
            pieces: vec![
                (Duration::ZERO, b"abcdef".as_slice().into()),
                (Duration::from_millis(30), b"ghi".as_slice().into()),
            ],
            finish: true,
        },
    );
    run(&Script::new(
        vec![route],
        vec![
            Stage::new(0, vec![open(b"/stream", PIECE)]),
            Stage { at: at(10), ..Stage::new(2, vec![Input::ReadOn]) },
            Stage { at: at(20), ..Stage::new(3, vec![Input::ReadOn]) },
            Stage::new(4, vec![Input::ReadOn]),
        ],
        vec![Report::StreamOpened, piece(b"abc"), piece(b"def"), piece(b"ghi"), Report::StreamEnded],
    ));
}
#[test]
fn a_refused_stream_reports_its_status_then_its_body() {
    const PIECE: u32 = 3; // The refusal body is two three-byte pieces.
    run(&Script::new(
        vec![fixed(b"/refuse", 403, b"denied")],
        vec![
            Stage::new(0, vec![open(b"/refuse", PIECE)]),
            Stage::new(2, vec![Input::ReadOn]),
            Stage::new(3, vec![Input::ReadOn]),
        ],
        vec![Report::StreamRefused { status: 403 }, piece(b"den"), piece(b"ied"), Report::StreamEnded],
    ));
}
#[test]
fn closing_and_replacing_a_stream_ends_each_connection() {
    let routes = [b"/first".as_slice(), b"/next".as_slice()]
        .into_iter()
        .map(|path| Route::new(path, 200, Body::Pieces { pieces: vec![(Duration::ZERO, path.into())], finish: false }))
        .collect();
    let mut script = Script::new(
        routes,
        vec![
            Stage::new(0, vec![open(b"/first", ANSWER)]),
            Stage::new(2, vec![open(b"/next", ANSWER)]),
            Stage::new(5, vec![Input::Stream(StreamDuty::Close)]),
        ],
        vec![
            Report::StreamOpened,
            piece(b"/first"),
            Report::StreamEnded,
            Report::StreamOpened,
            piece(b"/next"),
            Report::StreamEnded,
        ],
    );
    script.closed = 2;
    run(&script);
}

const HEADERS: &[u8] = b"Host:application.example:8080\nContent-Length:0\nConnection:close\n";
#[test]
fn the_host_and_origin_are_the_applications() {
    let mut post = exchange(2, "POST", b"/echo", b"body", ANSWER);
    if let Input::Exchange(exchange) = &mut post {
        exchange.headers.push(("X-Page".into(), "given".into()));
    }
    run(&Script::new(vec![Route::new(b"/echo", 200, Body::Headers)],
        vec![Stage::new(0, vec![get(1, b"/echo")]), Stage::new(1, vec![post])],
        vec![answered(1, 200, HEADERS), answered(2, 200, b"X-Page:given\nHost:application.example:8080\nOrigin:http://application.example:8080\nContent-Length:4\nConnection:close\n")]));
}
#[test]
fn a_client_error_fails_without_sending_a_request() {
    let mut invalid = get(1, b"/echo");
    if let Input::Exchange(exchange) = &mut invalid {
        exchange.headers.push(("Connection".into(), "close".into()));
    }
    run(&Script::new(
        vec![Route::new(b"/echo", 200, Body::Headers)],
        vec![Stage::new(0, vec![invalid])],
        vec![Report::Failed { exchange: 1 }],
    ));
}

fn setting(path: &[u8], cookies: &[&[u8]]) -> Route {
    let mut route = fixed(path, 200, b"set");
    route.headers = cookies
        .iter()
        .map(|value| skein_http::Header { name: b"Set-Cookie".as_slice().into(), value: (*value).into() })
        .collect();
    route
}
fn echo(path: &[u8]) -> Route {
    Route::new(path, 200, Body::Headers)
}
#[test]
fn the_jar_sends_a_cookie_under_its_path() {
    run(&Script::new(
        vec![setting(b"/app/set", &[b"a=1; HttpOnly", b"b=2; Path=/"]), echo(b"/app/x"), echo(b"/apple")],
        vec![
            Stage::new(0, vec![get(1, b"/app/set")]),
            Stage::new(1, vec![get(2, b"/app/x")]),
            Stage::new(2, vec![get(3, b"/apple")]),
        ],
        vec![
            answered(1, 200, b"set"),
            answered(2, 200, b"Host:application.example:8080\nCookie:a=1; b=2\nContent-Length:0\nConnection:close\n"),
            answered(3, 200, b"Host:application.example:8080\nCookie:b=2\nContent-Length:0\nConnection:close\n"),
        ],
    ));
}
#[test]
fn a_cleared_cookie_is_not_sent() {
    run(&Script::new(
        vec![setting(b"/set", &[b"a=1; Path=/"]), setting(b"/clear", &[b"a=; Path=/; Max-Age=0"]), echo(b"/echo")],
        vec![
            Stage::new(0, vec![get(1, b"/set")]),
            Stage::new(1, vec![get(2, b"/clear")]),
            Stage::new(2, vec![get(3, b"/echo")]),
        ],
        vec![answered(1, 200, b"set"), answered(2, 200, b"set"), answered(3, 200, HEADERS)],
    ));
}
#[test]
fn an_expired_cookie_is_not_sent() {
    run(&Script::new(
        vec![setting(b"/set", &[b"a=1; Max-Age=1"]), echo(b"/echo")],
        vec![Stage::new(0, vec![get(1, b"/set")]), Stage { at: at(2_000), ..Stage::new(1, vec![get(2, b"/echo")]) }],
        vec![answered(1, 200, b"set"), answered(2, 200, HEADERS)],
    ));
}
#[test]
fn a_cookie_past_the_jar_is_refused() {
    // Two slots, and four bytes of name plus value: both at-bound cookies stay.
    const COOKIE_BYTES: u32 = 4;
    for (cookies, expected) in [
        (
            vec![b"a=1".as_slice(), b"b=2", b"c=3"],
            b"Host:application.example:8080\nCookie:a=1; b=2\nContent-Length:0\nConnection:close\n".as_slice(),
        ),
        (
            vec![b"a=123".as_slice(), b"a=1234"],
            b"Host:application.example:8080\nCookie:a=123\nContent-Length:0\nConnection:close\n".as_slice(),
        ),
    ] {
        let mut script = Script::new(
            vec![setting(b"/set", &cookies), echo(b"/echo")],
            vec![Stage::new(0, vec![get(1, b"/set")]), Stage::new(1, vec![get(2, b"/echo")])],
            vec![answered(1, 200, b"set"), answered(2, 200, expected)],
        );
        script.config.cookie_bytes = COOKIE_BYTES;
        run(&script);
    }
}
#[test]
fn the_streams_opening_sets_the_jar() {
    let mut route = never(b"/stream");
    route.headers = setting(b"/", &[b"a=1; Path=/"]).headers;
    run(&Script::new(
        vec![route, echo(b"/echo")],
        vec![
            Stage::new(0, vec![open(b"/stream", ANSWER)]),
            Stage::new(1, vec![Input::Stream(StreamDuty::Close), get(1, b"/echo")]),
        ],
        vec![
            Report::StreamOpened,
            Report::StreamEnded,
            answered(1, 200, b"Host:application.example:8080\nCookie:a=1\nContent-Length:0\nConnection:close\n"),
        ],
    ));
}
#[test]
fn a_zero_bound_accepts_only_an_empty_answer() {
    run(&Script::new(
        vec![fixed(b"/empty", 200, b""), fixed(b"/byte", 200, b"x")],
        vec![
            Stage::new(0, vec![exchange(1, "GET", b"/empty", b"", 0)]),
            Stage::new(1, vec![exchange(2, "GET", b"/byte", b"", 0)]),
        ],
        vec![answered(1, 200, b""), Report::Failed { exchange: 2 }],
    ));
}

fn full_visit() -> Script {
    const BODY: u32 = 256; // The site's HTTP read/send bound; each body fills it.
    let held = |path: &[u8], end| {
        Route::new(
            path,
            200,
            Body::Pieces {
                pieces: vec![
                    (Duration::ZERO, vec![b'x'; usize::try_from(BODY).expect("body fits")].into_boxed_slice()),
                    (Duration::from_millis(end), Box::from([])),
                ],
                finish: true,
            },
        )
    };
    // Each name plus its value fills cookie_bytes = 32; both slots are used.
    let set =
        setting(b"/set", &[b"a=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx; Path=/", b"b=yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy; Path=/"]);
    let mut script = Script::new(
        vec![set, held(b"/one", 20), held(b"/two", 30), held(b"/stream", 40)],
        vec![
            Stage::new(0, vec![get(9, b"/set")]),
            Stage {
                at: at(1),
                ..Stage::new(
                    1,
                    vec![
                        exchange(1, "GET", b"/one", b"", BODY),
                        exchange(2, "GET", b"/two", b"", BODY),
                        open(b"/stream", BODY),
                    ],
                )
            },
            Stage { at: at(10), requests: 4, ..Stage::new(3, vec![get(4, b"/one")]) },
            Stage::new(6, vec![Input::ReadOn]),
        ],
        vec![
            answered(9, 200, b"set"),
            Report::StreamOpened,
            piece(&vec![b'x'; usize::try_from(BODY).expect("body fits")]),
            Report::Failed { exchange: 4 },
            answered(1, 200, &vec![b'x'; usize::try_from(BODY).expect("body fits")]),
            answered(2, 200, &vec![b'x'; usize::try_from(BODY).expect("body fits")]),
            Report::StreamEnded,
        ],
    );
    script.config.io.sockets = 3; // Two in-flight exchanges and the stream; the server adds its listener.
    script
}
#[test]
fn the_visitors_peak_is_bounded() {
    // Measured site peak 15,874 B, visitor peak 14,006 B; plus 25%, rounded up.
    const CEILING: u64 = 20_000;
    for seed in SEEDS {
        let outcome = site::run(seed, &full_visit(), Memory::Within(CEILING));
        let peaks = outcome.peaks().expect("metered processes");
        eprintln!("seed {seed}: process heap peaks {peaks:?} B");
        assert!(peaks.into_iter().all(|peak| peak <= CEILING), "seed {seed}");
    }
}

#[test]
fn a_reset_fails_an_exchange_and_ends_a_stream() {
    // A send after 10 ms hits the reset while both connections wait on the peer.
    let reset = |path: &[u8]| {
        Route::new(
            path,
            200,
            Body::Pieces { pieces: vec![(Duration::from_millis(10), b"late".as_slice().into())], finish: true },
        )
    };
    let mut script = Script::new(
        vec![reset(b"/call"), reset(b"/stream")],
        vec![Stage::new(0, vec![get(1, b"/call"), open(b"/stream", ANSWER)])],
        vec![Report::StreamOpened, Report::Failed { exchange: 1 }, Report::StreamEnded],
    );
    script.reset_at = Some(at(10));
    run(&script);
}
#[test]
fn an_abandoned_id_can_be_reused_before_its_connection_closes() {
    run(&Script::new(
        vec![never(b"/wait"), fixed(b"/answer", 200, b"new answer")],
        vec![Stage::new(0, vec![get(1, b"/wait"), Input::Abandon { exchange: 1 }, get(1, b"/answer")])],
        vec![answered(1, 200, b"new answer")],
    ));
}
