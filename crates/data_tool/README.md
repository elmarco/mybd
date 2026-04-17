# Data Tool

A standalone CLI utility for managing `mybd` metadata as "Content as Code". This tool allows you to synchronize the SQLite database with human-readable TOML files, enabling a Git-backed data workflow.

## Overview

The `data_tool` provides two primary commands: `export` and `import`. It maps the `series` and `albums` tables from your SQLite database into structured TOML files.

### Workflow
1. **Export:** Dumps the database state into `data/series/*.toml`.
2. **Git:** You can commit these files to track history or receive Pull Requests.
3. **Import:** Reads the TOML files and updates the database using `UPSERT` logic.

## Usage

Run the tool from the project root using `cargo`:

```bash
# Export database records to TOML files
cargo run -p data_tool -- export

# Import/Update database records from TOML files
cargo run -p data_tool -- import
```

### Options
- `--db <URL>`: Database URL (defaults to `sqlite:mybd.db`).
- `--out-dir <PATH>` (Export): Where to save files (defaults to `data/series`).
- `--in-dir <PATH>` (Import): Where to read files from (defaults to `data/series`).

## TOML Schema

Each TOML file represents a single series and its associated albums.

```toml
title = "The Nice House"
work_type = "comic"
author = "Alvaro Martinez"
description = "..."
year = 2023
bubble_id = "fMvy8bVe8hgr1i"

[[albums]]
title = "The nice house on the lake"
tome = 1
ean = "9791026827887"
bubble_id = "6dCBW7wcQCE4KM"
```

## ID Strategy

The tool uses `bubble_id` as the unique key for synchronization.
- If a record with that `bubble_id` exists, it is **updated**.
- If it does not exist, it is **inserted**.
- If a series is missing `bubble_id`, it will be inserted as a new record on every import (avoid this for managed data).
