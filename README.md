# tt

A fast command-line translator. Give it text, get the translation, plus the
synonyms and their back translations, the way the Google Translate web page
shows alternatives.

```console
$ tt sl=pt tl=en "bom dia"
good morning

good morning   bom dia, bom-dia
morning        manhã
```

## Why

Opening a browser tab to translate one phrase is slow. `tt` lives in your shell,
does one request, prints the result, and exits. It reads defaults from a config
file so the common case is just `tt "some text"`.

## Install

### One-liner

```sh
curl -fsSL https://raw.githubusercontent.com/paulo-granthon/tt/main/install.sh | sh
```

`install.sh` is self-adjusting: run inside a clone (with `cargo` available) it
builds from source, otherwise it downloads the release binary for your platform.
Either way it installs the binary and shell completions for bash, zsh, and fish.

### From crates.io

```sh
cargo install ttranslate   # the crate is ttranslate, the binary is still tt
```

### From source

```sh
git clone https://github.com/paulo-granthon/tt
cd tt
./install.sh      # builds and installs binary + completions
# or:
just install      # same thing via cargo install
# or:
cargo install --path .   # binary only; then ./completions.sh for completions
```

## Updating

```sh
tt update
```

Fetches the latest release and installs it over the current binary, in place.

## Usage

```
tt [sl=<lang>] [tl=<lang>] [p=<profile>] [flags] <text>
```

Language and profile are named parameters, so order does not matter and the
source can be omitted:

```sh
tt "bom dia"                 # default profile
tt tl=es "good morning"      # source auto-detected, target Spanish
tt sl=pt tl=en "bom dia"     # fully explicit
tt p=br "good morning"       # use the 'br' profile
tt p=br sl=de "hallo"        # profile, with the source overridden inline
```

### Input from a pipe or a file

```sh
echo "bom dia" | tt tl=en    # translate piped stdin
tt tl=en -                   # a lone - also forces reading stdin
tt tl=en f=notes.txt         # translate the contents of a file
```

When no text is given and stdin is piped, tt reads stdin. When source and target
are the same explicit language, tt returns the input unchanged without a request.

Languages accept aliases and any capitalization: `pt`, `br`, `ptbr`, `pt-BR`
all mean Brazilian Portuguese. Use `auto` (the default source) to detect.

### Flags

| Flag | Meaning |
| --- | --- |
| `-q`, `--quiet` | print only the primary translation |
| `-s`, `--synonyms` | print only the synonyms block |
| `-v`, `--verbose` | a labeled breakdown: languages, original text, translation, synonyms |
| `-j`, `--json` | the full result as one JSON line, for scripts |
| `--no-cache` | skip the on-disk cache for this call |
| `-h`, `--help` | show help |

Output is colored when printed to a terminal and plain when piped, so
`tt -q tl=en "oi" | pbcopy` copies just the translation.

`--json` prints one line with every field the engine returned, for `jq` and
friends:

```sh
tt -j tl=en "bom dia" | jq -r '.synonyms[].word'
```

```json
{"sl":"auto","tl":"en","detected":"pt","text":"bom dia","primary":"good morning","correction":null,"source_translit":null,"target_translit":null,"synonyms":[{"word":"good morning","back":["bom dia"]}],"engine":"google","cached":false}
```

## Cache

Every translation is stored on disk, so repeating a phrase answers without
touching the network. Entries never expire, one file per entry, oldest evicted
past 1000 entries.

```sh
tt cache          # entries, size on disk, path
tt cache clear    # delete every cached translation
tt --no-cache "bom dia"
```

The cache lives in the platform cache directory, `~/.cache/tt` on Linux.

## Defaults and profiles

`tt "text"` uses a profile named `default`. Set it once:

```sh
tt default sl=pt tl=en
tt default             # show the current default
```

Profiles bundle a source and target under a name:

```sh
tt profile add br sl=en tl=pt
tt profile list
tt profile patch br tl=es
tt profile patch br name=brazil
tt profile delete brazil
```

Config lives at `~/.config/tt/config.toml` (or the platform equivalent).

## Resilience

Google throttles its unofficial endpoints per client and per host. `tt` keeps a
list of eight lanes (host, client and response format combinations) and walks
them in order. A lane that fails on the network is retried in the background
after 1s, 2s and 4s while the next lane is tried immediately, the first answer
wins, and the whole attempt gives up after 20 seconds with a link to the web
page. Progress goes to stderr so pipes stay clean.

## Status

[![google lanes](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/overall.json)](https://github.com/paulo-granthon/tt/actions/workflows/live.yml)

A scheduled workflow probes every lane every six hours from GitHub's runners.
Google throttles datacenter addresses harder than home connections, so a red
lane can be a false alarm for you, while green is a reliable signal that the
endpoint and its response shape still work.

| lane | status |
| --- | --- |
| googleapis/dict-chrome-ex | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/googleapis_dict_chrome_ex.json) |
| googleapis/at | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/googleapis_at.json) |
| clients5/dict-chrome-ex | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/clients5_dict_chrome_ex.json) |
| clients5/at | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/clients5_at.json) |
| google.com/dict-chrome-ex | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/google_com_dict_chrome_ex.json) |
| google.com/at | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/google_com_at.json) |
| googleapis/t/dict-chrome-ex | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/googleapis_t_dict_chrome_ex.json) |
| googleapis/t/at | ![](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/paulo-granthon/tt/status/googleapis_t_at.json) |

## Building

```sh
cargo build --release
cargo test
```

## License

MIT
