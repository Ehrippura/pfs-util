# pfs-util

A command-line utility to archive and unarchive PFS packed resource files.

## Build

```sh
cargo build --release
```

## Usage

### Archive

Pack a single file or an entire folder into a PFS archive:

```sh
pfs-util archive <input> <output>
```

**Examples:**

```sh
# Pack a single file
pfs-util archive data.bin data.pfs

# Pack an entire folder
pfs-util archive ./assets/ output.pfs
```

### Unarchive

Extract files from a PFS archive:

```sh
pfs-util unarchive <input> [output] [--dry]
```

| Argument / Flag | Description |
|---|---|
| `input` | Path to the `.pfs` file |
| `output` | (Optional) Target folder. Defaults to the archive's filename stem |
| `--dry` / `-d` | Dry run — list files without extracting |

**Examples:**

```sh
# Extract to a folder named after the archive
pfs-util unarchive data.pfs

# Extract to a specific folder
pfs-util unarchive data.pfs ./output/

# Preview contents without extracting
pfs-util unarchive data.pfs --dry
```
