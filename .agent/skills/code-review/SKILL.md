---
name: Rust Code Review
description: Comprehensive code review process for Rust projects following idiomatic patterns, safety guidelines, and Tokio async best practices
---

# Rust Code Review Skill

A rigorous code review process that analyzes git differences or the entire project codebase, producing comprehensive impediment reports based on Rust safety, idiomatic patterns, API design, and asynchronous programming guidelines.

---

## Overview

This skill enables systematic code review for Rust projects. It checks:
- **Diff-based reviews**: Changes between the current branch and the `main` branch.
- **Full project reviews**: Complete codebase quality audits.

The output is a detailed markdown report saved to `features/{feature-name}/codereview.md` (or standard reviews at `docs/code-reviews/codereview_{date}_{time}.md`).

---

## Issue Classification

### Severity Levels

| Level | Description | Examples |
|-------|-------------|----------|
| 🔴 **Critical** | Security vulnerabilities, data corruption, panic loops, compile/link errors | Unchecked `unsafe` code, stack overflow, deadlocks |
| 🟠 **High** | Functional bugs, runtime panic risks, significant memory leaks, blocking executor | Unhandled `unwrap()`/`expect()` on user inputs, blocking async loop, logic bugs |
| 🟡 **Medium** | Performance issues, maintainability, testability, poor error handling | Unnecessary `.clone()`, broad trait bounds, missing tests, swallow errors |
| 🔵 **Low** | Style, conventions, minor clippy warnings, documentation | Non-standard naming, missing docstrings, minor efficiency improvements |
| ⚪ **Info** | Suggestions, idiomatic refactoring, modern Rust syntax | Suggesting `.entry()` in HashMaps, minor refactor with combinators |

---

## Rust Code Review Checklist

### 1. Safety & Correctness
- [ ] **No `unsafe`**: Ensure there are no `unsafe` blocks unless absolutely necessary, fully documented, and sound.
- [ ] **Panic Prevention**: Avoid `unwrap()` and `expect()` in runtime code unless it is mathematically proven to be safe. Use pattern matching (`let Some(x) = ... else { ... }`, `match`) or propagate errors using `?`.
- [ ] **Error Propagation**: Use idiomatic error handling (e.g. `anyhow` or custom `thiserror` enums) and propagate errors.
- [ ] **Resource Cleanup**: Ensure resources (file handles, terminal state via `crossterm`) are cleaned up properly when errors occur (RAII pattern).
- [ ] **Edge Cases**: Validate boundary inputs, empty slices, index out-of-bounds risks, and integer overflow/underflow (using `checked_add`, `saturating_add`, etc., where relevant).

### 2. Async & Concurrency (Tokio & Crossterm)
- [ ] **Never Block the Async Executor**: Ensure no synchronous block/blocking IO (`std::fs::File`, `std::thread::sleep`) is run on the Tokio runtime without `tokio::task::spawn_blocking` or async counterparts.
- [ ] **Locks & Mutexes**: Prefer Tokio's async locks (`tokio::sync::Mutex`) when holding the guard across `await` points. For CPU-only synchronization, prefer `std::sync::Mutex` or `parking_lot` for better performance.
- [ ] **Select Loops & Cancellation**: Verify that `tokio::select!` branches are cancellation-safe (e.g. state isn't partially modified and lost on drop).
- [ ] **Channel Bounds**: Use bounded channels (e.g. `tokio::sync::mpsc::channel` with capacity) to prevent unbounded memory growth.

### 3. Performance & Memory
- [ ] **Avoid Unnecessary Cloning**: Minimize `.clone()`, `.to_string()`, `.to_owned()` on complex structures. Pass variables by reference (`&str`, `&[T]`, `&Path`) instead of owned types.
- [ ] **Iterator Optimizations**: Prefer lazy iterators (`map`, `filter`, `fold`) over manual loops when it is more readable and optimal.
- [ ] **Allocations**: Avoid allocating memory in tight loops. Pre-allocate collections with `Vec::with_capacity` if the size is known.

### 4. API Design & Idiomatic Rust
- [ ] **Standard Traits**: Implement standard traits where appropriate: `Debug`, `Clone`, `Default`, `PartialEq`, `Eq`.
- [ ] **Naming Conventions**: Follow Rust API guidelines: `snake_case` for files/folders/variables/functions, `PascalCase` for structs/enums/traits, `SCREAMING_SNAKE_CASE` for constants/statics.
- [ ] **Clippy & Formatting**: The code must compile without warning. Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt -- --check`.
- [ ] **Public vs Private**: Expose only the minimum API required. Keep fields private by default.

### 5. Test Quality
- [ ] **No Mocks unless Necessary**: Focus functional/integration tests on actual features and behaviors, not internal implementations.
- [ ] **Unit Test Coverage**: Add unit tests in the same module using `#[cfg(test)] mod tests` to cover internal logic and boundary conditions.
- [ ] **Integration Tests**: Place high-level scenario tests in the `tests/` folder.

---

## Report Template

The generated report should follow this structure:

```markdown
# Rust Code Review Report

**Feature / Branch:** [Branch Name]  
**Date:** [YYYY-MM-DD HH:MM]  
**Scope:** [Diff-based / Full Project]  
**Reviewer:** AI Code Review Agent  

---

## Executive Summary

[Brief overview of code quality, major findings, and recommendations]

---

## Issues Summary

| # | Severity | Category | File | Location | Brief Description |
|---|----------|----------|------|----------|-------------------|
| 1 | 🔴 Critical | Concurrency | `src/main.rs` | Line 42 | Async Mutex held across await point deadlock |
| 2 | 🟠 High | Correctness | `src/ui.rs` | Line 85 | Panic risk: unhandled .unwrap() on user input |
| ... | ... | ... | ... | ... | ... |

**Total Issues:** X (Y Critical, Z High, W Medium, V Low)

---

## Detailed Findings

### Issue #1: [Title]

**Severity:** 🔴 Critical / 🟠 High / 🟡 Medium / 🔵 Low  
**Category:** [Safety / Concurrency / Performance / Error Handling / Correctness / Style]  
**File:** `[Path]`  
**Location:** Line [Line Number]  

**Description:**  
[Detailed explanation of the issue and why it is problematic]

**Code with Issue:**
```rust
// Problematical code
```

**Proposed Fix:**
```rust
// Fixed code
```

---

## Recommendations

[Summary of recommendations for improving overall codebase quality]

---

## Appendix

### Files Reviewed
- `src/main.rs`
- ...
```
