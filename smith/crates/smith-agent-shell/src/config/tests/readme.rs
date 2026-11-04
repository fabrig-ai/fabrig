//! The README's configuration is one the agent accepts
//! (smith/docs/design/protocol/agent.md, section 4).

use super::*;

const README: &str = include_str!("../../../README.md");

/// Each JSON fenced block of `text`, in order.
fn json_blocks(text: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut remaining = text;
    while let Some((_, contents)) = remaining.split_once("```json\n") {
        let (block, rest) = contents.split_once("\n```").expect("JSON fence has a closing fence");
        blocks.push(block);
        remaining = rest;
    }
    blocks
}

#[test]
fn the_readme_configuration_decodes() {
    let blocks = json_blocks(README);
    assert!(!blocks.is_empty(), "the README shows a configuration");
    for block in blocks {
        decode(block.as_bytes()).expect("the README's configuration decodes");
    }
}
