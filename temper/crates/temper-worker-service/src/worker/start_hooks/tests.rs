use super::*;
use crate::tests::fixtures;
use kernel::{Complete, Done, Error, Fd, Kind, Op, OpenHow, Stat};

const ROOT: Fd = Fd::new(9);
const HOOKS: Fd = Fd::new(10);
const CONFIG: Fd = Fd::new(11);
const CHECKOUT: Token = Token::new(17);
const OWNER: Token = Token::new(23);
const CEILING: u32 = 3600;

#[derive(Clone, Copy)]
enum Config {
    Text(&'static [u8]),
    Missing,
    Link,
    Directory,
    NoHooks(Error),
}

struct Fixture {
    reader: Reader,
    submissions: Queue<kernel::Submit>,
    prefix: u64,
}

impl Fixture {
    fn new(directories: &[(&[u8], bool)]) -> Self {
        let mut start = fixtures::worker_peers::decoded_chat_start(&fixtures::worker_agent());
        start.workspace = Some(CHECKOUT);
        let mut mounts = skein_lib::List::with_capacity(u32::try_from(directories.len()).expect("bounded mounts"));
        for (name, writable) in directories {
            mounts
                .push(smith::Directory {
                    root: CHECKOUT,
                    name: (*name).into(),
                    host: Box::default(),
                    writable: *writable,
                    git: true,
                    conflicts: Box::default(),
                    results: None,
                    hooks: None,
                })
                .expect("one start directory per fixture mount");
        }
        start.directories = mounts.into_boxed();
        let mut reader = Reader::new(1);
        reader.start(Ready { owner: OWNER, start: *start, window: smith::Window { turns: 1, bytes: 4096 } }, CEILING);
        Self { reader, submissions: Queue::uncapped(), prefix: crate::tokens::worker(0).unwrap() }
    }

    fn step(&mut self) -> Option<Ready> {
        self.reader.step(Time::ZERO, ROOT, self.prefix, &mut self.submissions)
    }

    fn pending(&mut self) -> kernel::Submit {
        self.submissions.pop().expect("a single file operation")
    }

    fn complete(&mut self, submit: kernel::Submit, result: Result<Done, Error>) {
        self.reader.complete(Time::ZERO, self.prefix, Complete { op: submit.op, kind: submit.kind, result }).unwrap();
    }

    fn directory(&mut self, name: &[u8], content: Config) {
        assert!(self.step().is_none(), "files drain before Start is sent");
        let open = self.pending();
        let Op::Open { root, path, how } = &open.kind else { panic!("expected a directory open: {open:?}") };
        assert_eq!((*root, *how), (ROOT, OpenHow::DirectoryNoFollow));
        assert_eq!(path.as_ref(), bytes::concat(&[b"workspaces/17/", name, b"/.temper/hooks"]).as_ref());
        match content {
            Config::NoHooks(error) => {
                self.complete(open, Err(error));
                return;
            }
            Config::Text(_) | Config::Missing | Config::Link | Config::Directory => {}
        }
        self.complete(open, Ok(Done::Fd(HOOKS)));
        assert!(self.step().is_none());
        let config = self.pending();
        let Op::Open { root, path, how } = &config.kind else { panic!("expected a config open: {config:?}") };
        assert_eq!((*root, path.as_ref(), *how), (HOOKS, b"config.toml".as_slice(), OpenHow::ReadNoFollow));
        match content {
            Config::Missing => self.complete(config, Err(Error::NotFound)),
            Config::Link => self.complete(config, Err(Error::TooManyLinks)),
            Config::Directory => self.load(config, Kind::Directory, b""),
            Config::Text(text) => self.load(config, Kind::File, text),
            Config::NoHooks(_) => unreachable!("the directory was opened"),
        }
        assert!(self.step().is_none());
        let close = self.pending();
        let Op::Close { fd } = close.kind else { panic!("expected directory close: {close:?}") };
        assert_eq!(fd, HOOKS);
        self.complete(close, Ok(Done::Nothing));
    }

    fn load(&mut self, open: kernel::Submit, kind: Kind, text: &[u8]) {
        self.complete(open, Ok(Done::Fd(CONFIG)));
        let stat = self.pending_after_step();
        let Op::Stat { fd } = stat.kind else { panic!("expected a stat: {stat:?}") };
        assert_eq!(fd, CONFIG);
        self.complete(
            stat,
            Ok(Done::Stat(Stat { kind, size: skein_lib::len_u64(text.len()), mode: 0o644, owner: 1000, links: 1 })),
        );
        if kind == Kind::File {
            for body in [text, b""] {
                let mut read = self.pending_after_step();
                let Op::Read { fd, buf, .. } = &mut read.kind else { panic!("expected read: {read:?}") };
                assert_eq!(*fd, CONFIG);
                assert!(body.len() <= buf.len());
                for (out, byte) in buf.iter_mut().zip(body) {
                    *out = *byte;
                }
                let count = u32::try_from(body.len()).expect("file within the read bound");
                self.complete(read, Ok(Done::Count(count)));
            }
        }
        let close = self.pending_after_step();
        let Op::Close { fd } = close.kind else { panic!("expected file close: {close:?}") };
        assert_eq!(fd, CONFIG);
        self.complete(close, Ok(Done::Nothing));
    }

    fn pending_after_step(&mut self) -> kernel::Submit {
        assert!(self.step().is_none());
        self.pending()
    }
}

#[test]
fn a_start_reads_times_only_inside_writable_hook_directories() {
    let bad = b"[deliver]\ntimeout_seconds = 900\ntimeout = 1";
    let cases = [
        (b"deliver".as_slice(), Config::Text(b"[deliver]\ntimeout_seconds = 900")),
        (b"finish", Config::Text(b"[finish]\ntimeout_seconds = 7200")),
        (b"bad", Config::Text(bad)),
        (b"missing-file", Config::Missing),
        (b"link-file", Config::Link),
        (b"directory-file", Config::Directory),
        (b"missing-hooks", Config::NoHooks(Error::NotFound)),
        (b"linked-hooks", Config::NoHooks(Error::TooManyLinks)),
        (b"regular-hooks", Config::NoHooks(Error::NotADirectory)),
    ];
    let mut mounts = skein_lib::List::with_capacity(10);
    for (name, _) in cases {
        mounts.push((name, true)).expect("one mount per case");
        if name == b"deliver" {
            mounts.push((b"read-only", false)).expect("one read-only mount between writable mounts");
        }
    }
    let mut fixture = Fixture::new(mounts.as_slice());
    for (name, content) in cases {
        fixture.directory(name, content);
    }
    let start = fixture.step().expect("all file operations closed before Start").start;
    assert!(fixture.reader.settled());
    let ceiling = Duration::from_secs(u64::from(CEILING));
    let empty = HookTimes { deliver: None, finish: None, ceiling, fault: Box::default() };
    let deliver = Duration::from_secs(900);
    let wrong_kind = b".temper/hooks/config.toml:0: not a regular file";
    let expected = [
        Some(HookTimes { deliver: Some(HookTime { time: deliver, asked: deliver }), ..empty.clone() }),
        None,
        Some(HookTimes { finish: Some(HookTime { time: ceiling, asked: Duration::from_secs(7200) }), ..empty.clone() }),
        Some(HookTimes {
            fault: b".temper/hooks/config.toml:3: unknown key `timeout`".as_slice().into(),
            ..empty.clone()
        }),
        Some(empty.clone()),
        Some(HookTimes { fault: wrong_kind.as_slice().into(), ..empty.clone() }),
        Some(HookTimes { fault: wrong_kind.as_slice().into(), ..empty }),
        None,
        None,
        None,
    ];
    assert_eq!(start.directories.len(), expected.len());
    for (directory, hooks) in start.directories.into_iter().zip(expected) {
        assert_eq!(directory.hooks, hooks, "start mount {:?}", directory.name);
    }
}

#[test]
fn an_overlong_hook_fault_is_cut_to_a_whole_text_prefix() {
    let file = bytes::concat(&[b"[deliver]\n", &[b'a'; 2000], b" = 1"]);
    let reported = hooks::times(&file).expect_err("unknown key").text(HOOK_CONFIG_READER);
    let start = load_result(Observed::Loaded(file), Duration::from_secs(u64::from(CEILING)));
    assert_eq!((start.deliver, start.finish), (None, None));
    assert!(reported.len() > 1024);
    assert_eq!(start.fault.as_ref(), reported.get(..1024).expect("reported line is over the bound"));
    assert!(bytes::is_utf8(&start.fault));
}
