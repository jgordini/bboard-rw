## Change

Ran Stylelint auto-fix on `style/main.scss` using standard SCSS rules.

## Command

```bash
cd /Users/jeremy/repos/bboard-rw/end2end
npx stylelint ../style/main.scss --config /tmp/stylelint-config.<random>.json --fix
```

Config content:

```json
{"extends":["stylelint-config-standard-scss"]}
```

## Result

- Auto-fix changed `style/main.scss` with diff stats:
  - `38 insertions(+), 37 deletions(-)`
- Lint now reports 3 remaining errors:
  - `no-invalid-position-at-import-rule` at line 54
  - `property-no-deprecated` (`clip`) at line 133
  - `declaration-property-value-keyword-no-deprecated` (`word-break: break-word`) at line 1830
