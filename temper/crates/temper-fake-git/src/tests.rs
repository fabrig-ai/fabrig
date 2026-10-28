//! The route's requests and machine effects, observed across discarded runs.
use crate::{Missing, Route, Transport};
use skein_fake_checkout::{
    Checkout,
    git::{self as kit, Remote, Tree},
};
use skein_fake_forge_domain::api;
use skein_fake_forge_protocol::{
    Limits,
    git::{self, Answer as RemoteAnswer, Call},
};
use skein_io::kernel::{self as k, Done, Exit, Fd, Op};
use skein_lib::{Secret, Time, Wall};
use skein_shell::Host;
use temper_worker_protocol_git::{
    Identity,
    invocation::{self, Command, Source},
    output,
};
mod disk;
mod net;
use disk::Disk;
use net::{Net, Response};
const REMOTE: &[u8] = b"http://127.0.0.1:3000/acme/app.git";
const AUTHORIZATION: &[u8] = b"Basic d29ya2VyOnJvdXRlLW9ubHktdG9rZW4=";
// The staged tree is {file: base\n}; its FNV tree id, as written by write-tree.
const BASE_TREE: u64 = 0xfe2b_b7b1_d1b6_d38c;
// Commit counts in the scripted store: base=1, topic=2, newly stored=3.
const BASE: u64 = 1;
const TOPIC: u64 = 2;
const STORED: u64 = 3;
// These scripts' largest JSON is below 1 KiB. Four KiB holds each whole body;
// 16 graph entries and 8 HTTP fields cover their objects and heads.
const BODY_BYTES: u32 = 4096;
fn limits() -> Limits {
    Limits {
        calls: 1,
        connections: 1,
        raw_bytes: 1024,
        http: skein_http::server::Limits {
            head: 2048,
            headers: 8,
            body: u64::from(BODY_BYTES),
            read: 128,
            response: 2048,
            send: 128,
        },
        documents: skein_forge::forgejo::limits::Limits {
            json: skein_json::tokenizer::Limits {
                chunk: 128,
                string: BODY_BYTES,
                number: 32,
                depth: 16,
                length: BODY_BYTES,
            },
            writer: skein_json::writer::Limits { depth: 16, length: BODY_BYTES },
            string_bytes: 512,
            body_bytes: BODY_BYTES,
            inner: 16,
            target_bytes: 2048,
        },
    }
}
fn commit(number: u64) -> temper_worker_domain_checkout::git::Commit {
    temper_worker_domain_checkout::git::Commit::new(skein_fake_forge_protocol::routes::object(number).bytes)
}
#[test]
fn git_ids_and_counts_share_the_forge_codec() {
    for count in [0, BASE, TOPIC, STORED, u64::MAX] {
        let object = skein_fake_forge_protocol::routes::object(count);
        assert_eq!(crate::id::id(count).as_slice(), skein_forge::forgejo::id::write(&object).as_ref());
        assert_eq!(crate::answer::count(commit(count)), count);
    }
    assert_ne!(&crate::id::id(BASE)[..7], &crate::id::id(TOPIC)[..7]);
}
fn spawn(command: &Command) -> k::Spawn {
    let kind = match command {
        Command::Clone { .. } => Some(temper_worker_domain_checkout::git::Kind::Clone),
        Command::Fetch { .. } => Some(temper_worker_domain_checkout::git::Kind::Fetch),
        Command::Create { .. } => Some(temper_worker_domain_checkout::git::Kind::Create),
        Command::Push { .. } | Command::IsAncestor { .. } => Some(temper_worker_domain_checkout::git::Kind::Push),
        Command::Unmerged
        | Command::Add
        | Command::DiffIndex { .. }
        | Command::WriteTree
        | Command::CommitTree { .. } => Some(temper_worker_domain_checkout::git::Kind::Commit),
        Command::CheckOut { .. } | Command::Clean => Some(temper_worker_domain_checkout::git::Kind::CheckOut),
        Command::Merge { .. } => Some(temper_worker_domain_checkout::git::Kind::Merge),
        Command::Version => None,
    };
    let identity = Identity {
        token: Secret::new(b"route-only-token".as_slice().into()),
        login: b"worker".as_slice().into(),
        name: b"Worker".as_slice().into(),
        email: b"worker@example.invalid".as_slice().into(),
    };
    k::Spawn {
        program: b"/git".as_slice().into(),
        args: invocation::arguments(command),
        env: invocation::environment(b"/home", kind, Some(&identity)),
        root: Fd::new(1),
        dir: b"repo".as_slice().into(),
        pipes: Box::new([]),
    }
}
fn files(remote: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    vec![
        (b"repo/file".to_vec(), b"base\n".to_vec()),
        (b"repo/.git/temper-index/file".to_vec(), b"base\n".to_vec()),
        (b"repo/.git/objects/1".to_vec(), vec![]),
        (b"repo/.git/objects/2".to_vec(), vec![]),
        (b"repo/.git/temper-head".to_vec(), BASE.to_le_bytes().to_vec()),
        (b"repo/.git/temper-remote".to_vec(), remote.to_vec()),
    ]
}
fn object_call(commit: u64) -> Call {
    Call::Objects { commits: Box::new([commit]) }
}
fn objects(commit: u64) -> Response {
    Response::Answer(RemoteAnswer::Objects {
        objects: Box::new([git::Object {
            commit,
            parent: (commit == TOPIC).then_some(BASE),
            merge_parent: None,
            tree: tree(if commit == BASE { b"base\n" } else { b"topic\n" }),
            message: b"message".as_slice().into(),
        }]),
        graph: if commit == TOPIC {
            Box::new([git::Parents { commit: BASE, parent: None, merge_parent: None }])
        } else {
            Box::new([])
        },
    })
}
fn tree(content: &[u8]) -> Box<[api::File]> {
    Box::new([api::File { path: b"file".as_slice().into(), content: content.into() }])
}
fn branch() -> Call {
    Call::Branch { repository: b"acme/app".as_slice().into(), branch: b"main".as_slice().into() }
}
fn branch_answer() -> (Call, Response) {
    (branch(), Response::Answer(RemoteAnswer::Commit(BASE)))
}
fn ancestor() -> Call {
    Call::Ancestor { ancestor: BASE, commit: TOPIC }
}
fn commit_tree() -> Command {
    Command::CommitTree { tree: commit(BASE_TREE), parent: commit(BASE), merging: Some(commit(TOPIC)) }
}
fn commit_tree_call() -> Call {
    Call::CommitTree {
        parent: BASE,
        merging: Some(TOPIC),
        tree: tree(b"base\n"),
        message: b"message\n".as_slice().into(),
    }
}
fn cloned() -> Response {
    Response::Answer(RemoteAnswer::Cloned {
        default: b"main".as_slice().into(),
        branches: Box::new([api::Head { branch: b"main".as_slice().into(), commit: BASE }]),
    })
}
fn clone_command() -> Command {
    Command::Clone { remote: REMOTE.into(), directory: b"copy".as_slice().into() }
}
fn clone_call() -> Call {
    Call::Clone { repository: b"acme/app".as_slice().into() }
}
fn push_command() -> Command {
    Command::Push {
        remote: REMOTE.into(),
        commit: commit(TOPIC),
        branch: b"main".as_slice().into(),
        expected: Some(commit(BASE)),
    }
}
fn push_call() -> Call {
    Call::Push {
        repository: b"acme/app".as_slice().into(),
        branch: b"main".as_slice().into(),
        commit: TOPIC,
        expected: Some(BASE),
    }
}
fn create_command() -> Command {
    Command::Create { remote: REMOTE.into(), branch: b"main".as_slice().into(), commit: commit(TOPIC) }
}
fn create_call() -> Call {
    Call::Create { repository: b"acme/app".as_slice().into(), branch: b"main".as_slice().into(), commit: TOPIC }
}
fn run(command: &Command, script: Vec<(Call, Response)>) -> (crate::Answer, Disk, Net) {
    let mut disk = Disk::new(&files(REMOTE));
    let mut net = Net::new(script);
    let answer = disk.run(spawn(command), &mut net, b"message\n");
    (answer, disk, net)
}
#[test]
fn missed_calls_replay_the_scanned_checkout_and_make_each_write_once() {
    let rows = [
        (
            "clone",
            clone_command(),
            vec![(clone_call(), cloned()), (object_call(BASE), objects(BASE))],
            b"repo/copy/.git/temper-remote".as_slice(),
            REMOTE,
        ),
        (
            "fetch",
            Command::Fetch { remote: REMOTE.into(), source: Source::Branch(b"topic".as_slice().into()) },
            vec![(
                Call::Fetch {
                    repository: b"acme/app".as_slice().into(),
                    want: api::Want::Branch(b"topic".as_slice().into()),
                },
                Response::Answer(RemoteAnswer::Commit(TOPIC)),
            )],
            b"repo/.git/refs/temper/fetched",
            b"\x02\0\0\0\0\0\0\0",
        ),
        (
            "create",
            create_command(),
            vec![
                (branch(), Response::Error(404, api::Error::Missing(api::What::Branch))),
                (create_call(), Response::Answer(RemoteAnswer::Created(api::Created::Created))),
            ],
            b"repo/file",
            b"base\n",
        ),
        (
            "push",
            push_command(),
            vec![branch_answer(), (push_call(), Response::Answer(RemoteAnswer::Pushed(api::Pushed::Pushed)))],
            b"repo/file",
            b"base\n",
        ),
        (
            "commit-tree",
            commit_tree(),
            vec![(commit_tree_call(), Response::Answer(RemoteAnswer::Stored(Some(STORED))))],
            b"repo/.git/objects/3",
            b"",
        ),
        (
            "ancestor",
            Command::IsAncestor { expected: commit(BASE), commit: commit(TOPIC) },
            vec![(ancestor(), Response::Answer(RemoteAnswer::Ancestor(true)))],
            b"repo/file",
            b"base\n",
        ),
        (
            "read",
            Command::CheckOut { commit: commit(TOPIC) },
            vec![(object_call(TOPIC), objects(TOPIC))],
            b"repo/file",
            b"topic\n",
        ),
        (
            "diff-index",
            Command::DiffIndex { parent: commit(BASE) },
            vec![(object_call(BASE), objects(BASE))],
            b"repo/file",
            b"base\n",
        ),
        (
            "merge",
            Command::Merge { theirs: commit(TOPIC) },
            vec![
                (Call::Ancestor { ancestor: TOPIC, commit: BASE }, Response::Answer(RemoteAnswer::Ancestor(false))),
                (object_call(BASE), objects(BASE)),
                (object_call(TOPIC), objects(TOPIC)),
            ],
            b"repo/file",
            b"topic\n",
        ),
    ];
    for (name, command, script, file, expected) in rows {
        let expected_calls: Vec<_> = script.iter().map(|(call, _)| call.clone()).collect();
        let (answer, mut disk, net) = run(&command, script);
        assert_eq!(answer.exit, 0, "{name}: {:?}", answer.stderr);
        assert!(answer.stderr.is_empty(), "{name}");
        assert_eq!(net.seen, expected_calls, "{name}: writes once");
        assert_eq!(net.connected, [([127, 0, 0, 1], 3000).into()], "{name}: one connection");
        assert_eq!(disk.read(file).as_deref(), Some(expected), "{name}: final checkout");
        assert_eq!(disk.read(b"repo/.git/objects/18446744073709551615"), None, "{name}: no placeholder retained");
        assert!(
            !disk.written.iter().any(|p| p.ends_with(b"/18446744073709551615")),
            "{name}: no placeholder write, including discarded runs"
        );
        assert!(!answer.stdout.windows(16).any(|b| b == b"ffffffffffffffff"), "{name}: no placeholder printed");
    }
}
#[test]
fn the_listener_comes_from_the_named_remote_or_clones_metadata() {
    let rows = [
        (
            clone_command(),
            b"http://127.0.0.1:4000/acme/app.git".as_slice(),
            vec![(clone_call(), cloned()), (object_call(BASE), objects(BASE))],
            3000,
        ),
        (
            Command::IsAncestor { expected: commit(BASE), commit: commit(TOPIC) },
            b"http://127.0.0.1:4000/acme/app.git",
            vec![(ancestor(), Response::Answer(RemoteAnswer::Ancestor(true)))],
            4000,
        ),
    ];
    for (command, remote, script, port) in rows {
        let mut disk = Disk::new(&files(remote));
        let mut net = Net::new(script);
        net.piece = 1;
        let answer = disk.run(spawn(&command), &mut net, b"");
        assert_eq!(answer.exit, 0);
        assert_eq!(net.connected, [([127, 0, 0, 1], port).into()]);
    }
}
#[test]
fn repository_refusals_keep_whether_the_route_named_the_identity() {
    let rows = [
        (
            Response::Raw(401, vec![]),
            128,
            b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': The requested URL returned error: 401\n"
                .as_slice(),
        ),
        (
            Response::Error(403, api::Error::Protected),
            1,
            b"error: failed to push some refs to 'http://127.0.0.1:3000/acme/app.git'\n",
        ),
    ];
    for (response, exit, stderr) in rows {
        let (answer, _, _) = run(&create_command(), vec![branch_answer(), (create_call(), response)]);
        assert_eq!(answer.exit, exit);
        assert_eq!(answer.stderr, stderr);
    }
}
#[test]
fn failures_without_a_fault_result_end_with_the_requested_git_diagnostic() {
    let local = Command::IsAncestor { expected: commit(BASE), commit: commit(TOPIC) };
    let message = b"message\n".as_slice();
    let oversized = vec![b'x'; usize::try_from(BODY_BYTES).expect("body bytes") + 1];
    let unreachable = (128, temper_worker_domain_checkout::git::Fault::Unreachable);
    let broken = (125, temper_worker_domain_checkout::git::Fault::Broken);
    let rows=[
        ("status",local.clone(),vec![(ancestor(),Response::Raw(503,vec![]))],false,message,unreachable,b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': The requested URL returned error: 503\n".as_slice()),
        ("refused",local.clone(),vec![],true,message,unreachable,b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': Failed to connect to 127.0.0.1 port 3000: Couldn't connect to server\n"),
        ("reset",local.clone(),vec![(ancestor(),Response::Reset)],false,message,unreachable,b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': Failed to connect to 127.0.0.1 port 3000: Couldn't connect to server\n"),
        ("early close",local,vec![(ancestor(),Response::Close)],false,message,unreachable,b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': Failed to connect to 127.0.0.1 port 3000: Couldn't connect to server\n"),
        ("store status",commit_tree(),vec![(commit_tree_call(),Response::Error(507,api::Error::Full))],false,message,unreachable,b"fatal: unable to access 'http://127.0.0.1:3000/acme/app.git': The requested URL returned error: 507\n"),
        ("oversized message",commit_tree(),vec![],false,oversized.as_slice(),broken,b"temper fake git: broken: request\n"),
    ];
    for (name, command, script, refused, input, (exit, fault), stderr) in rows {
        let mut disk = Disk::new(&files(REMOTE));
        let expected_calls: Vec<_> = script.iter().map(|(call, _)| call.clone()).collect();
        let mut net = Net::new(script);
        net.refused = refused;
        let answer = disk.run(spawn(&command), &mut net, input);
        assert_eq!(answer.exit, exit, "{name}");
        assert_eq!(answer.stderr, stderr, "{name}");
        assert_eq!(net.seen, expected_calls, "{name}: only scripted requests sent");
        assert_eq!(
            output::classify(&answer.stderr, temper_worker_domain_checkout::git::Kind::Fetch),
            fault,
            "{name}: classifier"
        );
    }
}
#[test]
fn missing_or_malformed_routes_have_their_own_fatal_exit() {
    let rows = [
        Response::Raw(404, vec![]),
        Response::Raw(405, vec![]),
        Response::Raw(200, b"{}".to_vec()),
        Response::Raw(200, b"{broken".to_vec()),
    ];
    for response in rows {
        let (answer, _, _) = run(&clone_command(), vec![(clone_call(), response)]);
        assert_eq!(answer.exit, 128);
        assert_eq!(answer.stderr, b"fatal: 127.0.0.1:3000 serves no fake git route\n");
    }
}
#[test]
fn repository_failures_are_logged_and_classified_without_retrying_a_write() {
    let rows = [
        (
            Response::Error(404, api::Error::Missing(api::What::Repository)),
            false,
            temper_worker_domain_checkout::git::Fault::Missing {
                missing: temper_worker_domain_checkout::git::Missing::Repository,
            },
        ),
        (Response::Reset, false, temper_worker_domain_checkout::git::Fault::Unreachable),
        (Response::Close, false, temper_worker_domain_checkout::git::Fault::Unreachable),
        (Response::Raw(502, vec![]), true, temper_worker_domain_checkout::git::Fault::Unreachable),
    ];
    for (response, refused, expected) in rows {
        let mut disk = Disk::new(&files(REMOTE));
        let mut net = Net::new(if refused { vec![] } else { vec![(clone_call(), response)] });
        net.refused = refused;
        let answer = disk.run(spawn(&clone_command()), &mut net, b"");
        assert_eq!(answer.exit, 128);
        assert_eq!(output::classify(&answer.stderr, temper_worker_domain_checkout::git::Kind::Clone), expected);
    }
}
#[test]
fn hosts_without_an_ip_and_port_fail_before_opening_a_socket() {
    for (remote, stderr) in [
        (
            b"https://forge.invalid/acme/app.git".as_slice(),
            b"fatal: unable to access 'https://forge.invalid/acme/app.git': Could not resolve host: forge.invalid\n"
                .as_slice(),
        ),
        (
            b"http://localhost/acme/app.git".as_slice(),
            b"fatal: unable to access 'http://localhost/acme/app.git': Could not resolve host: localhost\n".as_slice(),
        ),
    ] {
        let mut disk = Disk::new(&files(REMOTE));
        let mut net = Net::new(vec![]);
        let answer = disk.run(
            spawn(&Command::Clone { remote: remote.into(), directory: b"copy".as_slice().into() }),
            &mut net,
            b"",
        );
        assert_eq!(answer.exit, 128);
        assert_eq!(answer.stderr, stderr);
        assert!(net.connected.is_empty());
    }
}
#[test]
fn the_runs_bound_lets_the_last_run_finish_and_ends_the_next() {
    // Every empty objects answer costs one discarded run. With MAX_RUNS - 2
    // of them, the answer that fills the object makes the last run MAX_RUNS;
    // with MAX_RUNS, the run past the bound is never made.
    let missing = object_call(TOPIC);
    let empty = Response::Answer(RemoteAnswer::Objects { objects: Box::new([]), graph: Box::new([]) });
    let runs = usize::try_from(crate::MAX_RUNS).expect("runs");
    let rows = [
        (runs - 2, Some(objects(TOPIC)), 0, b"".as_slice(), runs - 1, b"topic\n".as_slice()),
        (runs, None, 125, b"temper fake git: broken: runs\n".as_slice(), runs, b"base\n".as_slice()),
    ];
    for (empties, last, exit, stderr, seen, file) in rows {
        let mut script = vec![(missing.clone(), empty.clone()); empties];
        script.extend(last.map(|last| (missing.clone(), last)));
        let (answer, mut disk, net) = run(&Command::CheckOut { commit: commit(TOPIC) }, script);
        assert_eq!(answer.exit, exit, "{empties} empty answers");
        assert_eq!(answer.stderr, stderr, "{empties} empty answers");
        assert_eq!(net.seen.len(), seen, "{empties} empty answers");
        assert_eq!(disk.read(b"repo/file").as_deref(), Some(file), "{empties} empty answers");
    }
}

fn answer_miss(route: &mut Route, net: &mut Net, connection: &mut crate::connection::Connection, missing: Call) {
    let exchange = crate::exchange::Exchange::new(missing, REMOTE.to_vec()).expect("listener");
    let (request, body) = exchange.request(None, &limits()).expect("request");
    connection.call(exchange.address, request, body);
    let reply = net.exchange(connection);
    let (call, answer, authenticated) = exchange.finish(reply, &limits()).expect("remote answer");
    route.receive(call, answer, authenticated);
}
#[test]
fn commits_and_merges_discard_their_placeholders_before_keeping_output_or_files() {
    let rows = [("commit", None, None), ("merge", Some(TOPIC), Some(u64::MAX))];
    for (name, merging, placeholder) in rows {
        let call = match merging {
            Some(merge_parent) => Call::Merge {
                parent: BASE,
                merge_parent,
                tree: tree(b"changed\n"),
                message: b"message\n".as_slice().into(),
            },
            None => Call::Commit { parent: BASE, tree: tree(b"changed\n"), message: b"message\n".as_slice().into() },
        };
        let mut net = Net::new(vec![(call.clone(), Response::Answer(RemoteAnswer::Stored(Some(STORED))))]);
        let mut route = Route::new(limits());
        let mut connection = route.connection().expect("route connection");
        let mut kept = None;
        let mut first = true;
        for _ in 0..4 {
            route.replay();
            let mut checkout = Checkout::new();
            for (path, bytes) in files(REMOTE) {
                checkout.write(&path, &bytes);
            }
            checkout.write(b"repo/file", b"changed\n");
            let result = match merging {
                Some(second) => Some(
                    kit::commit_merging(&mut route, &mut checkout, b"repo", BASE, second, b"message\n")
                        .expect("locally present parents"),
                ),
                None => {
                    kit::commit(&mut route, &mut checkout, b"repo", BASE, b"message\n").expect("locally present parent")
                }
            };
            if let Missing::Call(missing) = route.missed() {
                assert_eq!(result, placeholder, "{name}: kit accepts placeholder");
                assert!(first, "{name}: one write miss");
                first = false;
                answer_miss(&mut route, &mut net, &mut connection, missing);
            } else {
                kept = Some((result.map(crate::id::id), checkout));
                break;
            }
        }
        net.close(&mut connection);
        let (printed, checkout) = kept.expect("a complete run");
        assert_eq!(printed, Some(*b"1d0b14e4db018fed000000000000000300000000"), "{name}: output");
        assert!(checkout.exists(b"repo/.git/objects/3"), "{name}: final commit");
        assert!(!checkout.exists(b"repo/.git/objects/18446744073709551615"), "{name}: discarded placeholder");
        assert_eq!(net.seen, [call], "{name}: write once");
    }
}
#[test]
fn object_reads_cache_whole_objects_and_the_ancestry_graph() {
    enum Read {
        Parent,
        MergeParent,
        Tree,
        Message,
    }
    for read in [Read::Parent, Read::MergeParent, Read::Tree, Read::Message] {
        let mut route = Route::new(limits());
        route.replay();
        match read {
            Read::Parent => {
                route.parent(TOPIC);
            }
            Read::MergeParent => {
                route.merge_parent(TOPIC);
            }
            Read::Tree => {
                route.tree(TOPIC);
            }
            Read::Message => {
                route.message(TOPIC);
            }
        }
        let Missing::Call(missing) = route.missed() else { panic!("first read misses") };
        assert_eq!(missing, object_call(TOPIC));
        let mut net = Net::new(vec![(missing.clone(), objects(TOPIC))]);
        let mut connection = route.connection().expect("connection");
        answer_miss(&mut route, &mut net, &mut connection, missing);
        net.close(&mut connection);
        route.replay();
        assert_eq!(route.parent(TOPIC), Some(BASE));
        assert_eq!(route.merge_parent(TOPIC), None);
        assert_eq!(route.parent(BASE), None);
        assert_eq!(route.tree(TOPIC), Tree::from([(b"file".to_vec(), b"topic\n".to_vec())]));
        assert_eq!(route.message(TOPIC), b"message");
        assert_eq!(route.missed(), Missing::None);
    }
}
#[test]
fn an_out_of_order_replay_names_the_requested_and_logged_calls() {
    let mut route = Route::new(limits());
    route.receive(branch(), Ok(RemoteAnswer::Commit(BASE)), true);
    route.replay();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| route.is_ancestor(BASE, TOPIC)))
        .expect_err("order mismatch panics");
    let message = panic.downcast_ref::<String>().expect("assertion message");
    assert!(message.contains("Ancestor") && message.contains("Branch"), "{message}");
}

#[test]
fn a_lost_push_answer_does_not_send_the_write_again() {
    let call = Call::Push {
        repository: b"acme/app".as_slice().into(),
        branch: b"main".as_slice().into(),
        commit: TOPIC,
        expected: Some(BASE),
    };
    let command = Command::Push {
        remote: REMOTE.into(),
        commit: commit(TOPIC),
        branch: b"main".as_slice().into(),
        expected: Some(commit(BASE)),
    };
    let (answer, _, net) = run(&command, vec![branch_answer(), (call.clone(), Response::Reset)]);
    assert_eq!(answer.exit, 128);
    assert_eq!(net.seen, [branch(), call]);
    assert_eq!(
        output::classify(&answer.stderr, temper_worker_domain_checkout::git::Kind::Push),
        temper_worker_domain_checkout::git::Fault::Unreachable
    );
}
