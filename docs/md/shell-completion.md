# Shell Completion

Knot can generate shell completion scripts to accelerate
command navigation.

To enable autocompletion, generate the script for your shell
and place it in the appropriate directory.

## Bash

To enable completion in Bash, run:

```bash
mkdir -p ~/.local/share/bash-completion/completions
knot complete bash > ~/.local/share/bash-completion/completions/knot
```

## Zsh

To enable completion in Zsh, create the completion directory
and output the script:

```bash
mkdir -p ~/.zsh/completions
knot complete zsh > ~/.zsh/completions/_knot
```

> [!NOTE]
> Add `fpath=(~/.zsh/completions $fpath)` to your `~/.zshrc`
> file **before** calling `compinit`.

## Fish

To enable completion in Fish, run:

```bash
mkdir -p ~/.config/fish/completions
knot complete fish > ~/.config/fish/completions/knot.fish
```

### Fish Prompt Integration

You can modify your Fish theme to display a custom icon when
navigating into a directory containing a `.knot` folder. Add
the following to your Fish configuration file:

```fish
function prompt_knot -d "Display Knot icon if .knot directory exists"
  if test -d .knot
    # prompt_segment <background> <foreground> <text/icon>
    prompt_segment red black "🪢 Knotable"
    # Or use  if you have installed Nerd Fonts
  end
end
prompt_knot
```

> [!NOTE]
> This integration only works if your Fish theme defines the
> `prompt_segment` function. Verify theme support before
> adding this function.

## Elvish

To enable completion in Elvish, generate the module file:

```bash
mkdir -p ~/.config/elvish/lib
knot complete elvish > ~/.config/elvish/lib/knot.elv
```

> [!NOTE]
> To load the generated module, add `use knot` to your
> `~/.config/elvish/rc.elv` file.

## PowerShell

To load completion in PowerShell for your current session,
run:

```sh
knot complete powershell | Out-String | Invoke-Expression
```

> [!NOTE]
> To make PowerShell completion persistent across sessions,
> add the command above to your PowerShell `$PROFILE`.
