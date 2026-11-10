#!/usr/bin/env python3
"""Citation checker: path, section and status-label citations across the monorepo.

Usage, from the repository root:

    python3 -I -B tools/citations/citations.py --base origin/main [--head HEAD]
    python3 -I -B tools/citations/citations.py --all [--head HEAD]

`--base` is the landing gate: it fails when HEAD has a broken citation that BASE lacks
(counted per file and target, renames mapped), so a branch may not add one, directly or by
removing or renumbering what another file cites. `--all` fails on every broken citation.
Both check committed trees (`git archive`), never the working tree, and skip what
`tools/citations/allow.txt` lists. In the trees `tools/citations/sized.txt` lists, a design
document over LIMIT bytes fails too: under `--base`, only one that crossed the limit or grew
while over it. Exit 0 clean, 1 findings, 2 usage or git failure.
"""
import collections
import fnmatch
import os
import re
import subprocess
import sys
import tempfile

TREES = ("temper", "smith", "skein", "jig", "hearth")
CODE_EXT = (".rs", ".py", ".sh", ".toml", ".js")
SLASH_COMMENTS = (".rs", ".js")
ALLOW = "tools/citations/allow.txt"
SIZED = "tools/citations/sized.txt"
LIMIT = 64 * 1024  # a design document's bytes (DESIGN.md, 5)

NUM = r"\d+(?:\.\d+)*"
SEP = r"(?:\s*,\s*(?:and\s+|or\s+)?|\s+and\s+|\s+or\s+|\s+to\s+|\s*[–-]\s*)"
SECTION_AFTER = re.compile(
    r"^[`\]]?(?:\([^)\s]*\))?(?:['’]s(?=\s*,|\s+sections?\s))?"
    r"\s*(?:,\s*|\s+|\(\s*(?=sections?\s))(?:(?:sections?|§)\s+)?(" + NUM + r"(?:" + SEP
    + r"(?:section\s+)?" + NUM + r")*)(?![\w.:/%-]*\d)(?!\.\d)(?![\w%])")
NUM_RE = re.compile(NUM)
MD_TOKEN = re.compile(r"(?<![\w/.<>-])((?:(?:\.\./)+|\./|/)?[\w][\w.+-]*(?:/[\w.+-]+)*\.md)(?![\w/-])")
LINK = re.compile(r"\[([^\]]*)\]\(([^)\s]+)\)")
OWNER = re.compile(r"(?i)\b(temper|smith|skein|jig|hearth)['’]s(?:\s+[\w-]+){0,2}\s*[`\[(]*$")
# A citation of any other repository's document is refused: never resolved, since resolve()
# would find a same-named file here by suffix.
OUTSIDE = re.compile(
    r"(?i)\b(?!(?:the|a|an|this|that|its|their|each|every|no|one|our|whose|which|any|same)\b)"
    r"[\w-]+\s+repository['’]s\s*[`\[(]*$")
BARE_SECTION = re.compile(r"(?<![\w.])(sections?)\s+(" + NUM + r"(?:" + SEP + r"(?:section\s+)?" + NUM
                          + r")*)(?![\w.]*\d)(?!\.\d)")
HEADING = re.compile(r"^#{1,6}\s+(\d+(?:\.\d+)*)\.?(?:\s|$)")
DATED = re.compile(r"\d{4}-\d{2}-\d{2}")
DATE = re.compile(r"^(?:\d{4}-)?\d{2}-\d{2}$")  # "kv.md, 2026-10-05", "reference-case.md, 11-02"


class Tree:
    """One commit's tracked paths, with its extracted Markdown index and headings cached."""

    def __init__(self, root, tracked):
        self.root = root
        self.tracked = tracked
        self.headings = {}
        self.docs = None

    def heading_numbers(self, rel):
        if rel not in self.headings:
            nums = set()
            try:
                with open(os.path.join(self.root, rel), encoding="utf-8", errors="replace") as fh:
                    fence = False
                    for line in fh:
                        if line.lstrip().startswith("```"):
                            fence = not fence
                        elif not fence:
                            m = HEADING.match(line)
                            if m:
                                nums.add(m.group(1).rstrip("."))
            except OSError:
                pass
            self.headings[rel] = nums
        return self.headings[rel]

    def exists(self, rel):
        rel = os.path.normpath(rel)
        if rel.startswith(".."):
            return None
        return rel if rel in self.tracked else None

    def markdown(self):
        if self.docs is None:
            self.docs = sorted((rel for rel, _ in walk(self.root, every_md=True)), key=document_order)
        return self.docs


def document_order(path):
    """Git archives' file-first directory walk, independent of filesystem insertion order."""
    parts = path.split("/")
    return tuple((1, part) for part in parts[:-1]) + ((0, parts[-1]),)


def tree_of(rel):
    first = rel.split("/", 1)[0]
    return first if first in TREES else None


def walk(root, every_md=False):
    """(relative path, is code) for citation sources, or every Markdown target when every_md."""
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in (".git", "target", "node_modules", ".claude")]
        rel_dir = os.path.relpath(dirpath, root)
        rel_dir = "" if rel_dir == "." else rel_dir
        for name in filenames:
            rel = os.path.join(rel_dir, name)
            parts = rel.split("/")
            if not every_md and parts[0] in TREES and parts[1:3] == ["docs", "plan"]:
                continue
            if name.endswith(".md"):
                if every_md:
                    yield rel, False
                elif rel == "DESIGN.md" or parts[0] == "tools":
                    yield rel, False
                elif name in ("AGENTS.md", "CLAUDE.md") and "fixtures" not in parts:
                    yield rel, False
                elif len(parts) >= 3 and parts[0] in TREES and parts[1] == "docs":
                    if not (parts[2] == "development" and DATED.search(name)):
                        yield rel, False
            elif name.endswith(CODE_EXT) and not every_md:
                yield rel, True


def resolve(tree, src, token, owner, is_code):
    """(resolved relative path or None, how it resolved)."""
    tok = re.split(r"[?#]", token, maxsplit=1)[0]
    src_dir = os.path.dirname(src)
    home = tree_of(src)
    if tok.startswith("/"):
        return tree.exists(tok.lstrip("/")), "root"
    first = tok.split("/", 1)[0]
    if first in TREES or first in ("tools", "tests", ".config"):
        return tree.exists(tok) or tree.exists(os.path.join(src_dir, tok)), "root"
    if owner and owner != home:
        design = os.path.join(owner, "docs", "design")
        if tok.startswith(".."):
            sub = ""
            if home and src.startswith(f"{home}/docs/design/"):
                sub = os.path.dirname(src[len(f"{home}/docs/design/"):])
            cands = [os.path.join(design, sub, tok)]
        else:
            cands = [os.path.join(design, tok.lstrip("./"))]
        cands += [os.path.join(owner, "docs", tok), os.path.join(owner, tok)]
        for cand in cands:
            found = tree.exists(cand)
            if found:
                return found, "cross"
        return suffix(tree, tok, owner, "cross")
    if is_code:
        d = src_dir
        while True:
            found = tree.exists(os.path.join(d, tok))
            if found:
                return found, "code-anc"
            if not d or d == home or "/" not in d:
                break
            d = os.path.dirname(d)
        for base in ([f"{home}/docs/design", f"{home}/docs", home] if home else []) + [""]:
            found = tree.exists(os.path.join(base, tok))
            if found:
                return found, "code-base"
        return suffix(tree, tok, home, "code")
    found = tree.exists(os.path.join(src_dir, tok))
    if found:
        return found, "rel"
    if tok == "AGENTS.md":
        return "AGENTS.md", "root-agents"
    if src.startswith("temper/docs/design/draft/") and "/" not in tok:
        # the reference drafts' convention: unqualified names are temper's ../domain/ documents
        found = tree.exists(os.path.join("temper/docs/design/domain", tok))
        if found:
            return found, "convention"
    for base in ([f"{home}/docs/design", f"{home}/docs", home] if home else []) + [""]:
        found = tree.exists(os.path.join(base, tok))
        if found:
            return found, "loose"
    return suffix(tree, tok, home, "rel")


def suffix(tree, tok, home, how):
    """Match by path suffix among docs/ files: same tree first, then skein's foundation, then any."""
    candidates, rule = suffix_candidates(tree, tok, home)
    return (candidates[0], rule) if candidates else (None, how + "-dead")


def suffix_candidates(tree, tok, home):
    """All candidates at the suffix reader's winning precedence, in its selection order."""
    t = tok
    while t.startswith(("../", "./")):
        t = t[3:] if t.startswith("../") else t[2:]
    hits = [d for d in tree.markdown() if d == t or d.endswith("/" + t)]
    docs = [d for d in hits if "/docs/" in d or d.startswith("docs/")]
    same = [d for d in docs if home and d.startswith(home + "/")]
    if same:
        return same, "suffix-same" if len(same) == 1 else "suffix-ambiguous"
    for group in ([d for d in docs if d.startswith("skein/docs/foundation/")], docs, hits):
        if group:
            return group, "suffix"
    return [], None


def doc_of_status(rel):
    return rel[: -len(".status.md")] + ".md" if rel.endswith(".status.md") else rel


def parse_nums(text):
    return [n.rstrip(".") for n in NUM_RE.findall(text)]


def comment_of(line, rel):
    """The comment text of a code line (whole-line or trailing), or None."""
    marker = "//" if rel.endswith(SLASH_COMMENTS) else "#"
    quotes = ('"',) if marker == "//" else ('"', "'")
    at = line.find(marker)
    while at >= 0:
        before = line[:at]
        url = marker == "//" and before.endswith(":")
        if not url and all(before.count(quote) % 2 == 0 for quote in quotes):
            body = line[at + len(marker):]
            return body.lstrip("/!") if marker == "//" else body
        at = line.find(marker, at + 1)
    return None


def logical_lines(lines, rel, is_code):
    """(text, [(offset, line, column)]) for wrapped prose and code comment runs.

    Lines and columns are one-based, including indentation and comment markers.
    """
    buf, marks, fence = [], [], False

    def flush():
        if not buf:
            return None
        text, at = "", []
        for part, (ln, col) in zip(buf, marks):
            at.append((len(text), ln, col))
            text += part + " "
        return text, at

    for ln, line in enumerate(lines, 1):
        if is_code:
            body = comment_of(line, rel)
            whole = body is not None and line.lstrip().startswith("//" if rel.endswith(SLASH_COMMENTS) else "#")
            if body is not None and body.strip() and whole:
                buf.append(body.strip())
                marks.append((ln, (len(line) - len(body)) + len(body) - len(body.lstrip()) + 1))
                continue
            out = flush()
            if out:
                yield out
            buf, marks = [], []
            if body is not None and ".md" in body:
                yield body, [(0, ln, (len(line) - len(body)) + 1)]
            continue
        if line.lstrip().startswith("```"):
            fence = not fence
            out = flush()
            if out:
                yield out
            buf, marks = [], []
            continue
        if fence:
            continue
        if not line.strip() or re.match(r"^\s*(?:[-*+]|\d+\.)\s|^#|^\|", line):
            out = flush()
            if out:
                yield out
            buf, marks = [], []
            if not line.strip():
                continue
        buf.append(line.strip())
        marks.append((ln, len(line) - len(line.lstrip()) + 1))
        if line.startswith(("|", "#")):
            out = flush()
            if out:
                yield out
            buf, marks = [], []
    out = flush()
    if out:
        yield out


def line_at(marks, offset):
    ln = marks[0][1]
    for at, number, _ in marks:
        if at <= offset:
            ln = number
    return ln


def check(root, tracked):
    """Every broken citation under root: (kind, file, line, target, detail, context)."""
    tree = Tree(root, tracked)
    out = []
    for rel, is_code in walk(root):
        try:
            with open(os.path.join(root, rel), encoding="utf-8", errors="replace") as fh:
                text_all = fh.read()
        except OSError:
            continue
        if is_code and ".md" not in text_all:
            continue
        out.extend(check_file(tree, rel, text_all.split("\n"), is_code))
    return out


# Positions are one-based and spans are half-open. Numbers retain all documents that
# satisfy the checker's reading, so a consumer can distinguish an ambiguous reading.
Position = collections.namedtuple("Position", "file line column")
Number = collections.namedtuple("Number", "value start end targets")
Citation = collections.namedtuple(
    "Citation", "kind file line context token start end target how owner numbers targets link readings")
Reading = collections.namedtuple("Reading", "target how")
Link = collections.namedtuple("Link", "text start end tail tail_start tail_end")


def position_at(rel, marks, offset):
    at, ln, col = marks[0]
    for mark in marks:
        if mark[0] <= offset:
            at, ln, col = mark
    return Position(rel, ln, col + offset - at)


def external_section(pre):
    """A bare section attributed to a possessive, RFC or immediately preceding path."""
    return bool(re.search(r"(\bits|\btheir|['’]s|\bthat|\bthis|\bRFC \d+,?)$", pre, re.IGNORECASE)
                or re.search(r"\bRFC \d+['’]s\s+(?:`[^`]+`\s*|(?:[\w-]+\s+){0,2})\($", pre, re.IGNORECASE)
                or re.search(r"\.md`?\)?,?\s*$", pre))


def citations(tree, rel, lines, is_code, bare_pattern=None, section_pattern=None):
    """Yield every reading used by check_file, with physical spans and resolutions.

    A path reading includes its following section numbers. Bare readings and status
    labels have no path token; their numbers carry the candidates that hold them.
    Readings distinguish self, sentence, list-item and same-name candidates.
    Link metadata covers its label and its query/anchor suffix separately.
    Optional section patterns let consumers broaden readings; check_file uses the defaults.
    """
    is_status = rel.endswith(".status.md")
    self_doc = doc_of_status(rel)
    by_step, seen_label = False, False
    for text, marks in logical_lines(lines, rel, is_code):
        def pos(offset):
            return position_at(rel, marks, offset)

        def numbers(match, group, offset, targets):
            return tuple(Number(m.group(), pos(offset + match.start(group) + m.start()),
                                pos(offset + match.start(group) + m.end()),
                                tuple(t for t in targets if m.group() in tree.heading_numbers(t)))
                         for m in NUM_RE.finditer(match.group(group)))

        if is_status and not seen_label:
            first_label = min(status_label_matches(rel, text), key=lambda m: m.start(), default=None)
            prefix = text[:first_label.start()] if first_label else text
            by_step = by_step or "By step" in prefix
            seen_label = first_label is not None
        spans, masked = [], text
        if not is_code:
            for m in LINK.finditer(text):
                target = m.group(2)
                if re.match(r"^[a-z]+:", target):
                    continue
                if target.startswith("#"):
                    # Local anchors do not take part in the gate's path/section checks.
                    # Expose their coordinates to consumers that move the heading.
                    link = Link(m.group(1), pos(m.start(1)), pos(m.end(1)), target,
                                pos(m.start(2)), pos(m.end(2)))
                    yield Citation("anchor", rel, line_at(marks, m.start()), m.group(0), "",
                                   pos(m.start(2)), pos(m.start(2)), rel, "local-anchor", None,
                                   (), (), link, (Reading(rel, "local-anchor"),))
                    continue
                tok = re.split(r"[?#]", target, maxsplit=1)[0]
                if tok:
                    link = Link(m.group(1), pos(m.start(1)), pos(m.end(1)), target[len(tok):],
                                pos(m.start(2) + len(tok)), pos(m.end(2)))
                    spans.append((m.start(), m.end(), tok, text[m.end():], m.start(2), link))
                masked = masked[: m.start()] + " " * (m.end() - m.start()) + masked[m.end():]
        for m in MD_TOKEN.finditer(masked):
            if "://" not in text[max(0, m.start() - 10): m.start() + 1]:
                spans.append((m.start(), m.end(), m.group(1), text[m.end():], m.start(1), None))
        spans.sort()
        covered, resolved, explicit_owners = [], [], []
        item_target = None
        for start, end, tok, after, token_start, link in spans:
            if "<" in tok or "*" in tok or tok.startswith("http") or (is_code and not tok.endswith(".md")):
                continue
            before = text[:start]
            ln = line_at(marks, start)
            ctx = text[max(0, start - 60): end + 60].strip()
            owner, nums, targets = None, (), ()
            if OUTSIDE.search(before[-50:]):
                res, how = None, "outside"
            else:
                om = OWNER.search(before[-40:])
                owner = om.group(1).lower() if om else None
                if owner:
                    explicit_owners.append((start, owner))
                elif re.search(r"\bits\s*[`\[]*$", before, re.IGNORECASE):
                    sentence = before.rfind(". ") + 2
                    owners = [o for st, o in explicit_owners if st >= sentence]
                    owner = owners[-1] if owners else None
                res, how = resolve(tree, rel, tok, owner, is_code)
            if res is not None:
                resolved.append((start, res))
                if re.match(r"^(?:[-*+]|\d+\.)\s+\*\*`?$", before) and re.match(r"^`?:\*\*", after):
                    item_target = res
                sm = (section_pattern or SECTION_AFTER).match(after) if not re.match(r":\d", after) else None
                if sm and res.endswith(".md") and not DATE.match(sm.group(1)):
                    count = (NUM_RE.fullmatch(sm.group(1)) and re.match(r"\s+places\b", after[sm.end():])
                             and not re.search(r"(?:sections?|§)\s", sm.group(0)))
                    if not count:
                        covered.append((end, end + sm.end()))
                        targets = [doc_of_status(res)]
                        if how in ("loose", "suffix-ambiguous", "code-base", "code-anc", "suffix-same") \
                                and "/" not in tok.lstrip("./"):
                            base = os.path.basename(tok)
                            home = owner or tree_of(rel)
                            prefix = f"{home}/docs/" if home else "docs/"
                            targets += [doc for doc in tree.markdown() if os.path.basename(doc) == base
                                        and doc.startswith(prefix) and doc not in targets]
                        nums = numbers(sm, 1, end, targets)
            readings = tuple(Reading(t, how if i == 0 else "same-name") for i, t in enumerate(targets))
            if not nums and res:
                paths = suffix_candidates(tree, tok, owner or tree_of(rel))[0] if how.startswith("suffix") else [res]
                readings = tuple(Reading(t, how) for t in paths)
            yield Citation("path", rel, ln, ctx, tok, pos(token_start), pos(token_start + len(tok)),
                           res, how, owner, nums, tuple(targets), link, readings)
        if is_code:
            continue
        for m in (bare_pattern or BARE_SECTION).finditer(text):
            if any(a <= m.start() < b for a, b in covered):
                continue
            pre = text[: m.start()].rstrip()
            if external_section(pre):
                continue
            sentence = max(pre.rfind(". "), pre.rfind("; "), 0)
            antecedent = [r for (st, r) in resolved if sentence <= st < m.start()]
            targets = [self_doc] + antecedent[-1:]
            readings = [Reading(self_doc, "self")] + [Reading(t, "sentence") for t in antecedent[-1:]]
            if item_target and item_target not in targets:
                targets.append(item_target)
                readings.append(Reading(item_target, "list-item"))
            yield Citation("section-self", rel, line_at(marks, m.start()),
                           text[max(0, m.start() - 80): m.end() + 40].strip(), None,
                           pos(m.start()), pos(m.end()), None, "bare", None,
                           numbers(m, 2, 0, targets), tuple(targets), None, tuple(readings))
        if is_status and not by_step:
            for m in status_label_matches(rel, text):
                yield Citation("status-label", rel, line_at(marks, m.start()),
                               text[max(0, m.start() - 20): m.end() + 80].strip(), None,
                               pos(m.start()), pos(m.end()), self_doc, "status", None,
                               numbers(m, 1, 0, [self_doc]), (self_doc,), None,
                               (Reading(self_doc, "status"),))


def check_file(tree, rel, lines, is_code):
    out = []
    for citation in citations(tree, rel, lines, is_code):
        c = citation
        if c.kind == "anchor":
            continue
        if c.kind == "path" and c.target is None:
            out.append(("path", rel, c.line, c.token, c.how, c.context))
        have = set().union(*(tree.heading_numbers(t) for t in c.targets))
        for n in c.numbers:
            if n.targets:
                continue
            if c.kind == "path":
                kind, target = "section", f"{c.target} {n.value}"
                detail = "missing" if have else "no-numbered-headings"
            elif c.kind == "section-self":
                kind, target = c.kind, f"{'|'.join(c.targets)} {n.value}"
                detail = "missing" if tree.heading_numbers(doc_of_status(rel)) else "no-numbered-headings"
            else:
                kind, target, detail = c.kind, f"{c.target} {n.value}", "missing"
            out.append((kind, rel, c.line, target, detail, c.context))
    return out


def status_label_matches(rel, text):
    """Numbered labels in a status file, including labels whose sections exist."""
    tail = r"(?:,\s*area\s+\d+)?\s*:"
    patterns = [r"\*\*(?:Not yet|Departs|Design lags|Design departs|Open):\*\*\s*(" + NUM + r"(?:" + SEP + NUM
                + r")*)" + tail + r"(?!\d)",
                r"^(?:-\s+)?(?:\*\*)?(" + NUM + r"(?:" + SEP + NUM + r")*)" + tail + r"?(?:\*\*)?\s*:?(?=\s)"]
    if "/draft/" not in rel:
        patterns.append(r"(?<=[.]\s)(" + NUM + r"(?:" + SEP + NUM + r")*)" + tail + r"(?!\d)")
    for pattern in patterns:
        for m in re.finditer(pattern, text):
            if pattern.startswith("^") and not re.match(r"^(?:-\s+)?(?:\*\*)?" + NUM + r"[^:]*:", text):
                continue
            yield m


def status_labels(tree, rel, self_doc, text, marks):
    """A status file's "N.N:" labels after Not yet, Departs and the like, against its document."""
    have = tree.heading_numbers(self_doc)
    out = []
    for m in status_label_matches(rel, text):
        for n in parse_nums(m.group(1)):
            if n not in have:
                out.append(("status-label", rel, line_at(marks, m.start()), f"{self_doc} {n}", "missing",
                            text[max(0, m.start() - 20): m.end() + 80].strip()))
    return out


def allowed(root):
    """allow.txt's (file glob, target) pairs; a target of * allows every target. Each line: FILE TARGET # reason."""
    pairs = set()
    try:
        with open(os.path.join(root, ALLOW), encoding="utf-8") as fh:
            for number, line in enumerate(fh, 1):
                if not line.strip() or line.lstrip().startswith("#"):
                    continue
                entry, hash_, reason = line.partition(" # ")
                parts = entry.split(None, 1)
                if not hash_ or not reason.strip() or len(parts) != 2:
                    raise ValueError(f"{ALLOW}:{number}: want `FILE TARGET # reason`")
                pairs.add((parts[0], parts[1].strip()))
    except FileNotFoundError:
        pass
    return pairs


def sized_trees(root):
    """The trees whose design documents LIMIT covers: sized.txt's lines, `#` starting a comment."""
    try:
        with open(os.path.join(root, SIZED), encoding="utf-8") as fh:
            names = [line.split("#", 1)[0].strip() for line in fh]
    except FileNotFoundError:
        return set()
    names = [name for name in names if name]
    unknown = [name for name in names if name not in TREES]
    if unknown:
        raise ValueError(f"{SIZED}: not a tree: {', '.join(unknown)}")
    return set(names)


def sizes(root, tracked, trees):
    """Bytes of each Markdown file under the given trees' docs/design/."""
    return {rel: os.path.getsize(os.path.join(root, rel)) for rel in tracked
            if rel.endswith(".md") and tree_of(rel) in trees and rel.split("/")[1:3] == ["docs", "design"]}


def oversized(head, base=None):
    """(file, bytes) over LIMIT; against a base, only those that crossed it or grew while over it."""
    return sorted((rel, size) for rel, size in head.items()
                  if size > LIMIT and (base is None or size > base.get(rel, 0)))


def show_sizes(big):
    for rel, size in big:
        print(f"{rel}: size {size:,} bytes, over the {LIMIT:,}-byte limit: split it along its concepts")


def git(*args):
    return subprocess.run(("git", *args), check=True, capture_output=True, text=True).stdout


def extract(rev, into):
    """Extract the checked contents of a commit and return all its tracked paths."""
    names = git("ls-tree", "-r", "--name-only", rev).splitlines()
    # git archive refuses a pathspec that matches nothing
    present = [f":(glob)**/*{ext}" for ext in (".md",) + CODE_EXT if any(n.endswith(ext) for n in names)]
    present += [name for name in (ALLOW, SIZED) if name in names]
    archive = subprocess.run(("git", "archive", "--format=tar", rev, "--", *present),
                             check=True, capture_output=True).stdout
    subprocess.run(("tar", "-x", "-C", into), input=archive, check=True)
    return set(names)


def renames(base, head):
    """Old path → new path for files renamed between base and head."""
    moved = {}
    for line in git("diff", "-M", "--name-status", base, head).splitlines():
        parts = line.split("\t")
        if parts[0].startswith("R") and len(parts) == 3:
            moved[parts[1]] = parts[2]
    return moved


def keys(findings, allow, moved=None):
    """Findings counted per (kind, file, target), the allowed left out, paths mapped through moved."""
    counted = collections.Counter()
    for kind, rel, _, target, _, _ in findings:
        if moved:
            rel = moved.get(rel, rel)
            target = " ".join(moved.get(word, word) for word in target.split(" "))
        if any(fnmatch.fnmatchcase(rel, glob) and want in ("*", target) for glob, want in allow):
            continue
        counted[(kind, rel, target)] += 1
    return counted


def show(findings, wanted):
    left = collections.Counter(wanted)
    for kind, rel, ln, target, detail, ctx in sorted(findings, key=lambda f: (f[1], f[2])):
        key = (kind, rel, target)
        if left[key] > 0:
            left[key] -= 1
            print(f"{rel}:{ln}: {kind} {target} ({detail}): {ctx}")


def main(argv):
    args = {"--head": "HEAD"}
    it = iter(argv)
    for arg in it:
        if arg == "--all":
            args[arg] = True
        elif arg in ("--base", "--head"):
            args[arg] = next(it, None)
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if ("--base" in args) == ("--all" in args) or None in args.values():
        print(__doc__, file=sys.stderr)
        return 2
    try:
        with tempfile.TemporaryDirectory(prefix="citations-") as tmp:
            head_root = os.path.join(tmp, "head")
            os.mkdir(head_root)
            head_tracked = extract(args["--head"], head_root)
            allow = allowed(head_root)
            head = check(head_root, head_tracked)
            head_keys = keys(head, allow)
            trees = sized_trees(head_root)
            head_sizes = sizes(head_root, head_tracked, trees)
            if "--all" in args:
                show(head, head_keys)
                big = oversized(head_sizes)
                show_sizes(big)
                total = sum(head_keys.values())
                print(f"citations: {total} broken in {args['--head']}", file=sys.stderr)
                if big:
                    print(f"citations: {len(big)} design documents over the size limit", file=sys.stderr)
                return 1 if total or big else 0
            base_root = os.path.join(tmp, "base")
            os.mkdir(base_root)
            base_tracked = extract(args["--base"], base_root)
            moved = renames(args["--base"], args["--head"])
            base_keys = keys(check(base_root, base_tracked), allow, moved)
            base_sizes = {moved.get(rel, rel): size for rel, size in sizes(base_root, base_tracked, trees).items()}
    except (subprocess.CalledProcessError, ValueError) as error:
        detail = getattr(error, "stderr", None) or error
        print(f"citations: {detail}", file=sys.stderr)
        return 2
    new = head_keys - base_keys
    show(head, new)
    big = oversized(head_sizes, base_sizes)
    show_sizes(big)
    total = sum(new.values())
    print(f"citations: {total} new broken since {args['--base']} ({sum(head_keys.values())} in all)",
          file=sys.stderr)
    if total:
        print(f"Fix each citation, or list a non-citation in {ALLOW} (FILE TARGET # reason).", file=sys.stderr)
    if big:
        print(f"citations: {len(big)} design documents crossed or grew past the size limit; move their "
              "concepts into documents of their own with tools/citations/move.py.", file=sys.stderr)
    return 1 if total or big else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
