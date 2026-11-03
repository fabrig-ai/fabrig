//! Bounded model panels (client/tools.md, 5.5 area 5).
//! Text and JSON retain the same model selection, omissions and digest.
use super::Text;
use skein_json::{Document, document::Kind};
use skein_lib::List;

pub(super) fn text(bytes: &[u8], out: &mut Text) {
    let length = u32::try_from(bytes.len()).expect("bounded model projection");
    let tokens = skein_json::tokenizer::Limits { depth: 32, string: length, number: 20, chunk: length, length };
    let retained = skein_json::document::Limits { tokens: length, text: length };
    let panel = Panel { document: skein_json::parse(bytes, &tokens, &retained).expect("encoded model projection") };
    for project in &panel.rows(0, b"models") {
        out.put(b"models for project ");
        panel.print(*project, b"project", out);
        out.put(b"\n");
        for provider in &panel.rows(*project, b"providers") {
            out.put(b"provider ");
            panel.print(*provider, b"provider", out);
            out.put(b" \xc2\xb7 ");
            panel.print(*provider, b"provider_name", out);
            out.put(b"\n");
            for model in &panel.rows(*provider, b"models") {
                panel.model(*model, out);
            }
        }
    }
    super::account_text::text(bytes, None, out);
}

struct Panel {
    document: Document,
}
impl Panel {
    fn rows(&self, object: u32, name: &[u8]) -> List<u32> {
        let array = self.document.member(object, name).expect("projected array");
        let end = self.document.value_end(array).expect("complete array").checked_sub(1).expect("array close");
        let first = array.checked_add(1).expect("first element");
        let mut at = first;
        let mut rows = List::with_capacity(end.checked_sub(first).expect("array span"));
        for cursor in first..end {
            if cursor == at {
                rows.push(cursor).expect("array span bounds rows");
                at = self.document.value_end(cursor).expect("projected element");
            }
        }
        rows
    }
    fn word(&self, at: u32, out: &mut Text) {
        let at = if self.document.token(at).expect("projected value").kind == Kind::ObjectStart {
            self.document.member(at, b"bytes").expect("projected document text")
        } else {
            at
        };
        let token = self.document.token(at).expect("projected scalar");
        if token.kind == Kind::Null {
            out.put(b"unknown");
        } else {
            out.words(self.document.text(token).expect("projected text or number"));
        }
    }
    fn print(&self, object: u32, name: &[u8], out: &mut Text) {
        self.word(self.document.member(object, name).expect("projected field"), out);
    }
    fn model(&self, model: u32, out: &mut Text) {
        out.put(b"  model ");
        self.print(model, b"name", out);
        out.put(b"\n    description: ");
        self.print(model, b"description", out);
        out.put(b"\n    window: ");
        self.print(model, b"window", out);
        out.put(b" tokens \xc2\xb7 output: ");
        self.print(model, b"output", out);
        out.put(b" tokens\n    efforts:");
        let efforts = self.rows(model, b"efforts");
        if efforts.is_empty() {
            out.put(b" none listed");
        }
        for effort in &efforts {
            out.put(b" ");
            self.word(*effort, out);
        }
        out.put(b"\n");
        self.prices(self.document.member(model, b"prices").expect("projected prices"), out);
    }
    fn prices(&self, prices: u32, out: &mut Text) {
        out.put(b"    prices (per ");
        self.print(prices, b"unit", out);
        out.put(b" tokens), deployment spend units:\n");
        for (key, label) in [
            (b"input".as_slice(), b"input".as_slice()),
            (b"cached", b"cached input"),
            (b"cache_write", b"cache write"),
            (b"output", b"output"),
        ] {
            out.put(b"      ");
            out.put(label);
            out.put(b": ");
            self.print(prices, key, out);
            out.put(b"\n");
        }
    }
}
