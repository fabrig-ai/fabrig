//! The newest opening line supplies each value (client/drive.md, 3.4).
use crate::line::BoxValue;

pub(super) fn read(log: &[u8], name: BoxValue) -> Result<Box<[u8]>, String> {
    let prefix = match name {
        BoxValue::Api | BoxValue::Git => b"box: fake forgejo ".as_slice(),
        BoxValue::Admin => b"box: Forgejo elsewhere admin token file ".as_slice(),
        BoxValue::GitProgram => b"box: effective data ".as_slice(),
    };
    let value = log
        .split(|byte| *byte == b'\n')
        .filter_map(|line| super::strip_log_time(line).strip_prefix(prefix))
        .next_back()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("session/box.log: lacks {} line", String::from_utf8_lossy(prefix).trim_end()))?;
    match name {
        BoxValue::Api => value
            .strip_prefix(b"http://")
            .map(Box::from)
            .ok_or_else(|| "box: fake forgejo: expected http://HOST:PORT".into()),
        BoxValue::Git | BoxValue::Admin => Ok(value.into()),
        BoxValue::GitProgram => Ok([value, b"/demo/bin/git"].concat().into()),
    }
}
