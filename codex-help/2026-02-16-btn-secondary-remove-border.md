## Change

Removed the border from `.btn-secondary`.

## File

- `style/main.scss`

## Detail

Updated:

```scss
border: 1px solid var(--border);
```

to:

```scss
border: none;
```

Also removed hover-only `border-color` override since border is disabled.
