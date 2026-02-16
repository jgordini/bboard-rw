## Change

Resolved the 3 remaining Stylelint errors in `style/main.scss`.

## Fixes Applied

- Moved Google Fonts `@import` to the top of the file to satisfy import ordering.
- Replaced deprecated `clip: rect(...)` with `clip-path: inset(50%)` in `.sr-only`.
- Replaced deprecated `word-break: break-word` with `overflow-wrap: anywhere`.

## Verification

Lint command:

```bash
cd /Users/jeremy/repos/bboard-rw/end2end
npx stylelint ../style/main.scss --config /tmp/stylelint-config.<random>.json
```

Result: exit code `0` (no Stylelint errors).
