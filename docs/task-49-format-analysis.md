# Task-49: Data Format Analysis - CSV vs Custom Pipe Format

## Current State

We currently use a custom pipe-delimited format:
```
1|open|Simple title
2|done|Title with escaping|Description with \| pipe and \\ backslash|3
```

This requires **62 lines of custom parsing code**:
- `escape()` - 3 lines
- `unescape()` - 16 lines  
- `split_unescaped()` - 27 lines
- Plus usage throughout read/write functions

## The Problem

1. We've reinvented CSV with custom escaping logic
2. More code to maintain and test
3. Potential for edge case bugs
4. Violates YAGNI principle - we're maintaining complexity we don't need

## Options Analysis

### Option 1: Use the `csv` crate ⭐ RECOMMENDED

**Format Example:**
```csv
1,open,"Simple title",,
2,done,"Title with, comma","Description with | pipe",3
```

**Pros:**
- Reduces ~62 lines of custom code to ~10 lines
- Battle-tested library (100M+ downloads)
- Zero dependencies (pure Rust std)
- Handles all edge cases correctly
- Standard format - works with Excel, csvkit, etc.
- Still git-friendly (line-based diffs)
- Still human-readable

**Cons:**
- Adds one dependency
- More visual noise (quotes when fields contain commas/quotes)
- Migration effort (but small - write converter)

**Code Reduction:**
```rust
// Current: 62 lines of escape/unescape/split_unescaped
// With csv crate: ~5-10 lines
let mut rdr = csv::ReaderBuilder::new()
    .has_headers(false)
    .from_reader(file);
```

### Option 2: JSON Lines (.jsonl)

**Format Example:**
```json
{"id":"1","status":"open","title":"Simple title"}
{"id":"2","status":"done","title":"Title","description":"Description","pain_count":3}
```

**Pros:**
- Structured, self-documenting
- Easy to extend with new fields
- No escaping issues

**Cons:**
- Much less human-readable
- More verbose (2-3x size)
- Harder to grep/scan visually
- Still requires serde dependency (already have it though)

### Option 3: Keep Custom Format

**Pros:**
- No migration needed
- Already working
- Minimal visual noise

**Cons:**
- Maintaining 62 lines of custom parsing code
- Risk of edge case bugs
- Violates YAGNI and simplicity principles
- Not a standard format

### Option 4: TOML/YAML

**Cons:**
- Multi-line format = bad git diffs
- Not line-based (can't append easily)
- Harder to parse incrementally

## Recommendation: Use CSV Crate

**Reasoning:**
1. **Simplicity**: 62 lines → ~10 lines. Massive reduction in code to maintain.
2. **YAGNI**: Stop maintaining custom parsing when std solution exists.
3. **Standard Format**: CSV is universal. Tools already exist.
4. **Git-Friendly**: Still line-based, still readable diffs.
5. **Zero Extra Dependencies**: The `csv` crate has no dependencies itself.
6. **Battle-Tested**: Used by millions, all edge cases handled.

**What We Keep:**
- Line-based format ✓
- Git-friendly diffs ✓
- Human-readable ✓
- Append-friendly ✓

**What We Improve:**
- Less code to maintain
- Standard format
- Robust edge case handling
- Ecosystem compatibility

## Migration Plan

1. **Write test with CSV format** (TDD!)
2. **Add csv crate dependency** to Cargo.toml
3. **Implement read/write with CSV** - replace escape/unescape/split_unescaped
4. **Write migration tool** `.knecht/tasks` → CSV format
5. **Update tests** to expect CSV format
6. **Remove custom parsing code**
7. **Update documentation**

## Decision

**Moving forward with Option 1: CSV crate**

The pain of maintaining custom parsing code is real, and using a standard library aligns perfectly with the project's simplicity and YAGNI principles.