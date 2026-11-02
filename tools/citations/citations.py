#!/usr/bin/env python3
"""Citation checker: path, section and status-label citations across the monorepo.

Usage, from the repository root:

    python3 -I -B tools/citations/citations.py --base origin/main [--head HEAD]
    python3 -I -B tools/citations/citations.py --all [--head HEAD]

`--base` is the landing gate: it fails when HEAD has a broken citation that BASE lacks
(counted per file and target, renames mapped), so a branch may not add one, directly or by
removing or renumbering what another file cites. `--all` fails on every broken citation.
Both check committed trees (`git archive`), never the working tree, and skip what
`tools/citations/allow.txt` lists. Exit 0 clean, 1 findings, 2 usage or git failure.
"""
import collections
import fnmatch
import os
import re
import subprocess
import sys
import tempfile

TREES = ("temper", "smith", "skein", "jig")
CODE_EXT = (".rs", ".py", ".sh", ".toml", ".js")
SLASH_COMMENTS = (".rs", ".js")
ALLOW = "tools/citations/allow.txt"

NUM = r"\d+(?:\.\d+)*"
SEP = r"(?:\s*,\s*(?:and\s+|or\s+)?|\s+and\s+|\s+or\s+|\s+to\s+|\s*[–-]\s*)"
SECTION_AFTER = re.compile(
    r"^[`\]]?(?:\([^)\s]*\))?(?:['’]s(?=\s*,|\s+sections?\s))?"
    r"\s*(?:,\s*|\s+|\(\s*(?=sections?\s))(?:(?:sections?|§)\s+)?(" + NUM + r"(?:" + SEP
    + r"(?:section\s+)?" + NUM + r")*)(?![\w.:/%-]*\d)(?!\.\d)(?![\w%])")
NUM_RE = re.compile(NUM)
MD_TOKEN = re.compile(r"(?<![\w/.<>-])((?:(?:\.\./)+|\./|/)?[\w][\w.+-]*(?:/[\w.+-]+)*\.md)(?![\w/-])")
LINK = re.compile(r"\[([^\]]*)\]\(([^)\s]+)\)")
OWNER = re.compile(r"(?i)\b(temper|smith|skein|jig)['’]s(?:\s+[\w-]+){0,2}\s*[`\[]*$")
# Another repository's documents (hearth's, the agents repository's) are not checked.
EXTERNAL = re.compile(
    r"(?i)(?:\bhearth['’]s|\bagents['’](?:s)?|\bagents repository['’]s)(?:,?\s+[\w-]+){0,2}\s*[`\[(]*$")
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
            self.docs = [rel for rel, _ in walk(self.root, every_md=True)]
        return self.docs


def tree_of(rel):
    first = rel.split("/", 1)[0]
    return first if first in TREES else None


def walk(root, every_md=False):
    """(relative path, is code) for the files read: docs and AGENTS.md/CLAUDE.md, code naming .md."""
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in (".git", "target", "node_modules", ".claude")]
        rel_dir = os.path.relpath(dirpath, root)
        rel_dir = "" if rel_dir == "." else rel_dir
        for name in filenames:
            rel = os.path.join(rel_dir, name)
            parts = rel.split("/")
            if name.endswith(".md"):
                if every_md:
                    yield rel, False
                elif name in ("AGENTS.md", "CLAUDE.md") and "fixtures" not in parts:
                    yield rel, False
                elif len(parts) >= 3 and parts[0] in TREES and parts[1] == "docs" and parts[2] != "plan":
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
    t = tok
    while t.startswith(("../", "./")):
        t = t[3:] if t.startswith("../") else t[2:]
    hits = [d for d in tree.markdown() if d == t or d.endswith("/" + t)]
    docs = [d for d in hits if "/docs/" in d or d.startswith("docs/")]
    same = [d for d in docs if home and d.startswith(home + "/")]
    if same:
        return same[0], "suffix-same" if len(same) == 1 else "suffix-ambiguous"
    for group in ([d for d in docs if d.startswith("skein/docs/foundation/")], docs, hits):
        if group:
            return group[0], "suffix"
    return None, how + "-dead"


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
    """(text, [(offset, line number)]) joining wrapped prose: Markdown paragraphs, code comment runs."""
    buf, marks, fence = [], [], False

    def flush():
        if not buf:
            return None
        text, at = "", []
        for part, ln in zip(buf, marks):
            at.append((len(text), ln))
            text += part + " "
        return text, at

    for ln, line in enumerate(lines, 1):
        if is_code:
            body = comment_of(line, rel)
            whole = body is not None and line.lstrip().startswith("//" if rel.endswith(SLASH_COMMENTS) else "#")
            if body is not None and body.strip() and whole:
                buf.append(body.strip())
                marks.append(ln)
                continue
            out = flush()
            if out:
                yield out
            buf, marks = [], []
            if body is not None and ".md" in body:
                yield body, [(0, ln)]
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
        marks.append(ln)
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
    for at, number in marks:
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


def check_file(tree, rel, lines, is_code):
    out = []
    is_status = rel.endswith(".status.md")
    self_doc = doc_of_status(rel)
    by_step, seen_label = False, False
    for text, marks in logical_lines(lines, rel, is_code):
        if is_status and not seen_label:
            first_label = min(status_label_matches(rel, text), key=lambda m: m.start(), default=None)
            prefix = text[:first_label.start()] if first_label else text
            by_step = by_step or "By step" in prefix
            seen_label = first_label is not None
        spans, masked = [], text
        if not is_code:
            for m in LINK.finditer(text):
                target = m.group(2)
                if re.match(r"^[a-z]+:", target) or target.startswith("#"):
                    continue
                tok = re.split(r"[?#]", target, maxsplit=1)[0]
                if tok:
                    spans.append((m.start(), m.end(), tok, text[m.end():]))
                masked = masked[: m.start()] + " " * (m.end() - m.start()) + masked[m.end():]
        for m in MD_TOKEN.finditer(masked):
            if "://" not in text[max(0, m.start() - 10): m.start() + 1]:
                spans.append((m.start(), m.end(), m.group(1), text[m.end():]))
        spans.sort()
        covered, resolved, explicit_owners = [], [], []
        item_target = None
        for start, end, tok, after in spans:
            if "<" in tok or "*" in tok or tok.startswith("http") or (is_code and not tok.endswith(".md")):
                continue
            before = text[:start]
            ln = line_at(marks, start)
            ctx = text[max(0, start - 60): end + 60].strip()
            if EXTERNAL.search(before[-50:]):
                continue
            om = OWNER.search(before[-40:])
            owner = om.group(1).lower() if om else None
            if owner:
                explicit_owners.append((start, owner))
            elif re.search(r"\bits\s*[`\[]*$", before, re.IGNORECASE):
                sentence = before.rfind(". ") + 2
                owners = [o for st, o in explicit_owners if st >= sentence]
                owner = owners[-1] if owners else None
            res, how = resolve(tree, rel, tok, owner, is_code)
            if res is None:
                out.append(("path", rel, ln, tok, how, ctx))
                continue
            resolved.append((start, res))
            if re.match(r"^(?:[-*+]|\d+\.)\s+\*\*`?$", before) and re.match(r"^`?:\*\*", after):
                item_target = res
            if re.match(r":\d", after):
                continue  # a line number
            sm = SECTION_AFTER.match(after)
            if not (sm and res.endswith(".md")) or DATE.match(sm.group(1)):
                continue
            if NUM_RE.fullmatch(sm.group(1)) and re.match(r"\s+places\b", after[sm.end():]) \
                    and not re.search(r"(?:sections?|§)\s", sm.group(0)):
                continue
            covered.append((end, end + sm.end()))
            have = set(tree.heading_numbers(doc_of_status(res)))
            if how in ("loose", "suffix-ambiguous", "code-base", "code-anc", "suffix-same") \
                    and "/" not in tok.lstrip("./"):
                # an ambiguous bare name: a section of any same-named document of the tree counts
                base = os.path.basename(tok)
                home = owner or tree_of(rel)
                prefix = f"{home}/docs/" if home else "docs/"
                for doc in tree.markdown():
                    if os.path.basename(doc) == base and doc.startswith(prefix):
                        have |= tree.heading_numbers(doc)
            for n in parse_nums(sm.group(1)):
                if n not in have:
                    detail = "missing" if have else "no-numbered-headings"
                    out.append(("section", rel, ln, f"{res} {n}", detail, ctx))
        if is_code:
            continue
        for m in BARE_SECTION.finditer(text):
            if any(a <= m.start() < b for a, b in covered):
                continue
            pre = text[: m.start()].rstrip()
            if re.search(r"(\bits|\btheir|['’]s|\bthat|\bthis|\bRFC \d+,?)$", pre, re.IGNORECASE) \
                    or re.search(r"\bRFC \d+['’]s\s+(?:`[^`]+`\s*|(?:[\w-]+\s+){0,2})\($",
                                 pre, re.IGNORECASE) \
                    or re.search(r"\.md`?\)?,?\s*$", pre):
                continue
            sentence = max(pre.rfind(". "), pre.rfind("; "), 0)
            antecedent = [r for (st, r) in resolved if sentence <= st < m.start()]
            targets = [self_doc] + antecedent[-1:]
            if item_target and item_target not in targets:
                targets.append(item_target)
            for n in parse_nums(m.group(2)):
                if not any(n in tree.heading_numbers(t) for t in targets):
                    detail = "missing" if tree.heading_numbers(self_doc) else "no-numbered-headings"
                    out.append(("section-self", rel, line_at(marks, m.start()), f"{'|'.join(targets)} {n}",
                                detail, text[max(0, m.start() - 80): m.end() + 40].strip()))
        if is_status and not by_step:
            out.extend(status_labels(tree, rel, self_doc, text, marks))
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


def git(*args):
    return subprocess.run(("git", *args), check=True, capture_output=True, text=True).stdout


def extract(rev, into):
    """Extract the checked contents of a commit and return all its tracked paths."""
    names = git("ls-tree", "-r", "--name-only", rev).splitlines()
    # git archive refuses a pathspec that matches nothing
    present = [f":(glob)**/*{ext}" for ext in (".md",) + CODE_EXT if any(n.endswith(ext) for n in names)]
    present += [ALLOW] if ALLOW in names else []
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
            if "--all" in args:
                show(head, head_keys)
                total = sum(head_keys.values())
                print(f"citations: {total} broken in {args['--head']}", file=sys.stderr)
                return 1 if total else 0
            base_root = os.path.join(tmp, "base")
            os.mkdir(base_root)
            base_tracked = extract(args["--base"], base_root)
            base_keys = keys(check(base_root, base_tracked), allow, renames(args["--base"], args["--head"]))
    except (subprocess.CalledProcessError, ValueError) as error:
        detail = getattr(error, "stderr", None) or error
        print(f"citations: {detail}", file=sys.stderr)
        return 2
    new = head_keys - base_keys
    show(head, new)
    total = sum(new.values())
    print(f"citations: {total} new broken since {args['--base']} ({sum(head_keys.values())} in all)",
          file=sys.stderr)
    if total:
        print(f"Fix each citation, or list a non-citation in {ALLOW} (FILE TARGET # reason).", file=sys.stderr)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
