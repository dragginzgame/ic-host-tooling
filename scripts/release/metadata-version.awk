# Own exact version-only mutation for the four workspace packages.
BEGIN {
    split("ic-host-artifacts ic-host-fs ic-host-process ic-host-tools", names, " ")
    for (i in names) owned[names[i]] = 1
}
/^\[/ { selected = (mode == "manifest" && $0 == "[workspace.package]"); package_name = "" }
mode == "lock" && /^name = / { package_name = $0; sub(/^name = "/, "", package_name); sub(/"$/, "", package_name) }
/^version = / && (selected || package_name in owned) {
    count++
    if (replacement != "") $0 = "version = \"" replacement "\""
    if (read_version) {
        value = $0; sub(/^version = "/, "", value); sub(/"$/, "", value)
        if (count == 1) { observed = value; print value } else if (observed != value) invalid = 1
    }
}
mode == "manifest" && $1 in owned && /version = "/ {
    count++
    value = $0
    sub(/^.*version = "/, "", value)
    sub(/".*$/, "", value)
    if (read_version && value != observed) invalid = 1
    if (replacement != "") sub(/version = "[^"]+"/, "version = \"" replacement "\"")
}
!read_version { print }
END { expected = mode == "manifest" ? 1 + length(names) : length(names); if (count != expected || invalid) exit 2 }
