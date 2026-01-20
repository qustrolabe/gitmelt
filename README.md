# gitmelt

[![GitHub Repo](https://img.shields.io/badge/GitHub-qustrolabe/gitmelt-blue?style=flat&logo=github)](https://github.com/qustrolabe/gitmelt)
[![Crates.io](https://img.shields.io/crates/v/gitmelt.svg)](https://crates.io/crates/gitmelt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful CLI tool for generating single-file digests of repositories, perfect for feeding context to LLMs. Inspired by [gitingest](https://github.com/coderamp-labs/gitingest).

## Installation

### From Crates.io (Recommended)
```bash
cargo install gitmelt
```

### From Source
```bash
git clone https://github.com/qustrolabe/gitmelt.git
cd gitmelt
cargo build --release
cargo install --path .
```

## Usage

### Basic Example
```bash
gitmelt -i '*.{rs,toml}'
```
Generates a `digest.txt` file containing all `.rs` and `.toml` files in the current directory and subdirectories.

### Advanced Example
```bash
gitmelt -i '*.py' -e 'test_*' --preset markdown --output my_digest.md
```
Creates a Markdown-formatted digest excluding test files.

### With Git URL
```bash
gitmelt https://github.com/user/repo.git --branch main -i '*.js' --stdout
```
Processes a remote repository in temp folder and outputs to stdout.

## --help

```
Concatenates file contents into a single digest file

Usage: gitmelt [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Path to traverse or Git URL [default: .]

Options:
      --branch <BRANCH>      Git branch to clone (if input is a git URL)
  -i, --include <INCLUDE>    Include patterns (glob)
  -e, --exclude <EXCLUDE>    Exclude patterns (glob)
  -o, --output <OUTPUT>      Output file path (default: digest.txt in current directory)
      --stdout               Print output to stdout instead of file
  -v, --verbose              Verbose logging (info level). Default is error only
      --preset <PRESET>      Output preset [default: default] [possible values: default, markdown, xml]
      --prologue <PROLOGUE>  Prologue mode (tree, list, off) [default: list] [possible values: list, tree, off]
      --dry                  Dry run (only token estimation)
      --no-tokens            Disable token counting
  -t, --timing               Show detailed timing information
  -h, --help                 Print help
```