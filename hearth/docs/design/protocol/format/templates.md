# The format's templates

## 1. Where templates are

- **Files:** `*.j2` under `tree/` and `templates/`, `customize.sh.j2` and
  `software.sh.j2`.
- **Declarations:** in `task.json`, the `if` and `for_each` fields are
  expressions, and keys and string fields are interpolated (`tasks.md`, 2).
- **Nowhere else.** Configuration documents are data (`README.md`, 2.2).

## 2. Delimiters and whitespace

- `{{ … }}` outputs an expression; `{% … %}` is a statement; `{# … #}` is
  a comment.
- **`trim_blocks` and `lstrip_blocks` are on,** for statement and
  comment tags alike, never for `{{ … }}`. The first line break after
  such a tag, one `\n`, `\r\n` or `\r`, is removed; the whitespace
  before one at the start of a line is removed, which is every Unicode
  White_Space character but `\r` and `\n`: a space, a tab, U+000B,
  U+000C and U+00A0 among them. A statement or a comment alone on its
  line leaves nothing behind.
- **Markers:** `-` inside a delimiter (`{%-`, `-%}`, `{{-`, `-}}`, `{#-`,
  `-#}`) strips all Unicode White_Space, newlines included, on that
  side. `+%}` and `+#}` keep the line break after a tag that
  `trim_blocks` would remove; `{%+` and `{#+` keep the indentation that
  `lstrip_blocks` would remove. `{{+` and `+}}` are errors: an output
  tag has nothing for them to keep.
- **Trailing newlines are kept as written.** A file that ends in a
  newline renders ending in one (os stripped one, so `/etc/hostname`
  shipped without it).
- These are MiniJinja 2's semantics with `trim_blocks`, `lstrip_blocks`
  and `keep_trailing_newline` on, and render's tests hold hearth's engine
  to MiniJinja on generated templates (hearth/docs/design/domain/render/README.md, 6.5).
- **`trim_blocks` eats the newline after a statement at the end of a
  line too,** which os's `grub.cfg` met: write `+%}` there.

  ```jinja
  mynetworks = 127.0.0.0/8 [::1]/128{% for name, peer in exports.mail | items %}{% if peer.role == 'backup' %} {{ peer.address }}/32{% endif %}{% endfor +%}
  ```

## 3. Statements

| Statement | Form | Notes |
|---|---|---|
| condition | `{% if e %}` … `{% elif e %}` … `{% else %}` … `{% endif %}` | |
| loop | `{% for x in e %}` … `{% endfor %}` | over a list, or a map's keys in order |
| loop with pairs | `{% for k, v in e \| items %}` | two names only |
| assignment | `{% set x = e %}` | inside a `for`, lasts for one iteration; else to the end of the template |
| raw | `{% raw %}` … `{% endraw %}` | the text between is output as written, but for its tags' whitespace rules (2): the line break after `{% raw %}` goes, and the indentation before an `{% endraw %}` that starts its line |

- **Inside a `for`, `loop`** holds `index` (from 1), `index0`, `first`,
  `last` and `length`. It is read by those five keys alone: another key
  is an error, and so is `loop` used whole, but in `loop is defined` and
  as a condition, where it is true. Outside a `for`, `loop` is an
  unknown name (6).
- **Names may not shadow the context's roots** (7): `{% set settings = … %}`
  is an error. os's `for-each` element shadowed any task of the same name.
  The eight roots, `key`, `value` and `loop` among them, are refused as a
  `set` or `for` name in every template, bound there or not, when the
  template is read.
- **What follows `in`** is an expression without a conditional at its
  top, as in MiniJinja: `{% for x in (a if c else b) %}` loops over a
  conditional, and `{% for x in a if c %}` is the refused `for … if`. A
  pair loop takes any list whose elements are lists of exactly two, as
  `items` gives; another element is an evaluation error at the `for`.
- **Nothing else:** no `macro` or `call`, `include`, `import` or `from`,
  `extends` or `block`, `filter`, `with`, `do`, `break` or `continue`,
  block `set`, `for … else`, `for … if`, `recursive`, `autoescape`.

## 4. Expressions

```
expression = or [ "if" or "else" expression ]
or         = and { "or" and }
and        = not { "and" not }
not        = "not" not | test
test       = compare [ "is" [ "not" ] "defined" ]
compare    = concat [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" | "not" "in" ) concat ]
concat     = sum { "~" sum }
sum        = filtered { ( "+" | "-" ) filtered }
filtered   = postfix { "|" name [ "(" [ expression { "," expression } ] ")" ] }
postfix    = primary { "." name | "[" expression "]" }
primary    = name | string | integer | "true" | "false" | list | "(" expression ")"
list       = "[" [ expression { "," expression } ] "]"
name       = [a-zA-Z_] [a-zA-Z0-9_]*
string     = '…' or "…", with the escapes \\ \' \" \n \t
integer    = [ "-" ] [0-9]+
```

- **Between tokens** stand spaces, tabs and line feeds only: any other
  byte outside a string literal is an error, a `\r` that a declaration's
  JSON string holds included. A string literal keeps the bytes between
  its quotes as written, its escapes read.

- **Grammar words are not names:** `and`, `or`, `not`, `if`, `else`,
  `is`, `in`, `true` and `false`, and `none`, `None`, `True` and
  `False`, which MiniJinja reads as constants, are refused where a name
  stands as an operand, and so as a `set` or `for` name. After `.` each
  is a key like any other: `settings.if`. `defined` is a name but after
  `is`.
- **Lookups:** `a.b` and `a['b']` find a map's key; `a[i]` a list's
  element. There are no methods, so `u.keys` is always the key `keys`
  (the trap os's survey found, where Jinja calls a method).
- **A lookup of the wrong type is an error,** in a chain too (6): a
  key into a list, an integer into a map, anything into a string, an
  integer or a boolean. A negative index is outside the list, where
  MiniJinja counts from the end. `in` takes a string to find in a map or
  a string; another value there is an error.
- **Values** are strings, integers, booleans, lists and maps, as the
  schema types them.
- **`and` and `or`** short-circuit and give an operand, as in Jinja.
  **Truthiness:** `false`, `0`, `""`, `[]`, `{}` and undefined are false.
- **`==` and `!=`** compare values of one type deeply; values of two types
  are unequal. **`<`, `<=`, `>`, `>=`** take two integers or two strings,
  compared by bytes. **`in`** finds a key in a map, an element in a list,
  a substring in a string.
- **`+` and `-`** take integers, checked: an overflow is an error. **`~`**
  joins strings and integers, written in decimal.
- **Output:** a string as it is, an integer in decimal, a boolean as
  `true` or `false`. A list or a map is an error: use `join` or `tojson`.
- **No calls:** no functions, no `range`, no methods, no slices, no map
  literals. Data belongs in settings.

## 5. Filters and tests

| Filter | Takes | Gives |
|---|---|---|
| `default(v)` | anything, or undefined | `v` where undefined, else the value |
| `join(sep)` | a list of strings or integers | a string |
| `split(sep)` | a string; `sep` not empty | a list of strings |
| `first`, `last` | a list that is not empty | an element |
| `length` | a string, list or map | its characters, elements or entries |
| `replace(a, b)` | a string; `a` not empty | every `a` replaced by `b` |
| `items` | a map | its `[key, value]` pairs, in key order |
| `keys`, `values` | a map | a list, in key order |
| `sort` | a list of strings or of integers | ascending: strings by bytes, as `<` compares them, integers by value; a list of both is an error |
| `tojson` | any value | compact JSON: keys in order; `"` and `\` as `\"` and `\\`; U+0008, U+000C, U+000A, U+000D and U+0009 as `\b`, `\f`, `\n`, `\r` and `\t`; the rest of U+0000 to U+001F as `\u00xx`, in lowercase hex; everything else, `/` and U+007F included, as it is, in UTF-8 |
| `shell_quote` | a string | single-quoted for sh, each `'` written `'\''` |
| `systemd_escape` | a string | as `systemd-escape`: `/` to `-`, every byte outside `[A-Za-z0-9:_.]` and a leading `.` to `\xNN` |
| `systemd_escape_path` | a path (`README.md`, 5.2) | as `systemd-escape --path`: slashes collapsed and trimmed, then escaped; `/` alone is `-` |
| `basename`, `dirname` | a path (`README.md`, 5.2), or a relative path of a `file`'s form | its last segment; the rest. Of `/`, both are `/`; `/a`'s `dirname` is `/`, and a one-segment relative path's is `.`, as coreutils gives them |

- **A value a filter does not take is an evaluation error:** an empty
  string, `a/` or `/a/../b` to `basename` or `dirname`, a relative path
  to `systemd_escape_path`.
- **The only test is `defined`:** `x is defined`, `x is not defined`.
- `tojson` is for JSON files (the bash fleet escaped its agents'
  `settings.json` with `sed`). `systemd_escape_path` turns
  `/dev/nvme0n1p3` into `dev-nvme0n1p3`, for unit names the bash fleet
  built with `cut -b 2- | tr / -`. `shell_quote` is for settings that go
  into `customize.sh.j2`.

## 6. Undefined

- **A lookup that finds nothing is an error,** with the template's file
  and line: an unknown name, a missing key, an index past a list's end.
- **Undefined exists in four places only:** an optional setting or object
  field that is absent; an optional read's task that is absent; an
  optional export field that is absent; `target.member` outside a
  cluster.
- **An absent optional value is no key of its map:** only a lookup of it
  gives undefined. `keys`, `values`, `items`, `length`, `tojson`, `in`,
  `==` and truthiness see the map without it.
- **Undefined may be tested, defaulted or used as a condition:** with
  `is defined`, `default(…)`, or as the condition of `if`, `elif`, `not`,
  `and`, `or` and `… if … else …`, where it is false. Printing it,
  looping over it, comparing it, looking into it or filtering it
  otherwise is an error.
- **Nor does it go further:** undefined as a list's element, as a
  filter's argument but `default`'s, as a `set` value, or as a branch of
  `… if … else …` is an error. `and` and `or` give an operand, which may
  be undefined: `settings.a or settings.b` is undefined where both are.
- **A lookup chain that goes straight into `is defined` or `default`**
  gives undefined at its first missing link instead of an error:
  `peer.endpoint is defined`.
- **A chain is a bound name and the `.name` and `[e]` lookups after
  it,** written straight before `is defined` or `| default`: never in
  parentheses, `(peer.endpoint) is defined`, and never from a literal,
  `[1][5] is defined`. The name is no link: an unknown name is refused
  when the template is read, and a `set` name read where no `set` has
  bound it is an error, before `is defined` too. A lookup that finds
  nothing makes the chain undefined; one of the wrong type (4) is
  still an error.
- So os's two rules (an undefined `a.b` printed empty in a file, and was
  false in a condition, while `a.b` on a missing `a` failed in a file)
  become one, and an agent's typo fails.

## 7. The context

| Name | Holds | Example |
|---|---|---|
| `target.name`, `.base`, `.environment` | the target | `x1-prod`, `x1`, `prod` |
| `target.member` | the member's name; undefined outside a cluster | `1` |
| `target.placement` | `kind`, `firmware`, `hardware`, `on` (`README.md`, 3.4) | |
| `target.containers` | the targets placed on this one: name to placement | `{"chat-prod": {…}}` on `east-prod` |
| `debian` | the sources (`tasks.md`, 3) | `debian.suite` |
| `settings` | the task's own settings, defaults filled | `settings.port` |
| `tasks.<task>.<setting>` | declared reads (`README.md`, 5.4) | `tasks.core.domain` |
| `exports.<task>` | declared export reads: target name to exports (`README.md`, 6.2) | `exports.vpn` |
| `key`, `value` | the element of a `for_each` entry, in its fields and its template; unknown elsewhere, in a `tree/` template too | |
| `loop` and assigned names | inside the template | |

- **Nothing else:** no process environment, no clock, no random, no
  other task's files, no secret values, no global functions. The context
  is a function of the revision and the target, which is what lets
  render be one (hearth/docs/design/domain/render/README.md, section 1).

## 8. From esh and from MiniJinja

The subset is what the two surveys found in use, less what schemas and
typed values make unneeded:

| The bash fleet's esh, or os's MiniJinja | hearth |
|---|---|
| `$(config_get Ssh Port)` | `{{ settings.port }}`, or a declared read |
| `if`/`elif`/`else`, `[ a = b ]`, `-n`, `-z` | `if`/`elif`/`else`, `==`, truthiness |
| `for x in $(…)`, `while read` | `for` over a list or a map |
| `cut -f N -d :` on `a:b:c` strings (33 sites) | gone: values are objects |
| `tail -n 1`, last value wins | gone: merging decides |
| `${v:-default}` | a schema default; `default` for an optional setting |
| `config_get --sections \| grep -q Wifi` | `tasks.wifi is defined`, an optional read |
| `s="$s $x"` | `join(' ')` |
| `case … esac` | `if`/`elif`, or a map setting indexed by the value |
| shell functions (5), the open question of macros | gone: defaults, objects and maps remove all five |
| `sed` as a JSON escape | `tojson` |
| `cut -b 2- \| tr / -` on a device path | `systemd_escape_path` |
| `dirname` | `dirname`, `basename` |
| `log_die`, os's `require` | types, and `requires` (`tasks.md`, 9) |
| `$TARGET_KIND`, the `container` task | `target.placement.kind` |
| `systemd-detect-virt`, `awk` on another task's file | impossible: no calls, no files |
| os's `for-each` dotted path and `as` | `for_each`, an expression, binding `key` and `value` |
| os's inline `set`, `raw`, `-%}`, `+%}` | kept |
| os's slice `[1:]`, `map(attribute=…)` | gone: `systemd_escape_path`, `keys` |
| os's Lenient and Chainable undefined | one rule (6) |
| os's stripped final newline | kept as written |
