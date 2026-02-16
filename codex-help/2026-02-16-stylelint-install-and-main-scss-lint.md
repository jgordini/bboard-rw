## Change

Installed Stylelint tooling in the existing Node workspace at `end2end/`:

- `stylelint`
- `stylelint-config-standard-scss`

## Commands Run

```bash
cd /Users/jeremy/repos/bboard-rw/end2end
npm install --save-dev stylelint stylelint-config-standard-scss
```

Lint command used (with temporary config file):

```bash
npx stylelint ../style/main.scss --config /tmp/stylelint-config.<random>.json
```

Config content:

```json
{"extends":["stylelint-config-standard-scss"]}
```

## Result

- `style/main.scss` lint failed with **72 errors** (`71` auto-fixable via `--fix`).
