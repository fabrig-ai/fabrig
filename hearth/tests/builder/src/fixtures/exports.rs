use super::{File, thin_with};
const VPN_TASK: File = File { path: "tasks/vpn/task.json", mode: 0o644, body: b"{\"settings\": {\"address\": {\"type\": \"string\", \"doc\": \"Address.\"}, \"endpoint\": {\"type\": \"string\", \"optional\": true, \"doc\": \"Endpoint.\"}},\n \"exports\": {\"address\": {\"type\": \"string\", \"value\": \"{{ settings.address }}\", \"doc\": \"Address.\"}, \"endpoint\": {\"type\": \"string\", \"optional\": true, \"value\": \"{{ settings.endpoint }}\", \"doc\": \"Endpoint.\"}}}" };
const ADDRESS: File =
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "vpn": {"address": "10.0.0.2"}}"# };
const ENDPOINT: File = File {
    path: "hosts/x1/x1-prod.json",
    mode: 0o644,
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"},"vpn":{"endpoint":"198.51.100.7"}}"#,
};
pub const EXPORTS: &[File] = &[VPN_TASK, ADDRESS, ENDPOINT];
pub const DEEP_EXPORT: u32 = 29;
pub const DEEP_NEST_JSON: &[u8] = br#""nest":[[[[[[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]"#;
#[must_use]
pub fn export_value(value: &str) -> Vec<File> {
    let declaration = format!(
        "{{\"settings\": {{\"address\": {{\"type\": \"string\", \"doc\": \"Address.\"}}, \"endpoint\": {{\"type\": \"string\", \"optional\": true, \"doc\": \"Endpoint.\"}}}},\n \"exports\": {{\"address\": {{\"type\": \"string\", \"value\": \"{value}\", \"doc\": \"Address.\"}}}}}}"
    );
    thin_with(&[File { body: declaration.leak().as_bytes(), ..VPN_TASK }, ADDRESS, ENDPOINT])
}
#[must_use]
pub fn optional_export() -> Vec<File> {
    thin_with(&[VPN_TASK, ADDRESS])
}
#[must_use]
pub fn deep(depth: u32) -> Vec<File> {
    let mut ty = "\"int\"".to_owned();
    let mut value = "1".to_owned();
    for _ in 0..depth {
        ty = format!("{{\"list\":{ty}}}");
        value = format!("[{value}]");
    }
    let declaration =
        format!("{{\"exports\":{{\"nest\":{{\"type\":{ty},\"value\":\"{{{{ {value} }}}}\",\"doc\":\"Nest.\"}}}}}}");
    thin_with(&[
        File { body: declaration.leak().as_bytes(), ..VPN_TASK },
        File { body: br#"{"motd":{},"vpn":{}}"#, ..ADDRESS },
    ])
}
