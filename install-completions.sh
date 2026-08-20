#!/bin/sh
set -eu

# Installs tt shell completions for bash, zsh, and fish from this repo.
# Runs for all three shells regardless of the one you use now, so completions
# are ready the moment you open any of them.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
src="$script_dir/completions"

data="${XDG_DATA_HOME:-$HOME/.local/share}"
config="${XDG_CONFIG_HOME:-$HOME/.config}"

bash_dir="${BASH_COMPLETION_USER_DIR:-$data/bash-completion/completions}"
zsh_dir="$data/zsh/site-functions"
fish_dir="$config/fish/completions"

install_one() {
    from="$1"
    to="$2"
    mkdir -p "$(dirname -- "$to")"
    cp "$from" "$to"
    echo "  installed $to"
}

echo "installing tt completions:"
install_one "$src/tt.bash" "$bash_dir/tt"
install_one "$src/_tt" "$zsh_dir/_tt"
install_one "$src/tt.fish" "$fish_dir/tt.fish"

echo
echo "bash: needs the bash-completion package; open a new shell to load it."
echo "zsh:  make sure this line is in ~/.zshrc before 'compinit':"
echo "        fpath=($zsh_dir \$fpath)"
echo "      then open a new shell."
echo "fish: ready in any new shell, no extra step."
echo
echo "completions only trigger for 'tt' on your PATH, not for './target/...' paths."
