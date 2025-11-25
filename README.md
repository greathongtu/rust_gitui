# rust_gitui

A minimal, Lazygit-like TUI built with Ratatui. It presents four panels:
- Left column (top to bottom): `Status` (changed files), `Branches`, `Commits`
- Right column: `Diff` (shows `git diff`)

You can navigate panels, stage/unstage files, checkout branches/commits, commit/reword, reset to a commit, merge/rebase branches, and handle push/pull — all via keyboard.

## Keymap

Global
- `q` Quit
- `p` Pull (`git pull`)
- `P` Push (`git push`). If push is rejected, a force-push popup appears.
- `1` Focus `Status`
- `2` Focus `Branches`
- `3` Focus `Commits`
- `4` Focus `Diff`
- `j` / `Down` Scroll down in the focused panel
- `k` / `Up` Scroll up in the focused panel

Status panel (`CurrentPanel::Status`)
- `Space` Stage/Unstage the selected file
- `a` Stage/Unstage all (toggle)
- `A` Amend last commit without editing message (`git commit --amend --no-edit`)

Branches panel (`CurrentPanel::Branch`)
- `Space` Checkout selected branch
- `n` Open "new/checkout branch" popup (type name to create or track remote)
- `M` Merge selected branch into current (`git merge --no-edit`)
  - If conflicts are detected, a conflict popup is shown
- `r` Rebase current branch onto selected (`git rebase`)
  - If conflicts are detected, a conflict popup is shown

Commits panel (`CurrentPanel::Commit`)
- `Space` Checkout selected commit (detached HEAD)
- `d` Drop selected commit (rebase onto its parent)
- `R` Reword the last commit (opens commit popup prefilled with HEAD message)
- `g` Open reset popup to reset to the selected commit (`soft/mixed/hard`)

Diff panel (`CurrentPanel::Diff`)
- Read-only; scroll with `j/k` or arrows

Commit popup (`New` or `Edit` modes)
- Open (new commit): `c`
- Open (reword last commit): `R` from `Commits`
- Type commit message (first line is subject, subsequent lines are body)
- `Enter` Submit (create commit or reword based on mode)
- `Esc` Cancel
- Editing keys: `Char`, `Backspace`, `Tab`

Reset popup (for `git reset <mode> <hash>`)
- Open: `g` in `Commits` on selected commit
- Select mode: `j/Down` next, `k/Up` prev, `Ctrl-n` next, `Ctrl-p` prev
- Quick select: `s` soft, `m` mixed (default), `h` hard
- Confirm: `Space`
- Cancel: `Esc`

Push force popup (when `P` fails)
- Confirm force-push: `Enter` or `Space` (`git push --force-with-lease`)
- Cancel: `Esc`

Conflict popup
- Close: `Esc` or `Enter`

## TODOs

- todo: output command
- render_xx_popup with same logic
- porcelain v2？
- push/pull error message not to stdio::null
- if not git repo?
- better diff
- handle_events judge active popup
- unit test
