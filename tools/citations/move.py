#!/usr/bin/env python3
"""Move documents and numbered sections, preserving the citation checker's readings.

Usage from the repository root: python3 -I -B tools/citations/move.py MAP [--dry-run]
Exit 0 on success, 2 on a map error or a git failure. Changes are staged, never committed.
"""
import collections
import fnmatch
import importlib.util
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unicodedata

# Isolated Python does not put the script's directory on sys.path.
_spec = importlib.util.spec_from_file_location("citations", Path(__file__).with_name("citations.py"))
c = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(c)

Entry = collections.namedtuple("Entry", "src number dst new_number line directory")
Row = collections.namedtuple("Row", "file line text")
Heading = collections.namedtuple("Heading", "number start end level")


def git(*args):
    return c.git(*args)


def address(text):
    parts = text.split()
    if len(parts) not in (1, 2) or (len(parts) == 2 and not re.fullmatch(c.NUM, parts[1])):
        raise ValueError("want PATH or PATH SECTION")
    path = parts[0]
    if path.startswith("/") or any(p in ("", ".", "..") for p in path.rstrip("/").split("/")):
        raise ValueError("want a repository-relative path without . or ..")
    if not any(path.startswith(f"{tree}/docs/") for tree in c.TREES):
        raise ValueError("path must be under one of the five trees' docs/")
    return path, parts[1] if len(parts) == 2 else None


def read_map(path):
    entries, titles, errors = [], {}, []
    for ln, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        text = raw.split("#", 1)[0].strip()
        if not text:
            continue
        try:
            if text.startswith("title "):
                dest, sep, title = text[6:].partition(":")
                dst, number = address(dest.strip())
                if not sep or not title.strip() or number or not dst.endswith(".md"):
                    raise ValueError("want title PATH.md: TITLE")
                if dst in titles:
                    raise ValueError(f"duplicate title for {dst}")
                titles[dst] = title.strip()
                continue
            parts = text.split("->")
            if len(parts) != 2:
                raise ValueError("want SOURCE -> DESTINATION")
            src, number = address(parts[0].strip())
            dst, new_number = address(parts[1].strip())
            directory = src.endswith("/")
            if bool(number) != bool(new_number) or directory != dst.endswith("/"):
                raise ValueError("source and destination must have the same kind")
            if number and (directory or not src.endswith(".md") or not dst.endswith(".md")):
                raise ValueError("sections need Markdown documents")
            if new_number and len(new_number.split(".")) > 5:
                raise ValueError("destination heading would exceed level 6")
            entries.append(Entry(src, number, dst, new_number, ln, directory))
        except ValueError as error:
            errors.append(f"map:{ln}: {error}")
    if not entries:
        errors.append("map: no moves")
    return entries, titles, errors


def headings(lines):
    """Numbered section intervals; every heading of equal or higher level ends one."""
    found, all_heads, fence = [], [], False
    for i, line in enumerate(lines):
        if line.lstrip().startswith("```"):
            fence = not fence
        elif not fence:
            plain = re.match(r"^(#{1,6})\s+", line)
            if plain:
                level = len(plain.group(1))
                all_heads.append((i, level))
                match = c.HEADING.match(line)
                if match:
                    found.append((match.group(1), i, level))
    return [Heading(n, i, next((j for j, d in all_heads if j > i and d <= level), len(lines)), level)
            for n, i, level in found]


def prefix(number, parent):
    return number == parent or number.startswith(parent + ".")


def number_key(number):
    return tuple(int(n) for n in number.split("."))


def slug(text):
    text = re.sub(r"^#{1,6}\s+|\s+#+\s*$", "", text.strip()).lower()
    text = re.sub(r"<[^>]*>", "", text)
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    return "".join(ch for ch in text if ch in "-_ " or unicodedata.category(ch)[0] in "LNM").replace(" ", "-")


def slugs(rows):
    counts, out, fence = collections.Counter(), {}, False
    for row in rows:
        line = row.text
        if line.lstrip().startswith("```"):
            fence = not fence
        elif not fence and re.match(r"^#{1,6}\s+", line):
            base = slug(line)
            value = base if not counts[base] else f"{base}-{counts[base]}"
            while value in out:
                counts[base] += 1
                value = f"{base}-{counts[base]}"
            counts[base] += 1
            out[value] = (row.file, row.line)
    return out


class Plan:
    def __init__(self, root, tracked, entries, titles, errors):
        self.root, self.tracked, self.entries = root, tracked, entries
        self.titles, self.errors = titles, errors
        self.moves, self.sections, self.texts, self.by_hand = {}, [], {}, set()
        self.edits = collections.defaultdict(dict)
        self.consumed = collections.defaultdict(list)
        self.rewritten, self.rewritten_files = 0, set()
        self.validate()

    def read(self, path):
        if path not in self.texts:
            self.texts[path] = (self.root / path).read_text(encoding="utf-8", errors="replace")
        return self.texts[path]

    def lines(self, path):
        return self.read(path).splitlines(keepends=True)

    def validate(self):
        seen, files, directories = set(), {}, []
        explicit_files = {e.src: e.dst for e in self.entries if not e.number and not e.directory}
        for e in self.entries:
            key = (e.src, e.number)
            if key in seen:
                self.errors.append(f"map:{e.line}: duplicate source {e.src} {e.number or ''}".rstrip())
            seen.add(key)
            if e.directory:
                directories.append(e)
                if not any(p.startswith(e.src) for p in self.tracked):
                    self.errors.append(f"map:{e.line}: source does not exist: {e.src}")
            elif e.src not in self.tracked or not (self.root / e.src).is_file():
                self.errors.append(f"map:{e.line}: source does not exist: {e.src}")
            elif e.number:
                hs = [h for h in headings(self.lines(e.src)) if h.number == e.number]
                if len(hs) != 1:
                    self.errors.append(f"map:{e.line}: source section does not exist uniquely: {e.src} {e.number}")
                else:
                    self.sections.append((e, hs[0]))
            else:
                files[e.src] = e.dst
                if e.src.endswith(".md") and not e.src.endswith((".status.md", ".execution.md")):
                    for suffix in (".status.md", ".execution.md"):
                        companion = e.src[:-3] + suffix
                        if companion in self.tracked:
                            dest = e.dst[:-3] + suffix
                            if companion in explicit_files and explicit_files[companion] != dest:
                                self.errors.append(f"conflicting companion move: {companion}")
                            files[companion] = dest
        for e, _ in self.sections:
            if any(other.src == e.src and not other.number and not other.directory for other in self.entries):
                self.errors.append(f"map:{e.line}: document and section entry for {e.src}")
        for path in sorted(self.tracked):
            choices = [e for e in directories if path.startswith(e.src)]
            dest = files.get(path)
            if dest is None and choices:
                e = max(choices, key=lambda e: len(e.src))
                dest = e.dst + path[len(e.src):]
            if dest is not None and dest != path:
                self.moves[path] = dest
        occupied = {}
        for path in sorted(self.tracked):
            dst = self.moves.get(path, path)
            if dst in occupied:
                self.errors.append(f"two sources onto one destination: {occupied[dst]}, {path} -> {dst}")
            occupied[dst] = path
        for src, dst in self.moves.items():
            if dst not in self.tracked and (self.root / dst).exists():
                self.errors.append(f"destination exists and is not moved away: {dst}")
            self.check_parents(dst)
        self.final_tracked = set(occupied)
        for e, _ in self.sections:
            if e.dst not in self.final_tracked:
                if e.dst not in self.titles:
                    self.errors.append(f"created document needs a title: {e.dst}")
                if (self.root / e.dst).exists() and e.dst not in self.moves:
                    self.errors.append(f"destination exists and is not moved away: {e.dst}")
                self.check_parents(e.dst)
            self.final_tracked.add(e.dst)
        if self.errors:
            return
        self.rows = self.layout()
        for path, rows in self.rows.items():
            nums = [h.number for h in headings([r.text for r in rows])]
            duplicates = [n for n, count in collections.Counter(nums).items() if count > 1]
            for n in duplicates:
                self.errors.append(f"destination section exists or has two sources: {path} {n}")
            for e, _ in self.sections:
                if e.dst == path and "." in e.new_number and e.new_number.rsplit(".", 1)[0] not in nums:
                    self.errors.append(f"destination parent section does not exist: {path} {e.new_number}")

    def check_parents(self, dst):
        for parent in Path(dst).parents:
            if str(parent) == ".":
                break
            if (self.root / parent).is_symlink() or ((self.root / parent).exists() and not (self.root / parent).is_dir()):
                self.errors.append(f"destination parent is not a directory: {parent}")

    def section_at(self, path, line):
        choices = [(e, h) for e, h in self.sections if e.src == path and h.start <= line - 1 < h.end]
        return max(choices, key=lambda pair: len(pair[0].number.split(".")), default=(None, None))[0]

    def text_destination(self, path, line):
        e = self.section_at(path, line)
        return e.dst if e else self.moves.get(path, path)

    def target(self, path, number):
        choices = [e for e, _ in self.sections if e.src == path and prefix(number, e.number)]
        if choices:
            e = max(choices, key=lambda e: len(e.number))
            return e.dst, e.new_number + number[len(e.number):]
        return self.moves.get(path, path), number

    def transformed(self, row):
        if row.file is None:
            return row
        e = self.section_at(row.file, row.line)
        if not e:
            return row
        line = row.text
        h = re.match(r"^(#{1,6})(\s+)", line)
        if not h:
            return row
        # Only headings outside fences are changed (layout supplies their line set).
        if (row.file, row.line) not in self.heading_lines:
            return row
        level = len(h.group(1)) + len(e.new_number.split(".")) - len(e.number.split("."))
        if not 1 <= level <= 6:
            self.errors.append(f"heading level outside 1..6: {row.file}:{row.line}")
            return row
        m = c.HEADING.match(line)
        if m and prefix(m.group(1), e.number):
            n = e.new_number + m.group(1)[len(e.number):]
            tail = line[m.end(1):]
            if tail.startswith("."):
                tail = tail[1:]
            line = "#" * level + h.group(2) + n + ("." if "." not in n else "") + tail
        else:
            line = "#" * level + line[h.end(1):]
        return row._replace(text=line)

    def layout(self):
        paths = set(self.moves) | {e.src for e, _ in self.sections}
        paths |= {p for p in self.tracked if self.moves.get(p, p) in {e.dst for e, _ in self.sections}}
        paths = {p for p in paths if p.endswith(".md")}
        self.heading_lines = set()
        for path in paths:
            fence = False
            for i, line in enumerate(self.lines(path), 1):
                if line.lstrip().startswith("```"):
                    fence = not fence
                elif not fence and re.match(r"^#{1,6}\s+", line):
                    self.heading_lines.add((path, i))
        rows, chunks = {}, collections.defaultdict(list)
        for path in sorted(paths):
            dest = self.moves.get(path, path)
            rows[dest] = []
            for i, text in enumerate(self.lines(path), 1):
                row = Row(path, i, text)
                e = self.section_at(path, i)
                if e:
                    chunks[e].append(self.transformed(row))
                else:
                    rows[dest].append(row)
        for e, _ in sorted(self.sections, key=lambda pair: (pair[0].dst, number_key(pair[0].new_number))):
            if e.dst not in rows:
                rows[e.dst] = [Row(None, 0, f"# {self.titles.get(e.dst, '')}\n"), Row(None, 0, "\n")]
            dest = rows[e.dst]
            hs = headings([r.text for r in dest])
            parent = e.new_number.rsplit(".", 1)[0] if "." in e.new_number else None
            siblings = [h for h in hs if (h.number.rsplit(".", 1)[0] if "." in h.number else None) == parent]
            larger = [h for h in siblings if number_key(h.number) > number_key(e.new_number)]
            smaller = [h for h in siblings if number_key(h.number) < number_key(e.new_number)]
            if larger:
                at = min(h.start for h in larger)
            elif smaller:
                at = max(smaller, key=lambda h: number_key(h.number)).end
            else:
                at = next((h.end for h in hs if h.number == parent), len(dest))
            chunk = list(chunks[e])
            if at and dest[at - 1].text and not dest[at - 1].text.endswith("\n"):
                chunk.insert(0, Row(None, 0, "\n"))
            if at < len(dest) and chunk and not chunk[-1].text.endswith("\n"):
                chunk.append(Row(None, 0, "\n"))
            dest[at:at] = chunk
        return rows

    def styled(self, tree, src, target, token, owner, is_code):
        if token.startswith("/"):
            candidate = "/" + target
        elif token.split("/", 1)[0] in c.TREES + ("tools", "tests", ".config"):
            candidate = target
        elif owner and owner != c.tree_of(src):
            candidate = os.path.relpath(target, f"{owner}/docs/design")
        elif is_code and not token.startswith(("./", "../")):
            home = c.tree_of(src)
            if token.startswith("docs/") and home:
                bases = [home]
            else:
                bases = [f"{home}/docs/design", f"{home}/docs", home] if home else [""]
            base = next((b for b in bases if target.startswith(b + "/")), "")
            candidate = os.path.relpath(target, base or ".")
        else:
            candidate = os.path.relpath(target, os.path.dirname(src) or ".")
            if token.startswith("./") and not candidate.startswith("."):
                candidate = "./" + candidate
        resolved, how = c.resolve(tree, src, candidate, owner, is_code)
        return candidate if resolved == target and how not in ("suffix-ambiguous", "suffix") else target

    def edit(self, start, end, replacement):
        """Record a physical replacement, including a token split across wrapped lines."""
        lines = self.lines(start.file)
        if start.line == end.line:
            self.edits[start.file][(start.line, start.column - 1, end.column - 1)] = replacement
        else:
            self.edits[start.file][(start.line, start.column - 1, len(lines[start.line - 1].rstrip("\r\n")))] = replacement
            for ln in range(start.line + 1, end.line + 1):
                a = 0
                b = end.column - 1 if ln == end.line else len(lines[ln - 1].rstrip("\r\n"))
                self.edits[start.file][(ln, a, b)] = ""

    def manual(self, citation, reason):
        self.by_hand.add(f"{citation.file}:{citation.line}: {reason}: {citation.context}")

    def rewrite(self):
        before = c.Tree(str(self.root), self.tracked)
        after = c.Tree(str(self.root), self.final_tracked)
        after.docs = sorted(p for p in self.final_tracked if p.endswith(".md"))
        anchors = {}
        for path, rows in self.rows.items():
            for name, origin in slugs(rows).items():
                anchors[origin] = (path, name)
        for path, is_code in sorted(c.walk(str(self.root))):
            if path not in self.tracked:
                continue
            text = self.read(path)
            if is_code and ".md" not in text:
                continue
            for cite in c.citations(before, path, text.split("\n"), is_code):
                source = self.text_destination(path, cite.start.line)
                if cite.kind == "path":
                    self.consumed[path].append((cite.start, cite.end))
                destinations, ambiguous = [], False
                for n in cite.numbers:
                    candidates = {self.target(t, n.value) for t in n.targets}
                    if len(candidates) > 1:
                        ambiguous = True
                    destinations.append(next(iter(candidates)) if len(candidates) == 1 else None)
                if ambiguous:
                    affected = source != path or any(self.target(t, n.value) != (t, n.value)
                                                     for n in cite.numbers for t in n.targets)
                    if not affected:
                        continue
                    self.manual(cite, "ambiguous bare section" if cite.kind == "section-self" else "ambiguous section")
                    continue
                files = {d[0] for d in destinations if d is not None}
                if len(files) > 1 and cite.kind != "status-label":
                    self.manual(cite, "split citation")
                    continue
                edits_before = dict(self.edits[path])
                if cite.kind == "status-label":
                    for n, dest in zip(cite.numbers, destinations):
                        if dest is None:
                            dest = self.target(cite.target, n.value)
                        if dest[0] != self.moves.get(cite.target, cite.target):
                            self.manual(cite, f"status label to move to {dest[0]} {dest[1]}")
                        elif dest[1] != n.value:
                            self.edit(n.start, n.end, dest[1])
                elif cite.kind == "section-self":
                    if files and next(iter(files)) != c.doc_of_status(source):
                        target = next(iter(files))
                        token = self.styled(after, source, target, os.path.basename(target), None, False)
                        self.edit(cite.start, cite.start, f"`{token}`, ")
                    for n, dest in zip(cite.numbers, destinations):
                        if dest and dest[1] != n.value:
                            self.edit(n.start, n.end, dest[1])
                elif cite.target:
                    target = next(iter(files)) if files else self.moves.get(cite.target, cite.target)
                    tail = cite.link.tail if cite.link else ""
                    if cite.link and "#" in tail and cite.target.endswith(".md"):
                        query, _, anchor = tail.partition("#")
                        old_rows = [Row(cite.target, i, line) for i, line in enumerate(self.lines(cite.target), 1)]
                        origin = slugs(old_rows).get(anchor)
                        if origin in anchors:
                            anchor_target, name = anchors[origin]
                            if files and anchor_target != target:
                                self.manual(cite, "split citation (anchor and sections)")
                                continue
                            target, tail = anchor_target, query + "#" + name
                    resolved, how = c.resolve(after, source, cite.token, cite.owner, is_code)
                    if cite.kind == "anchor" and target == source:
                        token = ""
                    elif target == cite.target and source == path and resolved == target and how != "suffix-ambiguous":
                        token = cite.token
                    else:
                        token = self.styled(after, source, target, cite.token, cite.owner, is_code)
                    if token != cite.token:
                        self.edit(cite.start, cite.end, token)
                    for n, dest in zip(cite.numbers, destinations):
                        if dest and dest[1] != n.value:
                            self.edit(n.start, n.end, dest[1])
                    if cite.link:
                        if tail != cite.link.tail:
                            self.edit(cite.link.tail_start, cite.link.tail_end, tail)
                        if cite.link.text == cite.token and token != cite.token:
                            self.edit(cite.link.start, cite.link.end, token)
                        elif cite.link.text == cite.token + cite.link.tail and token + tail != cite.link.text:
                            self.edit(cite.link.start, cite.link.end, token + tail)
                if edits_before != self.edits[path]:
                    self.rewritten += 1
                    self.rewritten_files.add(source)
        self.rewrite_allow()
        self.names_by_hand()
        return self.contents()

    def rewrite_allow(self):
        if c.ALLOW not in self.tracked:
            return
        c.allowed(str(self.root))
        lines = self.lines(c.ALLOW)
        for ln, line in enumerate(lines, 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            entry, _, reason = line.partition(" # ")
            glob, target = entry.split(None, 1)
            old = (glob, target)
            if any(ch in glob for ch in "*?["):
                if any(fnmatch.fnmatchcase(p, glob) for p in self.moves):
                    self.by_hand.add(f"{c.ALLOW}:{ln}: glob matches a moved path: {glob}")
            else:
                glob = self.moves.get(glob, glob)
            parts = target.split()
            if len(parts) == 2 and re.fullmatch(c.NUM, parts[1]):
                parts = list(self.target(parts[0], parts[1]))
            else:
                parts = [self.moves.get(p, p) for p in parts]
            target = " ".join(parts)
            if old != (glob, target):
                self.edits[c.ALLOW][(ln, 0, len(line.rstrip("\r\n")))] = f"{glob} {target} # {reason.rstrip()}"
                self.rewritten += 1
                self.rewritten_files.add(c.ALLOW)

    def names_by_hand(self):
        # A line's rewritten spans are excluded; strings beside comments still get reported.
        tokens = {}
        for old in self.moves:
            tokens[old] = old
            home = c.tree_of(old)
            if home:
                tokens[old[len(home) + 1:]] = old
                if old.startswith(f"{home}/docs/design/"):
                    tokens[old[len(f'{home}/docs/design/'):]] = old
        if not tokens:
            return
        pattern = re.compile(r"(?<![\w./-])(?:" + "|".join(re.escape(t) for t in sorted(tokens, key=len, reverse=True)) + r")(?![\w/-])")
        for path in sorted(self.tracked):
            # Credential-shaped files are never opened by this advisory scan.
            if (self.root / path).is_symlink() or re.search(r"(?i)(?:^|/)(?:auth\.json|\.credentials\.json|[^/]*(?:secret|token|credential|private[-_]key)[^/]*)$|\.(?:pem|key|p12|pfx)$", path):
                continue
            raw = (self.root / path).read_bytes()
            if b"\0" in raw:
                continue
            for ln, line in enumerate(raw.decode("utf-8", errors="replace").splitlines(), 1):
                edits = [(a, b) for (number, a, b) in self.edits.get(path, {}) if number == ln]
                for m in pattern.finditer(line):
                    consumed = [(a.column - 1, b.column - 1) for a, b in self.consumed.get(path, [])
                                if a.line == ln == b.line]
                    if path == c.ALLOW or any(a <= m.start() and m.end() <= b for a, b in edits + consumed):
                        continue
                    self.by_hand.add(f"{path}:{ln}: name in code or other text: {m.group()}")

    def patched(self, path):
        lines = self.lines(path)
        per_line = collections.defaultdict(list)
        for (ln, a, b), replacement in self.edits.get(path, {}).items():
            per_line[ln].append((a, b, replacement))
        for ln, edits in per_line.items():
            end = len(lines[ln - 1])
            for a, b, replacement in sorted(edits, reverse=True):
                if b > end:
                    raise ValueError(f"overlapping citation edits: {path}:{ln}")
                lines[ln - 1] = lines[ln - 1][:a] + replacement + lines[ln - 1][b:]
                end = a
        return lines

    def contents(self):
        patched = {p: self.patched(p) for p in set(self.texts) | set(self.edits)}
        out = {}
        for dst, rows in self.rows.items():
            out[dst] = "".join(self.transformed(row._replace(text=patched[row.file][row.line - 1])).text
                               if row.file else row.text for row in rows)
        for path in self.edits:
            dst = self.moves.get(path, path)
            if dst not in out:
                out[dst] = "".join(patched[path])
        # Only write changed content. Non-Markdown moves keep their bytes and mode via git mv.
        return {p: text for p, text in out.items()
                if p not in self.tracked or text != self.read(p)}

    def summary(self):
        for e in self.entries:
            if e.number:
                print(f"Section {e.src} {e.number} -> {e.dst} {e.new_number}")
            else:
                print(f"{'Directory' if e.directory else 'Document'} {e.src} -> {e.dst}")
        print(f"Rewrote {self.rewritten} citations in {len(self.rewritten_files)} files.")
        print("By hand:")
        for item in sorted(self.by_hand):
            print(f"  {item}")
        if not self.by_hand:
            print("  (none)")


def apply(plan, contents):
    """Use git mv even for cycles; restore the clean starting tree if any step fails."""
    root = plan.root
    original = {p: (root / p).read_bytes() for p in contents if p in plan.tracked and p not in plan.moves}
    locations, directories = {}, set()

    def mkdir(path):
        missing = []
        while not path.exists():
            missing.append(path)
            path = path.parent
        for d in reversed(missing):
            d.mkdir()
            directories.add(d)

    with tempfile.TemporaryDirectory(prefix=".citations-move-", dir=root) as tmp:
        try:
            for i, src in enumerate(sorted(plan.moves)):
                staging = str(Path(tmp).relative_to(root) / str(i))
                git("mv", "--", src, staging)
                locations[src] = staging
            for src, dst in sorted(plan.moves.items()):
                mkdir((root / dst).parent)
                git("mv", "--", locations[src], dst)
                locations[src] = dst
            for path, text in sorted(contents.items()):
                mkdir((root / path).parent)
                (root / path).write_text(text, encoding="utf-8", newline="")
            changed = sorted(set(plan.moves.values()) | set(contents))
            if changed:
                git("add", "-A", "--", *changed)
        except (OSError, subprocess.CalledProcessError):
            # Gather before restoring, since destinations can also be sources.
            for i, (src, loc) in enumerate(locations.items()):
                rollback = Path(tmp) / f"rollback-{i}"
                os.replace(root / loc, rollback)
                locations[src] = str(rollback)
            for path in contents:
                if path not in plan.tracked and (root / path).exists():
                    (root / path).unlink()
            for src, loc in locations.items():
                (root / src).parent.mkdir(parents=True, exist_ok=True)
                os.replace(loc, root / src)
            # Files rewritten after moving need their original text, too.
            for src in locations:
                if src in plan.texts:
                    (root / src).write_text(plan.texts[src], encoding="utf-8", newline="")
            for path, data in original.items():
                (root / path).write_bytes(data)
            git("read-tree", "HEAD")
            for d in sorted(directories, key=lambda p: len(p.parts), reverse=True):
                if d.exists() and not any(d.iterdir()):
                    d.rmdir()
            raise


def main(argv):
    if len(argv) not in (1, 2) or (len(argv) == 2 and argv[1] != "--dry-run") or argv[0].startswith("--"):
        print(__doc__, file=sys.stderr)
        return 2
    try:
        root = Path.cwd()
        if Path(git("rev-parse", "--show-toplevel").strip()) != root:
            raise ValueError("run from the repository root")
        if git("status", "--porcelain=v1", "--untracked-files=no"):
            raise ValueError("working tree has tracked changes; commit or restore them first")
        tracked = set(git("ls-files", "-z").rstrip("\0").split("\0")) - {""}
        entries, titles, errors = read_map(Path(argv[0]))
        plan = Plan(root, tracked, entries, titles, errors)
        if plan.errors:
            for error in plan.errors:
                print(f"move: {error}", file=sys.stderr)
            return 2
        contents = plan.rewrite()
        if len(argv) == 1:
            apply(plan, contents)
        plan.summary()
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "stderr", None) or error
        print(f"move: {detail}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
