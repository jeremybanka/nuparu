# Style Decisions for Tasks 0007-0009

These decisions are now encoded in formatter regression tests in
`crates/nuparu-core/src/formatter.rs`.

## 0007. Grouped Multiline Expressions

Chosen style:

- If a grouped expression is clearly multiline, keep the opener separate from
  the first inner stage.

Examples:

```nu
return (
  $groups
  | each { |group|
      {
        dep_name: $group.dep_name
        current_version: $group.current_version
        target_version: $group.current_version
        has_update: false
      }
    }
)
```

```nu
(
  open --raw ($scrubs_dir | path join "seed.yaml")
  | str replace "REPLACE_WITH_SEED_ISO" $iso_location
  | str replace "REPLACE_WITH_SEED_DIR" $seed_dir
) | save --force $template_file
```

## 0008. Simple Catch Clauses

Chosen style:

- Keep `catch` clauses multiline even when the body is short.

Example:

```nu
try {
  print "configured"
} catch {|err|
  print --stderr $err.msg
}
```

## 0009. Multiline Command Call Heads

Chosen style:

- If a command call is already expressed as a multiline grouped expression,
  keep the command head and its arguments fully broken across lines.

Example:

```nu
let highest_version_dir = (
  find-highest-version-dir
    $illustrator_prefs_dir
    '^Adobe Illustrator (?P<version>\d+) Settings$'
    "No Adobe Illustrator settings directories found."
)
```

This keeps the whole grouped value as one clear multiline expression while
avoiding partial compaction like `find-highest-version-dir $illustrator_prefs_dir`
on the first line.
