//! A real CLI export interrupted at its Written boundary (temper/docs/design/client/ctl/testing.md, 4).
#![expect(
    clippy::disallowed_methods,
    reason = "The process fixture joins the socket relay's workers; its gate and process waits use the story's patience."
)]
use super::{
    WITHIN,
    patience::Patience,
    story_socket::{Scope, Socket},
};
use skein_world::{Deadline, Expired};
use std::{
    fs,
    io::{self, Read},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration as WallDuration,
};

pub struct Running {
    child: Option<Child>,
    tool: String,
    in_world: bool,
}
impl Running {
    pub fn terminate(&self) {
        let status = Command::new("/usr/bin/kill")
            .args(["-TERM", &self.child.as_ref().expect("ctl").id().to_string()])
            .env_clear()
            .status()
            .expect("signal ctl");
        assert!(status.success());
    }
    #[must_use]
    pub fn finish(mut self) -> Output {
        let mut deadline = (!self.in_world).then(|| Deadline::arm("the account-move ctl probe", WITHIN));
        let mut patience = self.in_world.then(|| Patience::arm("the account-move ctl probe", WITHIN));
        let pid = self.child.as_ref().expect("ctl").id();
        if let Some(deadline) = &mut deadline {
            deadline.watch_process(pid);
        }
        if let Some(patience) = &mut patience {
            patience.watch_process(pid);
        }
        while self.child.as_mut().expect("ctl").try_wait().expect("poll ctl").is_none() {
            if let Some(patience) = &mut patience
                && let Some(figures) = patience.failed()
            {
                panic!("ctl {} exceeded its probe deadline: {figures}", self.tool);
            }
            if let Some(deadline) = &deadline
                && deadline.passed()
            {
                match deadline.expired() {
                    Expired::Starved(starved) => {
                        super::parent::end_child(self.child.as_mut().expect("ctl"));
                        starved.pass();
                    }
                    Expired::Failed(figures) => panic!("ctl {} exceeded its probe deadline: {figures}", self.tool),
                }
            }
            thread::sleep(WallDuration::from_millis(2));
        }
        self.child.take().expect("ctl").wait_with_output().expect("ctl output")
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn success(output: &Output) {
    assert!(output.status.success(), "ctl {:?}: {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
}
/// Leave the real CLI's export durably moving, with Written held off the engine.
/// Reuse the account-transfer relay: no moved record or client credential is inspected.
#[must_use]
pub fn export_pending(directory: &Path, upstream: SocketAddr, account: u32, file: &Path) -> serde_json::Value {
    let client = directory.join("client");
    let relay = Relay::start(upstream);
    super::write_private(&client.join("address"), format!("http://{}\n", relay.address).as_bytes());
    let gate = relay.arm(2, Boundary::BeforeRequest);
    let child = Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("ctl")
        .arg(&client)
        .args(["account", "export"])
        .arg(account.to_string())
        .arg(file)
        .current_dir(directory)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("pending fixture export");
    let export = Running { child: Some(child), tool: "export".into(), in_world: true };
    gate.wait();
    assert!(fs::metadata(file).expect("synced fixture move file").len() > 0, "record reached the CLI before Written");
    export.terminate();
    assert_eq!(export.finish().status.code(), Some(143), "stop the CLI while its Written is held");
    gate.release(false);
    super::write_private(&client.join("address"), format!("http://{upstream}\n").as_bytes());
    drop(relay);
    let output = Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("ctl")
        .arg(&client)
        .args(["account", "list", "--json"])
        .current_dir(directory)
        .env_clear()
        .output()
        .expect("pending account read");
    success(&output);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("pending accounts JSON");
    let rows = value["accounts"]["party"].as_array().expect("fixture accounts");
    let row = rows.iter().find(|row| row["number"] == account).expect("export keeps its account until Written");
    assert!(
        row["sources"]
            .as_array()
            .expect("fixture sources")
            .iter()
            .any(|source| source["kind"] == "sign_in" && source["state"]["kind"] == "moving"),
        "the source is held moving before SIGTERM: {row}"
    );
    value
}

#[derive(Clone, Copy)]
pub enum Boundary {
    BeforeRequest,
    AfterReply,
}
struct Rule {
    left: usize,
    boundary: Boundary,
    reached: mpsc::Sender<()>,
    release: mpsc::Receiver<bool>,
}
pub struct Gate {
    reached: mpsc::Receiver<()>,
    release: Option<mpsc::Sender<bool>>,
}
impl Gate {
    pub fn wait(&self) {
        let mut patience = Patience::arm("the account relay's armed request", WITHIN);
        loop {
            let received = match patience.socket_timeout() {
                Some(timeout) => self.reached.recv_timeout(timeout),
                None => self.reached.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected),
            };
            match received {
                Ok(()) => return,
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("relay disconnected before its armed request"),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(figures) = patience.failed() {
                        panic!("relay reaches the armed request: {figures}");
                    }
                }
            }
        }
    }
    pub fn release(mut self, deliver: bool) {
        let _ = self.release.take().expect("gate sender").send(deliver);
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        if let Some(sender) = self.release.take() {
            let _ = sender.send(false);
        }
    }
}
pub struct Relay {
    address: SocketAddr,
    rule: Arc<Mutex<Option<Rule>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Relay {
    fn start(upstream: SocketAddr) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("relay listener");
        listener.set_nonblocking(true).expect("nonblocking relay");
        let address = listener.local_addr().expect("relay address");
        let rule = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_rule = Arc::clone(&rule);
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            let mut handlers = Vec::new();
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((socket, _)) => {
                        let rule = Arc::clone(&thread_rule);
                        let stop = Arc::clone(&thread_stop);
                        handlers.push(thread::spawn(move || {
                            if let Err(error) = relay(socket, upstream, &rule, &stop) {
                                eprintln!("account-transfer relay connection ended: {error}");
                            }
                        }));
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(WallDuration::from_millis(1));
                    }
                    Err(error) => panic!("relay accept: {error}"),
                }
            }
            for handler in handlers {
                handler.join().expect("relay handler");
            }
        });
        Self { address, rule, stop, thread: Some(thread) }
    }
    /// No other keyed acts run while armed: export sends `move_out`, then `written`.
    #[must_use]
    pub fn arm(&self, request: usize, boundary: Boundary) -> Gate {
        let (reached, reached_receiver) = mpsc::channel();
        let (release, release_receiver) = mpsc::channel();
        *self.rule.lock().expect("relay rule") =
            Some(Rule { left: request, boundary, reached, release: release_receiver });
        Gate { reached: reached_receiver, release: Some(release) }
    }
}
impl Drop for Relay {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.rule.lock().expect("relay rule").take();
        self.thread.take().expect("relay thread").join().expect("relay drains");
    }
}

fn length(head: &[u8]) -> usize {
    std::str::from_utf8(head)
        .expect("HTTP head")
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().expect("content length"))
        })
        .unwrap_or(0)
}
fn gate(rule: &Rule) -> bool {
    let _ = rule.reached.send(());
    let mut patience = Patience::arm("the account relay's gate release", WITHIN);
    loop {
        let received = match patience.socket_timeout() {
            Some(timeout) => rule.release.recv_timeout(timeout),
            None => rule.release.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        match received {
            Ok(deliver) => return deliver,
            Err(mpsc::RecvTimeoutError::Disconnected) => return false,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(figures) = patience.failed() {
                    eprintln!("relay gate release timed out: {figures}");
                    return false;
                }
            }
        }
    }
}
fn relay(
    downstream: TcpStream,
    upstream: SocketAddr,
    rules: &Mutex<Option<Rule>>,
    stop: &AtomicBool,
) -> io::Result<()> {
    let mut downstream = Socket::new(downstream, Scope::World);
    let request = downstream.head("the account relay's request head", WITHIN)?;
    let mut body = vec![0; length(&request)];
    assert!(body.len() <= 262_144, "bounded request");
    downstream.read_exact("the account relay's request body", WITHIN, &mut body)?;
    let text = std::str::from_utf8(&request).expect("request head");
    let rule = if text.lines().next().expect("request line").contains("/v1/request ") {
        let mut rules = rules.lock().expect("relay rule");
        if let Some(rule) = rules.as_mut() {
            rule.left -= 1;
            if rule.left == 0 { rules.take() } else { None }
        } else {
            None
        }
    } else {
        None
    };
    if let Some(rule) = &rule
        && matches!(rule.boundary, Boundary::BeforeRequest)
        && !gate(rule)
    {
        return Ok(());
    }
    let mut server = Socket::connect("the account relay's upstream connection", WITHIN, upstream, Scope::World)?;
    server.stream().set_read_timeout(Some(WallDuration::from_millis(50)))?;
    let mut forwarded = String::new();
    for line in text.lines() {
        if line.is_empty() || line.to_ascii_lowercase().starts_with("connection:") {
            continue;
        }
        if line.to_ascii_lowercase().starts_with("host:") {
            forwarded.push_str("Host: ");
            forwarded.push_str(&upstream.to_string());
            forwarded.push_str("\r\n");
        } else {
            forwarded.push_str(line);
            forwarded.push_str("\r\n");
        }
    }
    forwarded.push_str("Connection: close\r\n\r\n");
    server.write_all("the account relay's upstream request", WITHIN, forwarded.as_bytes())?;
    server.write_all("the account relay's upstream body", WITHIN, &body)?;
    if let Some(rule) = &rule
        && matches!(rule.boundary, Boundary::AfterReply)
    {
        let mut response = server.head("the account relay's upstream reply head", WITHIN)?;
        let size = length(&response);
        assert!(size <= 262_144, "bounded reply");
        let offset = response.len();
        response.resize(offset + size, 0);
        server.read_exact("the account relay's upstream reply body", WITHIN, &mut response[offset..])?;
        if gate(rule) {
            downstream.write_all("the account relay's held reply", WITHIN, &response)?;
        }
    } else {
        let mut buffer = [0; 8192];
        while !stop.load(Ordering::Relaxed) {
            match server.stream().read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => downstream.write_all("the account relay's forwarded reply", WITHIN, &buffer[..count])?,
                Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
                Err(error) => return Err(error),
            }
        }
    }
    let _ = downstream.stream().shutdown(Shutdown::Both);
    Ok(())
}
