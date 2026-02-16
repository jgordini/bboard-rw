## Change

Made admin export status hidden by default and visible only when active.

## File

- `style/main.scss`

## Detail

Updated `.admin-export-status`:

```scss
display: none;

&.active {
  display: block;
}
```

This works with the existing Leptos class toggle (`class:active`) that becomes true after an export action button click.
