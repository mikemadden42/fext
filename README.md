# fext

File extension organizer: lists the files in a directory grouped by extension.

## Usage

```
fext [DIR]
```

`DIR` defaults to the current directory. Extensions are compared
case-insensitively, hidden files are skipped, and files without an extension
are listed first under `(none)`.

```
$ fext
(none):
- LICENSE
- Makefile

md:
- README.md

toml:
- Cargo.toml
```
