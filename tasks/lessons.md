# Lessons

- For large TUI log buffers, cache the filtered visible-entry index and render only a viewport window on scroll redraws. Rebuild the index only when log content or filter state changes.
