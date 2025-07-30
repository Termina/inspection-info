## Inspect Info

> a bunch of commands for dev informations.

Install by cloning the demo and run:

```bash
cargo install --path .
```

### Usages

Install it with cargo install git source.

```bash
in help
```

##### IP address

```bash
in ip
10.143.35.141
```

```bash
in ip -d
lo0:	127.0.0.1
lo0:	::1
lo0:	fe80::1
en0:	192.168.35.141
```

##### Copy file to clipboard

```bash
in cpfile Cargo.toml
Copiled 192 characters to clipboard
```

### List largest files

```bash
in large --min 1k --sort
5.6 MB ./target/release/deps/libserde-3f58fa041f951e55.rmeta
5.6 MB ./target/release/deps/libserde-8238757ab41c1ecb.rmeta
5.7 MB ./target/release/deps/libserde-3f58fa041f951e55.rlib
5.7 MB ./target/release/deps/libserde-8238757ab41c1ecb.rlib
```

Filter by file extension:

```bash
in large --min 1k --ext rs
2.1 KB ./src/main.rs
3.4 KB ./src/args.rs
4.2 KB ./src/show_file_size.rs
```

### Dir mark

```bash
in dir add demo # add a bookmark "demo" of current directory
in dir jump demo # jump to it
in dir ls # show all bookmarks
in dir rm demo # unlink it
code $(in dir lookup demo) # open it in vscode
```

Since every shell only allow `cd <dir>` from functions, you need to add extra code to you `.bashrc` or `.zshrc`:

```bash
in dir gg >> ~/.zshrc
```

or eval function directly:

```bash
eval "$(in dir gg)"
```

then jump to `demo` with:

```bash
gg demo
```

### Show Git Tags

List git tags in chronological order. By default, it shows the latest 10 tags.

```bash
in tags
```

Example output:
```
Showing the last 10 of 12 tags (from 2025-07-31 to 2025-07-31). Use --all to see all.

2025-07-31  v0.1.03   Test tag 3
2025-07-31  v0.1.04   Test tag 4
2025-07-31  v0.1.05   Test tag 5
2025-07-31  v0.1.06   Test tag 6
2025-07-31  v0.1.07   Test tag 7
2025-07-31  v0.1.08   Test tag 8
2025-07-31  v0.1.09   Test tag 9
2025-07-31  v0.1.10   Test tag 10
2025-07-31  v0.1.11   Test tag 11
2025-07-31  v0.1.12   Test tag 12
```

To view all tags, use the `--all` flag.

```bash
in tags --all
```

### License

MIT
