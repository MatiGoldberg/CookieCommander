# Spec: Prettify JSON Feature

## Overview
CookieCommander users often view minified or single-line JSON files (e.g., config files, API responses). Reading them in a single line is difficult. This feature allows users to format (prettify) a single-line JSON file by pressing `p` while in the file viewer. If they make changes, they will be prompted to save them upon exiting the viewer.

## Requirements
1. **Trigger Condition**:
   - The user is in `FileViewer` mode.
   - The open file is a JSON file (by `.json` extension, case-insensitive).
   - The JSON file is minified/all on one line (non-empty content, having exactly 1 line or 1 non-empty line).

2. **Prettification Execution**:
   - The user presses the `p` key.
   - The application checks the file size against a configurable limit (default: 512 KB).
   - If the file is within the size limit, the application parses the JSON and reformats it to an indented form.
   - The viewer updates to display the reformatted JSON.
   - The file viewer state is marked as dirty (unsaved changes).
   - If the file exceeds the limit, a status message is shown in the viewer: `"File too large to prettify (max {limit} KB)"`.

3. **Configurability**:
   - The size limit is configurable in `config.json` via a new key `max_prettify_size_kb`.
   - The default value is `512` (representing 512 KB).

4. **Exit and Save Flow**:
   - When the user attempts to exit the file viewer (via `Esc` or `q`):
     - If the file has been prettified (is dirty), the user is prompted: `Save changes? (y/n)`.
     - Pressing `y` or `Y` writes the prettified content back to the file on disk using the VFS, clears the dirty flag, and closes the viewer.
     - Pressing `n` or `N` discards the changes and closes the viewer.
     - Pressing `Esc` or `q` cancels the exit prompt and returns to the file viewer.
     - If the file was not prettified, the viewer closes immediately as before.

5. **Extensibility**:
   - The formatting logic must use a common trait or abstraction (`Prettifier`) so that in the future, we can easily add prettifiers for other formats (like HTML, XML, YAML, etc.) without altering the core state machine.

## User Stories
- **As a developer**, when I open a minified config file in CookieCommander, I want to press `p` to format it so that I can read its structure easily.
- **As a developer**, if I format a JSON file to inspect it, I want the option to save the formatted version back to disk when I close the viewer, or discard the format if I only wanted to read it temporarily.
- **As a developer**, I want to configure the maximum size of files that can be prettified to prevent the application from hanging on multi-megabyte JSON logs.
