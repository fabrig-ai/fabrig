//! Render/parse round trips at the rows' bounds (file.md, 10).
use crate::*;

struct Seed(u64);
impl Seed {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn yes(&mut self) -> bool {
        self.next().is_multiple_of(2)
    }
    fn text(&mut self, bound: usize) -> String {
        let len = if self.yes() {
            bound
        } else {
            1 + usize::try_from(self.next() % u64::try_from(bound).expect("bound")).expect("bounded")
        };
        (0..len).map(|_| char::from(b'a' + u8::try_from(self.next() % 26).expect("letter"))).collect()
    }
    fn path(&mut self) -> std::path::PathBuf {
        format!("secrets/{}-é-\"-\\-\n", self.text(12)).into()
    }
    fn transport(&mut self) -> Transport {
        if self.yes() {
            Transport::Plaintext
        } else {
            Transport::Tls { server_name: self.text(30), trust: if self.yes() { Some(self.path()) } else { None } }
        }
    }
}
fn file(seed: &mut Seed, shape: Shape) -> File {
    let mut f = File::default();
    if shape != Shape::WorkerAlone {
        engine(seed, &mut f);
    }
    if shape != Shape::EngineAlone {
        if seed.yes() {
            f.programs.git = Some(format!("/tools/{}", seed.text(10)).into());
        }
        if seed.yes() {
            f.programs.claude = Some("/tools/claude".into());
        }
        if shape == Shape::Standalone {
            if seed.yes() {
                f.programs.forgejo = Some("/tools/forgejo".into());
            }
            if seed.yes() {
                f.programs.runner = Some("/tools/runner".into());
            }
        }
        if seed.yes() {
            f.environment = vec![
                ("z.first".into(), seed.text(20)),
                ("a".into(), "quotes\"\\ and unicode é\n\t\r\u{000B}\u{007F}".into()),
            ];
        }
    }
    if shape == Shape::EngineAlone {
        f.workers = Some("127.0.0.1:7200".parse().expect("socket"));
    }
    if shape == Shape::WorkerAlone {
        f.engine = Some("[::1]:7200".parse().expect("socket"));
        if seed.yes() {
            f.name = Some(seed.text(32));
        }
    }
    f
}
fn engine(seed: &mut Seed, f: &mut File) {
    if seed.yes() {
        f.listen = Some("127.0.0.2:0".parse().expect("socket"));
    }
    if seed.yes() {
        f.webhooks = Some("[::1]:7101".parse().expect("socket"));
    }
    if seed.yes() {
        f.public = Some("https://example.com".into());
    }
    if seed.yes() {
        f.owner = Some("ada@localhost".into());
    }
    if seed.yes() {
        let count = if seed.yes() { 64 } else { 1 };
        f.sign_in = Some(SignIn {
            client: seed.text(256),
            secret: seed.path(),
            members: (0..count)
                .map(|i| Member { address: format!("person{i}@example.com"), login: format!("person{i}") })
                .collect(),
            forges: Vec::new(),
        });
    }
    if seed.yes() {
        let count = usize::try_from(seed.next() % 4).expect("small count");
        f.openid = Some(OpenId {
            key: if seed.yes() { Some(seed.path()) } else { None },
            addresses: if seed.yes() { Some(if seed.yes() { vec![1, 3] } else { Vec::new() }) } else { None },
            clients: (0..count)
                .map(|_| OpenIdClient {
                    id: seed.text(255),
                    secret: if seed.yes() { Some(seed.path()) } else { None },
                    redirect: format!("https://example.com/{}", seed.text(20)),
                })
                .collect(),
        });
        if let Some(sign_in) = &mut f.sign_in {
            sign_in.forges = f
                .openid
                .as_ref()
                .expect("provider")
                .clients
                .iter()
                .enumerate()
                .map(|(i, c)| ForgeSignIn { forge: u16::try_from(i + 1).expect("few forges"), client: c.id.clone() })
                .collect();
        }
    }
    if seed.yes() {
        f.push = Some(Push { key: seed.path(), subject: "mailto:operator@example.com".into() });
        if seed.yes() {
            f.test.push_services = Some(vec!["http://127.0.0.1:8000".into(); if seed.yes() { 8 } else { 1 }]);
        }
    }
    if seed.yes() {
        f.internal.bundle = Some(format!("/bundle/{}", seed.text(20)).into());
    }
    if seed.yes() {
        f.test.github = Some(TestGitHub {
            api: if seed.yes() { "127.0.0.1:8300".into() } else { "[::1]:8300".into() },
            transport: seed.transport(),
            remote: "http://127.0.0.1:8300/git".into(),
            web: "http://127.0.0.1:8300".into(),
            git: if seed.yes() { Some(seed.path()) } else { None },
        });
    }
    if f.sign_in.is_some() && seed.yes() {
        f.test.google = Some(TestGoogle {
            issuer: "http://127.0.0.1:9000".into(),
            authorization: "http://127.0.0.1:9000/auth".into(),
            token: "http://127.0.0.1:9000/token".into(),
            keys: "http://127.0.0.1:9000/keys".into(),
            transport: seed.transport(),
        });
    }
}
#[test]
fn rendered_files_round_trip_without_materializing_defaults() {
    for case in 1..=512 {
        for shape in [Shape::Standalone, Shape::EngineAlone, Shape::WorkerAlone] {
            let f = file(&mut Seed(case), shape);
            let text = render(&f, shape);
            let parsed = parse(&text, shape, &tests::facts()).expect("rendered file parses");
            assert_eq!(parsed.file, f, "seed {case}, {shape:?}: {}", String::from_utf8_lossy(&text));
        }
    }
}

#[test]
fn old_spelling_converts_and_round_trips_at_the_newest_format() {
    use crate::{
        formats::tests::TWO,
        parse::parse_at,
        render::{quote, render_at},
    };
    use std::fmt::Write;

    let facts = tests::facts();
    for case in 1..=512 {
        for shape in [Shape::Standalone, Shape::EngineAlone, Shape::WorkerAlone] {
            let mut f = file(&mut Seed(case), shape);
            let git = f.programs.git.take();
            let listen = f.listen.take();
            let mut old = String::from("seed_count = 11\nretired_flag = true\n");
            if let Some(git) = &git {
                writeln!(old, "git_binary = {}", quote(git.to_str().expect("git path"))).expect("string");
            }
            if let Some(listen) = listen {
                writeln!(old, "listen_on = [\"{listen}\"]").expect("string");
            }
            let text = [old.as_bytes(), &render(&f, shape)].concat();
            f.programs.git = git;
            f.listen = listen;
            let parsed = parse_at(&text, shape, &facts, TWO).expect("old format parses");
            assert_eq!(parsed.file, f, "old format, seed {case}, {shape:?}");
            let newest = render_at(&parsed.file, shape, TWO);
            let again = parse_at(&newest, shape, &facts, TWO).expect("newest format parses");
            assert_eq!(again.file, f, "newest format, seed {case}, {shape:?}");
        }
    }
}
