//! Help bytes for the host to write, bounded by the tools' text window.
use crate::{jig, limits, table};
use skein_lib::{Overflow, Writer};
/// None: the ctl page and tools. Some: the selected table row's record.
pub fn text(tool: Option<u8>, out: &mut Writer) -> Result<(), Overflow> {
    let mut measured = Writer::new(usize::try_from(limits::TEXT).expect("text fits usize"));
    write(tool, &mut measured)?;
    let mut exact = Writer::new(measured.written());
    write(tool, &mut exact)?;
    out.put(&exact.finish())
}
fn write(tool: Option<u8>, out: &mut Writer) -> Result<(), Overflow> {
    match tool {
        None => {
            out.put(temper_docs::find(b"ctl").expect("compiled ctl page").text.as_bytes())?;
            jig::help::tools(table::TOOLS, out)
        }
        Some(index) => {
            let row = table::TOOLS.get(usize::from(index)).ok_or(Overflow)?;
            for later in table::TOOLS.get(usize::from(index)..).ok_or(Overflow)? {
                if later.name == row.name {
                    jig::help::tool(later, out)?;
                    if later.name == b"pending" {
                        out.put(PENDING)?;
                    }
                }
            }
            Ok(())
        }
    }
}

/// The local pending tool's result, without opening a client.
const PENDING: &[u8] = br#"Reads only the store: no lock, client or send.
Rows: operation, party N, first sending in wall-time words or not sent.
chat, party 3, not sent
{"pending":[{"operation":"chat","party":3,"first_sent":null}],"unread":2}
2 kept records these tools cannot read; a call that sends drops them
Unread records are counted, never listed; exit 0.
An invalid store: DIRECTORY's store cannot be read (exit 1).
"#;
