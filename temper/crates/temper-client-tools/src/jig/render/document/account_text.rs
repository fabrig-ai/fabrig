//! Account lines (tools.md, 5.5 area 5 and 6.1).
//! The encoder walks the supplied contract and charges every structural take;
//! presenting that bounded projection preserves JSON's omissions and digest.
use super::Text;
use skein_json::{Document, document::Kind};
use skein_lib::List;

const DOT: &[u8] = b" \xc2\xb7 ";

pub(super) fn text(bytes: &[u8], viewer: Option<u64>, out: &mut Text) {
    let length = u32::try_from(bytes.len()).expect("bounded account projection");
    let limits = skein_json::tokenizer::Limits { depth: 32, string: length, number: 20, chunk: length.max(1), length };
    let document = skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length })
        .expect("encoded account projection");
    let p = Projection { document, viewer };
    out.put(b"current: ");
    p.scalar(p.field(0, b"current"), out);
    out.put(b"\n");
    let accounts = p.field(0, b"accounts");
    if p.kind(accounts) == Kind::Null {
        out.put(b"accounts: unavailable\n");
    } else {
        let hosts = p.field(accounts, b"program_hosts");
        p.accounts(p.field(accounts, b"party"), hosts, out);
        for signing in &p.items(p.field(accounts, b"signing")) {
            out.put(b"signing: source ");
            p.scalar(p.field(*signing, b"source"), out);
            out.put(DOT);
            out.put(b"provider ");
            p.scalar(p.field(*signing, b"provider"), out);
            out.put(DOT);
            p.holder(p.field(*signing, b"holder"), out);
            out.put(b"\n");
        }
        let last = p.field(accounts, b"last_end");
        if p.kind(last) != Kind::Null {
            out.put(b"last end: source ");
            p.scalar(p.field(last, b"source"), out);
            let end = p.field(last, b"end");
            for key in [
                b"kind".as_slice(),
                b"account",
                b"failed",
                b"refused",
                b"why",
                b"bound",
                b"provider",
                b"scope",
                b"after",
                b"at",
                b"declined",
            ] {
                if let Some(value) = p.document.member(end, key) {
                    out.put(DOT);
                    out.put(key);
                    out.put(b" ");
                    p.label(value, out);
                }
            }
            out.put(b"\n");
        }
        p.readiness(accounts, None, b"readiness: ", out);
        let deployment = p.field(accounts, b"deployment_facts");
        if p.kind(deployment) != Kind::Null {
            out.put(b"deployment:\n");
            p.accounts(p.field(accounts, b"deployment"), p.field(deployment, b"program_hosts"), out);
            p.offers(deployment, out);
            p.readiness(deployment, Some(accounts), b"deployment readiness: ", out);
        }
        if p.flag(accounts, b"readiness_cut") {
            out.put(b"readiness cut: true (jig)\n");
        }
        if p.kind(deployment) != Kind::Null && p.flag(deployment, b"readiness_cut") {
            out.put(b"deployment readiness cut: true (jig)\n");
        }
    }
    let left = p.field(0, b"bytes_left");
    if p.bytes(left) != b"0" {
        out.put(b"bytes_left: ");
        p.scalar(left, out);
        out.put(b"\n");
    }
    if p.flag(0, b"cut") || out.left() > 0 {
        out.put(b"cut: true\n");
    }
    if p.kind(accounts) != Kind::Null {
        for row in &p.items(p.field(accounts, b"party")) {
            p.acts(*row, out);
        }
        for row in &p.items(p.field(accounts, b"deployment")) {
            p.acts(*row, out);
        }
    }
}

struct Projection {
    document: Document,
    viewer: Option<u64>,
}
impl Projection {
    fn field(&self, object: u32, key: &[u8]) -> u32 {
        self.document.member(object, key).expect("encoded field")
    }
    fn kind(&self, at: u32) -> Kind {
        self.document.token(at).expect("encoded value").kind
    }
    fn bytes(&self, at: u32) -> &[u8] {
        self.document.text(self.document.token(at).expect("encoded value")).expect("encoded text")
    }
    fn flag(&self, object: u32, key: &[u8]) -> bool {
        self.kind(self.field(object, key)) == Kind::True
    }
    fn items(&self, array: u32) -> List<u32> {
        let mut rows = List::with_capacity(self.document.len());
        if self.kind(array) != Kind::ArrayStart {
            return rows;
        }
        let mut at = array.checked_add(1).expect("array item");
        for _ in array..self.document.len() {
            if self.kind(at) == Kind::ArrayEnd {
                break;
            }
            rows.push(at).expect("projection row bound");
            at = self.document.value_end(at).expect("encoded value end");
        }
        rows
    }
    fn scalar(&self, at: u32, out: &mut Text) {
        match self.kind(at) {
            Kind::True => out.put(b"true"),
            Kind::False => out.put(b"false"),
            Kind::Null => out.put(b"unknown"),
            Kind::Number => out.put(self.bytes(at)),
            Kind::String => out.words(self.bytes(at)),
            Kind::ObjectStart | Kind::ObjectEnd | Kind::ArrayStart | Kind::ArrayEnd | Kind::Key | Kind::Long => {
                unreachable!("encoded scalar")
            }
        }
    }
    fn label(&self, at: u32, out: &mut Text) {
        if self.kind(at) != Kind::String {
            self.scalar(at, out);
            return;
        }
        let bytes = self.bytes(at);
        let mut from = 0;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'_' {
                out.words(bytes.get(from..index).expect("enum word"));
                out.put(b" ");
                from = index.checked_add(1).expect("enum separator");
            }
        }
        out.words(bytes.get(from..).expect("enum word"));
    }
    fn whole(&self, at: u32, out: &mut Text) {
        if self.kind(at) == Kind::String {
            let mut window = super::accounts::Window { left: 0, cut: false };
            super::offers::whole_text(self.bytes(at), out, &mut window);
        } else {
            self.scalar(at, out);
        }
    }
    fn holder(&self, holder: u32, out: &mut Text) {
        if let Some(party) = self.document.member(holder, b"party") {
            let number = skein_lib::bytes::decimal(self.bytes(party)).expect("party number");
            if self.viewer == Some(number) {
                out.put(b"yours");
            } else {
                out.put(b"party ");
                self.scalar(party, out);
            }
        } else {
            out.put(b"deployment");
        }
    }
    fn accounts(&self, rows: u32, hosts: u32, out: &mut Text) {
        for row in &self.items(rows) {
            out.put(b"account ");
            self.scalar(self.field(*row, b"number"), out);
            out.put(DOT);
            out.put(b"provider ");
            self.scalar(self.field(*row, b"provider"), out);
            for key in [b"provider_name".as_slice(), b"display"] {
                out.put(DOT);
                let text = self.field(*row, key);
                if self.kind(text) == Kind::ObjectStart {
                    self.scalar(self.field(text, b"bytes"), out);
                } else {
                    self.scalar(text, out);
                }
            }
            out.put(DOT);
            self.holder(self.field(*row, b"holder"), out);
            out.put(b"\n");
            for payer in &self.items(self.field(*row, b"uses")) {
                out.put(b"  uses: ");
                self.payer(*payer, out);
                out.put(b"\n");
            }
            let usage = self.field(*row, b"manage_usage");
            if self.kind(usage) != Kind::Null {
                out.put(b"  manage usage: ");
                self.whole(usage, out);
                out.put(b"\n");
            }
            for source in &self.items(self.field(*row, b"sources")) {
                self.source(*source, hosts, out);
            }
            for key in [b"runs".as_slice(), b"waiting"] {
                let activity = self.field(*row, key);
                out.put(b"  ");
                out.put(key);
                out.put(b":");
                if self.kind(activity) == Kind::Null {
                    out.put(b" unavailable");
                } else {
                    for task in &self.items(activity) {
                        out.put(b" ");
                        self.scalar(*task, out);
                    }
                }
                out.put(b"\n");
            }
        }
    }
    fn acts(&self, account: u32, out: &mut Text) {
        let holder = self.field(account, b"holder");
        if self.flag(account, b"cut") {
            return;
        }
        if let Some(party) = self.document.member(holder, b"party")
            && self.viewer != skein_lib::bytes::decimal(self.bytes(party))
        {
            return;
        }
        let mut programs = 0u32;
        for source in &self.items(self.field(account, b"sources")) {
            if self.bytes(self.field(*source, b"kind")) == b"program" {
                programs = programs.checked_add(1).expect("bounded sources");
            }
        }
        if programs == 1 {
            out.put(b"  verify: account verify ");
            self.scalar(self.field(account, b"number"), out);
            out.put(b" --source program\n");
        }
        out.put(b"  forget: account forget ");
        self.scalar(self.field(account, b"number"), out);
        out.put(b"\n");
    }
    fn source(&self, source: u32, hosts: u32, out: &mut Text) {
        out.put(b"  source ");
        self.scalar(self.field(source, b"number"), out);
        out.put(DOT);
        self.label(self.field(source, b"kind"), out);
        let name = self.field(source, b"name");
        if self.kind(name) != Kind::Null {
            out.put(DOT);
            self.whole(name, out);
        }
        out.put(DOT);
        let state = self.field(source, b"state");
        self.label(self.field(state, b"kind"), out);
        out.put(DOT);
        out.put(if self.flag(source, b"usable") { b"usable" } else { b"not usable" });
        for key in [b"until".as_slice(), b"at", b"why"] {
            if let Some(value) = self.document.member(state, key) {
                out.put(DOT);
                out.put(key);
                out.put(b" ");
                self.label(value, out);
            }
        }
        let reset = self.field(source, b"spent_until");
        if self.kind(reset) != Kind::Null {
            out.put(DOT);
            out.put(b"reset ");
            self.scalar(reset, out);
        }
        out.put(b"\n");
        if self.bytes(self.field(source, b"kind")) == b"program" {
            self.hosts(hosts, name, out);
        }
    }
    fn hosts(&self, hosts: u32, name: u32, out: &mut Text) {
        for host in &self.items(hosts) {
            for program in &self.items(self.field(*host, b"programs")) {
                if self.kind(name) == Kind::Null || self.bytes(self.field(*program, b"name")) != self.bytes(name) {
                    continue;
                }
                out.put(b"    host ");
                self.whole(self.field(*host, b"name"), out);
                out.put(b": ");
                let state = self.field(*program, b"state");
                if self.kind(state) == Kind::ObjectStart {
                    if let Some(method) = self.document.member(state, b"signed_in_otherwise") {
                        out.put(b"signed in otherwise (");
                        self.scalar(method, out);
                        out.put(b")");
                    } else {
                        out.put(b"failed ");
                        self.label(self.field(state, b"failed"), out);
                        out.put(DOT);
                        self.scalar(self.field(state, b"detail"), out);
                    }
                } else {
                    self.label(state, out);
                }
                out.put(b"\n");
            }
        }
    }
    fn payer(&self, payer: u32, out: &mut Text) {
        for key in [b"party".as_slice(), b"project"] {
            if let Some(number) = self.document.member(payer, key) {
                out.put(key);
                out.put(b" ");
                self.scalar(number, out);
            }
        }
    }
    fn readiness(&self, object: u32, party: Option<u32>, prefix: &[u8], out: &mut Text) {
        for row in &self.items(self.field(object, b"readiness")) {
            let mut same = false;
            if let Some(party) = party {
                for previous in &self.items(self.field(party, b"readiness")) {
                    same |= self.same_readiness(*row, *previous);
                }
            }
            if same {
                continue;
            }
            out.put(prefix);
            out.put(b"provider ");
            self.scalar(self.field(*row, b"provider"), out);
            out.put(DOT);
            self.payer(self.field(*row, b"payer"), out);
            out.put(DOT);
            self.scalar(self.field(*row, b"readiness"), out);
            let settled = self.field(*row, b"settled");
            if self.kind(settled) != Kind::Null {
                out.put(DOT);
                out.put(b"settled ");
                self.scalar(settled, out);
            }
            out.put(b"\n");
        }
    }
    fn same_readiness(&self, row: u32, previous: u32) -> bool {
        for key in [b"provider".as_slice(), b"readiness", b"settled"] {
            let a = self.field(row, key);
            let b = self.field(previous, key);
            if self.kind(a) != self.kind(b) || self.bytes(a) != self.bytes(b) {
                return false;
            }
        }
        let a = self.field(row, b"payer");
        let b = self.field(previous, b"payer");
        for key in [b"party".as_slice(), b"project"] {
            match (self.document.member(a, key), self.document.member(b, key)) {
                (Some(a), Some(b)) if self.bytes(a) == self.bytes(b) => {}
                (None, None) => {}
                _ => return false,
            }
        }
        true
    }
    fn offers(&self, deployment: u32, out: &mut Text) {
        for offer in &self.items(self.field(deployment, b"providers")) {
            out.put(b"  provider ");
            self.scalar(self.field(*offer, b"provider"), out);
            out.put(DOT);
            self.scalar(self.field(self.field(*offer, b"name"), b"bytes"), out);
            out.put(DOT);
            let mut separator = false;
            for (key, name) in [(b"sign_in".as_slice(), b"sign-in".as_slice()), (b"key", b"key"), (b"none", b"none")] {
                if self.flag(*offer, key) {
                    if separator {
                        out.put(b", ");
                    }
                    out.put(name);
                    separator = true;
                }
            }
            let programs = self.items(self.field(*offer, b"programs"));
            if !programs.is_empty() {
                if separator {
                    out.put(b", ");
                }
                out.put(b"programs");
                for program in &programs {
                    out.put(b" ");
                    self.whole(*program, out);
                }
            }
            let usage = self.field(*offer, b"manage_usage");
            if self.kind(usage) != Kind::Null {
                out.put(DOT);
                out.put(b"manage usage ");
                self.whole(usage, out);
            }
            out.put(b"\n");
        }
    }
}
