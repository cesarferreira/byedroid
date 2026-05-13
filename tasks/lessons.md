# Lessons

- For large TUI log buffers, cache the filtered visible-entry index and render only a viewport window on scroll redraws. Rebuild the index only when log content or filter state changes.
- Bound log ingestion work per tick so historical logcat backlogs cannot block terminal redraws or input handling.
