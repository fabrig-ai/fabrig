# Codecs, translation, goldens and versions: a survey

2026-11-17, read-only, on main at `a525e539f7`. The facts behind the drafts `temper/docs/design/draft/codec-shape.md` and `temper/docs/design/draft/layouts-both-ways.md`: what the codecs generate, the translation written around them, the golden files and who reads them, the versions and conversions of the store and each wire, and where concurrent changes collide. No build, test or regeneration was run.

Aliases (crate roots, append /path): JR = J/protocol-store-records; JE = J/protocol-engine-clients; JL = J/client-protocol-link; JW = J/protocol-worker-wire; EW = J/protocol-engine-workers; WL = J/protocol-worker-link; TS = T/protocol-store; TC = T/client-protocol-wire; TW = T/protocol-worker-wire; MX = M/protocol-translate.

Counts include comments and blank lines, and tracked files only. No file beneath a `secrets/` directory was opened. Such golden files are counted from Git tree metadata only. Source files describing secret types are schema/code, not credential values; no key or credential file was opened.

Path abbreviations throughout: `S/` = `skein/crates/skein-`, `M/` = `smith/crates/smith-`, `J/` = `jig/crates/jig-`, `T/` = `temper/crates/temper-`. All other paths start at the repository root. `CG` = `skein/testing/skein-codegen/src/`.

## 1. Codec inventory

Schemas and generated Rust are often in different crates. Registered map families have no kept-text/patch history here; wire families still do. Counts below preserve that ownership split.

Counting commands (C is the row's full crate path; filters apply to the tracked path list):

```sh
# S: current schema text, including split parts; excludes schema/kept.
git ls-files "$C/schema/*.schema" | rg -v '/kept/' | xargs -r wc -l
# G and T: literal src/generated and tests/generated Rust, respectively.
git ls-files "$C/src/generated/*.rs" | xargs -r wc -l
git ls-files "$C/tests/generated/*.rs" | xargs -r wc -l
# B: golden files/bytes, including nested and patches/golden directories.
git ls-files "$C" | rg '/golden/' | wc -l
git ls-tree -rl HEAD "$C"  # sum the size column for those same paths
# P and K: patch texts / kept schema texts.
git ls-files "$C/patches/[0-9]*.patch" | wc -l
git ls-files "$C/schema/kept/*" | wc -l
# D: unique direct dependent workspace manifests, all dependency kinds/targets.
rg -l '^CRATE[ =]' WORKSPACE_CARGO_TOMLS | wc -l
# Family/version and registration:
rg -n '^family ' "$C/schema" -g '*.schema' -g '!kept/**'
cat "$C/schema/registered"
```

B bytes sum Git blob sizes without opening golden contents. D uses explicit workspace members and dependency aliases, all dependency kinds/targets, cross-checked by parsing manifests; no Cargo. `→` links a codec-only crate to its schema owner; R=registered, all others unregistered. Store family prefix is jig-store-. Cursor is an opaque client continuation. Zero=no matching paths.

|ID|Crate|Families @ version / registration|Protocol|S|G|T|B files/bytes|P/K|D|
|---|---|---|---|---:|---:|---:|---:|---:|---:|
|C1|J/client-protocol-wire|jig-client-wire@30, jig-public-wire@1|client/public wire|1125|135843|4940|4348/153859|0/0|56|
|C2|J/client-view|jig-client-view-records@1|view|12|0|0|1688/2020757|0/0|11|
|C3|J/client-view-codec|→ J/client-view|view|0|1313|142|0/0|0/0|1|
|C4|J/protocol-accounts|jig-accounts@1|store|10|0|0|2/70|0/1|11|
|C5|J/protocol-accounts-codec|→ J/protocol-accounts|store|0|302|88|0/0|0/0|1|
|C6|JE|jig-engine-cursor@2|cursor|43|0|0|150/1807|0/0|23|
|C7|J/protocol-engine-clients-codec|→ JE|cursor|0|4390|236|0/0|0/0|1|
|C8|J/protocol-store|jig-store-accounts@2 R, jig-store-component@3, jig-store-core@6 R, jig-store-notes@1 R, jig-store-openings@1, jig-store-people@17 R, jig-store-registry@1, jig-store-secrets@1, jig-store-settings@3 R, jig-store-tasks@13 R|store|1252|16|0|3979/67957|0/0|33|
|C9|J/protocol-store-core|→ J/protocol-store|store|0|14764|662|0/0|0/0|2|
|C10|J/protocol-store-families|→ J/protocol-store|store|0|9761|713|0/0|0/0|2|
|C11|J/protocol-store-people|→ J/protocol-store|store|0|41029|1702|0/0|0/0|2|
|C12|J/protocol-store-settings|→ J/protocol-store|store|0|14445|471|0/0|0/0|2|
|C13|J/protocol-store-tasks|→ J/protocol-store|store|0|42813|1560|0/0|0/0|2|
|C14|JW|jig-worker-wire@16|worker wire|248|16931|729|5554/62318|6/7|26|
|C15|jig/examples/ops/client-protocol-wire|jig-ops-client-wire@1|client/public wire|137|16284|797|446/12232|0/0|3|
|C16|jig/testing/jig-client-test-protocol-wire|jig-test-client-wire@2|client/public wire|107|0|0|526/5928|0/0|8|
|C17|jig/testing/jig-client-test-protocol-wire-codec|→ jig/testing/jig-client-test-protocol-wire|client/public wire|0|12341|616|0/0|0/0|1|
|C18|jig/testing/jig-test-patches-jig|fixture-frame@6, fixture-note@1, fixture-note@2|reference/fixtures|6|0|0|4/108|2/4|2|
|C19|jig/testing/jig-test-patches-smith|fixture-turn@2|reference/fixtures|2|0|0|2/27|1/2|3|
|C20|jig/testing/jig-test-store|test-connector@2|store|77|0|0|432/4047|0/0|8|
|C21|jig/testing/jig-test-store-codec|→ jig/testing/jig-test-store|store|0|9191|522|0/0|0/0|1|
|C22|S/oauth|skein-oauth@3|store|15|0|0|2/119|0/1|29|
|C23|S/oauth-codec|→ S/oauth|store|0|459|89|0/0|0/0|1|
|C24|skein/examples/ui/view|notes@1|view|23|0|0|15/1284|0/0|3|
|C25|skein/examples/ui/view-codec|→ skein/examples/ui/view|view|0|679|99|0/0|0/0|1|
|C26|skein/tests/codec|every@1, every@2, lint-fixture@1, sample@1, scalars@1, secrets@1|reference/fixtures|328|676|711|138/3696|1/6|0|
|C27|skein/tests/codec-patches|label@1, label@2, shelf@1, shelf@2, shelf@3|reference/fixtures|16|0|0|26/131|3/5|0|
|C28|M/channel|smith-channel@10, smith-pieces@1|smith channel|225|12763|634|4040/24922|3/5|31|
|C29|M/charter|smith-charter@5|agent|83|3555|165|216/5444|1/2|33|
|C30|M/local-protocol|smith-local-store@1|store|20|5|0|63/778|0/0|6|
|C31|M/local-protocol-codec|→ M/local-protocol|store|0|2275|153|0/0|0/0|1|
|C32|M/transcript|smith-transcript@7|agent|113|8230|399|568/4503|0/1|45|
|C33|smith/tests/transcript-patches|fixture-turn@2|reference/fixtures|2|0|0|2/23|1/2|1|
|C34|TC|temper-client-wire@34, temper-public-wire@3|client/public wire|582|64218|2688|7892/383554|0/0|37|
|C35|T/protocol-records|temper-secrets@1|store|48|1988|168|78/538|0/1|9|
|C36|TS|temper-store@25 R|store|627|0|0|2222/49644|0/0|13|
|C37|T/protocol-store-codec|→ TS|store|0|73745|2686|0/0|0/0|1|
|C38|TW|temper-worker-wire@10|worker wire|57|4880|269|1430/21471|5/6|13|
`jig-protocol-store-families` generates accounts/component/notes/openings/registry/secrets; remaining store crate suffixes identify their family. Non-codec schema directories: S/codec/src/schema (parser, 3,312 lines), hearth/crates/hearth-domain-render/src/schema (JSON validation, 704), hearth/tests/render/errors/schema (fixtures); no codec family. Counts: `git ls-files DIR | xargs wc -l`.

S excludes kept duplicates; K counts them. G is literal src/generated only. Skein reference generated `src/every/{v1,v2}.rs`, `src/{scalars,lints,secrets}/v1.rs` add 7,466 lines outside G (`wc -l` on these files). Smith local's old generated mod.rs adds 5 G lines.

Totals by protocol, summing the section 1 commands over the assigned crates (not counting dependencies multiple times as source):

|Protocol|S|G|T|B files/bytes|P/K|
|---|---:|---:|---:|---:|---:|
|client/public wire|1951|228686|9041|13212/555573|0/0|
|view|35|1992|241|1703/2022041|0/0|
|store|2049|210793|8814|6778/123153|0/3|
|cursor|43|4390|236|150/1807|0/0|
|worker wire|305|21811|998|6984/83789|11/13|
|reference/fixtures|354|676|711|172/3985|8/19|
|smith channel|225|12763|634|4040/24922|3/5|
|agent|196|11785|564|784/9947|1/3|
|**All**|5158|492896|21239|33823/2825217|23/43|

Golden directories: the section 1 B cell is also the count/bytes for `C/golden` where that is the crate's only directory. Other/combined directories below use `git ls-files DIR | wc -l` and summed sizes from `git ls-tree -rl HEAD DIR`; recursive, disjoint. Non-codec directories are included.

- `hearth/testing/hearth-test-render/src/golden` 1/3935; `hearth/tests/render/fleet/rendered/carbon-lab/etc/golden` 2/8; `hearth/tests/render/fleet/rendered/carbon-prod/etc/golden` 2/8; `hearth/tests/render/fleet/rendered/chat-prod/etc/golden` 2/8.
- `hearth/tests/render/fleet/rendered/east-prod/etc/golden` 2/8; `hearth/tests/render/fleet/rendered/west-prod/etc/golden` 2/8; `hearth/tests/render/fleet/rendered/x1-prod/etc/golden` 2/8; `hearth/tests/render/fleet/tasks/core/tree/etc/golden` 3/15.
- `C2/golden` 1688/2020757; `C8/golden` 3978/67913; `C8/src/tests/golden` 1/44; `C14/golden` 554/6659.
- `C14/patches/golden` 5000/55659; `C18/patches/golden` 4/108; `C19/patches/golden` 2/27; `jig/tests/client/tests/golden` 9/24771.
- `jig/tests/store/golden` 2/400482; `S/ui-bundle/src/tests/golden` 2/934; `S/ui-screen/src/tests/golden` 18/33526; `S/ui-text/src/tests/golden` 9/4906.
- `S/ui-web/src/tests/golden` 21/16002; `S/ui/src/tests/golden` 1/317; `C24/golden` 12/166; `C24/src/tests/golden` 3/1118.
- `C27/patches/golden` 22/105; `C27/story/06-wire/golden` 4/26; `C26/golden` 112/2920; `C26/patches/golden` 26/776.
- `skein/tests/kv/golden` 4/501; `skein/tests/ui-web/golden` 2/2794; `C28/golden` 2526/15571; `C28/patches/golden` 1514/9351.
- `C29/golden` 108/2910; `C29/patches/golden` 108/2534; `M/events/golden` 233/122243; `C33/patches/golden` 2/23.
- `T/client-shell-terminal/src/tests/golden` 23/21563; `T/client-tools/src/tests/golden` 177/662072; `T/client-tui/src/tests/golden` 78/264513; `C38/golden` 150/2245.
- `C38/patches/golden` 1280/19226; `T/stage-one-tool/golden` 2/49769; `temper/tests/one-process/golden` 133/740374; `temper/tests/one-process/src/golden` 1/10463.

Within `J/protocol-store/golden`, the family directories contain files/bytes: accounts 94/937; component 30/1,308; core 560/5,495; notes 44/741; openings 6/173; people 1,512/16,077; registry 6/227; secrets 28/258; settings 336/11,872; tasks 1,362/30,825. Same two DIR commands, with the family appended. The secrets figure uses Git metadata only.

## 2. Generated lines by emitter and item

All requested files are current. Helpers leave type APIs and metadata; estimates are conditional and unmeasured.

Emitters: `CG/lib.rs:190` orders limits/types/codec/bounds/tests; `emit_types.rs:145,236`, `emit_codec.rs:19,70,115,202,308`, `emit_bounds.rs:8,30,50,73,99`. `emit_functions.rs:9,63` splits long functions; split helpers count with parent kind.

A=`J/protocol-store-tasks/src/generated/tasks_v13.rs`; B=`T/protocol-store-codec/src/generated/v25.rs`; C=`J/client-protocol-wire/src/generated/v30.rs`. Each cell: `wc -l` on selected item spans, found in memory from fn/struct/enum signature through balanced closing brace, ignoring string braces, including preceding docs/attributes. Function regex `^\s*(?:pub(?:\(crate\))? )?(?:const )?fn (\w+)`; types `^pub (struct|enum) (\w+)`; method names select categories. Residual includes all unassigned lines.

|Kind (emitter)|A|B|C|Estimated remaining fraction; assumption|
|---|---:|---:|---:|---|
|Record/enum definitions (`emit_types`)|2,919|4,898|9,007|100%; keep concrete public types|
|`*Parts` structs (`emit_types`)|2,554|4,260|7,726|100%; keep the movable-field API|
|Constructors (`emit_types`)|1,975|3,239|6,296|100%; same construction API|
|Accessors (`emit_types`)|2,259|4,014|6,684|100%; same public accessors|
|`into_parts` (`emit_types`)|786|1,203|2,487|100%; same ownership API|
|Value validation/check helpers (`emit_types`)|753|1,817|2,874|75–100%; generic container guards only|
|Measure, including splits (`emit_codec`)|3,758|6,754|11,986|50–75%; replace expanded list/option walks by calls|
|Encode, including splits (`emit_codec`)|4,174|7,757|13,216|50–75%; same, retaining field order/tag dispatch|
|Public decode (`emit_codec`)|1,992|3,240|6,300|25–50%; common limits/tail wrapper in trait default|
|`decode_from`, including splits (`emit_codec`)|2,458|4,226|7,565|50–75%; generic list/option readers, typed fields retained|
|Encoded/decoded bounds and arithmetic (`emit_bounds`)|8,508|13,880|26,696|80–100%; checked container arithmetic helper calls only|
|Limits/path/check_limits (`emit_limits`)|3,742|7,087|11,325|100%; retain named limits and paths|
|Other: imports, blank lines, impl shells/closures|6,935|11,370|21,802|100%; retained for this estimate|
|**Total** (`wc -l F`)|**42,813**|**73,745**|**133,964**|No measured post-change total|

Multiply each count by its fraction and round up for estimated remaining lines. Assumes one generated trait impl/type, associated family limits/problem, preserved paths/limits/allocation checks/encoding. New trait overhead is unmeasured. Container helpers alone do not remove concrete types, Parts, accessors or names. Reflective field traversal is outside this estimate.

Conditional remaining-line ranges A/B/C (ceil(count×fraction), unmeasured): validation 565–753/1363–1817/2156–2874; measure 1879–2819/3377–5066/5993–8990; encode 2087–3131/3879–5818/6608–9912; decode 498–996/810–1620/1575–3150; decode_from 1229–1844/2113–3170/3783–5674; bounds 6807–8508/11104–13880/21357–26696. Other rows remain their measured counts under stated assumptions.

Limits detail (command `sed -n 'START,ENDp' F | wc -l`; the following are inclusive spans in A/B/C):

|Item|A span/count|B span/count|C span/count|
|---|---|---|---|
|`Limits`|6–110 / 105|6–341 / 336|6–354 / 349|
|`CEILINGS`|111–214 / 104|342–676 / 335|355–702 / 348|
|`LIMIT_NAMES`|215–318 / 104|677–1011 / 335|703–1050 / 348|
|`values` impl|319–340 / 22|1012–1062 / 51|1051–1102 / 52|
|`Path` and names|341–3625 / 3,285|1063–6729 / 5,667|1103–10954 / 9,852|
|`Problem`|3626–3632 / 7|6730–6736 / 7|10955–10961 / 7|
|`check_limits`, splits|3633–3747 / 115|6737–7092 / 356|10962–11330 / 369|

Physical lines measure source volume, not tokens, compile cost or machine code; those were not measured.

## 3. Translation

Mechanical copying coexists with packing, model remapping and owner checks. Counts are a bounded sample, not extrapolated savings.

Class 1: unchanged field meaning, including nesting/enum/container copies. Class 2: same plus scalar representation conversions. Class 3: packing, splitting/joining, lookup/remapping or owner checks. Strongest class wins; ordinary serialization/construction is mechanical.

Scope totals: `git ls-files SCOPE | rg '\.rs$' | rg -v '/tests/|tests\.rs$|_tests\.rs$|/secrets/' | xargs wc -l`, removing inline test modules before wc. Includes docs/imports/glue. Root file plus matching directory are included where shown `+/**`.

|Owner / source scope|Lines outside tests|Sample class 1/2/3 lines|Sample selection|
|---|---:|---:|---|
|J store: `JR/src/codec.rs` +/**|17,796|31/12/117|value::{funder,read_funder,wall,read_wall,name,read_name}; models::encode; pieces::{fields,read_fields,rules}|
|T store: `TS/src/records.rs` +/**|10,429|8/80/56|names::{encode_repository,decode_repository,encode_piece,encode_segments,decode_segments}; status::{encode_facts,decode_facts,encode_looked}|
|J engine client: `JE/src/translate.rs` +/**|11,590|18/51/42|accounts::{payer,holder,facts}; parts::{plan,served,read}|
|J client: `JL/src/translate.rs` +/**|11,104|15/0/29|me::{symbols,wire_symbols,projects}; envelope::{versions,family}|
|T engine client parts: `T/engine-domain-forge-vocabulary/src/parts.rs`|135|15/42/42|chat; goal,totals; milestone|
|T client parts: `T/client-domain-forge/src/parts.rs`|89|—/—/27|milestone only (other wrappers not field translators)|
|J engine worker: `EW/src/translate.rs`|1,177|10/19/23|phase; turn,optional_token; part,transcript|
|J worker: `WL/src/translate.rs`|1,003|10/3/9|phase; name; part|
|Smith shared agent/channel: `MX/src/**`|2,988|27/0/166|result::{decode_fields,copy_fields}; answer::answer_record,result::decode_declared,start::start_context (shape)|
|Smith host: `M/host-protocol/src/translate.rs`|348|—/—/104|encode_start|
|Temper worker: `T/worker-service/src/worker/translate.rs`|281|unclassified|typed payloads, identity/workspace handling|
|Temper private records: `T/protocol-records/src/codec.rs`|246|unclassified|private origin/item codecs|

Sample command: `sed -n 'L,Ep' FILE | wc -l`, E the balanced closing brace; signature/body only. Purposive sample of copies, scalars and packing; no extrapolation. Zero means none sampled, not none in the owner. Unclassified lines remain unassigned.

Examples, with verified line counts from that command:

|Class|Examples (`file:line`, function, lines)|
|---|---|
|1, plain|`TS/src/records/names.rs:7` encode_repository, 4; `:11` decode_repository, 4; `JL/src/translate/me.rs:77` symbols, 7; `:85` wire_symbols, 8; `MX/src/result.rs:55` decode_fields, 10 and `:111` copy_fields, 17; `JR/src/codec/tasks/value.rs:7` funder, 14 and `:21` read_funder, 17|
|2, scalar|`JR/src/codec/tasks/value.rs:92` wall, 6; `:98` read_wall, 6; `TS/src/records/status.rs:6` encode_facts, 44; `:51` decode_facts, 18; `:70` encode_looked, 18; `JE/src/translate/accounts.rs:7` facts, 51; `EW/src/translate.rs:236` optional_token, 3; `WL/src/translate.rs:250` name, 3|
|3, shape|`JR/src/codec/tasks/models.rs:6` encode, 37 (sort referenced model names and construct bit remap); `.../tasks/pieces.rs:18` fields, 22 (byte-summed pieces/admission); `TS/src/records/names.rs:43` decode_segments, 28 (piece framing and aggregate name bound); `JE/src/translate/parts.rs:38` read, 19 (reconcile cursors and served parts); `MX/src/result.rs:10` decode_declared, 44 (form-specific forbidden-field checks); `M/host-protocol/src/translate.rs:22` encode_start, 104 (join domain grants/directories with service-held values)|

Scalar spellings in the full scopes: `rg -o 'PATTERN' SCOPE_TEXT | wc -l` after test removal. Counts are call sites, not type-inferred semantic conversions; time names can include non-Wall types, try_from includes lengths. Exhaustive only for these patterns.

|Scope|`.as_nanos` / `::from_nanos`|`.raw` / `Token::new`|`.bits` / `::from_parts`|`::try_from`|`u64::from`|
|---|---:|---:|---:|---:|---:|
|J store|79/70|3/3|4/2|76|25|
|T store|67/67|0/0|0/0|110|2|
|J engine client|93/3|2/0|4/2|25|13|
|J client|2/104|0/3|0/0|8|28|
|T engine parts|4/0|0/0|0/0|0|0|
|T client parts|0/1|0/0|0/0|0|0|
|J engine worker|0/0|22/25|0/0|5|3|
|J worker|0/0|22/22|0/0|16|0|
|Smith shared|2/2|1/1|0/0|17|0|
|Smith host|0/0|1/0|0/0|1|0|
|T worker|0/0|0/0|0/0|3|0|
|T private records|0/0|0/3|0/0|7|2|

Meanings verified: Wall↔u64 nanos (JR/src/codec/tasks/value.rs:92/98 and TS/src/records/status.rs:6/51); Token↔u64 (worker files above); Models↔bits/all (JR/src/codec/tasks/value.rs:650,675), with class-3 vocabulary remapping in models.rs.

Unclassified: other temper domain/service client translations (e.g. engine-domain-forge/src/public/**), views, OAuth/accounts, cursors, smith local/channel receive/calls/live and worker routing. Semantic conversion counts/class totals outside the sample and three examples/class/owner are **unverified**.

## 4. Goldens, readers and reference coverage

Generated family tests embed literal bytes; they do not read the separate golden files. Drift tests, patch tests and handwritten consumers do read them, and the reference has fingerprint pins as well as consistency properties.

Discovery: `rg -n 'golden|include_bytes!|include_str!|fs::read' . -g '*.rs' -g '!**/secrets/**' -g '!**/generated/**' -g '!**/golden/**'`, then inspect paths. Entries give readers/first calls, not each fixture literal; inline tests included.

Reader prefixes: SK/ = skein/tests/codec/tests/suite/; SC/ = jig/tests/store-codecs/tests/suite/; MC/ = smith/tests/codecs/tests/suite/; PC/ = temper/tests/codecs/tests/suite/; CW/ = jig/tests/client-wire/tests/suite/; CH/ = smith/tests/channel/tests/.

|Reader location|What it checks|
|---|---|
|`CG/emit_tests.rs:157,274,332`; each section 1 `tests/generated/*.rs`|Literal round trips, tail, measure and bounds; no file IO.|
|`SK/drift.rs:28,60,75`|Reference golden inventory/bytes/source/test drift.|
|`skein/tests/oauth/tests/suite/drift.rs:13,35`|OAuth generated code, bytes and fingerprint.|
|`skein/examples/ui/view/src/tests/drift.rs:20,30`|Notes view generated codec and golden drift.|
|`jig/tests/codec-families.rs:229,257,284`|Shared jig family/code/test/golden inventory and exact bytes; used by jig codec drift worlds.|
|`SC/drift.rs:29,58`|Store generated layout fingerprint file as well as shared family drift.|
|`jig/tests/worker-wire/tests/suite/drift.rs:39`; `jig/tests/protocol-accounts/tests/suite/drift.rs:40`|Worker wire and moved-account golden bytes.|
|`MC/drift.rs:44,59,78,97`|Charter/channel/transcript/local-store generated files and bytes; event re-encoding.|
|`PC/drift.rs:43,61,163,176,189`|Store, worker, client, private and public codecs; removes/refuses stale generated goldens, retains handwritten ones.|
|`CG/patch/golden.rs:258,417,435,442`|Synthesized inputs from kept texts; forward/reverse output/refusal and stale-file checks.|
|`JW/patches/mod.rs:6994`; `M/channel/patches/mod.rs:3600`; `M/charter/patches/mod.rs:541`; `TW/patches/mod.rs:1616`|One generated wrapper per numbered patch invokes that patch reader.|
|`skein/tests/codec/patches/mod.rs:755`; `skein/tests/codec-patches/patches/mod.rs:175`; `jig/testing/jig-test-patches-jig/patches/mod.rs:118`; `jig/testing/jig-test-patches-smith/patches/mod.rs:87`; `smith/tests/transcript-patches/patches/mod.rs:73`|Same reader for reference/fixture owners.|
|`SK/patch.rs:351,382`; `suite/fuzzy/decode.rs:76`; `suite/fuzzy/layout.rs:214`|Patch/mutation/layout oracles. generic.rs:49 and memory.rs:34,182,332 generate/build values in memory, without family golden IO.|
|`SC/memory.rs:9`; `suite/fuzzy/decode.rs:70,76`; `suite/placed.rs:9`|Full row heap bounds, mutation/panic checks and placed-hold round trips.|
|`MC/support/goldens.rs:7`; `suite/memory.rs:88`; `suite/fuzzy/decode.rs:83`|Shared fixture reads, heap bounds, mutations.|
|`MC/{charter.rs:28,channel.rs:65,transcript.rs:26,events.rs:23,patches.rs:20}`|Record semantics, tail/version checks, JSON event round trips, patch inventory.|
|`temper/tests/codecs/src/lib.rs:9,16`; `tests/suite/{client_wire.rs:26,worker_wire.rs:78,fuzzy/worker_wire.rs:59,store_queue.rs:39,store_protection.rs:31}`|Shared family mutations/round trips, payload translation, queue/protection bytes.|
|`CW/{terminal_login.rs:27,settings.rs:58,settings_proposals.rs:9,placed_holds.rs:88,fuzzy/client_wire.rs:720,fuzzy/public_wire.rs:96}`; `jig/tests/engine-clients/tests/suite/fuzzy/cursor.rs:15,32`|Typed client/settings/login translation and mutation corpora; cursor decoder mutations.|
|`JL/src/tests/{settings.rs:16,settings_proposals.rs:11}`; `TC/src/{tests.rs:1053,tests/checkpoints.rs:14,tests/drafts.rs:38,tests/forgejo_tokens.rs:28,tests/forgejo_admin.rs:55}`|Client document/enum round trips, checkpoint/draft projections, handwritten fixtures.|
|`J/client-protocol-wire/src/tests/public/schema.rs:23`; `TC/src/tests/public.rs:26`|`.fields` snapshots of reachable public wire shape, outside a golden directory.|
|`J/protocol-store/src/tests/{checkpoints.rs:202,history_logs.rs:249}`; `TS/src/tests/registered_layouts.rs:37,94`|Checkpoint/header/history compatibility and old-layout→current registered rows.|
|`jig/tests/store/tests/suite/{layout_golden.rs:8,transcript_golden.rs:12}`|Directory layout and transcript framing, distinct from codec family goldens.|
|`MX/src/charter.rs:308,328`; `M/charter/src/lib.rs:80`; `M/agent-service/src/live/tests/room.rs:85`|Charter unknown-version/capacity tests and v4 bridge up.|
|`CH/{suite/stories.rs:103,suite/stories/limits.rs:42,suite/fuzzy/stream_cuts.rs:11,suite/composed.rs:157,live.rs:11}`; `smith/tests/channel/src/lib.rs:538,1270`; `smith/tests/agent-process/src/{lib.rs:415,program_world.rs:1088}`|Resumed charter/turn fixtures through channel and agent processes.|
|`jig/testing/jig-test-worker/src/tests/fixtures.rs:74`; `jig/testing/jig-client-test-records/src/activity_reads.rs:4`; `jig/testing/jig-client-protocol-kit/src/fixtures.rs:279`; `J/protocol-transcript/src/tests.rs:640,659,952`|Shared current/old charter/turn fixtures, transcript indexing and patch reading.|

Non-codec readers (not encoding pins):

- UI/view snapshots: `J/client-view/src/tests/kit.rs:107`; `jig/tests/client/tests/suite/text_pages.rs:18`; `skein/examples/ui/view/src/tests/{text.rs:20,chart.rs:81}`; `skein/tests/ui-web/tests/suite/pages.rs:79`; `S/ui/src/tests/scene.rs:204`; `S/ui-web/src/tests/{svg.rs:33,mapping.rs:468,page.rs:67,page/events.rs:284}`; `S/ui-text/src/tests.rs:26`; `S/ui-screen/src/{tests.rs:375,tests/render.rs:57}`; `S/ui-bundle/src/tests.rs:99`.
- Temper render/tool snapshots: `T/client-tools/src/tests/{fixtures.rs:551,json.rs:217,help.rs:494,jig/settings.rs:667,jig/settings_overlays.rs:19}`; `T/client-shell-terminal/src/tests/render.rs:22`; `T/client-tui/src/tests/{welcome.rs:498,talk.rs:178,launchpad.rs:871,goldens.rs:40,front.rs:33,foot.rs:1484,chat.rs:1033}`; `temper/testing/temper-box-kit/src/terminal.rs:229`.
- Events/bench fixtures: `M/events/src/tests.rs:9`; `M/protocol-events/src/{sinks/tests.rs:144,sinks/tests/view.rs:96}`; `smith/testing/smith-bench-fixtures/src/events.rs:109`; `smith/benchmarks/src/{fixture/events.rs:109,process/lower_tests.rs:188}`; `smith/benchmarks/tests/suite/{slow_completion_probe.rs:111,fuzzy/parsers.rs:73,events.rs:111,events/delegation.rs:35,common/tool.rs:8,budget_visibility.rs:38}`.
- Storage/render/upgrade/tooling: `skein/tests/kv/tests/suite/{golden.rs:36,logs.rs:378}`; `hearth/testing/hearth-test-render/src/golden.rs:68`; `hearth/testing/hearth-test-fixtures/src/golden.rs:24`; `hearth/tests/builder/tests/suite/goldens.rs:192`; `T/stage-one-tool/src/{golden.rs:508,tests/golden.rs:7,tests/judge.rs:71}`; `temper/tests/one-process/src/golden.rs:628,726` and `src/upgrade/story/image.rs:105`; `M/local-shell/src/settings/tests.rs:102` (`built-in.golden`, outside a golden directory).

Static audit; shared readers represent indirect calls. Additional dynamically assembled/hidden-helper paths are **not proven absent**.

Every: `skein/tests/codec/schema/every-v{1,2}.schema` + kept copies; `src/every/v{1,2}.rs` (1,470/1,543 lines, wc -l); tests/generated/every{,_v2}.rs; golden/every-v1 26 files/666 bytes, every-v2 28/750 (section 1 DIR commands); patches/0001.patch and patches/golden/0001/every. Readers/oracles are listed above.

Coverage (`every-v2.schema:7,9,19,25,27,33,40,54`): all primitive Type variants in CG/lib.rs:78; empty/versioned/plain records; named fields; payloadless, record and boxed enum variants; record/enum lists, nested lists, lists of options, optional lists/records/enums/text; bytes holds a foreign record and bytes pieces local records.

section 1 uses but every lacks: defaults (tasks-v13:35/306; store-v25:302/369), secret holds (jig accounts moved-v1:9, jig secrets-v1:6/12), held enum (temper store-v25:233). Every's synthetic foreign hold does not exercise resolved production family graphs. Multiple bools and long records exercise bool grouping/function splitting absent in every; lints-v1 is a separate fixture. `was` exists in grammar but no current section 1 text uses it (`git ls-files '*schema/*.schema' ':!:**/kept/**' | xargs rg -n ' was '`: none). No missing primitive production found.

Fingerprint pins: `SK/fingerprint.rs:146,156` fixes every-v1/secrets-v1 family digest literals; `S/codec/src/schema/fingerprint/tests.rs:25,68` fixes canonical bytes/nested digests; `.../value_tests.rs:264` fixes every root-layout digest. Other properties compare reprints/renames, generated arrays and .layouts values for consistency. Fingerprints are **not only consistency-checked**.

## 5. Versions, conversion and opening

Store value conversion derives byte plans from texts, while store opening still checks versions, levels and floor. Worker wires use numbered patch bridges and negotiated fingerprints; clients refuse incompatible versions and reload.

All family versions are in section 1. Commands: `rg -n '^family|VERSION|FLOOR|FORMAT_VERSION'` on the following files; `git ls-files '*schema/*' '*patches/*.patch' '*contract/*'` inventories persisted artefacts.

|Owner/wire|Current / floor; definition|Texts, patches and checks|
|---|---|---|
|Jig store|component 3, core 6, tasks 13, people 17, notes 1, accounts 2, settings 3; `J/protocol-store/src/format.rs:35`|`schema/registered`: core/tasks/people/notes/accounts/settings. No map kept texts or patches; registry persists value-layout fingerprints/texts in the store.|
|Temper store|25; `TS/src/lib.rs:20`|registered temper-store; no kept history/patches; `application()` supplies format version.|
|Other stores|component 3, registry/openings/jig secrets 1, OAuth 3, jig accounts/temper secrets/smith local 1, cursor 2|Fixed encoding outside registry; section 1 lists kept baselines. OAuth fingerprint: `S/oauth/src/lib.rs:59`; store digest array: `J/protocol-store/src/generated/layouts.rs:5`.|
|Jig worker|16 / 16; `JW/src/lib.rs:140`, `schema/wires`|Kept 10–16; patches 0001–0006. 0006:17 declares credential-opening-purpose change breaking; older wire cannot negotiate today.|
|Temper worker|10 / 5; `TW/src/lib.rs:213`, `schema/wires`|Kept 5–10; patches 0001–0005. Filenames remain worker-wire-v1.schema/v1.rs.|
|Smith charter/results|5 / 4; `M/charter/src/lib.rs:44`, `schema/wires`|Kept 4/5, patch 0001. Charter FORMAT_VERSION derives from wire VERSION.|
|Smith turns|7 / 7; `M/transcript/src/lib.rs:80`, `schema/wires`|Kept 7 only, no patch; Turn FORMAT_VERSION derives from wire VERSION; archived reader :62 handles current framing.|
|Smith channel/pieces|channel 10 build-only, `M/channel/src/lib.rs:70`; pieces 1/1, `src/pieces.rs:11`|Kept channel 7–10/pieces 1; channel patches 0001–0003. schema/wires declares build-only channel and pieces floor 1. Pieces moves are breaking.|
|Client/public|Jig 30/1, `J/client-protocol-wire/src/lib.rs:22,31`; temper 34/3, `TC/src/lib.rs:48,103`|Four schema/*.layouts lists; two public-v1.fields snapshots. No kept text or bridge. Constants checked against headers in src/tests/versions.rs and temper src/tests.rs:19.|
|Host tools, no codec schema|8 / 1; `T/engine-smith/src/wire.rs:13,15`|contract/host-tools-v1.txt through v8 and contract/floor. contract.rs:278 checks range, unchanged tool lines/adapters. Charter/call carries contract version; no codec patch.|

Generated versioned records check/write schema version (CG/emit_codec.rs:202). Owner-wide four-digit patch numbers differ from version numbers; blocks say family F A to B. CG/patch/renumber.rs:15 moves unlanded files when base took their number; landed text is immutable. patches/mod.rs embeds texts/patches, dispatch, floors/layouts/factories/tests. .layouts stores version/digest; .fields stores public shape, neither is runtime text exchange.

Store checks: `J/protocol-store/src/open.rs:55,65,72,92,138`, format.rs:9,54. Ahead level, then unlevelled/behind; missing/below floor refuses OlderBuild::Floor before planning. Floor 1 (lib.rs:18), bridge build :21. Levels: skein/smith 0 (owners.rs:9,26), jig 6 (patch_set.rs:6), temper 14 (TS/src/patch_set.rs:40). Format versions/deployment and header/commit counts must agree. Registry checks types/texts/recomputed digest and derivable plan (registry/check.rs:72,93,99); underived changes need jobs. Format, levels and floor are separate.

Worker: skein opening.rs:47 selects highest common channel version/checks terms. EW/src/wires.rs:11 refuses duplicate names, invalid ranges, missing wire, disjoint ranges or fingerprint mismatch at **any common version**; selects highest. WL/src/wires.rs:16 rejects unsupported/duplicate agreed entries. Negotiated bounds: JW/src/kinds.rs:123. Ranges/digests/limits cross, no texts. Smith offers only version 10 (M/channel/src/kinds.rs:105,139), framing/terms checks, no cross-build channel bridge.

Clients: JE/src/served.rs and public/component.rs:197 refuse owner/public-kind versions outside ranges; JL/src/component.rs:337,418,797,853 checks owner/version/hello. T/client-domain-forge/src/parts.rs:20 checks family version. NotServed/unreadable current jig records mean reload/upgrade; no text/fingerprint negotiation here.

Redeploys: temper/docs/design/draft/redeploys.md **and status file**: engine first, workers stay if new build check permits; drain/restart in turn when requested. All-parts moves let workers drain against old engine. Worker pins agent build; channel per-build. Clients may outlive builds, then reload.

Bridges: JW/src/translation.rs:66,86: newer sends current→agreed down, receives agreed→current up, including charter/turns/temper payloads; TW/src/application.rs:14 names Items/Left/Saved/Preparation/JobSpec/Identity/Ask/Answer. Already-spoken stored bytes can cross unchanged. Reverse totality (:106) and value checks refuse unheld assignments without a task try; fleet avoids task/channel. **Jig floor=current 16 blocks older jig versions today**. Temper 5–10/charter 4–5 remain bridgeable; pieces/turns speak only 1/7; no client/public bridge.

Layout plans: `skein/docs/design/data/codec/layouts.md, 2`; `S/codec/src/layout.rs:17,27`, layout/derive.rs. Match names/former names within root reach; derive added defaulted fields, drops/reorders, enum additions/tag moves, bound/integer widening, option wrapping, versioned heads, held/pieces conversions. Underived Why: Root, NoDefault, Retyped, Reshaped, VariantDropped, Payload, Held, Pieces, Deep. derive_dropping changes dropped variants alone into per-value refusal. Plan owns tables and converts bytes with reusable Workspace, without generic values; reverse derivation is not the patch bridge's checked reverse.

Patch machine: `skein/docs/design/data/codec/{patches,bridges}.md`; `S/codec/src/bridge.rs:185,200,237`. Patcher uses generic decoded values, written owner sites and held-field stops, then re-encodes; Bridge accepts/returns bytes. Up chains forward plans; down chains reverses and runs forward again, requiring equal bytes (Halt::Altered). New fields go down only at their old assumed value; new variants/narrowing failures refuse Unheld. Drops need down defaults/sites; retype/reshape/payload changes need owner sites or breaking floor. Reads all kept versions, speaks floor..current. This differs from the store's two-text byte pass.

## 6. Conflict hotspots

Counts are first-parent commits touching paths, not inferred conflicts. Historical removed/moved paths remain included.

|Artefact|Schema edit|Version move|
|---|---|---|
|src|Affected types/paths/guards/codecs/bounds|Header/prefixes, sometimes rename/module paths|
|tests|Literal cases/bounds/chunks|Header/paths/version bytes|
|goldens|Changed/transitively containing samples|Prefix bytes, new patch results|
|.layouts|Changed wire layout requires version move|Append version/digest|
|fingerprints|Canonical digest propagation|Family/head digest changes; value-layout hash omits family version|
|version-named files|Same file for fixed map version|Rename/add; some temper filenames remain v1|
|patches/mod.rs|Wire block/skeleton/module changes; maps none|Owner number, TEXTS/PATCHES/layouts/dispatch/tests append|
|kept texts|Landed immutable|Add current; maps retain registry texts instead|
|.fields|Reachable public shape|Filename need not move|
|counts/lists|LIMIT_NAMES/values size, Path/name arms, CEILINGS, split/test tables|TEXTS/LAYOUTS/PATCHES/version lists append|

For every cell: `git -c diff.renames=false log origin/main --first-parent --since=2026-11-10T00:00:00Z --format=%H -- PATHS | wc -l`. Select within row C: S=src/generated/**; T=tests/generated/**; G=**/golden/**; L=schema/*.layouts; F=src/generated/layouts.rs + patches/mod.rs; V=basenames `(?:.*[-_]v[0-9]+|v[0-9]+).(rs|schema)`; P=patches/[0-9]*.patch; M=patches/mod.rs; K=schema/kept/**; D=schema/*.fields. F counts fingerprint-bearing files, not only changed digest lines.

A adds `-G 'LIMIT_NAMES|pub fn values|pub enum Path|pub const CEILINGS' -- C/src/generated/`: a verified declaration-change subset, **not all edits inside appended lists**. Batched counting used `--diff-merges=first-parent --name-only --format='COMMIT %H'`, selected those paths and deduplicated hashes before wc. No rename following; deleted/moved historical paths count. Zero means no selected commit; no fetch.

|section 1 crate ID (or historical crate)|S|T|G|L|F|V|P|M|K|D|A|
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|C1|23|12|18|12|0|25|0|0|0|0|7|
|C2|2|0|4|0|0|2|0|0|0|0|2|
|C3|3|2|0|0|0|4|0|0|0|0|1|
|C4|2|0|0|0|2|3|0|2|1|0|2|
|C5|3|2|0|0|0|4|0|0|0|0|1|
|C6|2|0|1|0|0|3|0|0|0|0|2|
|C7|4|3|0|0|0|5|0|0|0|0|2|
|C8|24|0|23|0|26|27|7|12|11|0|1|
|C9|8|5|0|0|0|9|0|0|0|0|4|
|C10|9|6|0|0|0|10|0|0|0|0|6|
|C11|9|4|0|0|0|10|0|0|0|0|5|
|C12|7|4|0|0|0|8|0|0|0|0|3|
|C13|14|10|0|0|0|15|0|0|0|0|5|
|C14|11|5|7|0|7|13|6|7|6|0|4|
|C15|3|2|0|0|0|4|0|0|0|0|1|
|C16|3|0|1|0|0|3|0|0|0|0|2|
|C17|3|2|0|0|0|4|0|0|0|0|1|
|jig/testing/jig-test-patches|0|0|1|0|4|1|1|4|1|0|0|
|C18|0|0|0|0|3|0|0|3|0|0|0|
|C19|0|0|0|0|3|0|0|3|0|0|0|
|C20|3|0|1|0|0|3|0|0|0|0|2|
|C21|3|2|0|0|0|4|0|0|0|0|1|
|C22|2|0|0|0|2|3|0|2|1|0|2|
|C23|3|2|0|0|0|4|0|0|0|0|1|
|C24|2|0|0|0|0|2|0|0|0|0|2|
|C25|3|2|0|0|0|4|0|0|0|0|1|
|C26|3|2|0|0|3|5|1|3|1|0|1|
|C27|0|0|1|0|3|1|0|3|0|0|0|
|C28|9|3|4|0|5|11|2|5|4|0|3|
|C29|6|2|3|0|3|8|1|3|2|0|2|
|C30|2|0|0|0|0|2|0|0|0|0|2|
|C31|3|2|0|0|0|4|0|0|0|0|1|
|C32|4|2|0|0|3|6|0|3|1|0|2|
|C33|0|0|0|0|3|0|0|3|0|0|0|
|C34|35|17|35|19|0|38|0|0|3|3|10|
|C35|3|2|0|0|2|5|0|2|1|0|1|
|C36|21|0|47|0|14|50|13|14|13|0|5|
|C37|28|22|0|0|0|29|0|0|0|0|19|
|C38|9|6|7|0|8|10|5|8|6|0|3|

Other crates with only G hits (same G command): jig/tests/client 1, jig/tests/store 1, S/ui-bundle 2, S/ui-screen 1, skein/tests/kv 1, M/events 1, T/client-shell-terminal 6, T/client-tools 29, T/client-tui 9, T/stage-one-tool 3, temper/tests/one-process 30. No other matched artefact commits were found in the selected range. S/T count directory artefacts; source generated elsewhere in the reference is only included in V where its filename matches.

Examples: `git show --format=fuller --stat HASH`; `git diff-tree --no-commit-id --name-only --no-renames -r HASH^1 HASH`.

- 35bcbfbeae, “regenerate codecs and restore what the merge dropped”: jig client v30/store layouts, temper client v1/store v25, client layout lists.
- 919f7c968f, “Settle ... conflicts across the ten goals”: jig client v28/v30, temper client v1, client layout lists.
- 04f8918aed, “293 paths left with conflicts”: jig client v28, cursor v1/v2, temper client v1, client layout lists.
- 1879e33780, “72 paths left with conflicts”: jig component_v3/store layouts, temper client v1/layout list.
- 684184788f, merge subject “conflicts resolved”: temper client v1 and then-current store v11 change against first parent.

Messages/first-parent diffs show churn; they do not prove each listed path conflicted. Conflict stages/provenance were not reconstructed.
