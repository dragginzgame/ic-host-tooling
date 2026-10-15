# Own exact manifest version-only projection for the selected workspace packages.
BEGIN {
    # Manifest version observations belong to the shared TOML reader. This
    # adapter projects this consumer's exact dependency requirement spelling.
    if (mode != "manifest") exit 2
    if (packages == "") exit 2
    split(packages, names, " ")
    for (i in names) owned[names[i]] = 1
}
/^\[/ { selected = ($0 == "[workspace.package]") }
/^version = / && selected {
    count++
}
$1 in owned && /version = "/ {
    count++
    dependencies++
    if (replacement != "") {
        requirement = $0
        sub(/^.*version = "/, "", requirement)
        sub(/".*$/, "", requirement)
        projected = replacement
        # Cargo preserves a bare major.minor requirement's precision when
        # updating workspace dependencies; package/lock versions stay complete.
        if (requirement ~ /^[0-9]+\.[0-9]+$/) sub(/\.[^.]+$/, "", projected)
        sub(/version = "[^"]+"/, "version = \"" projected "\"")
    }
}
{ print }
END { if (packages == "" || count != 1 + dependencies) exit 2 }
