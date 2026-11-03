//! The browser tier's static host (temper/docs/design/client/browser.md, 4.3;
//! temper/docs/design/playground.md, 7).
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
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
        if !path.starts_with('/') || path.contains(['%', '\\', '\0'])
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
        let (status, body, content_type) = if !matches!(method, "GET" | "HEAD") {
            fields.insert("Allow".into(), "GET, HEAD".into());
            ("405 Method Not Allowed", Vec::new(), "text/plain")
        } else {
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
        };
        write!(stream, "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\n", body.len())?;
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

fn main() -> io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let root = args.next().expect("usage: playground-serve OUT");
    assert!(args.next().is_none(), "usage: playground-serve OUT");
    let host = Host::open(Path::new(&root))?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    println!("http://{}/", listener.local_addr()?);
    io::stdout().flush()?;
    for stream in listener.incoming() {
        if let Err(why) = host.serve(&mut stream?) {
            eprintln!("playground-serve: {why}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Host;
    use std::{fs, os::unix::fs::symlink};

    const FILES_CACHE: &[u8] = b"public, max-age=31536000, immutable";
    const PAGE_CACHE: &[u8] = b"no-cache";

    fn bundle() -> skein_scratch::Scratch {
        let scratch = skein_scratch::Scratch::new("playground-host");
        fs::create_dir_all(scratch.path().join("out/digest")).expect("digest directory");
        fs::write(scratch.path().join("out/index.html"), b"page").expect("page");
        fs::write(scratch.path().join("out/digest/boot.js"), b"script").expect("script");
        let headers = include_str!("../../../../crates/temper-client-shell-web/playground/headers")
            .replace("{bundle}", "/bundle/digest");
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
