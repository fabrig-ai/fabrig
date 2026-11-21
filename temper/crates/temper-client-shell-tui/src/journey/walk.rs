//! Read a recording’s last browse address (temper/docs/design/client/drive.md, 5).
use super::{Line, Unreadable, lines};

/// Read the last browse address, including saves without one.
pub fn browsed(save: &[u8]) -> Result<Option<Box<[u8]>>, Unreadable> {
    Ok(lines(save)?.into_iter().rev().find_map(
        |line| {
            if let Line::Browse(address) = line { Some(address) } else { None }
        },
    ))
}

#[cfg(test)]
mod tests;
