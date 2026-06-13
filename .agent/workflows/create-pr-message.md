---
description: Create a comprehensive Pull Request message based on the current changes in CookieCommander
---

# Create PR Message Workflow

Generates a structured Pull Request message by analyzing the changes between the current branch/state and the `main` branch.

## Steps

### 1. Fetch Latest Changes (Optional but Recommended)
```bash
git fetch origin main || echo "Could not fetch origin/main, proceeding with local main"
```

### 2. Identify Changed Files
Get the list of files that have changed compared to the `main` branch.
```bash
git diff --name-status main
```

### 3. Analyze Content Changes
Get the detailed diff to understand *what* changed.
```bash
git diff main
```

### 4. Analyze Test Coverage
Identify new or modified tests.
```bash
# Find changed test files
git diff --name-only main | grep -i "test" || echo "No test files changed"

# Count tests in new/changed files (heuristic)
# Look for #[test] attributes added in the diff
git diff main | grep -E "^\+.*#\[test\]" | wc -l || echo "0"
```

### 5. Generate PR Message
Based on the diffs and analysis above, generate the PR message in the following format:

**Output Format:**

1. **Title**: `feat({feature-name}): [A concise and descriptive title for the PR]`
2. **PR Message**:
```markdown
## 🚀 Features Added
- [Feature 1]
- [Feature 2]

## 🛠️ Code & Design Changes
- [Change 1]
- [Change 2]

## 🧪 Tests Status
- **Status**: [Green/Passing/WIP]
- **Count**: [Number] tests added/modified
  - [X] Unit Tests
  - [Y] Functional/Integration Tests

## 📄 New Documents
- [Features folder files](link)

## 🤖 New Commands
- [Any new scripts or tools]

---
```

**Instructions for Generation:**
- **Final Output**: Always provide the final PR message (including the title) inside a markdown code block (using triple backticks) so it can be easily copied and pasted into GitHub.
- **Features**: Summarize the "what" and "why" of the changes.
- **Code/Design**: Mention significant refactors, styling updates, or architectural updates.
- **Tests**: Be specific about the *count* and *types* of tests based on the file paths or test definitions.
- **Docs**: List any new markdown or documentation files (like the PRD, spec, or design documents under `features/{feature-name}/`).
- **Commands**: List any new scripts or tools added to the repository.
- Use emojis and dividers exactly as requested.
