# LiveSpeedTimer
LiveSpeedTimer is a lightweight program that runs in the terminal for quick and easy split timing.

## Usage
When the program is started, a `cfg.yaml` containing various settings will be created if one does not already exist.
This file contains keybindings corresponding to crossterm KeyCode enums, which may be modified.

---

Initially, default keybindings are generated as follows:

- `s` starts the timer if stopped, and stops if started.
- <code>&nbsp;</code>&nbsp;creates a new split at the current time.
- `r` stops and resets the timer and removes all splits.
- `Esc` closes the program.
