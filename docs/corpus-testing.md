# Testing against a private AGT corpus

Ken’s imagery ground-truth files live on a **local RAID or bulk storage mount** you choose—not in this repository. Typical layouts are **intentionally messy**: `.agt` files sit at irregular depths, mixed with datasets, exports, and non-AGT assets. Use that tree for **real-world** parser and CLI checks; keep checked-in fixtures under `tests/fixtures/` for CI.

Some datasets also follow the **classic per-sensor layout** (`$DATASET_ROOT/$SENSOR/arf/` + `agt/`, basename pairing between `.arf` and `.agt`). That convention is documented in [arf-frame-authority.md](arf-frame-authority.md#dataset-layout-classic-arfagt-pairing); use `newagt pairs --dataset-root /path/to/dataset` to list pairs locally without parsing ARF. Messy recursive scans (`newagt info` on a broad mount) and classic pairing scans answer different questions—use whichever matches your tree.

Set a shell variable for the corpus root (example only; pick whatever path is stable on your machine):

```bash
export CORPUS_ROOT='/your/raid/mount'   # do not commit this value
```

## Expectations

- **Layout:** No single “corpus root” convention; recursive discovery is required.
- **Compliance:** Many files load successfully while diverging from [agt-format-spec-1992.md](agt-format-spec-1992.md); see [agt-real-world-format.md](agt-real-world-format.md).
- **Scale:** A full recursive scan can mean **thousands** of `.agt` files. Plan for minutes of I/O and parsing unless you cap the batch.
- **Privacy:** Do **not** commit customer paths, filenames, mount points, or scan output into git. Keep reports local (redirect to files under `/tmp` or your home directory).

## Prerequisites

- Read access to `$CORPUS_ROOT` on the machine where your data is mounted.
- A release build of the CLI: `cargo build --release -p newagt-cli`.

Quick access check (run locally; do not paste output into the repo):

```bash
test -r "$CORPUS_ROOT" && echo ok
```

Approximate file count (still **do not** commit counts or paths into docs):

```bash
find "$CORPUS_ROOT" -name '*.agt' 2>/dev/null | wc -l
```

## Directory mode: `newagt info` (M10)

When `PATH` is a directory, `newagt info` **walks subdirectories by default** (`--recursive` is true) and prints **one tab-separated summary line per `.agt`**: path, frame count, sections present, parse ok/fail.

Single-file mode is unchanged (multi-line human summary).

```bash
# Sample scan: first N files after sorted walk (recommended for huge trees)
newagt info "$CORPUS_ROOT" --limit 50

# Full tree (can take a long time on a large mount)
newagt info "$CORPUS_ROOT" > /tmp/newagt-corpus-info.tsv

# Non-recursive: only `.agt` in the top level of PATH
newagt info "$CORPUS_ROOT/subtree" --recursive=false
```

### Without `--limit`

If you need a fixed sample without processing the whole directory listing in Rust:

```bash
find "$CORPUS_ROOT" -name '*.agt' 2>/dev/null | sort | head -n 20 | xargs -r newagt info
```

(`xargs` invokes `info` once **per file** in that pipeline; use `--limit` on a directory when you want one sorted walk and a single batch.)

## Suggested workflows

| Goal | Approach |
|------|----------|
| Smoke test after a code change | `newagt info "$CORPUS_ROOT" --limit 20` and spot-check parse ok/fail ratio |
| Compare profiles | Same sample with `--profile agtj` vs `--profile pdf1999` |
| Deep validation on a sample | Pipe paths from `find … \| head` into `newagt validate` |
| Full corpus inventory | `newagt info "$CORPUS_ROOT"` redirected to a **local** TSV; analyze offline |
| CI / public repo | Use checked-in fixtures under `tests/fixtures/` only |

## Redirecting output

Keep corpus-derived paths off the terminal history you might paste into issues:

```bash
newagt info "$CORPUS_ROOT" --limit 500 > ~/agt-corpus-sample.tsv 2> ~/agt-corpus-sample.err
newagt validate "$CORPUS_ROOT/example.agt" --format json > /tmp/one.json
```

## Related docs

- [agt-implementation-plan.md](agt-implementation-plan.md) — milestone status and post-M10 optional work
- [agt-real-world-format.md](agt-real-world-format.md) — wild-format patterns seen in training data
