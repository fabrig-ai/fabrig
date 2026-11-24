use super::File;
pub const ACCOUNTS: File = File { path: "tasks/motd/task.json",mode: 0o644,
body: br#"{"groups":{"ops":{"gid":1500},"mail":{"from":"package","gid":8}},"users":{"ci":{},"alice":{"uid":1000,"home":"/home/alice","shell":"/bin/bash","groups":["sudo","ops"]},"postfix":{"from":"package","uid":107,"gid":113}}}"# };
pub const ACCOUNTS_MOVED: File = File { path: "tasks/motd/task.json",mode: 0o644,
body: br#"{"groups":{"mail":{"from":"package","gid":8}},"users":{"ci":{},"alice":{"uid":1001,"home":"/home/alice","shell":"/bin/bash","groups":["sudo","video"]},"postfix":{"from":"package","uid":107,"gid":113}}}"# };
