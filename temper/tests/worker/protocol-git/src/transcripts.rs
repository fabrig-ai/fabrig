//! Raw captured bytes; tests never run git to refresh them.
/// The four files of a real-git invocation.
#[derive(Debug)]
pub struct Transcript {
    pub argv: Box<[u8]>,
    pub exit: u8,
    pub stdout: Box<[u8]>,
    pub stderr: Box<[u8]>,
}
/// Read the capture by case directory, including its masked argument vector.
#[must_use]
pub fn read(case: &str) -> Transcript {
    let path = skein_scratch::paths::tree().join("tools/git-transcripts/transcripts").join(case);
    let bytes = |name| std::fs::read(path.join(name)).expect("captured file").into_boxed_slice();
    let exit = bytes("exit");
    let exit = std::str::from_utf8(&exit).expect("exit is ASCII").trim().parse().expect("exit code");
    Transcript { argv: bytes("argv"), exit, stdout: bytes("stdout"), stderr: bytes("stderr") }
}
/// Drop each line, cut it midway, uppercase a hex digit, and drop the terminator.
#[must_use]
pub fn mutations(bytes: &[u8]) -> Vec<Box<[u8]>> {
    let mut out = Vec::new();
    let mut start = 0;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' && *byte != 0 {
            continue;
        }
        let end = at + 1;
        let mut dropped = bytes[..start].to_vec();
        dropped.extend_from_slice(&bytes[end..]);
        out.push(dropped.into_boxed_slice());
        let mut cut = bytes[..start + (at - start) / 2].to_vec();
        cut.extend_from_slice(&bytes[end..]);
        out.push(cut.into_boxed_slice());
        start = end;
    }
    // Captured ids have only digits after masking; A raises one into an
    // uppercase hex digit, which the strict id reader must refuse.
    if let Some(at) = bytes.windows(7).position(|part| part.iter().all(u8::is_ascii_hexdigit)) {
        let mut raised = bytes.to_vec();
        raised[at] = b'A';
        out.push(raised.into_boxed_slice());
    }
    if !bytes.is_empty() {
        out.push(bytes[..bytes.len() - 1].into());
    }
    out
}
