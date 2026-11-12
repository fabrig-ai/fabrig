//! The browser tier's static host (temper/docs/design/client/web/testing.md, 3;
//! temper/docs/design/playground.md, 7), with its table and front
//! (temper/docs/design/testing/tools.md, 3 and 4).
use skein_shell::{Argument, Command, Front, Reach, Set, Table};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

const SERVE: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &["OUT"],
    line: "Serve the playground's bundle as static files with its headers.",
    arguments: &[Argument { shown: "OUT", line: "The built playground's output directory." }],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};

/// The static host's command line (temper/docs/design/testing/tools.md, 4).
pub const TABLE: Table = Table {
    binary: "temper-playground-serve",
    line: "the browser tier's static host for the playground's bundle.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[SERVE],
    settings: &[],
    variables: &[],
    more: "Read temper/docs/design/playground.md, section 7.",
};

struct Host {
    root: PathBuf,
    headers: Vec<(String, String, String)>,
}

impl Host {
    fn open(root: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let text = fs::read_to_string(root.join("headers"))?;
        let mut headers: Vec<(String, String, String)> = Vec::new();
        for line in text.lines() {
            let (path, field) = line.split_once(": ").expect("header path");
            let (name, value) = field.split_once(": ").expect("header value");
            assert!(!name.contains(['\r', '\n']) && !value.contains(['\r', '\n']));
            headers.push((path.strip_suffix('*').expect("header wildcard").into(), name.into(), value.into()));
        }
        headers.sort_by_key(|(prefix, _, _)| prefix.len());
        Ok(Self { root, headers })
    }

    fn fields(&self, path: &str) -> BTreeMap<String, String> {
        let mut fields = BTreeMap::new();
        for (prefix, name, value) in &self.headers {
            if path.starts_with(prefix) {
                fields.insert(name.clone(), value.clone());
            }
        }
        fields
    }

    fn file(&self, path: &str) -> io::Result<PathBuf> {
        // Bundle names are literal file names; refuse escaped separators and traversal too.
        if !path.starts_with('/')
            || path.contains(['%', '\\', '\0'])
            || Path::new(path).components().any(|part| matches!(part, Component::ParentDir))
        {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        let relative = path.strip_prefix("/bundle/").unwrap_or("index.html");
        let file = self.root.join(relative).canonicalize()?;
        if !file.starts_with(&self.root) || !file.is_file() {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        Ok(file)
    }

    fn serve(&self, stream: &mut TcpStream) -> io::Result<()> {
        stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
        let mut head = Vec::new();
        for _ in 0..8192 {
            let mut byte = [0];
            if stream.read(&mut byte)? == 0 {
                return Ok(());
            }
            head.push(byte[0]);
            if head.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        let text = String::from_utf8_lossy(&head);
        let mut words = text.split_whitespace();
        let method = words.next().unwrap_or("");
        let path = words.next().unwrap_or("/").split('?').next().expect("path");
        let mut fields = self.fields(path);
        let (status, body, content_type) = if matches!(method, "GET" | "HEAD") {
            match self.file(path) {
                Ok(file) => {
                    let body = fs::read(&file)?;
                    ("200 OK", body, mime(&file))
                }
                Err(why) => {
                    let status = if why.kind() == io::ErrorKind::NotFound { "404 Not Found" } else { "403 Forbidden" };
                    (status, Vec::new(), "text/plain")
                }
            }
        } else {
            fields.insert("Allow".into(), "GET, HEAD".into());
            ("405 Method Not Allowed", Vec::new(), "text/plain")
        };
        write!(
            stream,
            "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\n",
            body.len()
        )?;
        for (name, value) in fields {
            write!(stream, "{name}: {value}\r\n")?;
        }
        stream.write_all(b"\r\n")?;
        if method != "HEAD" {
            stream.write_all(&body)?;
        }
        Ok(())
    }
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        Some("webmanifest") => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

/// Capture the arguments once and answer through the table before opening a host.
#[must_use]
pub fn main() -> ExitCode {
    #[expect(
        clippy::disallowed_methods,
        reason = "the binary's capture: temper-playground-serve's arguments, once (skein/docs/foundation/cli.md, 6)"
    )]
    let arguments = skein_shell::arguments();
    match TABLE.front(&arguments) {
        Front::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            ExitCode::from(2)
        }
        Front::Run { command, rest } => {
            let root = match rest {
                [] => return refuse(command, "OUT is not given"),
                [root] => root,
                [_, _, ..] => return refuse(command, "expected one OUT"),
            };
            if root.starts_with(b"-") {
                return refuse(command, &format!("no flag {}", String::from_utf8_lossy(root)));
            }
            match serve(Path::new(std::ffi::OsStr::from_bytes(root))) {
                Ok(()) => ExitCode::SUCCESS,
                Err(why) => {
                    eprintln!("temper-playground-serve: {why}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn refuse(command: &Command, why: &str) -> ExitCode {
    eprint!("{}", TABLE.refusal(command, why));
    ExitCode::from(2)
}

fn serve(root: &Path) -> io::Result<()> {
    let host = Host::open(root)?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    println!("http://{}/", listener.local_addr()?);
    io::stdout().flush()?;
    for stream in listener.incoming() {
        if let Err(why) = host.serve(&mut stream?) {
            eprintln!("temper-playground-serve: {why}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Host, TABLE};
    use std::{fs, os::unix::fs::symlink};

    #[test]
    fn the_reference_documents_match_the_table() {
        let write = temper_test_env::rewrite();
        let directory = skein_scratch::paths::tree().join("docs/design/cli");
        for (name, rendered) in [
            ("temper-playground-serve-args.md", TABLE.arguments_document()),
            ("temper-playground-serve-env.md", TABLE.environment_document(&[&TABLE])),
        ] {
            let path = directory.join(name);
            if write {
                fs::create_dir_all(&directory).expect("reference directory");
                fs::write(&path, &rendered).expect("write table reference");
            } else {
                assert_eq!(
                    fs::read_to_string(&path).expect("table reference"),
                    rendered,
                    "{name}; regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
                );
            }
        }
    }

    const FILES_CACHE: &[u8] = b"public, max-age=31536000, immutable";
    const PAGE_CACHE: &[u8] = b"no-cache";

    fn bundle() -> skein_scratch::Scratch {
        let scratch = skein_scratch::Scratch::new("playground-host");
        fs::create_dir_all(scratch.path().join("out/digest")).expect("digest directory");
        fs::write(scratch.path().join("out/index.html"), b"page").expect("page");
        fs::write(scratch.path().join("out/digest/boot.js"), b"script").expect("script");
        let headers =
            include_str!("../../temper-playground-page/playground/headers").replace("{bundle}", "/bundle/digest");
        fs::write(scratch.path().join("out/headers"), headers).expect("headers");
        scratch
    }

    #[test]
    fn digest_files_and_page_routes_apply_the_headers_file() {
        let scratch = bundle();
        let root = scratch.path().join("out");
        let host = Host::open(&root).expect("host");
        for (path, file, cache) in [
            ("/bundle/digest/boot.js", "digest/boot.js", FILES_CACHE),
            ("/", "index.html", PAGE_CACHE),
            ("/chats/1", "index.html", PAGE_CACHE),
        ] {
            assert_eq!(host.file(path).expect("mapped file"), root.join(file).canonicalize().expect("file"));
            let fields = host.fields(path);
            assert_eq!(fields["Cache-Control"].as_bytes(), cache);
            assert!(fields["Content-Security-Policy"].contains("worker-src 'self'"));
            for (name, value) in [
                ("X-Content-Type-Options", "nosniff"),
                ("Cross-Origin-Opener-Policy", "same-origin"),
                ("Cross-Origin-Resource-Policy", "same-origin"),
                ("Referrer-Policy", "no-referrer"),
            ] {
                assert_eq!(fields[name], value);
            }
        }
    }

    #[test]
    fn paths_and_symlinks_outside_the_output_are_refused() {
        let scratch = bundle();
        fs::write(scratch.path().join("outside"), b"outside").expect("outside file");
        symlink(scratch.path().join("outside"), scratch.path().join("out/digest/escape")).expect("symlink");
        let host = Host::open(&scratch.path().join("out")).expect("host");
        for path in ["/bundle/../outside", "/../outside", "/bundle/%2e%2e/outside", "/bundle/digest/escape"] {
            assert_eq!(host.file(path).expect_err("outside refused").kind(), std::io::ErrorKind::PermissionDenied);
        }
    }
}
