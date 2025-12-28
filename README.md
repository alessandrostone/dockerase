# dockerase

```
 ___     ___      __  __  _    ___  ____    ____  _____   ___
|   \   /   \    /  ]|  l/ ]  /  _]|    \  /    T/ ___/  /  _]
|    \ Y     Y  /  / |  ' /  /  [_ |  D  )Y  o  (   \_  /  [_
|  D  Y|  O  | /  /  |    \ Y    _]|    / |     |\__  TY    _]
|     ||     |/   \_ |     Y|   [_ |    \ |  _  |/  \ ||   [_
|     |l     !\     ||  .  ||     T|  .  Y|  |  |\    ||     T
l_____j \___/  \____jl__j\_jl_____jl__j\_jl__j__j \___jl_____j
```

A Docker cleaning utility CLI that helps you reclaim disk space by removing unused Docker resources.

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap alessandrostone/tap
brew install dockerase
```

### Cargo (Rust)

```bash
cargo install dockerase
```

### From Source

```bash
git clone https://github.com/alessandrostone/dockerase.git
cd dockerase
cargo install --path .
```

## Usage

```bash
# Show disk usage overview
dockerase

# Safely remove unused resources (dangling images, stopped containers, unused volumes)
dockerase purge

# Interactively select what to remove
dockerase select

# Remove ALL Docker resources (nuclear option)
dockerase --nuclear
```

### Flags

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Skip confirmation prompts |
| `--dry-run` | Preview what would be removed without making changes |
| `--nuclear` | Remove ALL Docker resources (containers, images, volumes, networks, build cache) |

### Examples

```bash
# Preview what purge would remove
dockerase purge --dry-run

# Force purge without confirmation
dockerase purge --force

# Interactively select and preview
dockerase select --dry-run

# Nuclear mode with confirmation skip
dockerase --nuclear --force
```

## Output Example

```
Docker Space Usage
══════════════════════════════════════════════════
┌───────────────────────────────────────┐
│ TYPE          TOTAL   RECLAIMABLE     │
╞═══════════════════════════════════════╡
│ Images        12.5 GB 3.2 GB (8)      │
│ Containers    245 MB  245 MB (5)      │
│ Volumes       1.8 GB  890 MB (3)      │
│ Build Cache   2.1 GB  2.1 GB          │
└───────────────────────────────────────┘

Total Reclaimable: 6.4 GB
```

## License

MIT
