//! The local usage command's scripted result (protocol/programs.md, 5.11 and 8).
use crate::support::{Invocation, run};
use skein_io::kernel::Exit;
use smith_fake_claude::{Story, UsageReply};

#[test]
fn the_usage_command_prints_the_storys_result_as_one_json_line() {
    let UsageReply::Result(default) = UsageReply::default() else { unreachable!("default usage result") };
    assert_eq!(default["result"], "Current session: 0% used · resets Nov 19, 4:30am (UTC)\n");
    for value in [
        default,
        serde_json::json!({"type":"result", "is_error":false, "result":"Current session: 16% used · resets Nov 19, 4:30am (UTC)\nCurrent week (all models): 60% used · resets Nov 23, 5am (UTC)\nCurrent week (Fable): 8% used · resets Nov 23, 4:59am (UTC)\n", "usage":{"input_tokens":0}, "total_cost_usd":0}),
        serde_json::json!({"type":"result", "is_error":true, "result":"usage refused"}),
        serde_json::json!({"type":"result", "is_error":false, "result":"spent", "usage":{"output_tokens":1}, "total_cost_usd":0.01}),
    ] {
        let story = Story { usage_read: UsageReply::Result(value.clone()), ..Story::default() };
        let result = run(
            Some(story),
            Some(b"claude.ai"),
            vec![Invocation::auth("status").arguments(&["-p", "/usage", "--output-format", "json"]).no_input()],
        );
        let record = result.records.first().expect("usage invocation");
        assert_eq!(record.exit, Some(Exit::Code(0)));
        assert!(record.stderr.is_empty());
        assert_eq!(record.stdout.iter().filter(|byte| **byte == b'\n').count(), 1);
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&record.stdout).expect("result JSON"), value);
    }
}
