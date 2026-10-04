# Corpus sample parse failures (2026-10-04)

Ken’s directory-mode `newagt info` sample scan on conure (atlas user). **Do not commit corpus paths**; this report uses basenames only.

## Sample file

- Machine: `conure`
- Scan output: `agt-sample.tsv` in the atlas user home directory (Ken referenced `~/.agt-sample.tsv`; on this host the file is **`~/agt-sample.tsv`**, not under a dot-prefixed name).

## TSV shape

| Field | Pattern |
|-------|---------|
| Column 1 | `path: <absolute path>` |
| Column 2 | `frames: <N>` or `frames: -` when parse failed |
| Column 3 | `sections: SenSect,TgtSect` (etc.) or `sections: -` on fail |
| Column 4 | `parse: ok` or `parse: fail` |

Tab-separated; one row per `.agt` file.

## Counts (pre-fix binary)

| Metric | Value |
|--------|------:|
| Total rows | 100 |
| `parse: ok` | 57 (57%) |
| `parse: fail` | 43 (43%) |

**Ok rows:** 56× `SenSect,TgtSect`, 1× `SenSect` only.  
**Fail rows:** all 43 had `frames: -` and `sections: -` (parse never reached summarization).

Profile: default **`agtj`**. Same failures with **`pdf1999`**.

## Failure analysis (43 files, all sampled)

Every failure was a **hard parse error** (`newagt validate` → `validation: not loadable`, category **`lex_error`**), not a validate warning counted as fail.

| Category | Count | Description |
|----------|------:|-------------|
| Signed integer / `PixLoc` | 43 | `unexpected character '-'` at **column 16** on a `PixLoc` line: negative pixel X (e.g. `PixLoc` then `-3` and a positive Y). |
| Other lexer | 0 | — |
| Unknown section / strict profile | 0 | — |
| Encoding / IO | 0 | — |
| Huge file / timeout | 0 | — |

Representative basenames (not exhaustive): `avco15421_2004.agt` (2 rows in sample), many `SC8300HD_Pickett_*` and `SC8300HD_Pickett_1010_1020_*` variants.

### Example pattern (structural, no corpus path)

```text
        PixLoc	-3 197
```

Column 16 is the `-` starting a **signed integer** token. The lexer accepted `-` only when starting a **real** literal (optional signs then digits and `.`); bare signed integers like `-3` were rejected after `try_lex_real` and `try_lex_integer` both declined.

## Ok files (contrast)

Sample ok files parse and validate as **`validation: loadable`**, often with informational **`missing_optional_section`** (no `PrjSect`) — expected for this corpus slice, not batch `info` failures.

## Fix (newagt-core)

**Change:** `try_lex_integer` in `crates/newagt-core/src/lex.rs` — optional leading `+`/`-` before digit run, consistent with real literal sign handling.

**Test:** `signed_integer_literals` unit test (`-3 197 +42`).

**Post-fix on same 100-file sample:** 100/100 `newagt validate` exit 0; former 43 failures load with thousands of frames (e.g. one former fail → 300 frames, `SenSect`/`TgtSect` present).

## Tolerant parsing gaps (for Ken)

- None of this sample’s failures were “wild but loadable” COMMENT/KEYWORD issues; they were a **single lexer gap** for signed integers in field values (especially `PixLoc` near image edges).
- After the fix, this **100-file slice is 100% parse-ok** under default profile; wider corpus scans may still surface other patterns (unknown keywords, validation warnings, etc.).
