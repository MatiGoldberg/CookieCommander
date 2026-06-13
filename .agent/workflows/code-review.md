---
description: Run a comprehensive Rust code review on the CookieCommander project
---

# Rust Code Review Workflow

Runs a rigorous code review following idiomatic Rust guidelines, async/concurrency checks, safety audits, and Clippy validation.

## Prerequisites

- Ensure you're on the correct branch
- The project should build successfully (`cargo build`)

## Steps

### 1. Read the Code Review Skill

First, read the skill instructions:
```
View file: .agent/skills/code-review/SKILL.md
```

### 2. Determine Review Scope

Ask the user (or inspect the context):
- **Diff review**: Analyze changes against `main` branch (default for features)
- **Full project review**: Analyze the entire codebase

### 3. For Diff-Based Review

```bash
git diff main --name-only -- '*.rs'
```

Then get the full diff:
```bash
git diff main --unified=5 -- '*.rs'
```

### 4. For Full Project Review

```bash
find src -name "*.rs" -type f
```

### 5. Analyze Code & Run Automated Checks

Go through each file/change systematically using the checklist from the skill.

Check for common Rust issues:
```bash
# Check for cargo clippy warnings (treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Check for unwrap/expect/panic usage in src (potential panic paths)
grep -rn "unwrap()\|expect(\|panic!" src --include="*.rs" || echo "No unwrap/expect/panic found"

# Check for unsafe blocks
grep -rn "unsafe {" src --include="*.rs" || echo "No unsafe blocks found"

# Check for TODO/FIXME/HACK comments
grep -rn "TODO\|FIXME\|HACK" src --include="*.rs" || echo "No TODO/FIXME found"
```

### 6. Generate Report

Create the report file.
- For feature branches: Save the report to the feature's folder:
  `features/{feature-name}/codereview.md`
- For general/ad-hoc reviews: Save to:
  `docs/code-reviews/codereview_{YYYY-MM-DD}_{HH-MM}.md`

Include:
1. Executive Summary
2. Issues Summary Table (ordered by severity: Critical, High, Medium, Low, Info)
3. Detailed Findings with code snippets and proposed fixes
4. Recommendations
5. List of files reviewed

### 7. Summarize to User

Provide a brief verbal summary of:
- Total issues found by severity
- Critical/High items requiring immediate attention (which MUST be fixed before completing the feature)
- General assessment of code quality and Idiomatic Rust status
