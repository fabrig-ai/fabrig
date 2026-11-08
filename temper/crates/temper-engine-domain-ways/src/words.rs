//! Agent card words (temper/docs/design/domain/forge.md, 8.13).

/// The named tool families, followed by the charter's merge boundary.
#[must_use]
#[expect(clippy::fn_params_excessive_bools, reason = "the contract names four independent tool families")]
pub fn can(inspect: bool, modify: bool, shell: bool, agents: bool) -> &'static [u8] {
    let families =
        u8::from(inspect) | (u8::from(modify) << 1_u8) | (u8::from(shell) << 2_u8) | (u8::from(agents) << 3_u8);
    match families {
        0 => b"finish alone; never merge or update",
        1 => b"read the code and the forge; never merge or update",
        2 => b"change and push its branch; never merge or update",
        3 => b"read the code and the forge; change and push its branch; never merge or update",
        4 => b"run commands; never merge or update",
        5 => b"read the code and the forge; run commands; never merge or update",
        6 => b"change and push its branch; run commands; never merge or update",
        7 => b"read the code and the forge; change and push its branch; run commands; never merge or update",
        8 => b"start sub-agents; never merge or update",
        9 => b"read the code and the forge; start sub-agents; never merge or update",
        10 => b"change and push its branch; start sub-agents; never merge or update",
        11 => b"read the code and the forge; change and push its branch; start sub-agents; never merge or update",
        12 => b"run commands; start sub-agents; never merge or update",
        13 => b"read the code and the forge; run commands; start sub-agents; never merge or update",
        14 => b"change and push its branch; run commands; start sub-agents; never merge or update",
        15 => b"read the code and the forge; change and push its branch; run commands; start sub-agents; never merge or update",
        _ => unreachable!("four tool-family bits"),
    }
}

/// Through the first ". " or ".\n", trimmed, at most `crate::VALUE` bytes at a character's end.
#[must_use]
pub fn first_sentence(instructions: &[u8]) -> &[u8] {
    let mut sentence = instructions;
    for (at, pair) in instructions.windows(2).enumerate() {
        if pair == b". " || pair == b".\n" {
            let end = at.checked_add(1).expect("full stop within input");
            sentence = instructions.get(..end).expect("sentence prefix");
            break;
        }
    }
    skein_lib::bytes::utf8_prefix(sentence.trim_ascii(), crate::VALUE).trim_ascii_end()
}
