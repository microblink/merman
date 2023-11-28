# Merman

A tool for generating svg diagrams from a `json` description. Inspired by `mermaid` tool.

## Usage instructions

### Transforming a single diagram

Transforming a json file (see `sample_files` directory).

```
cargo run some-diagram.json
```

This will print the svg to the standard output.

To print the svg to a file, use the `-o` or `--output` option.

```
cargo run some-diagram.json -o some-diagram.svg
```

### Transforming an `.md` file containing diagrams

The `merman` tool can also transform an `.md` file containing diagrams.
The diagrams are enclosed in `` ```mermaid `` code blocks.

The output diagram will have `svg` data in place of the `` ```merman `` code tags.

Transfor an `.md` file in-place:

```
cargo run some-documentation.md
```

Transform an `.md` file and write the output to a new file:

```
cargo run some-documentation.md -o some-processed-documentation.md
```