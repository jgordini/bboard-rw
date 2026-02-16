## Change

Added a scoped 1px border for buttons inside `.detail-card-actions`.

## File

- `style/main.scss`

## Detail

Added:

```scss
.detail-card-actions {
  .btn {
    border: 1px solid var(--border);
  }
}
```

This restores borders for detail action buttons after global `.btn-secondary` border removal.
