---
name: shell-scripting
description: Bash scripting best practices including error handling, argument parsing, and shellcheck compliance. Use when writing shell scripts, reviewing bash code, or automating tasks with shell commands.
allowed-tools: Bash(shellcheck:*) Bash(bash:*) Read Write
---

# Shell Scripting

## When to Use

- Writing new shell scripts or functions
- Reviewing existing bash code for correctness and safety
- Adding error handling and input validation to scripts
- Parsing command-line arguments
- Automating tasks with portable shell commands
- Fixing shellcheck warnings or improving script reliability

## Instructions

### Script Header

Every script must start with a shebang and strict mode:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

- `set -e` -- exit on any command failure
- `set -u` -- error on unset variables
- `set -o pipefail` -- propagate pipe failures

Use `#!/usr/bin/env bash` instead of `#!/bin/bash` for portability.

### Variable Quoting

Always double-quote variable expansions to prevent word splitting and globbing:

```bash
# Correct
cp "$source" "$dest"
for file in "$dir"/*; do
  echo "Processing: $file"
done

# Wrong -- breaks on paths with spaces
cp $source $dest
```

Use `${var:-default}` for defaults, `${var:?error message}` for required variables:

```bash
config_file="${1:?Usage: $0 <config-file>}"
log_dir="${LOG_DIR:-/var/log/myapp}"
```

### Argument Parsing

For simple positional arguments:

```bash
readonly PROG="$(basename "$0")"
readonly ARG1="${1:?Usage: $PROG <input> <output>}"
readonly ARG2="${2:?Usage: $PROG <input> <output>}"
```

For options, use getopts:

```bash
verbose=false
output=""
while getopts ":vo:h" opt; do
  case $opt in
    v) verbose=true ;;
    o) output="$OPTARG" ;;
    h) usage; exit 0 ;;
    :) echo "Option -$OPTARG requires an argument" >&2; exit 1 ;;
    \?) echo "Unknown option: -$OPTARG" >&2; exit 1 ;;
  esac
done
shift $((OPTIND - 1))
```

### Functions

Define functions with the `name()` syntax. Use `local` for all function variables:

```bash
log() {
  local level="$1"
  shift
  printf '[%s] [%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$level" "$*" >&2
}

die() {
  log "ERROR" "$@"
  exit 1
}
```

Return values via stdout (capture with `$()`), not via return codes (which are limited to 0-255).

### Error Handling and Cleanup

Use trap for cleanup on exit:

```bash
cleanup() {
  rm -rf "${tmpdir:-}"
}
trap cleanup EXIT

tmpdir="$(mktemp -d)"
```

The EXIT trap fires on normal exit, errors (set -e), and signals. For additional signal handling:

```bash
trap 'echo "Interrupted" >&2; exit 130' INT TERM
```

### Arrays

Use arrays for lists of items, especially file paths:

```bash
files=()
while IFS= read -r -d '' file; do
  files+=("$file")
done < <(find /path -name "*.txt" -print0)

# Iterate safely
for file in "${files[@]}"; do
  process "$file"
done

# Array length
echo "Found ${#files[@]} files"
```

### Here-Documents

Use here-docs for multi-line strings. Quote the delimiter to prevent variable expansion when needed:

```bash
# With variable expansion
cat << EOF
Hello, $USER. Today is $(date).
EOF

# Without expansion (literal content)
cat << 'EOF'
This $variable is not expanded.
EOF
```

### Common Pitfalls

- Never parse `ls` output -- use globs (`for f in *.txt`) or `find -print0` with `read -d ''`
- Use `[[ ]]` instead of `[ ]` for conditionals (safer, supports regex and pattern matching)
- Use `$(command)` instead of backticks for command substitution
- Always use `"$@"` (quoted) to pass arguments through, never `$*`
- Check command existence with `command -v cmd` instead of `which cmd`
- Use `printf` instead of `echo` for portable output (echo behavior varies)

### Shellcheck Compliance

Run shellcheck on every script before committing:

```bash
shellcheck -o all script.sh        # enable all optional checks
shellcheck -s bash script.sh       # explicit bash dialect
```

When a shellcheck warning is a false positive, disable it locally with a comment:

```bash
# shellcheck disable=SC2034  # variable used in sourced file
unused_var="value"
```

Never disable shellcheck globally. Fix warnings rather than suppressing them.

See `references/bash-patterns.md` for reusable pattern library.

## Examples

### Write a script with argument parsing and validation

```bash
#!/usr/bin/env bash
set -euo pipefail

readonly PROG="$(basename "$0")"
usage() { echo "Usage: $PROG [-v] [-o output] <input-file>" >&2; }

verbose=false
output="/dev/stdout"
while getopts ":vo:h" opt; do
  case $opt in
    v) verbose=true ;;
    o) output="$OPTARG" ;;
    h) usage; exit 0 ;;
    *) usage; exit 1 ;;
  esac
done
shift $((OPTIND - 1))

input="${1:?$(usage)}"
[[ -f "$input" ]] || { echo "File not found: $input" >&2; exit 1; }

$verbose && echo "Processing $input -> $output" >&2
wc -l < "$input" > "$output"
```

### Add safe temp file handling to an existing script

```bash
#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

# All temp files go in the managed directory
curl -sS "https://example.com/data" > "$tmpdir/raw.json"
jq '.items[]' "$tmpdir/raw.json" > "$tmpdir/filtered.json"
wc -l < "$tmpdir/filtered.json"
# tmpdir is automatically cleaned up on exit
```

### Review and fix a script with common mistakes

```bash
# Before (problematic):
#   files=$(ls *.txt)
#   for f in $files; do cat $f; done

# After (correct):
for f in *.txt; do
  [[ -e "$f" ]] || continue   # handle no-match case
  cat "$f"
done
```
