"""Shared ratchet for tracked Rust code and comments.

Tree entries supply ordered FORMS and COMMENT_FORMS, with optional MARKERS,
WORDS, EXCLUDE, INCLUDE, EXEMPT and EXEMPT_DIRECTORIES. Regex forms count
matches; computed forms receive text, relative filename, tree and repository.
SYNTAX_FORMS see code with literals masked. Other code forms retain strings
and replace ordinary literal escapes with separators. Comment forms see only
comments, including doc comments. Generated path components and each crate's
patches/mod.rs are excluded; authored patch functions and context still count.
"""
import argparse
from functools import lru_cache
from pathlib import Path
import re
import subprocess
import sys

# Match whole quoted literals before looking for comment markers inside them.
# A closing quote distinguishes a character from a lifetime such as 'a.
TOKEN = re.compile(
    r'//[^\n]*|/\*|(?:br|cr|r)(?P<hashes>#{0,255})"'
    r'|"(?:\\[\s\S]|[^"\\])*"'
    r"|'(?:[^'\\\n]|\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\n]))'"
)
BLOCK = re.compile(r'/\*|\*/')
ESCAPE = re.compile(r'\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[\s\S])')


def split_source(source):
    """Return legacy code, comments, and code with literals masked.

    Comments retain their delimiters and line breaks. A separator between
    comments prevents a match from joining distinct comment tokens.
    """
    code, comments, syntax = [], [], []
    position = 0
    while token := TOKEN.search(source, position):
        between = source[position:token.start()]
        code.append(between)
        syntax.append(between)
        end = token.end()
        is_comment = token.group().startswith(('//', '/*'))
        if token.group() == '/*':
            depth = 1
            while depth and (marker := BLOCK.search(source, end)):
                depth += 1 if marker.group() == '/*' else -1
                end = marker.end()
            if depth:
                end = len(source)
        elif token.group('hashes') is not None:
            closing = '"' + token.group('hashes')
            close = source.find(closing, end)
            end = len(source) if close == -1 else close + len(closing)
        text = source[token.start():end]
        if is_comment:
            code.append(' ')
            comments.append(text + '\n')
        else:
            code.append(text if token.group('hashes') is not None else ESCAPE.sub(' ', text))
        syntax.append(re.sub(r'[^\n]', ' ', text))
        position = end
    code.append(source[position:])
    syntax.append(source[position:])
    return ''.join(code), ''.join(comments), ''.join(syntax)


def without_comments(source):
    return split_source(source)[0]


QUALIFIER = re.compile(r'(\b\w+(?:\s*::\s*\w+)*)\s*::\s*$')
QUALIFIED_IDENTIFIER = re.compile(r'\b\w+(?:\s*::\s*\w+)*\b')


def exempt(config, form, match, text):
    names = config.get('EXEMPT', {}).get(form, ())
    if not isinstance(names, dict):
        return match.group() in names
    qualifiers = names.get(match.group(), ())
    qualifier = QUALIFIER.search(text, 0, match.start()) if qualifiers else None
    return qualifier is not None and qualifier.group(1) in qualifiers


def all_forms(config):
    return [*config['FORMS'], *('comment:' + name for name in config.get('COMMENT_FORMS', {}))]


def count_text(config, forms, text, name, tree, repository, *, comments=False):
    totals = {}
    words = None
    for form, pattern in forms.items():
        key = 'comment:' + form if comments else form
        totals[key] = 0
        if name.startswith(config.get('EXCLUDE', {}).get(key, ())):
            continue
        include = config.get('INCLUDE', {}).get(key)
        if include is not None and not include.search(name):
            continue
        markers = config.get('MARKERS', {}).get(key, ())
        if markers and not any(marker in text for marker in markers):
            continue
        candidate = text
        if key in config.get('WORDS', ()):
            if words is None:
                # Preserve qualified names for jig and temper's exemptions.
                words = QUALIFIED_IDENTIFIER.findall(text)
            candidate = ' '.join(word for word in words if not markers or any(marker in word for marker in markers))
        if callable(pattern):
            totals[key] = pattern(candidate, name, tree, repository)
        else:
            totals[key] = sum(1 for match in pattern.finditer(candidate)
                              if not exempt(config, key, match, candidate))
    return totals


def counts(config):
    tree = Path.cwd()
    repository = Path(subprocess.run(
        ['git', 'rev-parse', '--show-toplevel'], check=True, capture_output=True,
        text=True, stdin=subprocess.DEVNULL,
    ).stdout.strip())
    paths = subprocess.run(
        ['git', 'ls-files', '-z', '*.rs'], check=True, capture_output=True,
        stdin=subprocess.DEVNULL,
    ).stdout.split(b'\0')
    totals = dict.fromkeys(all_forms(config), 0)
    syntax_forms = {key: value for key, value in config['FORMS'].items()
                    if key in config.get('SYNTAX_FORMS', ())}
    code_forms = {key: value for key, value in config['FORMS'].items() if key not in syntax_forms}
    for path in paths:
        parts = path.split(b'/')
        if not path or b'generated' in parts or parts[-2:] == [b'patches', b'mod.rs']:
            continue
        name = path.decode('utf-8')
        if name.startswith(config.get('EXEMPT_DIRECTORIES', ())):
            continue
        code, comments, syntax = split_source(Path(name).read_text(encoding='utf-8'))
        for forms, text, is_comment in ((code_forms, code, False), (syntax_forms, syntax, False),
                                        (config.get('COMMENT_FORMS', {}), comments, True)):
            for form, count in count_text(config, forms, text, name, tree, repository,
                                          comments=is_comment).items():
                totals[form] += count
    return totals


RUST_TOKEN = re.compile(r'\b\w+\b|::|\.\.|=>|->|[^\s]')


def full_limits(source, name, tree, repository):
    """Count complete Limits literals, with no top-level struct update.

    Braces inside field expressions do not end the literal; nested ranges
    and nested struct updates do not count as an update of the outer Limits.
    Declarations, impl bodies, function bodies and destructuring are skipped.
    A `file_layer::Limits` is skipped: skein/docs/design/runtime/io/files.md, has each reader
    write its bounds out and keep them, so that literal is the new form.
    """
    tokens = RUST_TOKEN.findall(source)
    count = 0
    for index, token in enumerate(tokens[:-1]):
        if token != 'Limits':
            continue
        brace = index + 1
        if tokens[brace:brace + 2] == ['::', '<']:
            angle, brace = 1, brace + 2
            while brace < len(tokens) and angle:
                angle += (tokens[brace] == '<') - (tokens[brace] == '>')
                brace += 1
        if brace >= len(tokens) or tokens[brace] != '{':
            continue
        before = tokens[index - 1] if index else ''
        if before in ('struct', 'enum', 'impl', 'for', '->'):
            continue
        # A qualified function return type ends in ::Limits as well.
        start = index
        while start >= 2 and tokens[start - 1] == '::':
            start -= 2
        if start and tokens[start - 1] in ('->', 'impl', 'for'):
            continue
        if tokens[index - 2:index] == ['file_layer', '::']:
            continue
        depth, update, end = 1, False, brace + 1
        while end < len(tokens) and depth:
            value = tokens[end]
            if value in ('{', '(', '['):
                depth += 1
            elif value in ('}', ')', ']'):
                depth -= 1
            elif value == '..' and depth == 1 and tokens[end - 1] in ('{', ','):
                update = True
            end += 1
        after = tokens[end] if end < len(tokens) else ''
        if not depth and not update and after not in ('=', '=>'):
            count += 1
    return count


MD_PATH = re.compile(r'(?<![\w/])(?:\.{1,2}/)?(?:[\w.-]+/)*[\w.-]+\.md\b')


@lru_cache(maxsize=16)
def markdown_paths(repository):
    """Short citations may name a tracked document in any tree."""
    result = subprocess.run(['git', 'ls-files', '-z', '*.md'], cwd=repository,
                            capture_output=True, stdin=subprocess.DEVNULL, check=True)
    paths = {}
    for path in result.stdout.split(b'\0'):
        if path:
            relative = Path(path.decode('utf-8'))
            paths.setdefault(relative.name, []).append(repository / relative)
    return paths


def dead_citations(source, name, tree, repository):
    """Count missing Markdown path occurrences in comments.

    Repository paths, tree-relative paths, source-relative paths and short
    design-document names are accepted. Bare names also resolve against tracked
    Markdown documents across all trees. Web URLs are not repository citations.
    """
    count = 0
    for match in MD_PATH.finditer(source):
        # A URL's host/path may contain an md suffix; it is not a local path.
        prefix = source[max(0, source.rfind(' ', 0, match.start()) + 1):match.start()]
        if '://' in prefix or prefix.endswith('/'):
            continue
        path = match.group()
        candidates = (repository / path, tree / path, tree / Path(name).parent / path,
                      tree / 'docs/design' / path)
        exists = False
        for candidate in candidates:
            resolved = candidate.resolve()
            if resolved.is_relative_to(repository.resolve()) and resolved.is_file():
                exists = True
                break
        if not exists and '/' not in path:
            exists = any(candidate.is_file() and candidate.resolve().is_relative_to(repository.resolve())
                         for candidate in markdown_paths(repository).get(path, ()))
        count += not exists
    return count


def banned_word(word):
    """Count a word or its -ly form in prose and snake/camel-case names."""
    parts = re.compile(r'[A-Z]+(?=[A-Z][a-z]|[^a-zA-Z]|$)|[A-Z]?[a-z]+|[A-Z]+')

    def count(source, name, tree, repository):
        return sum(part.lower() in (word, word + 'ly') for part in parts.findall(source))

    return count


TEST_PATH = re.compile(r'(?:^|/)(?:tests|benchmarks)/|(?:^|/)(?:tests|[^/]+_tests)\.rs$')
TEST_ATTRIBUTE = re.compile(r'#\s*\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]')


def test_regions(source, name):
    """Find test files and #[cfg(test)] / #[test] item bodies in syntax text."""
    if TEST_PATH.search(name):
        return [(0, len(source))]
    regions = []
    for attribute in TEST_ATTRIBUTE.finditer(source):
        start = source.find('{', attribute.end())
        semicolon = source.find(';', attribute.end())
        if semicolon != -1 and (start == -1 or semicolon < start):
            # An external test module ends here; the following item is separate.
            end = semicolon + 1
        elif start == -1:
            continue
        else:
            depth, end = 1, start + 1
            while depth and end < len(source):
                depth += (source[end] == '{') - (source[end] == '}')
                end += 1
        if not regions or attribute.start() >= regions[-1][1]:
            regions.append((attribute.start(), end))
    return regions


def test_windows(source, name, tree, repository):
    return sum(len(re.findall(r'\.\s*windows\s*\(', source[start:end]))
               for start, end in test_regions(source, name))


def domain_keyword(keyword):
    """Domains use the production subset; their test harnesses are ordinary Rust."""
    pattern = re.compile(r'\b' + keyword + r'\b')

    def count(source, name, tree, repository):
        pieces, position = [], 0
        for start, end in test_regions(source, name):
            pieces.append(source[position:start])
            position = end
        pieces.append(source[position:])
        return len(pattern.findall(' '.join(pieces)))

    return count


def main(config, argv=None):
    parser = argparse.ArgumentParser(description=config.get("description", __doc__))
    parser.add_argument('--write', action='store_true', help='initialize or lower the baseline')
    args = parser.parse_args(argv)
    baseline_path = config["BASELINE"]
    header = config["HEADER"]
    rule = config["RULE"]
    forms = all_forms(config)
    current = counts(config)
    try:
        lines = baseline_path.read_text(encoding='utf-8').splitlines(keepends=True)
    except FileNotFoundError:
        if not args.write:
            parser.error(f'{baseline_path} is missing; initialize with --write')
        baseline = current
        lines = header.splitlines(keepends=True) + [
            f'{form} {count}\n' for form, count in current.items()
        ]
    else:
        try:
            entries = [line.split() for line in lines if line.strip() and not line.startswith('#')]
            baseline = {form: int(count) for form, count in entries}
            if (list(baseline) != list(forms)[:len(entries)] or len(entries) != len(baseline)
                    or any(count < 0 for count in baseline.values())):
                raise ValueError('expected each form once, in order, with nonnegative counts')
        except ValueError as error:
            parser.error(f'invalid {baseline_path}: {error}')
        # Forms added since the baseline was written: their rows start at
        # today's counts.
        missing = list(forms)[len(baseline):]
        if missing and not args.write:
            parser.error(f'{baseline_path} records no {", ".join(missing)}; initialize with --write')
        if missing and lines and not lines[-1].endswith('\n'):
            lines[-1] += '\n'
        for form in missing:
            baseline[form] = current[form]
            lines.append(f'{form} {current[form]}\n')
    raised = False
    for form, count in current.items():
        if count > baseline[form]:
            print(f'{form}: count {count}, baseline {baseline[form]}', file=sys.stderr)
            raised = True
    if raised:
        print(rule, file=sys.stderr)
        return 1
    if args.write:
        for index, line in enumerate(lines):
            if line.strip() and not line.startswith('#'):
                form = line.split()[0]
                lines[index] = f'{form} {current[form]}\n'
        baseline_path.write_text(''.join(lines), encoding='utf-8')
    else:
        for form, count in current.items():
            if count < baseline[form]:
                print(f'{form}: count {count}, baseline {baseline[form]}; lower with '
                      '`python3 -I -B scripts/old_forms.py --write`', file=sys.stderr)
    return 0
