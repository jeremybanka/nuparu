# Rejoining Strategy

This note describes where `nufmt` should prefer to rejoin lines when the input
contains unnecessary breaks, and what to do when rejoining would exceed the
target line width.

For now, assume:

- `lineWidth = 80`
- preserving semantics is more important than preserving the user's original
  wrapping
- if a construct fits comfortably on one line, prefer the single-line form
- if it does not fit, choose the "best break" for that construct rather than
  preserving an arbitrary user break

## General rule

Prefer rejoining when a newline appears where a space would be the normal,
unambiguous separator between tokens.

Examples:

```nu
let parsed =
  ($line | parse --regex "...")
```

becomes:

```nu
let parsed = ($line | parse --regex "...")
```

and:

```nu
if
(
  $cond
)
{
  body
}
```

becomes:

```nu
if (
  $cond
) {
  body
}
```

## Places that should prefer rejoining

### 1. Assignment-like operators

Rejoin after operators that normally bind a right-hand expression inline:

- `=`
- mutation/assignment forms, if Nushell tokenization exposes them distinctly
- redirection operators when followed by a simple target

Examples:

```nu
let repo_root =
  ($env.FILE_PWD | path dirname)
```

```nu
default_value: any =
  null
```

Preferred result when it fits:

```nu
let repo_root = ($env.FILE_PWD | path dirname)
default_value: any = null
```

If it does not fit, keep the operator on the first line and break after it:

```nu
let very_long_value =
  ($input | some command | another command | final-step)
```

### 2. Parameter type/default clauses

In parameter lists, these pieces should stay glued together when possible:

- `name: type`
- `name: type = default`
- flags plus typed values

Examples:

```nu
export def get-setting [
  settings:
    record
  default_value:
    any =
    null
]
```

Preferred result when pieces fit:

```nu
export def get-setting [
  settings: record
  default_value: any = null
]
```

If the whole parameter line is too long, prefer breaking after `=` first, not
between `name` and `:` or `:` and `type`:

```nu
some_really_long_parameter_name: list<string> =
  ($env.SOME_VALUE | split row ",")
```

### 3. Command heads and their first arguments

A command name should usually stay on the same line as the first simple
argument or flag set.

Examples:

```nu
open
  --raw
  $settings_file
```

Preferred result when it fits:

```nu
open --raw $settings_file
```

If too long, keep the command head on the first line and break after a natural
argument boundary:

```nu
open --raw
  ($scrubs_dir | path join "very-long-file-name.env")
```

### 4. Pipeline separators

Pipelines should rejoin when the entire pipeline fits under the width.

Examples:

```nu
$env.FILE_PWD
| path dirname
```

```nu
$acc
| upsert $entry.key $value
```

Preferred result when it fits:

```nu
$env.FILE_PWD | path dirname
$acc | upsert $entry.key $value
```

If the pipeline is too long, prefer one of these shapes:

```nu
open --raw $settings_file
| lines
| each {|line| $line | str trim }
```

or, for pipelines nested inside another expression:

```nu
let dns_entries = (
  $dns_servers
  | split row ","
  | each {|entry| $entry | str trim }
)
```

The formatter should not preserve arbitrary mixed styles like:

```nu
open --raw $settings_file |
  lines | each {|line|
  $line | str trim
}
```

### 5. Boolean operators in conditions

Rejoin around boolean operators when the condition still fits:

- `and`
- `or`
- `not` with its immediate operand

Examples:

```nu
if (
  $line != ""
  and not ($line | str starts-with "#")
) {
```

If the whole condition fits, prefer:

```nu
if ($line != "" and not ($line | str starts-with "#")) {
```

If it does not fit, prefer breaks at boolean operators with a hanging indent:

```nu
if (
  ($raw_value | str length) >= 2
  and (
    (($raw_value | str starts-with '"') and ($raw_value | str ends-with '"'))
    or (($raw_value | str starts-with "'") and ($raw_value | str ends-with "'"))
  )
) {
```

### 6. Prefix keywords and their expressions

These should usually stay attached to the expression they govern:

- `if (`
- `else if (`
- `while (`
- `match <expr>`
- `not <expr>`
- `return <expr>`

Examples:

```nu
return
  {}
```

Preferred result when it fits:

```nu
return {}
```

If it does not fit, keep the keyword on the first line and indent the governed
expression:

```nu
return (
  $records
  | where active
  | first
)
```

### 7. Call-site parentheses and brackets

Opening delimiters that begin an argument or subexpression should usually
rejoin with the token before them:

- function call arguments in `(...)`
- subexpressions after `if`, `while`, `not`, and assignment
- list literals `[...]`
- record literals `{...}` when they begin immediately after `=`, `return`, or
  a command argument position

Examples:

```nu
let settings_file =
  (scrubs-dir | path join "settings.env")
```

```nu
return
{
  key: value
}
```

Preferred result when it fits:

```nu
let settings_file = (scrubs-dir | path join "settings.env")
return {
  key: value
}
```

### 8. Block openers

Opening braces should rejoin with the preceding completed header:

- `def foo [] {`
- `if (...) {`
- `else {`
- closure heads like `{|line| ... }` when they fit

Examples:

```nu
if ($parsed | is-empty)
{
  $acc
}
```

Preferred result:

```nu
if ($parsed | is-empty) {
  $acc
}
```

### 9. Closure signatures

Closure heads should stay compact if possible:

- `{|line| ... }`
- `{|line, acc| ... }`

Examples:

```nu
| each {
  |line|
  $line | str trim
}
```

Preferred result when it fits:

```nu
| each {|line| $line | str trim }
```

If the body is too long, prefer:

```nu
| reduce --fold {} {|line, acc|
  let parsed = ($line | parse --regex "...")
  ...
}
```

That is: keep the closure signature on the opening line and break inside the
body, not between `{` and `|line|`.

### 10. String interpolation heads

Interpolated strings should rejoin with the token that introduces them:

- `$"..."` after assignment or return
- `^cmd $"..."` at call sites

Examples:

```nu
let latest_url =
  $"https://channels.nixos.org/($channel)/latest-nixos-($flavor)-($arch)-linux.iso"
```

Preferred result when it fits.

If it does not fit, prefer breaking before a pipeline or after `=`, not inside
the interpolation syntax unless the parser provides a structured way to do that
safely.

### 11. Redirection and completion tails

These should stay on the same line as the pipeline or command they modify when
possible:

- `| complete`
- `| ignore`
- `> file`
- `err> file`

Examples:

```nu
do { ^limactl stop $instance_name }
| complete
| ignore
```

Preferred result when it fits:

```nu
do { ^limactl stop $instance_name } | complete | ignore
```

### 12. Method-style chains and path operations

Short path and string chains are especially good candidates for rejoining:

- `$env.HOME | path join ...`
- `$value | str replace ...`
- `open --raw ... | lines | each ...`

These are common in the fixture set and often read best as a single unit when
under the width limit.

## Where to prefer breaking instead of rejoining

Even with an 80-column target, not every construct should be collapsed.

Prefer multiline formatting for:

- long boolean conditions with nested grouping
- pipelines with three or more substantial stages
- list literals with many elements
- records with multiple fields
- multiline strings and heredoc-like content
- closures whose body contains statements rather than a single short expression
- nested parenthesized expressions that become dense when flattened

## Best-break strategy when rejoining would exceed 80

Once a candidate construct is too wide, choose breaks in this order.

### Pipelines

Best break points:

1. before `|`
2. before a closure body inside a pipeline stage
3. after `=` if the whole pipeline is the right-hand side of an assignment

Preferred shape:

```nu
let dns_entries = (
  $dns_servers
  | split row ","
  | each {|entry| $entry | str trim }
  | where {|entry| $entry != "" }
)
```

### Conditions

Best break points:

1. at `and` / `or`
2. inside nested grouped subconditions
3. before the block opener only as a last resort

Preferred shape:

```nu
if (
  long-left-side
  and another-check
  and (
    nested-condition
    or alternate-condition
  )
) {
```

### Assignments

Best break points:

1. after `=`
2. inside a pipeline or grouped expression on the right-hand side

Preferred shape:

```nu
let output_path =
  ($cache_dir | path join $file_name)
```

### Parameter lists

Best break points:

1. one parameter per line
2. after `=` for long defaults
3. inside the default expression

Avoid:

- breaking between `name` and `:`
- breaking between `:` and `type` unless the parser forces it

### Call arguments

Best break points:

1. one argument per line after the command head
2. inside grouped arguments
3. at pipeline boundaries inside an argument expression

## Likely implementation order

The next parser-aware rejoining passes should probably land in this order:

1. assignments and `keyword (` / `) {` rejoining
2. `name: type` and `name: type = default` parameter compaction
3. pipeline rejoining under width
4. command-head and simple-argument compaction
5. condition flattening when it fits, multiline normalization when it does not
6. closure-head compaction

This order matches the real fixture pressure in `fixtures/dotfiles/scrubs`.
