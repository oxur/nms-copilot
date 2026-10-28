# Shell completions

`nms completions` generates scripts for Bash, Zsh, Fish, PowerShell, and Elvish. Generation needs only the `nms` executable; loading a script needs the matching shell. These scripts complete one-shot CLI commands and are separate from Tab completion inside the REPL.

## Bash

Save the generated script and load it:

```bash
mkdir -p "$HOME/.bash_completion.d"
nms completions bash > "$HOME/.bash_completion.d/nms"
source "$HOME/.bash_completion.d/nms"
```

Add the `source` line to `~/.bashrc` for future sessions.

## Zsh

Generate `_nms` in a directory on `fpath`:

```zsh
mkdir -p "$HOME/.zfunc"
nms completions zsh > "$HOME/.zfunc/_nms"
```

In `~/.zshrc`, put the directory assignment before the existing `compinit` call:

```zsh
fpath=("$HOME/.zfunc" $fpath)
```

If your configuration does not initialize completion yet, add it once after that assignment:

```zsh
autoload -Uz compinit
compinit
```

## Fish

Save the script in Fish's completion directory. Future sessions load it automatically:

```fish
mkdir -p "$HOME/.config/fish/completions"
nms completions fish > "$HOME/.config/fish/completions/nms.fish"
```

## PowerShell and Elvish

Generate a script for the shell you use:

```text
nms completions powershell
nms completions elvish
```

Save the output and load it through that shell's normal profile/completion setup. Regenerate installed scripts after upgrading `nms` so command and option names stay current.
