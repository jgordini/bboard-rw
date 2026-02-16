## Change

Removed rounded corners from board sort tab buttons by adding:

```scss
.sort-tab {
  border-radius: 0;
}
```

## Why

`button.sort-tab` also uses `btn btn-secondary`, and `.btn-secondary` applies `border-radius: var(--radius)`.
The override ensures active sort tabs render with square corners.

## File

- `style/main.scss`
