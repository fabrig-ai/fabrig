//! Forge choice and requests awaiting their bodies (temper/docs/plan/playground-reset-r4/notes/contracts.md, 5).
use crate::{Answered, Domain, Mode, Request};
use skein_lib::{Queue, Token};

pub(crate) fn refuse(d: &mut Domain, request: Token, token: Token, answer: Answered, out: &mut Queue<Request>) {
    d.connect.answer(request, token, answer, out);
}

pub(crate) fn set_up_forge(d: &Domain) -> Option<u16> {
    let mut selected = None;
    for (forge, config) in &d.configured {
        let lower = match selected {
            Some(number) => *forge < number,
            None => true,
        };
        let removing = match d.forge_record(*forge) {
            Some(row) => row.removing,
            None => false,
        };
        if config.mode == Mode::Owned && config.asked && !removing && !d.forge_ended(*forge) && lower {
            selected = Some(*forge);
        }
    }
    selected
}
