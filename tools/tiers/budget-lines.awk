# tools/tiers/test's budget lines (tools/tiers/design.md, 11.9): main's allowances.tsv on stdin, then the
# run's counts file; one line for each tree and layer the run's tests are in, and one for each binary
# main's counts lack. awk, not Python, so that the lines cost a few milliseconds.
BEGIN { FS = "\t" }
FILENAME == "-" {
    if ($1 == "main") main = $2
    else if ($1 == "binary" && $2 == profile) place[$3] = $4 " " $5
    else if ($1 == "layer" && $2 == profile) { count[$3 " " $4] = $5; allow[$3 " " $4] = $6 }
    next
}
match($0, /"binary_id": *"[^"]*"/) {
    binary = substr($0, RSTART, RLENGTH)
    sub(/^"binary_id": *"/, "", binary)
    sub(/"$/, "", binary)
    if (!match($0, /"instructions": *[0-9]+/)) next
    n = substr($0, RSTART, RLENGTH)
    sub(/^"instructions": */, "", n)
    n += 0
    if (binary in place) sum[place[binary]] += n
    else unplaced[binary] = 1
}
END {
    for (key in sum) {
        split(key, part, " ")
        printf "budgets: %s %s %s: this run's tests count %.1f G; a change may grow the layer by %.1f G at main %s (2%% of %.1f G, at least 1 G)\n", part[1], profile, part[2], sum[key] / 1e9, allow[key] / 1e9, substr(main, 1, 10), count[key] / 1e9
    }
    for (binary in unplaced)
        printf "budgets: %s is not in main's counts; the gate places a new binary by tools/tiers/layers.toml\n", binary
}
