use crate::{Configuration, format_text};

#[test]
fn formats_pipelines_and_comments() {
    let input = "ls   |where size > 10| sort-by name   # comment";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "ls | where size > 10 | sort-by name # comment");
}

#[test]
fn preserves_double_pipe_tokens() {
    let input = "do { foo } | complete || true\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "do { foo } | complete || true\n");
}

#[test]
fn normalizes_block_indentation() {
    let input = "def greet [] {\nprint \"hi\"\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "def greet [] {\n  print \"hi\"\n}\n");
}

#[test]
fn preserves_pipeline_indentation_inside_blocks() {
    let input = "def demo [] {\nopen --raw foo\n| lines\n| each {|line| $line }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "def demo [] {\n  open --raw foo\n  | lines\n  | each {|line| $line }\n}\n"
    );
}

#[test]
fn preserves_blank_line_limit() {
    let input = "let x = 1\n\n\n\nlet y = 2\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "let x = 1\n\nlet y = 2\n");
}

#[test]
fn removes_blank_lines_at_block_edges() {
    let input = "def demo [] {\n\n  print \"hi\"\n\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "def demo [] {\n  print \"hi\"\n}\n");
}

#[test]
fn preserves_comment_spacing_from_source() {
    let input = "bun install # repo dependencies\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "bun install # repo dependencies\n");
}

#[test]
fn rejoins_parenthesized_assignments_split_at_spaces() {
    let input = "let parsed =\n($line | parse --regex \"x\")\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "let parsed = ($line | parse --regex \"x\")\n");
}

#[test]
fn rejoins_if_conditions_and_block_openers_split_at_spaces() {
    let input = "if\n(\n$raw_value | str starts-with \"~/\"\n)\n{\n$env.HOME\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "if (\n  $raw_value | str starts-with \"~/\"\n) {\n  $env.HOME\n}\n"
    );
}

#[test]
fn rejoins_parameter_types_and_defaults() {
    let input =
        "export def get-setting [\n  settings:\n    record\n  default_value: any =\n    null\n]\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "export def get-setting [\n  settings: record\n  default_value: any = null\n]\n"
    );
}

#[test]
fn keeps_distinct_parameters_on_separate_lines() {
    let input = "export def get-setting [\n  settings:\n    record\n  key: string\n  default_value: any =\n    null\n]\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "export def get-setting [\n  settings: record\n  key: string\n  default_value: any = null\n]\n"
    );
}

#[test]
fn keeps_long_assignments_broken_when_they_exceed_line_width() {
    let input = "let latest_url =\n  $\"https://channels.nixos.org/($channel)/latest-nixos-($flavor)-($arch)-linux.iso\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn rejoins_short_pipelines() {
    let input = "$env.FILE_PWD\n| path dirname\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "$env.FILE_PWD | path dirname\n");
}

#[test]
fn keeps_long_pipelines_broken_when_they_exceed_line_width() {
    let input = "open --raw $settings_file\n| lines\n| each {|line| $line | str trim }\n| where {|line| $line != \"\" and not ($line | str starts-with \"#\") }\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn rejoins_command_heads_and_arguments() {
    let input = "open\n  --raw\n  $settings_file\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "open --raw $settings_file\n");
}

#[test]
fn rejoins_boolean_conditions_when_they_fit() {
    let input =
        "if (\n  $line != \"\"\n  and not ($line | str starts-with \"#\")\n) {\n  $line\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "if (\n  $line != \"\" and not ($line | str starts-with \"#\")\n) {\n  $line\n}\n"
    );
}

#[test]
fn rejoins_return_record_literals() {
    let input = "return\n{\n  key: value\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "return {\n  key: value\n}\n");
}

#[test]
fn preserves_dense_multiline_ssh_arg_list_from_fixture_shape() {
    let input = "def ssh-base-args [] {\n  [\n    \"-o\" \"ControlMaster=no\"\n    \"-o\" \"ControlPath=none\"\n    \"-o\" \"ControlPersist=no\"\n    \"-o\" \"StrictHostKeyChecking=no\"\n    \"-o\" \"UserKnownHostsFile=/dev/null\"\n    \"-o\" \"NoHostAuthenticationForLocalhost=yes\"\n    \"-o\" \"PreferredAuthentications=publickey\"\n    \"-o\" \"Compression=no\"\n    \"-o\" \"BatchMode=yes\"\n    \"-o\" \"IdentitiesOnly=yes\"\n    \"-o\" \"GSSAPIAuthentication=no\"\n    \"-i\" ($env.HOME | path join \".lima\" \"_config\" \"user\")\n    \"-p\" $ssh_port\n    $\"($guest_user)@127.0.0.1\"\n  ]\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn preserves_dense_multiline_filename_list_from_fixture_shape() {
    let input = "for file_name in [\n  \"carapace-init.nu\"\n  \"config.nu\"\n  \"config.shared.nu\"\n  \"config.darwin.nu\"\n  \"config.linux.nu\"\n  \"env.nu\"\n  \"env.shared.nu\"\n  \"env.darwin.nu\"\n  \"env.linux.nu\"\n  \"kolo.nu\"\n  \"mise.nu\"\n  \"ni-completions.nu\"\n  \"vite-plus.nu\"\n] {\n  print $file_name\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn still_compacts_short_simple_multiline_lists() {
    let input = "let values = [\n  \"a\"\n  \"b\"\n]\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "let values = [\n  \"a\" \"b\"\n]\n");
}

#[test]
fn keeps_record_literal_separate_from_preceding_let_in_fixture_shape() {
    let input = "items | each { |row|\n  let entry = ($row | first)\n  {\n    name: $row.name\n    version: ($entry.version | into int)\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_group_record_literal_separate_from_preceding_let() {
    let input = "items | each { |group|\n  let first = ($group.occurrences | first)\n  {\n    group_key: $group.group_key\n    action_name: $first.action_name\n    current_version: $first.current_version\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_version_group_record_literal_separate_from_preceding_let() {
    let input = "items | each { |group|\n  let parsed = (parse-version $group.current_version)\n  {\n    dep_name: 'jdx/mise'\n    current_version: $group.current_version\n    sort_major: ($parsed | get -o major | default (-1))\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_branch_record_literal_separate_from_preceding_lets() {
    let input = "items | each { |group|\n  let repository = $group.action_name\n  let available_tags = ($tag_cache | get $repository)\n  let target_tag = (select-latest-tag $available_tags)\n\n  if $target_tag == null {\n    {\n      dep_name: $group.action_name\n      current_version: $group.current_version\n    }\n  } else {\n    let target_ref = (resolve-tag-commit $repository $target_tag)\n    let target_version = (normalize-version $target_tag)\n    {\n      dep_name: $group.action_name\n      target_ref: $target_ref\n      target_version: $target_version\n    }\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn rejoins_closure_signatures_and_simple_bodies() {
    let input = "items\n| each {\n  |line|\n  $line | str trim\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, "items | each {|line| $line | str trim }\n");
}

#[test]
fn rejoins_completion_tails() {
    let input = "do { ^limactl stop $instance_name }\n| complete\n| ignore\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "do { ^limactl stop $instance_name } | complete | ignore\n"
    );
}

#[test]
fn keeps_distinct_function_invocations_on_separate_lines() {
    let input = "def main [] {\n  configure-vs-code\n  configure-vscodium\n  configure-cursor\n  configure-illustrator\n  configure-indesign\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_print_and_return_as_distinct_statements() {
    let input = "if ($port_forwards | is-empty) {\n  print \"  no port forwards configured\"\n  return\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_print_and_return_as_distinct_statements_in_another_fixture_shape() {
    let input = "if ($pids | is-empty) {\n  print \"No running hx sessions found.\"\n  return\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_sequential_filesystem_commands_on_separate_lines() {
    let input = "rm -rf $payload_dir\nmkdir $cache_dir\nmkdir $key_dir\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn preserves_multiline_string_command_bodies_and_following_statements() {
    let input = "$\"\n#!/bin/sh\nsudo systemctl reset-failed cloud-final.service || true\nexit 0\n\nexit \\\"$status\\\"\n\" | save --force $guest_apply\nchmod +x $guest_apply\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_print_and_mutation_as_distinct_statements() {
    let input = "print \"Waiting for SSH access to the guest\"\nmut ready = false\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_real_fixture_if_branch_body_on_its_own_line() {
    let input = "export def get-setting [\n  settings: record\n  key: string\n  default_value: any = null\n] {\n  if ($env | columns | any {|column| $column == $key }) {\n    $env | get $key\n  } else if ($settings | columns | any {|column| $column == $key }) {\n    $settings | get $key\n  } else {\n    $default_value\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_adjacent_print_statements_on_separate_lines() {
    let input = "print \"\"\nprint \"Scrubs guest is ready.\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_metadata_print_lines_on_separate_lines() {
    let input =
        "print \"Metadata:\"\nprint $\"  ($resolved_url_file)\"\nprint $\"  ($sha256_file)\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_print_and_do_block_as_distinct_statements() {
    let input = "if $delete_instance == \"true\" {\n  print $\"Deleting temporary Lima instance ($instance_name)\"\n  do {\n    ^limactl delete $instance_name\n  } | complete | ignore\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_side_effects_and_branch_values_on_separate_lines() {
    let input = "let iso_location = if ($existing_iso | path exists) {\n  print $\"Reusing local installer ISO at ($existing_iso)\"\n  $existing_iso\n} else {\n  $seed_iso\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_instruction_prints_on_separate_lines() {
    let input = "print \"Inside the installer console, run:\"\nprint \"  sudo -i\"\nprint \"  /mnt/host-scrubs-seed/install.sh\"\nprint \"\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn normalizes_repeated_internal_spaces_in_simple_pipeline() {
    let input = "export    def repo-root [] {\n  $env.FILE_PWD     |   path dirname\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "export def repo-root [] {\n  $env.FILE_PWD | path dirname\n}\n"
    );
}

#[test]
fn keeps_noisy_adjacent_function_invocations_on_separate_lines() {
    let input = "def    main [] {\n  configure-vs-code configure-vscodium\n  configure-cursor configure-illustrator\n  configure-indesign\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "def main [] {\n  configure-vs-code\n  configure-vscodium\n  configure-cursor\n  configure-illustrator\n  configure-indesign\n}\n"
    );
}

#[test]
fn keeps_noisy_instruction_prints_on_separate_lines() {
    let input = "print    \"Inside the installer console, run:\"\nprint    \"  sudo -i\" print    \"  /mnt/host-scrubs-seed/install.sh\"\nprint    \"\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "print \"Inside the installer console, run:\"\nprint \"  sudo -i\"\nprint \"  /mnt/host-scrubs-seed/install.sh\"\nprint \"\"\n"
    );
}

#[test]
fn normalizes_noisy_grouped_pipeline_indentation() {
    let input = "(\n     open    --raw ($scrubs_dir  |   path join \"seed.yaml\")\n       |      str replace \"REPLACE_WITH_SEED_ISO\" $iso_location\n      |      str replace \"REPLACE_WITH_SEED_DIR\" $seed_dir\n  )     |   save --force $template_file\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(
        output,
        "(\n  open --raw ($scrubs_dir | path join \"seed.yaml\")\n  | str replace \"REPLACE_WITH_SEED_ISO\" $iso_location\n  | str replace \"REPLACE_WITH_SEED_DIR\" $seed_dir\n) | save --force $template_file\n"
    );
}

#[test]
fn keeps_sync_copy_steps_on_separate_lines() {
    let input = "mkdir $local_dir\ncp --force $source_path $dest_path\n\nprint $\"Copied ($source_path)\"\nprint $\"to ($dest_path)\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_sync_to_icloud_steps_on_separate_lines() {
    let input = "mkdir $icloud_dir\ncp --force $source_path $dest_path\n\nprint $\"Copied ($source_path)\"\nprint $\"to ($dest_path)\"\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_update_reporting_closure_multiline() {
    let input = "items | each { |update|\n  if $update.has_update {\n    let current_ref_label = (color-cyan ('(' + $update.current_short_ref + ')'))\n    let target_ref_label = (color-cyan ('(' + (short-sha $update.target_ref) + ')'))\n    print \"updated\"\n  } else {\n    let current_ref_label = (color-cyan ('(' + $update.current_short_ref + ')'))\n    print \"current\"\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_workflow_change_builder_closure_multiline() {
    let input = "items | each { |workflow_use|\n  let update = (\n    $workflow_updates\n    | where group_key == (group-key $workflow_use.action_name $workflow_use.current_version)\n    | first\n  )\n\n  if (not $update.has_update) {\n    null\n  } else {\n    $workflow_use.file_path\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_mise_change_builder_closure_multiline() {
    let input = "items | each { |mise_use|\n  let update = (\n    $mise_updates | where current_version == $mise_use.current_version | first\n  )\n\n  if (not $update.has_update) {\n    null\n  } else {\n    $mise_use.file_path\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_file_update_closure_multiline() {
    let input = "$file_paths | each { |file_path|\n  let file_text = (open --raw $file_path)\n  let had_trailing_newline = ($file_text | str ends-with \"\\n\")\n  let file_lines = ($file_text | split row \"\\n\")\n  let updated_text = ($file_lines | str join \"\\n\")\n\n  if $had_trailing_newline {\n    ($updated_text + \"\\n\") | save --force $file_path\n  } else {\n    $updated_text | save --force $file_path\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_line_update_closure_multiline() {
    let input = "$file_lines\n| enumerate\n| each { |line|\n  let workflow_change = (\n    $workflow_changes\n    | where file_path == $file_path and line_index == $line.index\n    | get replacement\n    | get -o 0\n    | default null\n  )\n  let mise_change = (\n    $mise_changes\n    | where file_path == $file_path and line_index == $line.index\n    | get replacement\n    | get -o 0\n    | default null\n  )\n\n  if ($workflow_change | is-not-empty) {\n    $workflow_change\n  } else if ($mise_change | is-not-empty) {\n    $mise_change\n  } else {\n    $line.item\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_inventory_scan_closure_multiline() {
    let input = "$file_lines\n| enumerate\n| each { |line|\n  let match = (regex-first $line.item $USES_PATTERN)\n  if $match == null {\n    null\n  } else if ($match.action | str starts-with './') {\n    null\n  } else {\n    let original_comment = ($match | get -o comment | default null)\n    if $original_comment == null {\n      null\n    } else {\n      $file_path\n    }\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_two_statement_workflow_use_lookup_closure_multiline() {
    let input = "items | each { |workflow_use|\n  let file_lines = (read-file-lines $workflow_use.file_path)\n  find-mise-version-use $workflow_use.file_path $file_lines $workflow_use.line_index\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_update_resolution_closure_multiline() {
    let input = "items | each { |group|\n  let repository = $group.action_name\n  let available_tags = ($tag_cache | get $repository)\n  let target_tag = (select-latest-tag $available_tags)\n\n  if $target_tag == null {\n    $group.action_name\n  } else {\n    let target_ref = (resolve-tag-commit $repository $target_tag)\n    let target_version = (normalize-version $target_tag)\n    $\"($target_ref)-($target_version)\"\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_two_statement_tag_parsing_closure_multiline() {
    let input = "$result.stdout\n| lines\n| each { |line|\n  let columns = ($line | split row --regex '\\s+')\n  $columns | get -o 1 | default ''\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multistatement_tag_selection_closure_multiline() {
    let input = "$tags | each { |tag|\n  let parsed = (parse-version $tag)\n  if $parsed == null {\n    null\n  } else if $parsed.prerelease != null {\n    null\n  } else {\n    $parsed.major\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_return_grouped_multiline_expression_opener_on_its_own_line() {
    let input = "if $target_tag == null {\n  return (\n    $groups\n    | each { |group|\n        {\n          dep_name: $group.dep_name\n          current_version: $group.current_version\n          target_version: $group.current_version\n          has_update: false\n        }\n      }\n  )\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_plain_grouped_multiline_expression_opener_on_its_own_line() {
    let input = "def build-template [] {\n  (\n    open --raw ($scrubs_dir | path join \"seed.yaml\")\n    | str replace \"REPLACE_WITH_SEED_ISO\" $iso_location\n    | str replace \"REPLACE_WITH_SEED_DIR\" $seed_dir\n  ) | save --force $template_file\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_simple_catch_clause_multiline_in_fixture_shape() {
    let input = "def configure-editor-settings [] {\n  try {\n    print \"configured\"\n  } catch {|err|\n    print --stderr $err.msg\n  }\n}\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multiline_command_call_head_fully_broken_in_fixture_shape() {
    let input = "let highest_version_dir = (\n  find-highest-version-dir\n    $illustrator_prefs_dir\n    '^Adobe Illustrator (?P<version>\\d+) Settings$'\n    \"No Adobe Illustrator settings directories found.\"\n)\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}

#[test]
fn keeps_multiline_command_call_head_fully_broken_for_single_line_argument_shapes() {
    let input = "let highest_version_dir = (\n  find-highest-version-dir\n    $indesign_prefs_dir\n    '^Version (?P<version>\\d+)\\.0$'\n    \"No Adobe InDesign settings directories found.\"\n)\n";
    let output = format_text(input, &Configuration::default());
    assert_eq!(output, input);
}
