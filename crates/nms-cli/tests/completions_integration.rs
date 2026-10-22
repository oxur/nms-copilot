//! End-to-end checks for generated shell completion scripts.

use assert_cmd::cargo::cargo_bin_cmd;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use tempfile::TempDir;

const ROOT_COMMANDS: &[&str] = &[
    "info",
    "find",
    "show",
    "stats",
    "route",
    "list",
    "convert",
    "export",
    "import",
    "completions",
    "saves",
    "help",
];
const ROOT_FLAGS: &[&str] = &["slot", "help", "version"];
const EXPORT_FLAGS: &[&str] = &[
    "save",
    "biome",
    "infested",
    "within",
    "nearest",
    "named",
    "discoverer",
    "from",
    "format",
    "slot",
    "help",
];
const FIND_FLAGS: &[&str] = &[
    "save",
    "biome",
    "infested",
    "within",
    "nearest",
    "named",
    "discoverer",
    "from",
    "slot",
    "help",
];
const SHOW_COMMANDS: &[&str] = &["system", "base"];

#[derive(Debug, PartialEq, Eq)]
enum SnapshotEntry {
    Directory,
    File(Vec<u8>),
    Other,
}

fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, SnapshotEntry> {
    fn visit(root: &Path, directory: &Path, output: &mut BTreeMap<PathBuf, SnapshotEntry>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                output.insert(relative, SnapshotEntry::Directory);
                visit(root, &path, output);
            } else if kind.is_file() {
                output.insert(relative, SnapshotEntry::File(fs::read(path).unwrap()));
            } else {
                output.insert(relative, SnapshotEntry::Other);
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn run_cli(args: &[&str]) -> Output {
    let home = TempDir::new().unwrap();
    let before = snapshot_tree(home.path());
    let config = home.path().join(".config");
    let data = home.path().join(".local/share");
    let cache = home.path().join(".cache");

    let mut command = cargo_bin_cmd!("nms");
    command
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", config)
        .env("XDG_DATA_HOME", data)
        .env("XDG_CACHE_HOME", cache)
        .timeout(Duration::from_secs(10));
    let output = command.output().unwrap();

    assert_eq!(
        snapshot_tree(home.path()),
        before,
        "CLI changed its isolated HOME"
    );
    output
}

fn assert_successful_output(output: Output) -> String {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let script = String::from_utf8(output.stdout).expect("completion output must be UTF-8");
    assert!(!script.is_empty(), "completion output was empty");
    assert!(
        !script.contains('\u{1b}'),
        "completion output contains ANSI escapes"
    );
    assert!(
        !script.contains("NMS Copilot CLI"),
        "completion output contains a CLI banner"
    );
    assert!(
        !script.contains("SAVE FILE SUMMARY"),
        "completion command attempted save loading"
    );
    script
}

fn normalized(values: Vec<String>, expected: &[&str]) -> bool {
    let mut actual = values;
    let mut expected = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    actual == expected
}

#[derive(Debug)]
struct Manifest {
    root_commands: Vec<String>,
    root_flags: Vec<String>,
    export_flags: Vec<String>,
    find_flags: Vec<String>,
    show_commands: Vec<String>,
}

impl Manifest {
    fn matches(&self) -> bool {
        normalized(self.root_commands.clone(), ROOT_COMMANDS)
            && normalized(self.root_flags.clone(), ROOT_FLAGS)
            && normalized(self.export_flags.clone(), EXPORT_FLAGS)
            && normalized(self.find_flags.clone(), FIND_FLAGS)
            && SHOW_COMMANDS
                .iter()
                .all(|command| self.show_commands.iter().any(|actual| actual == command))
    }

    fn find_has_named(&self) -> bool {
        self.find_flags.iter().any(|flag| flag == "named")
    }
}

fn bash_context_words(script: &str, context: &str) -> Option<Vec<String>> {
    let lines = script.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != format!("{context})") {
            continue;
        }
        let opts = lines
            .get(index + 1)?
            .trim()
            .strip_prefix("opts=\"")?
            .strip_suffix('"')?;
        return Some(opts.split_whitespace().map(str::to_owned).collect());
    }
    None
}

fn bash_flags(words: Vec<String>) -> Vec<String> {
    words
        .into_iter()
        .filter_map(|word| word.strip_prefix("--").map(str::to_owned))
        .collect()
}

fn bash_registrations_are_valid(script: &str) -> bool {
    let registrations = script
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("complete "))
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    !registrations.is_empty()
        && registrations.iter().all(|tokens| {
            tokens.first() == Some(&"complete")
                && tokens.last() == Some(&"nms")
                && tokens.windows(2).filter(|pair| pair[0] == "-F").count() == 1
                && tokens.windows(2).any(|pair| pair == ["-F", "_nms"])
        })
}

fn bash_manifest(script: &str) -> Option<Manifest> {
    if !script.contains("_nms() {") || !bash_registrations_are_valid(script) {
        return None;
    }

    let root = bash_context_words(script, "nms")?;
    let root_commands = root
        .iter()
        .filter(|word| !word.starts_with('-'))
        .cloned()
        .collect();
    Some(Manifest {
        root_commands,
        root_flags: bash_flags(root),
        export_flags: bash_flags(bash_context_words(script, "nms__export")?),
        find_flags: bash_flags(bash_context_words(script, "nms__find")?),
        show_commands: bash_context_words(script, "nms__show")?
            .into_iter()
            .filter(|word| !word.starts_with('-'))
            .collect(),
    })
}

fn zsh_function_body<'a>(script: &'a str, signature: &str) -> Option<&'a str> {
    let (_, body) = script.split_once(signature)?;
    let (body, _) = body.split_once("\n}")?;
    Some(body)
}

fn zsh_helper_commands(script: &str, helper: &str) -> Option<Vec<String>> {
    let body = zsh_function_body(script, &format!("{helper}() {{"))?;
    let (_, commands) = body.split_once("commands=(")?;
    let mut result = Vec::new();
    for line in commands.lines() {
        let line = line.trim();
        if line == ")" {
            break;
        }
        let Some(command) = line.strip_prefix('\'') else {
            continue;
        };
        let Some((name, _)) = command.split_once(':') else {
            continue;
        };
        result.push(name.to_owned());
    }
    Some(result)
}

fn zsh_long_options(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches('\\').trim();
            if !line.starts_with('\'') || !line.contains("--") {
                return None;
            }
            let head = line.split('[').next()?;
            let head = head.rsplit(')').next().unwrap_or(head).trim();
            let long = head.find("--")?;
            let token = head[long + 2..]
                .split_whitespace()
                .next()?
                .trim_matches('\'')
                .trim_end_matches('=');
            (!token.is_empty()).then(|| token.to_owned())
        })
        .collect()
}

fn zsh_subcommand_block<'a>(function: &'a str, subcommand: &str) -> Option<&'a str> {
    let state = function.find("case $state in")?;
    let tail = &function[state..];
    let marker = format!("({subcommand})");
    let start = tail.lines().position(|line| line.trim() == marker)?;
    let start_offset = tail
        .lines()
        .take(start)
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let block = &tail[start_offset..];
    let end = block.find(";;")?;
    Some(&block[..end])
}

fn zsh_manifest(script: &str) -> Option<Manifest> {
    if !script.lines().any(|line| line.trim() == "#compdef nms")
        || !script.contains("compdef _nms nms")
    {
        return None;
    }
    let function = zsh_function_body(script, "_nms() {")?;
    let state = function.find("case $state in")?;
    let root_commands = zsh_helper_commands(script, "_nms_commands")?;
    let show_commands = zsh_helper_commands(script, "_nms__show_commands")?;
    Some(Manifest {
        root_commands,
        root_flags: zsh_long_options(&function[..state]),
        export_flags: zsh_long_options(zsh_subcommand_block(function, "export")?),
        find_flags: zsh_long_options(zsh_subcommand_block(function, "find")?),
        show_commands,
    })
}

fn fish_completion_lines(script: &str) -> Vec<&str> {
    script
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("complete -c "))
        .collect()
}

fn fish_condition(line: &str) -> Option<&str> {
    let (_, condition) = line.split_once(" -n \"")?;
    condition.split_once('"').map(|(value, _)| value)
}

fn fish_context_matches(line: &str, context: &str) -> bool {
    let Some(condition) = fish_condition(line) else {
        return false;
    };
    match context {
        "root" => condition == "__fish_nms_needs_command",
        "export" | "find" => condition == format!("__fish_nms_using_subcommand {context}"),
        "show" => {
            condition
                == "__fish_nms_using_subcommand show; and not __fish_seen_subcommand_from system base help"
        }
        _ => false,
    }
}

fn fish_long_options(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let words = line.split_whitespace().collect::<Vec<_>>();
            let index = words.iter().position(|word| *word == "-l")?;
            words.get(index + 1).map(|value| (*value).to_owned())
        })
        .collect()
}

fn fish_subcommands(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let marker = "-a \"";
            let start = line.find(marker)? + marker.len();
            let value = &line[start..];
            let end = value.find('"')?;
            Some(value[..end].to_owned())
        })
        .collect()
}

fn fish_manifest(script: &str) -> Option<Manifest> {
    let lines = fish_completion_lines(script);
    if lines.is_empty()
        || !lines
            .iter()
            .all(|line| line.starts_with("complete -c nms "))
        || !script.contains("function __fish_nms_needs_command")
        || !script.contains("function __fish_nms_using_subcommand")
    {
        return None;
    }

    let root = lines
        .iter()
        .copied()
        .filter(|line| fish_context_matches(line, "root"))
        .collect::<Vec<_>>();
    let export = lines
        .iter()
        .copied()
        .filter(|line| fish_context_matches(line, "export"))
        .collect::<Vec<_>>();
    let find = lines
        .iter()
        .copied()
        .filter(|line| fish_context_matches(line, "find"))
        .collect::<Vec<_>>();
    let show = lines
        .iter()
        .copied()
        .filter(|line| fish_context_matches(line, "show"))
        .collect::<Vec<_>>();

    Some(Manifest {
        root_commands: fish_subcommands(&root),
        root_flags: fish_long_options(&root),
        export_flags: fish_long_options(&export),
        find_flags: fish_long_options(&find),
        show_commands: fish_subcommands(&show),
    })
}

fn manifest_matches(shell: &str, script: &str) -> bool {
    let manifest = match shell {
        "bash" => bash_manifest(script),
        "zsh" => zsh_manifest(script),
        "fish" => fish_manifest(script),
        _ => None,
    };
    manifest.is_some_and(|manifest| manifest.matches())
}

fn has_export_context(shell: &str, script: &str) -> bool {
    match shell {
        "bash" => bash_context_words(script, "nms__export").is_some(),
        "zsh" => zsh_function_body(script, "_nms() {")
            .and_then(|function| zsh_subcommand_block(function, "export"))
            .is_some(),
        "fish" => fish_completion_lines(script)
            .iter()
            .any(|line| fish_context_matches(line, "export")),
        _ => false,
    }
}

fn find_has_named(shell: &str, script: &str) -> bool {
    let manifest = match shell {
        "bash" => bash_manifest(script),
        "zsh" => zsh_manifest(script),
        "fish" => fish_manifest(script),
        _ => None,
    };
    manifest.is_some_and(|manifest| manifest.find_has_named())
}

fn remove_bash_word(script: &str, context: &str, word: &str) -> Option<String> {
    let mut lines = script.lines().map(str::to_owned).collect::<Vec<_>>();
    for index in 0..lines.len().saturating_sub(1) {
        if lines[index].trim() != format!("{context})") {
            continue;
        }
        let prefix = "opts=\"";
        let value = lines[index + 1]
            .trim()
            .strip_prefix(prefix)?
            .strip_suffix('"')?;
        let words = value.split_whitespace().collect::<Vec<_>>();
        if words.iter().filter(|candidate| **candidate == word).count() != 1 {
            return None;
        }
        let changed = words
            .into_iter()
            .filter(|candidate| *candidate != word)
            .collect::<Vec<_>>()
            .join(" ");
        let indentation =
            &lines[index + 1][..lines[index + 1].len() - lines[index + 1].trim_start().len()];
        lines[index + 1] = format!("{indentation}{prefix}{changed}\"");
        return Some(lines.join("\n") + "\n");
    }
    None
}

fn remove_zsh_helper_command(script: &str, helper: &str, command: &str) -> Option<String> {
    let lines = script.lines().collect::<Vec<_>>();
    let marker = format!("{helper}() {{");
    let start = lines.iter().position(|line| line.trim() == marker)?;
    let end = (start + 1..lines.len()).find(|index| lines[*index].trim() == "}")?;
    let matching = (start + 1..end)
        .filter(|index| lines[*index].trim().starts_with(&format!("'{command}:")))
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return None;
    }
    Some(
        lines
            .into_iter()
            .enumerate()
            .filter(|(index, _)| *index != matching[0])
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
}

fn remove_zsh_export_flag(script: &str, flag: &str) -> Option<String> {
    let function = zsh_function_body(script, "_nms() {")?;
    let block = zsh_subcommand_block(function, "export")?;
    let matching = block
        .lines()
        .enumerate()
        .filter(|(_, line)| zsh_long_options(line).iter().any(|option| option == flag))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matching.len() != 1 || script.matches(block).count() != 1 {
        return None;
    }

    let mut changed_block = block
        .lines()
        .enumerate()
        .filter(|(index, _)| *index != matching[0])
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    if block.ends_with('\n') {
        changed_block.push('\n');
    }

    let mut result = script.to_owned();
    let start = result.find(block)?;
    result.replace_range(start..start + block.len(), &changed_block);
    Some(result)
}

fn fish_remove_line(script: &str, predicate: impl Fn(&str) -> bool) -> Option<String> {
    let lines = script.lines().collect::<Vec<_>>();
    let matching = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| predicate(line))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return None;
    }
    Some(
        lines
            .into_iter()
            .enumerate()
            .filter(|(index, _)| *index != matching[0])
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
}

fn wrong_namespace(shell: &str, script: &str) -> Option<String> {
    match shell {
        "bash" => {
            let mut lines = script.lines().map(str::to_owned).collect::<Vec<_>>();
            let index = lines.iter().position(|line| {
                let line = line.trim();
                line.starts_with("complete -F _nms ") && line.ends_with(" nms")
            })?;
            lines[index] = lines[index].trim_end().strip_suffix(" nms")?.to_owned() + " other";
            Some(lines.join("\n") + "\n")
        }
        "zsh" => Some(script.replacen("#compdef nms", "#compdef other", 1)),
        "fish" => {
            let mut count = 0;
            let changed = script.replace("complete -c nms ", "complete -c other ");
            if changed == script {
                return None;
            }
            for line in script.lines() {
                if line.starts_with("complete -c nms ") {
                    count += 1;
                }
            }
            (count > 0).then_some(changed)
        }
        _ => None,
    }
}

fn remove_root_export(shell: &str, script: &str) -> Option<String> {
    match shell {
        "bash" => remove_bash_word(script, "nms", "export"),
        "zsh" => remove_zsh_helper_command(script, "_nms_commands", "export"),
        "fish" => fish_remove_line(script, |line| {
            line.trim().starts_with("complete -c nms ")
                && line.contains("__fish_nms_needs_command")
                && line.contains("-a \"export\"")
        }),
        _ => None,
    }
}

fn remove_export_named(shell: &str, script: &str) -> Option<String> {
    match shell {
        "bash" => remove_bash_word(script, "nms__export", "--named"),
        "zsh" => remove_zsh_export_flag(script, "named"),
        "fish" => fish_remove_line(script, |line| {
            line.trim().starts_with("complete -c nms ")
                && fish_context_matches(line, "export")
                && line.split_whitespace().any(|word| word == "named")
                && line.contains("-l named")
        }),
        _ => None,
    }
}

fn scripts_manifest(shell: &str) -> String {
    let output = run_cli(&["completions", shell]);
    assert_successful_output(output)
}

#[test]
fn test_completions_all_shells_generate_scoped_manifests_and_reject_mutants() {
    for shell in ["bash", "zsh", "fish"] {
        let script = scripts_manifest(shell);
        assert!(
            manifest_matches(shell, &script),
            "{shell} manifest mismatch: {script}"
        );

        let without_root_export = remove_root_export(shell, &script).expect("root export entry");
        assert_ne!(
            without_root_export, script,
            "root export mutation was ineffective"
        );
        assert!(
            has_export_context(shell, &without_root_export),
            "export subtree was removed for {shell}"
        );
        assert!(
            !manifest_matches(shell, &without_root_export),
            "{shell} accepted a missing root export entry"
        );

        let without_export_named = remove_export_named(shell, &script).expect("export named entry");
        assert_ne!(
            without_export_named, script,
            "export named mutation was ineffective"
        );
        assert!(
            find_has_named(shell, &without_export_named),
            "find lost its independent named option for {shell}"
        );
        if shell == "zsh" {
            let original = zsh_manifest(&script).expect("original zsh manifest");
            let mutated = zsh_manifest(&without_export_named).expect("mutated zsh manifest");
            assert_eq!(mutated.root_commands, original.root_commands);
            assert_eq!(mutated.find_flags, original.find_flags);
        }
        assert!(
            !manifest_matches(shell, &without_export_named),
            "{shell} accepted missing export --named"
        );

        let wrong_command = wrong_namespace(shell, &script).expect("shell command namespace");
        assert_ne!(wrong_command, script, "namespace mutation was ineffective");
        assert!(
            !manifest_matches(shell, &wrong_command),
            "{shell} accepted the wrong command namespace"
        );
    }
}

fn assert_argument_error(output: Output, needle: &str) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "usage error wrote to stdout");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(needle), "missing {needle:?} in {stderr:?}");
    assert!(
        !stderr.to_ascii_lowercase().contains("panicked"),
        "child panic: {stderr}"
    );
}

#[test]
fn test_completions_cli_help_and_shell_argument_errors() {
    let help = run_cli(&["completions", "--help"]);
    assert_eq!(
        help.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&help.stderr)
    );
    assert!(help.stderr.is_empty());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(
        help.contains("<SHELL>"),
        "missing positional shell in {help}"
    );
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        assert!(
            help.contains(shell),
            "missing accepted shell {shell} in {help}"
        );
    }
    assert!(!help.contains('\u{1b}'));

    let missing = run_cli(&["completions"]);
    assert_argument_error(missing, "<SHELL>");

    let invalid = run_cli(&["completions", "not-a-shell"]);
    let invalid_stderr = String::from_utf8(invalid.stderr.clone()).unwrap();
    assert_argument_error(invalid, "not-a-shell");
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        assert!(
            invalid_stderr.contains(shell),
            "missing accepted shell {shell} in {invalid_stderr}"
        );
    }
}
