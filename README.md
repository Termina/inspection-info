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

### License

MIT
