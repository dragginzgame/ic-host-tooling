# Own exact version-only mutation for the four workspace packages.
BEGIN {
    # Manifest version observations belong to the shared TOML reader. This
    # adapter only projects exact release payloads and reads owned lock entries.
    if (read_version && mode != "lock") exit 2
    split("ic-host-artifacts ic-host-fs ic-host-process ic-host-tools", names, " ")
    for (i in names) owned[names[i]] = 1
}
/^\[/ { selected = (mode == "manifest" && $0 == "[workspace.package]"); package_name = "" }
mode == "lock" && /^name = / { package_name = $0; sub(/^name = "/, "", package_name); sub(/"$/, "", package_name) }
/^version = / && (selected || package_name in owned) {
    count++
    if (replacement != "") sub(/"[^"]+"/, "\"" replacement "\"")
    if (read_version) {
        value = $0; sub(/^version = "/, "", value); sub(/"$/, "", value)
        if (count == 1) { observed = value; print value } else if (observed != value) invalid = 1
    }
}
mode == "manifest" && $1 in owned && /version = "/ {
    count++
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
!read_version { print }
END { expected = mode == "manifest" ? 1 + length(names) : length(names); if (count != expected || invalid) exit 2 }
