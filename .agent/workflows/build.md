---
description: Build a feature from specification through TDD, code review, clean warning-free build, and PR creation
---

# Feature Build Workflow (`/build`)

A structured, end-to-end development process for implementing a new feature in CookieCommander. It enforces high quality, TDD best practices, clean compiler checks, and comprehensive code reviews.

---

## Steps

### Step 1: Create Feature Branch

1. Ask the user for the feature name (use lowercase kebab-case, e.g., `file-sorting`).
2. Pull the latest from `main` and checkout the new branch:
```bash
git checkout main && git pull
git checkout -b build/{feature-name}
```

---

### Step 2: Spec, Design, and Planning

Before writing any code:
1. Write a short **Spec / Product Requirements Document (PRD)** outlining requirements and user stories.
2. Ask clarifying questions to the user if anything is ambiguous.
3. Write a high-level **Design Document** outlining the files and modules to be changed.
4. Derive an **Implementation Plan** consisting of small, incremental milestones (TDD checkpoints).
5. Save these documents under a feature folder inside the workspace root:
   - `features/{feature-name}/spec.md`
   - `features/{feature-name}/design.md`
   - `features/{feature-name}/plan.md`

Ensure the user is aligned on the spec and plan before proceeding to the code.

---

### Step 3: Implement Functional Tests (TDD)

For the current milestone:
1. Write **functional tests** verifying the feature's behavior according to the spec *before* writing the implementation.
2. Focus tests on actual business logic and user actions.
3. **Best Practice**: Do not test mocks or infrastructure. Keep assertions focused on concrete outputs or state transitions.
4. Place tests in appropriate Rust locations (e.g. integration tests in `tests/` or functional modules).

---

### Step 4: Code Implementation & Unit Testing

1. Write the minimal implementation code to make the functional tests pass.
2. Write granular **unit tests** in the implementation files (using `#[cfg(test)] mod tests`) to cover helper functions, data structures, and edge cases.
3. Run tests using `cargo test` to verify.

---

### Step 5: Iterative Commits

After completing each milestone chunk and verifying that all tests pass:
1. Commit the code to the branch:
```bash
git add -A
git commit -m "feat({feature-name}): [Milestone Name]

- [Detail 1]
- [Detail 2]

Tests: All passing (X unit, Y functional)"
```

---

### Step 6: Code Review

Once all milestones of the feature are complete:
1. Run the code review workflow:
   ```
   Run workflow: .agent/workflows/code-review.md
   ```
2. Save the generated review report to the feature's folder:
   `features/{feature-name}/codereview.md`

---

### Step 7: Fix Issues

1. Analyze the issues identified in `features/{feature-name}/codereview.md`.
2. Fix all **Critical** (🔴) and **High** (🟠) issues immediately.
3. Re-run tests to ensure stability.

---

### Step 8: Clean and Verify Build

Before final delivery, run a complete clean build and check for warnings. Warnings must be treated as errors:
```bash
# Clean previous build artifacts
cargo clean

# Build all targets and treat warnings as errors
RUSTFLAGS="-D warnings" cargo build --all-targets --all-features

# Run the test suite
cargo test
```
**Gate**: Zero compiler warnings, zero clippy warnings, and all tests passing.

---

### Step 9: Create Pull Request Description

1. Run the PR message workflow:
   ```
   Run workflow: .agent/workflows/create-pr-message.md
   ```
2. Copy the generated template.
3. Push the branch and create a PR:
```bash
git push origin build/{feature-name}

# If GitHub CLI is available:
gh pr create --title "feat({feature-name}): [Title]" --body "[PR Message content]" --base main --head build/{feature-name}
```
If GitHub CLI is not available, push the branch and print the PR description and the GitHub PR link so the user can easily open it in the browser:
`https://github.com/MatiGoldberg/CookieCommander/pull/new/build/{feature-name}`
