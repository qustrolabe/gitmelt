# gitmelt

a tool for quickly generating single file digest of a folder, inspired by
[gitingest](https://github.com/coderamp-labs/gitingest)

## Usage

```bash
gitmelt -i '*.{rs,toml}'
```

produces a `digest.txt` file in the current directory containing all the `.rs`
and `.toml` files in the current directory and its subdirectories

## Build

```bash
cargo build --release
cargo install --path .
```
