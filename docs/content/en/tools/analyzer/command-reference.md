+++
title = "Analyzer command reference"
description = "Every flag mago analyze accepts."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Command reference

```sh
Usage: mago analyze [OPTIONS] [PATHS]...
```

`mago analyse` is an alias for `mago analyze`. Both work.

Global flags must come before `analyze`. See the [CLI overview](/fundamentals/command-line-interface/) for the global list.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATHS]...` | Files or directories to analyze. When provided, these replace the `paths` from `mago.toml` for this run. |

## Analyzer-specific options

| Flag | Description |
| :--- | :--- |
| `--no-stubs` | Skip the built-in PHP standard-library stubs. Use only when you have a reason. |
| `--skip-ignores` | Ignore the `ignore` list from `mago.toml` and report every issue found. Inline suppressions still apply. |
| `--slow-file-warning-threshold-ms <MILLISECONDS>` | Override the per-file slow analysis warning threshold from `mago.toml`. |
| `--cache <FILE_OR_DIR>` | Persist raw analysis results and reuse them when all inputs are unchanged. See [Persistent cache](#persistent-cache). |
| `--staged` | Analyze only files staged in git. Fails outside a git repository. |
| `--stdin-input` | Read file content from stdin and use the single path argument for baseline lookup and reporting. Intended for editor integrations. |
| `--substitute <ORIG=TEMP>` | Replace one host file with another for this invocation. Intended for mutation testing. Repeatable. |
| `--watch` | Run continuously, re-analysing on file changes. See [watch mode](#watch-mode). |
| `--list-codes` | List every analyzer issue code as JSON. |
| `-h`, `--help` | Print help and exit. |

The shared flags for reporting, fixing, and baselines are documented on the [reporting and fixing options](/fundamentals/shared-reporting-options/) page.

## Persistent cache

Use `--cache` to avoid repeating type analysis when the effective configuration and every loaded source, dependency, stub, patch, and extension-provided file are unchanged:

```sh
mago analyze --cache .mago-cache
```

If the path is an existing directory, or a non-existing path without an extension, Mago stores the cache in `<path>/analyze.bin`. Otherwise the path is used as the exact cache filename. Parent directories are created automatically.

The cache stores analyzer issues before ignore, baseline, and reporting filters are applied, so the same entry can be used with different reporting options. Cache corruption and I/O errors never prevent analysis: Mago warns, performs a full analysis, and attempts to replace the cache. The option conflicts with `--watch`, which already keeps incremental analysis state in memory.

## Reading from stdin

For editor and IDE integrations that pipe unsaved buffer content:

```sh
cat src/Example.php | mago analyze --stdin-input src/Example.php
```

Exactly one path argument is required. It is used as the logical (workspace-relative) file name for baseline matching and diagnostics. The path is normalised, so `./src/Example.php` is treated the same as `src/Example.php`. Conflicts with `--staged` and `--watch`.

## Substituting files

`--substitute ORIG=TEMP` replaces one host file with another for the duration of a single run without writing anything to disk. Designed for mutation-testing frameworks (Infection and friends) that produce a mutated copy of a source file and want the analyzer to evaluate the mutation against the rest of the project. If the analyzer reports a new error on the mutated file, the mutation can be killed without running the test suite.

```sh
mago analyze --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Rules:

- Both `ORIG` and `TEMP` must be absolute paths and both files must exist.
- `ORIG` must be a host file under one of your configured `paths`. Vendored or excluded files cannot be substituted.
- The flag can be repeated to substitute several files at once.
- Conflicts with `--stdin-input` and `--staged`.

Under the hood, `TEMP` is added to host paths and `ORIG` is added to excludes for this run, so cross-file type inference continues to see the mutation. Reported issues and baseline entries reference `TEMP` rather than `ORIG`.

## Watch mode

`--watch` keeps the analyzer running and re-runs whenever a PHP file in the workspace is created, modified, or deleted.

```sh
mago analyze --watch
```

### Automatic restart

The analyzer also watches the files that change its own configuration:

- The loaded `mago.toml` (or whichever config Mago picked up).
- The baseline file referenced from `[analyzer].baseline`.
- `composer.json` and `composer.lock`.

When any of those change, the analyzer restarts with the reloaded configuration. So you can edit `mago.toml`, save, and the next analysis pass uses the new settings without a manual restart.

If no configuration file exists when watch mode starts, the analyzer watches for the creation of any supported config file (`mago.toml`, `mago.yaml`, `mago.json`, …) and restarts when one appears.

Press **Ctrl+C** to stop watching.
