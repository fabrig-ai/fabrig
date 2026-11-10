#!/usr/bin/env python3
"""Move documents and numbered sections, preserving the citation checker's readings.

Usage from the repository root: python3 -I -B tools/citations/move.py MAP [--dry-run]
Exit 0 on success, 2 on a map error or a git failure. Changes are staged, never committed.
"""
import collections
import datetime
import fnmatch
import html
import importlib.util
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unicodedata
from urllib.parse import quote, unquote

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
        text = re.split(r"(?<!\S)#", raw, maxsplit=1)[0].strip()
        if not text:
            continue
        try:
            if text.startswith("renumber "):
                src, number = address(text[9:].strip())
                if number or not src.endswith(".md"):
                    raise ValueError("want renumber PATH.md")
                entries.append(Entry(src, None, src, None, ln, "renumber"))
                continue
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
            if (new_number and not number) or directory != dst.endswith("/"):
                raise ValueError("source and destination must have the same kind")
            if not directory and not dst.endswith(".md"):
                raise ValueError("document destination must end in .md")
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
    # Code spans contain literal markup: protect them before reading the visible text.
    code = []

    def protect(match):
        code.append(match.group(2))
        return f"\0{len(code) - 1}\0"

    text = re.sub(r"(`+)(.+?)\1", protect, text)
    text = html.unescape(re.sub(r"<[^>]*>", "", text))
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"(?<![\w\\])_{1,3}(.+?)_{1,3}(?!\w)", r"\1", text)
    text = re.sub(r"\0(\d+)\0", lambda m: code[int(m.group(1))], text)
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
        self.line_cache = {}
        self.edits = collections.defaultdict(dict)
        self.consumed = collections.defaultdict(list)
        self.rewritten, self.rewritten_files = 0, set()
        self.bare_rewritten = 0
        self.expectations, self.wrong = [], []
        self.status_locations = {}
        self.status_copies = collections.defaultdict(list)
        self.edit_destination = None
        self.paragraphs = {}
        self.renumbers = {e.src for e in entries if e.directory == "renumber"}
        self.renumbered = {}
        self.validate()

    def read(self, path):
        if path not in self.texts:
            with (self.root / path).open(encoding="utf-8", errors="surrogateescape", newline="") as fh:
                self.texts[path] = fh.read()
        return self.texts[path]

    def lines(self, path):
        if path not in self.line_cache:
            parts = self.read(path).split("\n")
            self.line_cache[path] = [part + "\n" for part in parts[:-1]] + ([parts[-1]] if parts[-1] else [])
        return self.line_cache[path]

    def validate(self):
        seen, files, directories = set(), {}, []
        explicit_files = {e.src: e.dst for e in self.entries if not e.number and not e.directory}
        for e in self.entries:
            key = (e.src, e.number, e.directory == "renumber")
            if key in seen:
                self.errors.append(f"map:{e.line}: duplicate source {e.src} {e.number or ''}".rstrip())
            seen.add(key)
            if e.directory == "renumber":
                if e.src not in self.tracked:
                    self.errors.append(f"map:{e.line}: source does not exist: {e.src}")
            elif e.directory:
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
        directory_destinations = {}
        for e in directories:
            if e.dst in directory_destinations:
                self.errors.append(f"two directory sources onto one destination: {directory_destinations[e.dst]}, {e.src} -> {e.dst}")
            directory_destinations[e.dst] = e.src
            existing = [p for p in self.tracked if p.startswith(e.dst)]
            moved_away = existing and all(p in self.moves for p in existing)
            if (self.root / e.dst).exists() and e.src != e.dst and not moved_away:
                self.errors.append(f"destination exists and is not moved away: {e.dst}")
        self.final_tracked = set(occupied)
        self.created = {e.dst for e, _ in self.sections} - self.final_tracked
        whole_destinations = collections.Counter(e.dst for e, _ in self.sections if not e.new_number)
        for dst, count in whole_destinations.items():
            if count > 1:
                self.errors.append(f"two sources onto one whole-document destination: {dst}")
        for e, _ in self.sections:
            if e.new_number is None and e.dst not in self.created:
                self.errors.append(f"map:{e.line}: whole-section destination must be a created document: {e.dst}")
            if e.dst not in self.final_tracked:
                if e.dst not in self.titles:
                    self.errors.append(f"created document needs a title: {e.dst}")
                if (self.root / e.dst).exists() and e.dst not in self.moves:
                    self.errors.append(f"destination exists and is not moved away: {e.dst}")
                self.check_parents(e.dst)
            self.final_tracked.add(e.dst)
        for path in self.created:
            status = path[:-3] + ".status.md"
            if status in self.final_tracked or ((self.root / status).exists() and status not in self.moves):
                self.errors.append(f"created status destination exists: {status}")
            self.final_tracked.add(status)
        for dst in sorted(set(self.moves.values()) | {e.dst for e, _ in self.sections}):
            for parent in Path(dst).parents:
                if str(parent) in self.final_tracked:
                    self.errors.append(f"destination parent is a file: {parent}")
        if self.errors:
            return
        for path in self.renumbers:
            remaining = [h for h in headings(self.lines(path))
                         if h.level == 2 and not self.section_at(path, h.start + 1)]
            for i, h in enumerate(remaining, 1):
                self.renumbered[path, h.number] = str(i)
        self.rows = self.layout()
        for path, rows in self.rows.items():
            nums = [h.number for h in headings([r.text for r in rows])]
            duplicates = [n for n, count in collections.Counter(nums).items() if count > 1]
            for n in duplicates:
                self.errors.append(f"destination section exists or has two sources: {path} {n}")
            for e, _ in self.sections:
                if e.dst == path and e.new_number and "." in e.new_number and e.new_number.rsplit(".", 1)[0] not in nums:
                    self.errors.append(f"destination parent section does not exist: {path} {e.new_number}")

    def check_parents(self, dst):
        for parent in Path(dst).parents:
            if str(parent) == ".":
                break
            if str(parent) in self.moves:
                continue
            if (self.root / parent).is_symlink() or ((self.root / parent).exists() and not (self.root / parent).is_dir()):
                self.errors.append(f"destination parent is not a directory: {parent}")

    def section_at(self, path, line):
        choices = [(e, h) for e, h in self.sections if e.src == path and h.start <= line - 1 < h.end]
        return max(choices, key=lambda pair: len(pair[0].number.split(".")), default=(None, None))[0]

    def text_destination(self, path, line):
        if (path, line) in self.status_locations:
            return self.status_locations[path, line][0]
        e = self.section_at(path, line)
        return e.dst if e else self.moves.get(path, path)

    def target(self, path, number):
        choices = [e for e, _ in self.sections if e.src == path and prefix(number, e.number)]
        if choices:
            e = max(choices, key=lambda e: len(e.number))
            return e.dst, (e.new_number + number[len(e.number):] if e.new_number
                           else number[len(e.number):].lstrip("."))
        first, dot, tail = number.partition(".")
        if (path, first) in self.renumbered:
            number = self.renumbered[path, first] + dot + tail
        return self.moves.get(path, path), number

    def transformed(self, row):
        if row.file is None:
            return row
        e = self.section_at(row.file, row.line)
        line = row.text
        h = re.match(r"^(#{1,6})(\s+)", line)
        if not h:
            return row
        # Only headings outside fences are changed (layout supplies their line set).
        if (row.file, row.line) not in self.heading_lines:
            return row
        level = len(h.group(1))
        if e:
            level += (len(e.new_number.split(".")) if e.new_number else 0) - len(e.number.split("."))
        if not 1 <= level <= 6:
            self.errors.append(f"heading level outside 1..6: {row.file}:{row.line}")
            return row
        m = c.HEADING.match(line)
        if m:
            _, n = self.target(row.file, m.group(1))
            if not n:
                return row._replace(text="")
            tail = line[m.end(1):]
            if tail.startswith("."):
                tail = tail[1:]
            line = "#" * level + h.group(2) + n + ("." if "." not in n else "") + tail
        else:
            line = "#" * level + line[h.end(1):]
        return row._replace(text=line)

    def layout(self):
        paths = set(self.moves) | {e.src for e, _ in self.sections} | self.renumbers
        paths |= {p for p in self.tracked if self.moves.get(p, p) in {e.dst for e, _ in self.sections}}
        paths = {p for p in paths if p.endswith(".md")}
        self.layout_sources = paths
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
                    rows[dest].append(self.transformed(row))
        for e, _ in sorted(self.sections, key=lambda pair: (pair[0].dst, number_key(pair[0].new_number or "0"))):
            if e.dst not in rows:
                rows[e.dst] = [Row(None, 0, f"# {self.titles.get(e.dst, '')}\n"), Row(None, 0, "\n")]
            dest = rows[e.dst]
            hs = headings([r.text for r in dest])
            parent = e.new_number.rsplit(".", 1)[0] if e.new_number and "." in e.new_number else None
            siblings = [h for h in hs if (h.number.rsplit(".", 1)[0] if "." in h.number else None) == parent]
            larger = [h for h in siblings if number_key(h.number) > number_key(e.new_number or "0")]
            smaller = [h for h in siblings if number_key(h.number) < number_key(e.new_number or "0")]
            if not e.new_number:
                at = 2
            elif larger:
                at = min(h.start for h in larger)
            elif smaller:
                at = max(smaller, key=lambda h: number_key(h.number)).end
            else:
                at = next((h.end for h in hs if h.number == parent), len(dest))
            chunk = list(chunks[e])
            if not e.new_number:
                while chunk and not chunk[0].text.strip():
                    chunk.pop(0)
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
            candidate = (os.path.relpath(target, os.path.dirname(src) or ".")
                         if c.tree_of(src) == c.tree_of(target) else target)
            if token.startswith("./") and not candidate.startswith("."):
                candidate = "./" + candidate
        candidates = [candidate]
        if c.tree_of(src) == c.tree_of(target):
            candidates.append(os.path.relpath(target, os.path.dirname(src) or "."))
            if is_code and c.tree_of(src):
                candidates.append(os.path.relpath(target, f"{c.tree_of(src)}/docs/design"))
        candidates.append(target)
        for candidate in candidates:
            resolved, how = c.resolve(tree, src, candidate, owner, is_code)
            if resolved == target and how not in ("suffix-ambiguous", "suffix"):
                return candidate
        return target

    def edit(self, start, end, replacement):
        """Record a physical replacement, including a token split across wrapped lines."""
        lines = self.lines(start.file)

        def put(line, a, b, text):
            if self.edit_destination is not None:
                self.status_edits[self.edit_destination][(start.file, line, a, b)] = text
            else:
                self.edits[start.file][(line, a, b)] = text

        if start.line == end.line:
            put(start.line, start.column - 1, end.column - 1, replacement)
        else:
            put(start.line, start.column - 1, len(lines[start.line - 1].rstrip("\r\n")), replacement)
            for ln in range(start.line + 1, end.line + 1):
                a = 0
                b = end.column - 1 if ln == end.line else len(lines[ln - 1].rstrip("\r\n"))
                put(ln, a, b, "")

    def manual(self, citation, reason):
        self.by_hand.add((citation.file, citation.line, reason, citation.context))

    def span_text(self, start, end):
        lines = self.lines(start.file)
        if start.line == end.line:
            return lines[start.line - 1][start.column - 1:end.column - 1]
        return (lines[start.line - 1][start.column - 1:]
                + "".join(lines[start.line:end.line - 1]) + lines[end.line - 1][:end.column - 1])

    def expanded(self, cite, tree):
        """Expand ranges from existing headings at the endpoints' depth, including holes."""
        out = []
        for i, n in enumerate(cite.numbers):
            if i:
                prev = cite.numbers[i - 1]
                if tree is self.before:
                    between = self.span_text(prev.end, n.start)
                else:
                    lines = Path(tree.root, cite.file).read_text(encoding="utf-8", errors="surrogateescape").splitlines(True)
                    between = (lines[prev.end.line - 1][prev.end.column - 1:n.start.column - 1]
                               if prev.end.line == n.start.line else
                               lines[prev.end.line - 1][prev.end.column - 1:]
                               + "".join(lines[prev.end.line:n.start.line - 1])
                               + lines[n.start.line - 1][:n.start.column - 1])
                if re.fullmatch(r"\s*(?:to|–|-)\s*", between):
                    depth = len(prev.value.split("."))
                    if depth == len(n.value.split(".")):
                        targets = prev.targets or cite.targets
                        middle = sorted({v for t in targets for v in tree.heading_numbers(t)
                                         if len(v.split(".")) == depth
                                         and number_key(prev.value) < number_key(v) < number_key(n.value)},
                                        key=number_key)
                        out.extend(c.Number(v, prev.end, n.start,
                                            tuple(t for t in targets if v in tree.heading_numbers(t)))
                                   for v in middle)
            out.append(n)
        return out

    def bare_numbers(self, tree, path, lines, readings):
        """Own-document dotted references, outside literal markup and numeric quantities."""
        if "/docs/design/" not in path or path.endswith((".status.md", ".execution.md")):
            return
        have = tree.heading_numbers(path)
        prose, fence = [], None
        for line in lines:
            marker = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
            if marker:
                token = marker.group(1)
                if fence is None:
                    fence = token
                elif token[0] == fence[0] and len(token) >= len(fence):
                    fence = None
                prose.append("")
            else:
                prose.append("" if fence else line)
        covered = [(n.start, n.end) for cite in readings for n in cite.numbers]
        for text, marks in c.logical_lines(prose, path, False):
            if re.match(r"^#", text):
                continue
            masked = text
            patterns = (r"(`+).*?\1", r"\]\([^)]+\)", r"(?:[a-zA-Z][\w+.-]*://|www\.)[^\s)]+")
            for pattern in patterns:
                masked = re.sub(pattern, lambda m: " " * len(m.group()), masked)
            found, last_end = [], 0
            for m in re.finditer(r"(?<![\w.])\d+(?:\.\d+)+(?![\w.]|[-+][\w])", masked):
                start, end = c.position_at(path, marks, m.start()), c.position_at(path, marks, m.end())
                if m.group() not in have or any(a <= start < b for a, b in covered):
                    continue
                tail = text[m.end():]
                pre = text[:m.start()]
                if re.search(r"(?i)\b(?:v|version|release|Linux|kernel|Python|Rust|Debian|Ubuntu|MiniJinja|OpenSSL)\s*$", pre):
                    continue
                if re.search(r"(?i)\bsince\s*$", pre) and re.search(r"(?i)\b(?:Linux|kernel|overlayfs)\b", pre):
                    continue
                if re.match(r"\s*(?:%|°[CF]|(?:[kMGTmunµ]?s|ms|seconds?|minutes?|hours?|days?|bytes?|bits?|[kMGT]?i?B|[kMGT]?bit|[kMGT]?bps|fps|[kMG]?Hz|[cmk]?m|kg|V|W|px|em|rem|percent)\b)", tail):
                    continue
                n = c.Number(m.group(), start, end, (path,))
                if found and not re.fullmatch(c.SEP, text[last_end:m.start()]):
                    yield self.bare_citation(path, text, found)
                    found = []
                found.append(n)
                last_end = m.end()
            if found:
                yield self.bare_citation(path, text, found)

    @staticmethod
    def bare_citation(path, text, nums):
        return c.Citation("bare-number", path, nums[0].start.line, text.strip(), None,
                          nums[0].start, nums[-1].end, path, "own-number", None,
                          tuple(nums), (path,), None, (c.Reading(path, "self"),))

    def status_rows(self):
        """Move complete entries, retaining row origins for citations and reporting."""
        self.status_edits = collections.defaultdict(dict)
        for path in sorted(self.created):
            status = path[:-3] + ".status.md"
            name = os.path.basename(path)
            stem = name[:-3]
            date = datetime.datetime.now(datetime.timezone.utc).date().isoformat()
            self.rows[status] = [Row(None, 0, f"# {stem}: status\n"), Row(None, 0, "\n"),
                                 Row(None, 0, f"{date}. Where the tree's code lags or departs from [{name}]({name}).\n"),
                                 Row(None, 0, "\n")]
        for path in sorted(p for p in self.tracked if p.endswith(".status.md")):
            doc = c.doc_of_status(path)
            if not any(e.src == doc for e, _ in self.sections) and doc not in self.renumbers:
                continue
            lines = self.lines(path)
            starts = [i for i, line in enumerate(lines)
                      if re.match(r"^(?:-\s+)?\*\*(?:Not yet|Departs|Design lags|Design departs|Open):\*\*", line)]
            if not starts:
                continue
            labels = [cite for cite in c.citations(self.before, path, [line.rstrip("\r\n") for line in lines], False)
                      if cite.kind == "status-label"]
            default = self.moves.get(path, path)
            rows = self.rows.setdefault(default, [Row(path, i, line) for i, line in enumerate(lines, 1)])
            for index, start in enumerate(starts):
                end = starts[index + 1] if index + 1 < len(starts) else len(lines)
                # Entries stop at another list item or heading; blank continuation lines go with them.
                end = next((i for i in range(start + 1, end)
                            if re.match(r"^(?:[-*+] |#)", lines[i])), end)
                text = "".join(lines[start:end])
                matches = [cite for cite in labels if start < cite.start.line <= end]
                groups = [(cite, [self.target(doc, n.value) for n in self.expanded(cite, self.before)])
                          for cite in matches]
                whole = re.search(r"\*\*(?:Not yet|Departs|Design lags|Design departs|Open):\*\*\s*((?:the )?whole document)", lines[start])
                if whole:
                    a = c.Position(path, start + 1, whole.start(1) + 1)
                    b = c.Position(path, start + 1, whole.end(1) + 1)
                    number = c.Number("", a, b, (doc,))
                    cite = c.Citation("status-label", path, start + 1, lines[start].strip(), None,
                                      a, b, doc, "whole-status", None, (number,), (doc,), None, ())
                    targets = {self.moves.get(doc, doc)} | {self.target(doc, n)[0] for n in self.before.heading_numbers(doc)}
                    groups.append((cite, [(target, "") for target in sorted(targets)]))
                if not groups:
                    continue
                destinations = {p[:-3] + ".status.md" for _, ds in groups for p, _ in ds}
                if not destinations:
                    continue
                if any(dst != default and c.doc_of_status(dst) not in self.created for dst in destinations):
                    self.by_hand.add((path, start + 1, "status entry needs an existing destination", text.strip()))
                    continue
                entry = [Row(path, i + 1, lines[i]) for i in range(start, end)]
                if destinations != {default}:
                    rows[:] = [r for r in rows if not (r.file == path and start < r.line <= end)]
                    for dst in sorted(destinations):
                        self.rows[dst].extend(entry)
                    self.by_hand.add((path, start + 1,
                                      "split status entry" if len(destinations) > 1 else "moved status entry",
                                      text.strip()))
                self.status_locations[path, start + 1] = (sorted(destinations)[0], 0)
                for ln in range(start + 1, end + 1):
                    self.status_locations[path, ln] = (sorted(destinations)[0], 0)
                for dst in destinations:
                    for cite, ds in groups:
                        nums = [n for p, n in ds if p[:-3] + ".status.md" == dst]
                        original = self.span_text(cite.numbers[0].start, cite.numbers[-1].end)
                        replacement = self.format_numbers(self.after, c.doc_of_status(dst), nums, original)
                        a = cite.numbers[0].start
                        b = cite.numbers[-1].end if nums else cite.end
                        self.edit_destination = dst
                        self.edit(a, b, replacement)
                        self.edit_destination = None
            if not any(re.match(r"^(?:-\s+)?\*\*(?:Not yet|Departs|Design lags|Design departs|Open):\*\*", r.text)
                       for r in rows):
                rows.append(Row(None, 0, "No gap.\n"))
        for path in self.created:
            rows = self.rows[path[:-3] + ".status.md"]
            if len(rows) == 4:
                rows.append(Row(None, 0, "No gap.\n"))
        for dst, rows in self.rows.items():
            if dst.endswith(".status.md"):
                for row in rows:
                    if row.file and dst not in self.status_copies[row.file, row.line]:
                        self.status_copies[row.file, row.line].append(dst)

    def citing_destinations(self, path, line):
        return self.status_copies.get((path, line)) or [self.text_destination(path, line)]

    def edit_snapshot(self, path, source):
        return dict(self.status_edits[source] if path.endswith(".status.md") else self.edits[path])

    @staticmethod
    def format_numbers(tree, target, numbers, original):
        nums = list(numbers)
        if "" in nums:
            return "the whole document"
        range_sep = re.search(r"\s*(to|–|-)\s*", original)
        if range_sep and len(nums) > 1:
            runs = []
            for number in nums:
                siblings = sorted((n for n in tree.heading_numbers(target)
                                   if len(n.split(".")) == len(number.split("."))), key=number_key)
                previous = runs[-1][-1] if runs else None
                adjacent = (previous in siblings and number in siblings
                            and siblings.index(number) == siblings.index(previous) + 1)
                if adjacent:
                    runs[-1].append(number)
                else:
                    runs.append([number])
            joiner = " to " if range_sep.group(1) == "to" else range_sep.group(1)
            return " and ".join(run[0] + joiner + run[-1] if len(run) > 1 else run[0] for run in runs)
        conjunction = "and" if re.search(r"\band\b", original) else "or" if re.search(r"\bor\b", original) else None
        if conjunction and len(nums) > 2:
            tail = (", " if re.search(r",\s*" + conjunction, original) else " ") + conjunction + " "
            return ", ".join(nums[:-1]) + tail + nums[-1]
        return (" " + conjunction + " " if conjunction else ", ").join(nums)

    def render_reference(self, after, source, cite, destinations, expanded, is_code=False, force_path=False):
        groups = []
        for target, number in destinations:
            if groups and groups[-1][0] == target:
                groups[-1][1].append(number)
            else:
                groups.append((target, [number]))
        original = self.span_text(cite.numbers[0].start, cite.numbers[-1].end)
        word = "sections " if re.search(r"\bsections\b", self.span_text(cite.start, cite.numbers[0].start)) else (
            "section " if cite.kind == "section-self" or re.search(r"\bsection\b", self.span_text(cite.start, cite.numbers[0].start)) else "")
        parts = []
        for target, numbers in groups:
            path_target = target[:-3] + ".status.md" if cite.target and cite.target.endswith(".status.md") and cite.kind == "path" else target
            token = self.styled(after, source, path_target, cite.token or os.path.basename(path_target), cite.owner, is_code)
            nums = list(numbers)
            if "" in nums:
                suffix = ""
            else:
                suffix = self.format_numbers(after, target, nums, original)
            label = word
            if len(nums) == 1:
                label = label.replace("sections", "section")
            if len(groups) == 1 and cite.kind in ("section-self", "bare-number") and target == c.doc_of_status(source) and suffix and not force_path:
                parts.append(label + suffix)
            else:
                start, _ = self.reference_span(cite)
                quoted = cite.kind != "path" or start != cite.start
                if cite.link:
                    link_text = self.link_label(cite, token, cite.link.tail)
                    path_text = f"[{link_text}]({token}{cite.link.tail})"
                else:
                    path_text = f"`{token}`" if quoted else token
                parts.append(path_text + (f", {label}{suffix}" if suffix else ""))
        conjunction = "or" if re.search(r"\bor\b", original) else "and"
        joiner = ", " + conjunction + " " if all(nums and "" not in nums for _, nums in groups) else " " + conjunction + " "
        if "," in original and not re.search(r"\b(?:and|or|to)\b|[–-]", original):
            joiner = ", "
        return joiner.join(parts)

    def reference_span(self, cite):
        start, end = cite.start, cite.numbers[-1].end if cite.numbers else cite.end
        if cite.link and not cite.numbers:
            end = cite.link.tail_end
        if cite.link:
            start = cite.link.start._replace(column=cite.link.start.column - 1)
        if cite.kind == "path" and not cite.link:
            line = self.lines(start.file)[start.line - 1]
            if start.column > 1 and line[start.column - 2] == "`":
                start = start._replace(column=start.column - 1)
        return start, end

    def numbers_to_rewrite(self, cite, tree):
        return self.expanded(cite, tree)

    def protect_antecedent(self, cite, introduced):
        paragraph = self.paragraphs.get((cite.file, cite.start.line))
        return cite.kind == "section-self" and any(
            a < cite.start and self.paragraphs.get((a.file, a.line)) == paragraph for a in introduced)

    def preserve_suffix_name(self, cite, shadowed, resolved):
        if shadowed and cite.how in ("suffix", "suffix-ambiguous"):
            self.manual(cite, f"the bare name now resolves to {resolved}")
            return True
        return False

    def dependent_grammar(self, cite):
        if cite.kind != "path" or not cite.target:
            return
        paragraph = self.paragraphs.get((cite.file, cite.start.line))
        lines = self.lines(cite.file)
        first = cite.start.line
        while first > 1 and self.paragraphs.get((cite.file, first - 1)) == paragraph:
            first -= 1
        pre = " ".join(line.strip() for line in lines[first - 1:cite.start.line - 1])
        pre += " " + lines[cite.start.line - 1][:cite.start.column - 1]
        for match in c.BARE_SECTION.finditer(pre):
            if not re.fullmatch(r"\s+of\s*[`\[]*", pre[match.end():]):
                continue
            nums = c.parse_nums(match.group(2))
            if any(self.target(cite.target, n) != (cite.target, n) for n in nums):
                self.manual(cite, "sections before the path need grammatical repair")

    @staticmethod
    def link_label(cite, token, tail):
        pairs = [(cite.token + cite.link.tail, token + tail), (cite.token, token)]
        if cite.token.endswith(".status.md"):
            pairs.append((c.doc_of_status(cite.token), c.doc_of_status(token)))
        label = cite.link.text
        for old, new in pairs:
            if old == new:
                continue
            if label.startswith(f"`{old}`"):
                return f"`{new}`" + label[len(old) + 2:]
            if label == old:
                return new
        return label

    def rewrite_link_label(self, cite, token, tail):
        label = self.link_label(cite, token, tail)
        if label != cite.link.text:
            self.edit(cite.link.start, cite.link.end, label)

    def rewrite(self):
        before = c.Tree(str(self.root), self.tracked)
        before.docs = sorted((p for p, _ in c.walk(str(self.root), every_md=True) if p in self.tracked), key=c.document_order)
        self.before = before
        self.anchor_expected = {}
        after = c.Tree(str(self.root), self.final_tracked)
        after.docs = [self.moves.get(p, p) for p in before.docs]
        after.docs += sorted(self.created | {p[:-3] + ".status.md" for p in self.created})
        after.docs.sort(key=c.document_order)
        self.final_docs = after.docs
        self.after = after
        # Future headings must come from the layout, never from an occupied old pathname.
        for path, rows in self.rows.items():
            after.headings[path] = {h.number for h in headings([r.text for r in rows])}
        self.status_rows()
        old_slugs, anchors = {}, {}
        for path, rows in self.rows.items():
            for name, origin in slugs(rows).items():
                anchors[origin] = (path, name)
        for e, h in self.sections:
            if not e.new_number:
                anchors[e.src, h.start + 1] = (e.dst, "")
        introduced = collections.defaultdict(list)
        for path, is_code in sorted(c.walk(str(self.root))):
            if path not in self.tracked:
                continue
            text = self.read(path)
            if is_code and ".md" not in text:
                continue
            readings = list(c.citations(before, path, text.split("\n"), is_code))
            for index, (_, marks) in enumerate(c.logical_lines(text.split("\n"), path, is_code)):
                for _, line, _ in marks:
                    self.paragraphs[path, line] = index
            if not is_code:
                readings += list(self.bare_numbers(before, path, text.split("\n"), readings))
            for cite, source in [(cite, source) for cite in readings
                                 for source in self.citing_destinations(path, cite.start.line)]:
                self.edit_destination = source if path.endswith(".status.md") else None
                if cite.kind == "status-label":
                    # Labels are rendered with their complete entry, including split copies.
                    for n in cite.numbers:
                        dest = self.target(cite.target, n.value)
                        managed = any(key[0] == path and key[1] == n.start.line
                                      for edits in self.status_edits.values() for key in edits)
                        if not managed and dest[0] == self.moves.get(cite.target, cite.target) and dest[1] != n.value:
                            self.edit(n.start, n.end, dest[1])
                    nums = [n for n in self.expanded(cite, before)
                            if self.target(cite.target, n.value)[0] == c.doc_of_status(source)]
                    self.expectations.append((cite, nums, is_code, source))
                    if any(self.target(cite.target, n.value) != (c.doc_of_status(source), n.value)
                           for n in cite.numbers):
                        self.rewritten += 1
                        self.rewritten_files.add(source)
                    continue
                if cite.kind == "path":
                    self.consumed[path].append((cite.start, cite.end))
                    if cite.link:
                        bases = {cite.token, cite.token + cite.link.tail, c.doc_of_status(cite.token)}
                        if any(cite.link.text == base or cite.link.text.startswith(f"`{base}`") for base in bases):
                            self.consumed[path].append((cite.link.start, cite.link.end))
                    self.dependent_grammar(cite)
                self.expectations.append((cite, self.expanded(cite, before), is_code, source))
                expanded = self.numbers_to_rewrite(cite, before)
                destinations, ambiguous = [], False
                for n in expanded:
                    targets = n.targets or ((c.doc_of_status(cite.target),) if cite.target else ())
                    candidates = {self.target(t, n.value) for t in targets}
                    if len(candidates) > 1:
                        # Candidates can share a number rewrite while following their
                        # documents. The final reader checks that no new candidate appears.
                        consistent = (len({number for _, number in candidates}) == 1
                                      and all(self.target(t, n.value)[0] == self.moves.get(t, t) for t in targets)
                                      and (cite.kind != "section-self" or
                                           (source == self.moves.get(path, path) and not self.protect_antecedent(cite, introduced[path]))))
                        if not consistent:
                            ambiguous = True
                        if cite.kind == "section-self" and consistent:
                            chosen = (c.doc_of_status(source), next(iter(candidates))[1])
                        else:
                            chosen = next((d for d in sorted(candidates) if cite.target and d[0] == self.moves.get(cite.target, cite.target)), None)
                            if chosen is None and consistent:
                                chosen = sorted(candidates)[0]
                        destinations.append(chosen)
                    else:
                        destinations.append(next(iter(candidates), None))
                if ambiguous:
                    self.manual(cite, "ambiguous bare section" if cite.kind == "section-self" else "ambiguous section")
                    continue
                affected = (source != path or cite.target in self.moves
                            or any(self.target(t, n.value) != (t, n.value) for n in expanded for t in n.targets))
                if self.protect_antecedent(cite, introduced[path]):
                    affected = True
                anchor_destination = None
                if cite.link and "#" in cite.link.tail and cite.target and cite.target.endswith(".md"):
                    if cite.target not in old_slugs:
                        old_slugs[cite.target] = slugs([Row(cite.target, i, line) for i, line in enumerate(self.lines(cite.target), 1)])
                    anchor_token = cite.link.tail.partition("#")[2]
                    anchor = unquote(anchor_token)
                    anchor_destination = anchors.get(old_slugs[cite.target].get(anchor))
                    if anchor_destination:
                        self.anchor_expected[cite.start] = anchor_destination
                    affected |= bool(anchor_destination and anchor_destination != (cite.target, anchor))
                if cite.kind == "path" and cite.target:
                    resolved, how = c.resolve(after, source, cite.token, cite.owner, is_code)
                    shadowed = resolved != cite.target and cite.target not in self.moves
                    if self.preserve_suffix_name(cite, shadowed, resolved):
                        continue
                    affected |= resolved != cite.target
                if not affected:
                    continue
                edits_before = self.edit_snapshot(path, source)
                files = {d[0] for d in destinations if d is not None}
                has_range = any(re.fullmatch(r"\s*(?:to|–|-)\s*", self.span_text(a.end, b.start))
                                for a, b in zip(cite.numbers, cite.numbers[1:]))
                render_range = has_range and all(d is not None for d in destinations) and not (cite.link and "#" in cite.link.tail)
                render_whole = any(d and not d[1] for d in destinations) and not (cite.link and "#" in cite.link.tail)
                if len(files) > 1 or render_range or render_whole:
                    if (cite.link and "#" in cite.link.tail) or any(d is None for d in destinations):
                        self.manual(cite, "split citation with dependent grammar")
                        continue
                    start, end = self.reference_span(cite)
                    self.edit(start, end, self.render_reference(after, source, cite, destinations, expanded, is_code,
                                                                 force_path=self.protect_antecedent(cite, introduced[path])))
                    introduced[path].append(start)
                elif cite.kind in ("section-self", "bare-number"):
                    if files and (next(iter(files)) != c.doc_of_status(source)
                                  or "" in {d[1] for d in destinations if d}
                                  or self.protect_antecedent(cite, introduced[path])):
                        start, end = self.reference_span(cite)
                        replacement = self.render_reference(after, source, cite, destinations, expanded, force_path=True)
                        self.edit(start, end, replacement)
                        introduced[path].append(start)
                    else:
                        for n, dest in zip(expanded, destinations):
                            if dest and dest[1] != n.value and n in cite.numbers:
                                self.edit(n.start, n.end, dest[1])
                elif cite.target:
                    target = next(iter(files)) if files else self.moves.get(cite.target, cite.target)
                    if files and cite.target.endswith(".status.md"):
                        target = target[:-3] + ".status.md"
                    tail = cite.link.tail if cite.link else ""
                    if anchor_destination:
                        anchor_target, name = anchor_destination
                        if files and anchor_target != target:
                            self.manual(cite, "split citation (anchor and sections)")
                            continue
                        if "%" in anchor_token:
                            name = quote(name)
                        target, tail = anchor_target, tail.partition("#")[0] + ("#" + name if name else "")
                    if target != self.moves.get(cite.target, cite.target):
                        introduced[path].append(cite.start)
                    resolved, how = c.resolve(after, source, cite.token, cite.owner, is_code)
                    if cite.kind == "anchor" and target == source:
                        token = ""
                    elif target == cite.target and source == path and resolved == target and how != "suffix-ambiguous":
                        token = cite.token
                    else:
                        token = self.styled(after, source, target, cite.token, cite.owner, is_code)
                    if expanded and destinations and all(d and not d[1] for d in destinations):
                        if cite.link:
                            self.edit(cite.start, cite.end, token)
                            end_of_link = cite.link.tail_end._replace(column=cite.link.tail_end.column + 1)
                            self.edit(end_of_link, cite.numbers[-1].end, "")
                        else:
                            start, end = self.reference_span(cite)
                            quoted = start != cite.start
                            self.edit(start, end, f"`{token}`" if quoted else token)
                    else:
                        if token != cite.token:
                            self.edit(cite.start, cite.end, token)
                        for n, dest in zip(expanded, destinations):
                            if dest and dest[1] != n.value and n in cite.numbers:
                                self.edit(n.start, n.end, dest[1])
                    if cite.link:
                        if tail != cite.link.tail:
                            self.edit(cite.link.tail_start, cite.link.tail_end, tail)
                        self.rewrite_link_label(cite, token, tail)
                if edits_before != self.edit_snapshot(path, source):
                    if cite.kind == "bare-number":
                        changed = {key: value for key, value in self.edits[path].items()
                                   if edits_before.get(key) != value}
                        self.bare_rewritten += sum(any(
                            ln == n.start.line and a <= n.start.column - 1 < b
                            for ln, a, b in changed) for n in cite.numbers)
                    else:
                        self.rewritten += 1
                    self.rewritten_files.add(source)
        self.edit_destination = None
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
                    self.by_hand.add((c.ALLOW, ln, "glob matches a moved path", glob))
            else:
                glob = self.moves.get(glob, glob)
            parts = target.split()
            if len(parts) == 2 and re.fullmatch(c.NUM, parts[1]):
                destinations = [self.target(p, parts[1]) for p in parts[0].split("|")]
                if len({n for _, n in destinations}) > 1:
                    self.by_hand.add((c.ALLOW, ln, "split allow target", target))
                    continue
                parts = ["|".join(p for p, _ in destinations)] + ([destinations[0][1]] if destinations[0][1] else [])
            else:
                parts = [self.moves.get(p, p) for p in parts]
            target = " ".join(parts)
            if old != (glob, target):
                self.edits[c.ALLOW][(ln, 0, len(line.rstrip("\r\n")))] = f"{glob} {target} # {reason.rstrip()}"
                self.rewritten += 1
                self.rewritten_files.add(c.ALLOW)

    def names_by_hand(self):
        # Match path shapes once, then look up tokens, independent of the map's size.
        tokens = collections.defaultdict(set)
        all_sources = set(self.moves) | {e.src for e, _ in self.sections}
        basenames = {os.path.basename(path) for path in all_sources}
        for old in all_sources:
            forms = {old, os.path.basename(old)}
            home = c.tree_of(old)
            if home:
                forms.add(old[len(home) + 1:])
                if old.startswith(f"{home}/docs/design/"):
                    forms.add(old[len(f'{home}/docs/design/'):])
            for form in forms:
                tokens[form].add(old)
        pattern = re.compile(r"(?<![\w./-])(?:/|(?:\.\./)+|\./)?(?:(?:[\w.+-]+/)+[\w.+-]+|[\w][\w.+-]*\.[\w+-]+)(?![\w/-])")
        for path in sorted(self.tracked):
            parts = path.split("/")
            if parts[0] in c.TREES and parts[1:3] == ["docs", "plan"]:
                continue
            if parts[0] in c.TREES and parts[1:3] == ["docs", "development"] and c.DATED.search(parts[-1]):
                continue
            if (self.root / path).is_symlink() or re.search(r"(?i)(?:^|/)(?:auth\.json|\.credentials\.json|[^/]*(?:secret|token|credential|private[-_]key)[^/]*)$|\.(?:pem|key|p12|pfx)$", path):
                continue
            for ln, line in enumerate(self.read(path).splitlines(), 1):
                for m in pattern.finditer(line):
                    tok = m.group().rstrip(".")
                    key = tok.lstrip("/")
                    while key.startswith(("../", "./")):
                        key = key[3:] if key.startswith("../") else key[2:]
                    sources = tokens.get(key)
                    if not sources and os.path.basename(key) not in basenames:
                        continue
                    owner_match = c.OWNER.search(line[:m.start()][-40:])
                    owner = owner_match.group(1).lower() if owner_match else None
                    resolved, _ = c.resolve(self.before, path, tok, owner, path.endswith(c.CODE_EXT))
                    if not sources:
                        sources = {resolved} if resolved in all_sources else set()
                    if resolved not in sources:
                        continue
                    consumed = [(a.column - 1, b.column - 1) for a, b in self.consumed.get(path, [])
                                if a.line == ln == b.line]
                    edits = [(a, b) for number, a, b in self.edits.get(path, {}) if number == ln]
                    edits += [(a, b) for changes in self.status_edits.values()
                              for (file, number, a, b) in changes if file == path and number == ln]
                    if path == c.ALLOW or any(a <= m.start() and m.start() + len(tok) <= b for a, b in consumed + edits):
                        continue
                    moved_file = any(old in self.moves for old in sources)
                    moved_section = False
                    for old in sources:
                        sm = c.SECTION_AFTER.match(line[m.start() + len(tok):])
                        if sm:
                            offset = m.start() + len(tok) + sm.start(1)
                            nums = tuple(c.Number(n.group(), c.Position(path, ln, offset + n.start() + 1),
                                                  c.Position(path, ln, offset + n.end() + 1), (old,))
                                         for n in c.NUM_RE.finditer(sm.group(1)))
                            cite = c.Citation("path", path, ln, line, tok, nums[0].start, nums[-1].end,
                                              old, "name", None, nums, (old,), None, ())
                            if any(self.target(old, n.value) != (old, n.value) for n in self.expanded(cite, self.before)):
                                moved_section = True
                    if moved_file or moved_section:
                        self.by_hand.add((path, ln, "name in code or other text", tok))

    def row_text(self, dst, row):
        if row.file is None:
            return row.text
        text = self.lines(row.file)[row.line - 1]
        edits = {(a, b): value for (ln, a, b), value in self.edits.get(row.file, {}).items() if ln == row.line}
        edits.update({(a, b): value for (path, ln, a, b), value in self.status_edits.get(dst, {}).items()
                      if path == row.file and ln == row.line})
        end = len(text)
        for (a, b), value in sorted(edits.items(), reverse=True):
            if b > end:
                raise ValueError(f"overlapping citation edits: {row.file}:{row.line}")
            text = text[:a] + value + text[b:]
            end = a
        return self.transformed(row._replace(text=text)).text

    def contents(self):
        self.final_positions = collections.defaultdict(list)
        self.generated_lines = set()
        out = {}
        paths = set(self.rows) | set(self.status_edits) | {self.moves.get(p, p) for p in self.edits}
        inverse = {dst: src for src, dst in self.moves.items()}
        for dst in paths:
            rows = self.rows.get(dst)
            if rows is None:
                src = inverse.get(dst, dst)
                rows = [Row(src, i, text) for i, text in enumerate(self.lines(src), 1)]
            texts, ln, col = [], 1, 1
            for row in rows:
                text = self.row_text(dst, row)
                if row.file and text:
                    self.final_positions[row.file, row.line].append((dst, ln, col))
                elif text:
                    self.generated_lines.add((dst, ln))
                texts.append(text)
                if "\n" in text:
                    ln += text.count("\n")
                    col = len(text.rsplit("\n", 1)[-1]) + 1
                else:
                    col += len(text)
            out[dst] = "".join(texts)
        return {p: text for p, text in out.items() if p not in self.tracked or text != self.read(p)}

    def final_position(self, pos, end=False, destination=None):
        locations = self.final_positions.get((pos.file, pos.line))
        if locations:
            dst, ln, col = next((loc for loc in locations if loc[0] == destination), locations[0])
        else:
            dst, ln, col = self.moves.get(pos.file, pos.file), pos.line, 1
        offset = pos.column - 1
        shift = 0
        edits = dict(self.edits.get(pos.file, {}))
        edits.update({(ln, a, b): value for (file, ln, a, b), value in self.status_edits.get(dst, {}).items()
                      if file == pos.file})
        for (line, a, b), replacement in sorted(edits.items()):
            if line != pos.line:
                continue
            if a <= offset < b:
                offset = a + shift + (len(replacement) if end else 0)
                break
            if b <= offset:
                if a == b == offset and not end:
                    continue
                shift += len(replacement) - (b - a)
        else:
            offset += shift
        return c.Position(dst, ln, col + offset)

    def reading(self, cite, tree):
        nums = self.expanded(cite, tree)
        if nums:
            return [frozenset((t, n.value) for t in n.targets) for n in nums]
        if cite.link and "#" in cite.link.tail:
            return [frozenset(((cite.target, "#" + unquote(cite.link.tail.partition("#")[2])),))]
        return [frozenset(((cite.target, ""),))] if cite.target else []

    def expected(self, cite, nums):
        if cite.kind == "status-label" and not nums:
            return []
        if nums:
            values = [frozenset(self.target(t, n.value) for t in n.targets) for n in nums]
            # A whole-document member subsumes its contiguous subsection members.
            out, index = [], 0
            while index < len(values):
                value = values[index]
                if len(value) != 1:
                    out.append(value)
                    index += 1
                    continue
                path, _ = next(iter(value))
                end = index + 1
                while end < len(values) and len(values[end]) == 1 and next(iter(values[end]))[0] == path:
                    end += 1
                group = values[index:end]
                if any((path, "") in v for v in group):
                    if cite.kind == "path" and cite.target.endswith(".status.md"):
                        path = path[:-3] + ".status.md"
                    out.append(frozenset(((path, ""),)))
                else:
                    out.extend(group)
                index = end
            return out
        if cite.link and "#" in cite.link.tail:
            target, anchor = self.anchor_expected.get(cite.start, (self.moves.get(cite.target, cite.target), unquote(cite.link.tail.partition("#")[2])))
            return [frozenset(((target, "#" + anchor if anchor else ""),))]
        return [frozenset((self.moves.get(r.target, r.target), "") for r in cite.readings)] if cite.target else []

    def verify(self, root):
        """Re-read the final text and compare each original region's mapped meaning."""
        tree = c.Tree(str(root), self.final_tracked)
        tree.docs = sorted((p for p, _ in c.walk(str(root), every_md=True) if p in self.final_tracked), key=c.document_order)
        readings, texts = {}, {}
        for path, is_code in c.walk(str(root)):
            if path not in self.final_tracked:
                continue
            with (root / path).open(encoding="utf-8", errors="surrogateescape") as fh:
                text = fh.read()
            texts[path] = text.split("\n")
            cites = list(c.citations(tree, path, texts[path], is_code))
            if not is_code:
                # Use final headings for recognising the newly written own numbers.
                cites += list(self.bare_numbers(tree, path, texts[path], cites))
            readings[path] = cites
        matched = set()
        for cite, nums, is_code, source in self.expectations:
            if cite.file in self.layout_sources and (cite.file, cite.start.line) not in self.final_positions:
                # A section's discarded heading contributes no text to the destination.
                continue
            expected = self.expected(cite, nums)
            start, end = self.reference_span(cite)
            a = self.final_position(start, destination=source)
            b = self.final_position(end, end=True, destination=source)
            found = [v for v in readings.get(a.file, [])
                     if a.line <= v.start.line <= b.line
                     and (v.start.line, v.start.column) >= (a.line, a.column)
                     and (v.start.line, v.start.column) < (b.line, b.column)]
            found.sort(key=lambda v: (v.start.line, v.start.column))
            matched.update(id(v) for v in found)
            actual = [value for v in found for value in self.reading(v, tree)]
            if cite.kind == "status-label" and not found and a.line == b.line:
                fragment = texts[a.file][a.line - 1][a.column - 1:b.column - 1]
                if "the whole document" in fragment:
                    actual = [frozenset(((c.doc_of_status(a.file), ""),))]
            if cite.kind == "bare-number" and not found and a.line == b.line:
                # A promoted dotted reference may become a single number. Its original
                # coordinates establish that it is a reference, even though the gate
                # intentionally does not discover single bare numbers.
                fragment = texts[a.file][a.line - 1][a.column - 1:b.column - 1]
                nums = [c.Number(m.group(), a._replace(column=a.column + m.start()),
                                 a._replace(column=a.column + m.end()),
                                 (a.file,) if m.group() in tree.heading_numbers(a.file) else ())
                        for m in c.NUM_RE.finditer(fragment)]
                own = cite._replace(file=a.file, start=a, end=b, numbers=tuple(nums))
                actual = self.reading(own, tree)
            if cite.how in ("suffix", "suffix-ambiguous") and cite.target not in self.moves and cite.token:
                if found and found[0].token != cite.token:
                    self.wrong.append((a.file, a.line, texts[a.file][a.line - 1].strip(),
                                       [frozenset(((found[0].token or "", "restyled name"),))],
                                       [frozenset(((cite.token, "preserved bare name"),))]))
            agrees = len(actual) == len(expected) and all(read == want or (read and read <= want)
                                                        for read, want in zip(actual, expected))
            if not agrees:
                text = texts.get(a.file, [""])[a.line - 1].strip()
                self.wrong.append((a.file, a.line, text, actual, expected))
        for path, cites in readings.items():
            for cite in cites:
                if id(cite) in matched:
                    continue
                generated = (path, cite.start.line) in self.generated_lines
                if generated and cite.target and all(n.targets for n in cite.numbers):
                    continue
                text = texts[path][cite.start.line - 1].strip()
                actual = self.reading(cite, tree)
                self.wrong.append((path, cite.start.line, text, actual,
                                   [frozenset((("an original reading" if not generated else "a resolving generated citation", ""),))]))
        return not self.wrong

    def dry_verify(self, contents):
        with tempfile.TemporaryDirectory(prefix="citations-move-verify-") as temporary:
            root = Path(temporary)
            for src in self.tracked:
                dst = self.moves.get(src, src)
                if dst in contents or not src.endswith((".md",) + c.CODE_EXT):
                    continue
                path = root / dst
                path.parent.mkdir(parents=True, exist_ok=True)
                if (self.root / src).is_symlink():
                    path.symlink_to(os.readlink(self.root / src))
                else:
                    path.write_bytes((self.root / src).read_bytes())
            for dst, text in contents.items():
                path = root / dst
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8", errors="surrogateescape", newline="")
            return self.verify(root)

    def summary(self):
        for e in self.entries:
            if e.directory == "renumber":
                print(f"Renumber {e.src}")
            elif e.number:
                print(f"Section {e.src} {e.number} -> {e.dst}" + (f" {e.new_number}" if e.new_number else ""))
            else:
                print(f"{'Directory' if e.directory else 'Document'} {e.src} -> {e.dst}")
        print(f"Rewrote {self.rewritten} citations and {self.bare_rewritten} bare numbers in {len(self.rewritten_files)} files.")
        print("By hand:")
        items = []
        for path, line, reason, context in self.by_hand:
            if path in self.layout_sources and (path, line) not in self.final_positions:
                continue
            locations = sorted(self.final_positions.get((path, line), [(self.moves.get(path, path), line, 1)]))
            if reason in ("moved status entry", "split status entry"):
                locations = locations[:1]
            for dst, ln, _ in locations:
                items.append((dst.endswith(".md"), dst, ln, reason, context))
        for _, path, ln, reason, context in sorted(set(items)):
            print(f"  {path}:{ln}: {reason}: {' '.join(context.split())}")
        if not items:
            print("  (none)")
        if self.wrong:
            print("Wrong:")
            for path, ln, text, actual, expected in self.wrong:
                def describe(values):
                    return "; ".join(" | ".join(p + (" " + n if n else "") for p, n in sorted(v)) or "unresolved"
                                     for v in values) or "nothing"
                print(f"  {path}:{ln}: {text}\n    reads: {describe(actual)}\n    should read: {describe(expected)}")


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
                (root / path).write_text(text, encoding="utf-8", errors="surrogateescape", newline="")
            changed = sorted(set(plan.moves.values()) | set(contents))
            if changed:
                git("add", "-A", "--", *(f":(literal){p}" for p in changed))
        except BaseException:
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
                    (root / src).write_text(plan.texts[src], encoding="utf-8", errors="surrogateescape", newline="")
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
        if git("--no-optional-locks", "status", "--porcelain=v1", "--untracked-files=no"):
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
            correct = plan.verify(root)
        else:
            correct = plan.dry_verify(contents)
        plan.summary()
        return 0 if correct else 1
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "stderr", None) or error
        print(f"move: {detail}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
