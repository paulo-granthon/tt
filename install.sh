#!/bin/sh
set -eu

REPO="paulo-granthon/tt"
BIN="tt"
PREFIX="${PREFIX:-$HOME/.local}"
BINDIR="$PREFIX/bin"

os() {
    case "$(uname -s)" in
        Linux) echo linux ;;
        Darwin) echo macos ;;
        *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
    esac
}

arch() {
    case "$(uname -m)" in
        x86_64 | amd64) echo x86_64 ;;
        aarch64 | arm64) echo aarch64 ;;
        *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
    esac
}

repo_root() {
    case "$0" in
        */*) dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd) ;;
        *) return 1 ;;
    esac
    [ -f "$dir/Cargo.toml" ] && [ -d "$dir/src" ] || return 1
    echo "$dir"
}

main() {
    if root=$(repo_root) && command -v cargo >/dev/null 2>&1; then
        install_from_source "$root"
    else
        install_from_release
    fi

    case ":$PATH:" in
        *":$BINDIR:"*) ;;
        *) echo "note: add $BINDIR to your PATH" ;;
    esac
}

install_from_source() {
    root="$1"
    echo "building $BIN from source in $root"
    ( cd "$root" && cargo build --release )
    mkdir -p "$BINDIR"
    install -m 0755 "$root/target/release/$BIN" "$BINDIR/$BIN"
    echo "installed $BIN to $BINDIR"
    install_completions "$root"
}

install_from_release() {
    target="$(arch)-$(os)"
    url="https://github.com/$REPO/releases/latest/download/$BIN-$target.tar.gz"
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    echo "downloading $url"
    curl -fsSL "$url" -o "$tmp/pkg.tar.gz"
    tar -xzf "$tmp/pkg.tar.gz" -C "$tmp"

    mkdir -p "$BINDIR"
    install -m 0755 "$tmp/$BIN" "$BINDIR/$BIN"
    echo "installed $BIN to $BINDIR"
    install_completions "$tmp"
}

install_completions() {
    src="$1"
    if [ -d "$src/completions" ]; then
        bash_dir="${BASH_COMPLETION_USER_DIR:-$HOME/.local/share/bash-completion/completions}"
        zsh_dir="$HOME/.local/share/zsh/site-functions"
        fish_dir="$HOME/.config/fish/completions"
        [ -f "$src/completions/tt.bash" ] && mkdir -p "$bash_dir" && cp "$src/completions/tt.bash" "$bash_dir/tt"
        [ -f "$src/completions/_tt" ] && mkdir -p "$zsh_dir" && cp "$src/completions/_tt" "$zsh_dir/_tt"
        [ -f "$src/completions/tt.fish" ] && mkdir -p "$fish_dir" && cp "$src/completions/tt.fish" "$fish_dir/tt.fish"
        echo "installed shell completions"
    fi
}

main
