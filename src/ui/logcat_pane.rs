//! Full-screen logcat pane — rustycat-style rendering with horizontal scrolling.
//!
//! Column layout:  [timestamp 12] [TAG right-aligned 23] [LEVEL 3] [unwrapped message]
//! Tags are suppressed on repeated runs (rustycat style).
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use crate::app::App;
use crate::modules::logcat::{level_style, tag_color};

const TAG_WIDTH: usize = 23;
const TS_WIDTH: usize = 12;

pub fn render(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    let paused = app.logcat_paused;
    let running = app.logcat_running;
    let scrolled = app.log_scroll > 0;

    let title = if app.exclude_focused {
        " Logcat  [EXCLUDE] "
    } else if app.filter_focused {
        " Logcat  [FILTER] "
    } else if app.level_picker_open {
        " Logcat  [LEVELS] "
    } else if paused {
        " Logcat  PAUSED "
    } else if scrolled {
        if app.new_lines_while_scrolled > 0 {
            &format!(
                " Logcat  {} logs behind — End/G to jump to bottom ",
                app.new_lines_while_scrolled
            )
        } else {
            " Logcat  ↑scrolled — End to tail "
        }
    } else {
        " Logcat "
    };

    let border_style = Style::default().fg(if app.exclude_focused {
        Color::Rgb(255, 150, 100)
    } else if app.level_picker_open {
        Color::Rgb(244, 162, 97)
    } else if app.filter_focused {
        Color::Cyan
    } else {
        Color::Rgb(50, 50, 70)
    });

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut header_lines: Vec<Line> = Vec::new();
    let level_summary = app.current_level_filter_summary();
    if level_summary != "all" {
        header_lines.push(Line::from(vec![
            Span::styled(" levels: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                level_summary,
                Style::default()
                    .fg(Color::Rgb(244, 162, 97))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if app.filter_focused || !app.filter_input.is_empty() {
        header_lines.push(if app.filter_focused {
            Line::from(vec![
                Span::styled(" filter: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    app.filter_input.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("█", Style::default().fg(Color::Cyan)),
            ])
        } else {
            Line::from(vec![
                Span::styled(" filter: ", Style::default().fg(Color::DarkGray)),
                Span::styled(app.filter_input.clone(), Style::default().fg(Color::Cyan)),
            ])
        });
    }
    if app.exclude_focused || !app.exclude_input.is_empty() {
        header_lines.push(if app.exclude_focused {
            Line::from(vec![
                Span::styled(" exclude: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    app.exclude_input.clone(),
                    Style::default()
                        .fg(Color::Rgb(255, 150, 100))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("█", Style::default().fg(Color::Rgb(255, 150, 100))),
            ])
        } else {
            Line::from(vec![
                Span::styled(" exclude: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    app.exclude_input.clone(),
                    Style::default().fg(Color::Rgb(255, 150, 100)),
                ),
            ])
        });
    }

    // New lines notification when scrolled up
    if scrolled && app.new_lines_while_scrolled > 0 {
        header_lines.push(Line::from(vec![
            Span::styled(" ▼ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} logs behind", app.new_lines_while_scrolled),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" — Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "End",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to jump", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let header_h = header_lines.len() as u16;
    let visible_rows = inner.height.saturating_sub(header_h).max(1) as usize;

    let total = app.log_lines.len();
    let viewport = app.log_viewport_window(app.log_scroll, visible_rows);
    app.log_scroll = viewport.scroll_from_bottom;

    let mut collected: Vec<Line> = Vec::new();
    let mut last_tag = viewport
        .newer_context_index
        .and_then(|idx| app.log_lines.get(idx))
        .map(|entry| entry.tag.clone())
        .unwrap_or_default();

    for idx in viewport.indices.iter().rev() {
        let entry = &app.log_lines[*idx];
        let lc = &entry.level;
        let lvl_color = level_style(lc);
        let tc = tag_color(&entry.tag, &mut app.tag_color_cache);
        let is_stack = entry.is_stack_trace;
        let crash_bg = entry.crash_start;
        let msg_fg = if crash_bg {
            Color::Rgb(255, 200, 200)
        } else {
            lvl_color
        };
        let msg_modifier = if crash_bg || is_stack || matches!(lc.as_str(), "E" | "F") {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };
        let lvl_style_span = if crash_bg {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(120, 0, 0))
                .add_modifier(Modifier::BOLD)
        } else {
            match lc.as_str() {
                "E" | "F" => Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                "W" => Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                _ => Style::default().fg(lvl_color).add_modifier(Modifier::BOLD),
            }
        };

        let show_tag = entry.tag != last_tag;
        let tag_display = if show_tag {
            right_pad(&entry.tag, TAG_WIDTH)
        } else {
            " ".repeat(TAG_WIDTH)
        };
        let tag_style = if crash_bg {
            Style::default()
                .fg(Color::Rgb(255, 180, 180))
                .bg(Color::Rgb(80, 0, 0))
        } else if show_tag {
            Style::default().fg(tc)
        } else {
            Style::default().fg(Color::Rgb(40, 40, 55))
        };
        let ts_style = if crash_bg {
            Style::default()
                .fg(Color::Rgb(255, 200, 200))
                .bg(Color::Rgb(80, 0, 0))
        } else {
            Style::default().fg(Color::Rgb(80, 80, 100))
        };
        collected.push(Line::from(vec![
            Span::styled(
                format!("{:<width$} ", entry.timestamp, width = TS_WIDTH),
                ts_style,
            ),
            Span::styled(tag_display, tag_style),
            Span::raw(" "),
            Span::styled(format!(" {} ", lc), lvl_style_span),
            Span::raw(" "),
            Span::styled(
                entry.message.clone(),
                Style::default()
                    .fg(msg_fg)
                    .bg(if crash_bg {
                        Color::Rgb(80, 0, 0)
                    } else {
                        Color::Reset
                    })
                    .add_modifier(msg_modifier),
            ),
        ]));
        last_tag = entry.tag.clone();
    }

    collected.reverse();
    let start = collected.len().saturating_sub(visible_rows);

    let header_h = header_h.min(inner.height);
    f.render_widget(
        Paragraph::new(header_lines),
        Rect {
            height: header_h,
            ..inner
        },
    );
    let log_area = Rect {
        y: inner.y + header_h,
        height: inner.height.saturating_sub(header_h),
        ..inner
    };
    if total == 0 {
        collected.push(Line::from(Span::styled(
            if running {
                " Waiting for log lines…"
            } else {
                " Press l to start logcat"
            },
            Style::default().fg(Color::DarkGray),
        )));
    }
    let max_scroll = collected
        .iter()
        .map(Line::width)
        .max()
        .unwrap_or(0)
        .saturating_sub(log_area.width as usize)
        .min(u16::MAX as usize) as u16;
    app.log_horizontal_max = max_scroll;
    // Keep the requested position when shorter logs enter the viewport.
    let horizontal_scroll = app.log_horizontal_scroll.min(max_scroll);
    f.render_widget(
        Paragraph::new(collected[start..].to_vec()).scroll((0, horizontal_scroll)),
        log_area,
    );
    if max_scroll > 0 {
        let mut state = ScrollbarState::new(max_scroll as usize + 1)
            .position(horizontal_scroll as usize)
            .viewport_content_length(log_area.width as usize);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
            Rect {
                x: inner.x,
                y: area.bottom().saturating_sub(1),
                width: inner.width,
                height: 1,
            },
            &mut state,
        );
    }
}

fn right_pad(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        s.chars().take(width).collect()
    } else {
        format!("{s}{}", " ".repeat(width - count))
    }
}
