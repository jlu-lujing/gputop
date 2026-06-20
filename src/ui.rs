use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Cell, Paragraph, Row, Table, TableState, Wrap};

use crate::app::{App, SortColumn};

// ── Color palette ──────────────────────────────────────────────
const CYAN: Color = Color::Rgb(0x2a, 0xc3, 0xde);
const GREEN: Color = Color::Rgb(0x2e, 0xcc, 0x71);
const YELLOW: Color = Color::Rgb(0xf3, 0x9c, 0x12);
const ORANGE: Color = Color::Rgb(0xe6, 0x7e, 0x22);
const RED: Color = Color::Rgb(0xe7, 0x4c, 0x3c);
const MAGENTA: Color = Color::Rgb(0x9b, 0x59, 0xb6);
const PURPLE: Color = Color::Rgb(0x8e, 0x44, 0xad);
const BLUE: Color = Color::Rgb(0x34, 0x98, 0xdb);
const GRAY: Color = Color::Rgb(0x95, 0xa5, 0xa6);
const DIM: Color = Color::Rgb(0x63, 0x6e, 0x72);
const WHITE: Color = Color::Rgb(0xec, 0xf0, 0xf1);

// ── Smooth RGB gradient interpolation ──────────────────────────
fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let (r1, g1, b1) = match a {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        _ => (0.0, 0.0, 0.0),
    };
    let (r2, g2, b2) = match b {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        _ => (0.0, 0.0, 0.0),
    };
    Color::Rgb(
        (r1 + (r2 - r1) * t) as u8,
        (g1 + (g2 - g1) * t) as u8,
        (b1 + (b2 - b1) * t) as u8,
    )
}

fn sample_gradient(stops: &[(f64, Color)], t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    for i in 0..stops.len().max(1) - 1 {
        let (pos_a, color_a) = stops[i];
        let (pos_b, color_b) = stops[i + 1];
        if t <= pos_b {
            let local_t = if pos_b > pos_a {
                (t - pos_a) / (pos_b - pos_a)
            } else {
                0.0
            };
            return lerp_color(color_a, color_b, local_t);
        }
    }
    stops.last().map(|(_, c)| *c).unwrap_or(DIM)
}

// ── Gradient stops ───────────────────────────────────────────
fn gpu_gradient_stops() -> Vec<(f64, Color)> {
    vec![
        (0.0, Color::Rgb(0x00, 0x80, 0x80)),
        (0.15, CYAN),
        (0.35, GREEN),
        (0.55, YELLOW),
        (0.75, ORANGE),
        (1.0, RED),
    ]
}

fn mem_gradient_stops() -> Vec<(f64, Color)> {
    vec![
        (0.0, Color::Rgb(0x00, 0x40, 0x80)),
        (0.2, BLUE),
        (0.4, PURPLE),
        (0.6, MAGENTA),
        (0.8, ORANGE),
        (1.0, RED),
    ]
}

// ── Gradient bar builder (smooth RGB gradient) ────────────────
/// Filled: solid █ with color. Empty: plain space.
fn gradient_bar<'a>(
    percent: u32,
    width: usize,
    stops: &[(f64, Color)],
    _empty_color: Color,
) -> Line<'a> {
    let filled = (percent as f64 / 100.0 * width as f64).round() as usize;
    let mut spans = Vec::new();

    for i in 0..width {
        let pos = i as f64 / width.max(1) as f64;
        let color = sample_gradient(stops, pos);

        if i < filled {
            spans.push(Span::styled("█", Style::new().fg(color)));
        } else {
            spans.push(Span::raw(" "));
        }
    }

    Line::from(spans)
}

/// Mini gradient bar: 6 chars wide
fn mini_bar(percent: u32, stops: &[(f64, Color)]) -> Line<'_> {
    gradient_bar(percent, 6, stops, DIM)
}

/// Helper: label + gradient bar + percentage (with space before %)
fn gradient_bar_line<'a>(
    label: &'a str,
    percent: u32,
    width: usize,
    stops: &[(f64, Color)],
    _empty_color: Color,
) -> Line<'a> {
    let bar = gradient_bar(percent, width, stops, _empty_color);
    let mut spans = vec![Span::styled(label, Style::new().bold().fg(WHITE))];
    spans.extend(bar.spans);
    spans.push(Span::styled(
        format!(" {:3}% ", percent),
        Style::new().fg(WHITE),
    ));
    Line::from(spans)
}

/// Single-color bar line: label + bar + percentage (used for compact 8-per-row mode)
fn solid_bar_line<'a>(
    label: &'a str,
    percent: u32,
    width: usize,
    color: Color,
    _empty_color: Color,
) -> Line<'a> {
    let filled = (percent as f64 / 100.0 * width as f64).round() as usize;
    let mut spans = vec![Span::styled(label, Style::new().bold().fg(WHITE))];
    for i in 0..width {
        if i < filled {
            spans.push(Span::styled("█", Style::new().fg(color)));
        } else {
            spans.push(Span::raw(" "));
        }
    }
    spans.push(Span::styled(
        format!(" {:3}% ", percent),
        Style::new().fg(WHITE),
    ));
    Line::from(spans)
}

// ── Timeline line chart ──────────────────────────────────────
/// Render a filled line chart (btop/gotop style) for GPU utilization history.
/// Uses block characters (▁▂▃▄▅▆▇█) for smooth vertical boundaries with gradient coloring.
fn render_timeline<'a>(
    data: &[u64],
    width: usize,
    height: usize,
    stops: &[(f64, Color)],
) -> Vec<Line<'a>> {
    if data.is_empty() || width < 2 || height < 1 {
        return (0..height).map(|_| Line::from("")).collect();
    }

    let max_val = 100.0_f64; // percent scale — consistent
    let data_len = data.len();

    // Compute fill level per column in "eighths" across total chart height.
    // E.g. height=3 → total_eighths=24, a 50% value fills 12 eighths from bottom.
    let total_eighths = (height * 8) as f64;
    let fill_eighths: Vec<f64> = (0..width)
        .map(|col| {
            let idx = if data_len >= width {
                data_len - width + col
            } else {
                let frac = col as f64 / (width - 1).max(1) as f64;
                ((data_len - 1) as f64 * frac) as usize
            };
            let val = data[idx.min(data_len - 1)] as f64;
            (val / max_val) * total_eighths
        })
        .collect();

    // Block chars: index 0 = empty, 1..8 = ▁..█
    const BLOCK_CHARS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Build chart rows from top (row=0) to bottom (row=height-1)
    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let mut spans = Vec::with_capacity(width);
        for &fe in &fill_eighths {
            // Row r spans eighths [(height-1-r)*8, (height-r)*8) from bottom.
            let row_bottom = (height - 1 - row) * 8;
            let row_top = row_bottom + 8;

            if fe <= row_bottom as f64 {
                // Fill entirely below this row → empty
                spans.push(Span::raw(" "));
            } else if fe >= row_top as f64 {
                // Fill covers this entire row → solid block
                let gradient_t = fe / total_eighths;
                let color = sample_gradient(stops, gradient_t);
                spans.push(Span::styled("█", Style::new().fg(color)));
            } else {
                // Partial fill at the vertical boundary
                let partial = (fe - row_bottom as f64).round().clamp(1.0, 8.0) as u8;
                let gradient_t = fe / total_eighths;
                let color = sample_gradient(stops, gradient_t);
                spans.push(Span::styled(BLOCK_CHARS[partial as usize].to_string(), Style::new().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}

// ── Public API ─────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if let Some(ref error) = app.error {
        render_error_popup(frame, area, error);
    }

    let gpu_count = app.gpu_data.len() as u16;
    let core_count = app.cpu_data.cores.len() as u16;
    // Ensure at least 5 rows for process table (3 data + 1 header + 1 border) + 2 for footer
    let max_header = area.height.saturating_sub(7);

    // Try 4 cores/row first — if it fits, use it; otherwise fall back to 8
    let core_rows_4 = (core_count + 3) / 4;
    let no_chart_needed = (gpu_count.max(1)) * 3 + 1 + 1 + core_rows_4 + 1 + 2;
    let cores_per_row = if no_chart_needed <= max_header {
        4u16
    } else {
        8u16
    };
    let core_rows = (core_count + cores_per_row - 1) / cores_per_row;

    // Compute available chart height per GPU (extra rows beyond 3-row base)
    let base_needed = (gpu_count.max(1)) * 3 + 1 + 1 + core_rows + 1 + 2;
    let extra_avail = max_header.saturating_sub(base_needed);
    let gpu_count_for_chart = gpu_count.max(1);

    // chart_height: 3 if lots of room, 2 if moderate, 1 if tight, 0 if no room
    let chart_height = if gpu_count > 0 && extra_avail >= gpu_count_for_chart * 3 {
        3u16
    } else if gpu_count > 0 && extra_avail >= gpu_count_for_chart * 2 {
        2u16
    } else if gpu_count > 0 && extra_avail >= gpu_count_for_chart {
        1u16
    } else {
        0u16
    };

    let gpu_rows = (gpu_count.max(1)) * (3 + chart_height);
    let needed_height = gpu_rows + 1 + 1 + core_rows + 1 + 2;
    let min_header = 9u16.min(max_header);
    let header_height = needed_height.clamp(min_header, max_header);

    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_gpu_header(frame, app, chunks[0], gpu_rows, cores_per_row, chart_height);
    render_process_table(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_error_popup(frame: &mut Frame, area: Rect, error: &str) {
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 4,
        width: area.width / 2,
        height: 4,
    };
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .title(" Error ")
        .border_style(Style::new().fg(RED));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    frame.render_widget(
        Paragraph::new(error)
            .style(Style::new().fg(RED).bold())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

// ── GPU Header Section ─────────────────────────────────────────

fn render_gpu_header(frame: &mut Frame, app: &App, area: Rect, gpu_rows: u16, cores_per_row: u16, chart_height: u16) {
    let time_str = crate::cpu::format_time();
    let uptime_str = crate::cpu::format_uptime();
    let title = format!(" GPU / CPU  {}  Uptime: {}", time_str, uptime_str);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::new().fg(CYAN));
    frame.render_widget(&block, area);

    let inner = block.inner(area);
    let chunks = Layout::vertical([
        Constraint::Length(gpu_rows.min(inner.height)),
        Constraint::Fill(1),
    ])
    .split(inner);

    render_gpu_cards(frame, app, chunks[0], chart_height);
    render_cpu_section(frame, app, chunks[1], cores_per_row);
}

fn render_gpu_cards(frame: &mut Frame, app: &App, area: Rect, chart_height: u16) {
    if app.gpu_data.is_empty() {
        frame.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" GPU "),
            area,
        );
        let inner = Block::bordered().title(" GPU ").inner(area);
        frame.render_widget(
            Paragraph::new("No GPU detected or NVML not available")
                .style(Style::new().fg(GRAY)),
            inner,
        );
        return;
    }

    let gpu_stops = gpu_gradient_stops();
    let mem_stops = mem_gradient_stops();
    let per_gpu_rows = 3 + chart_height;

    for (i, gpu) in app.gpu_data.iter().enumerate() {
        let y = area.y + (i as u16 * per_gpu_rows);
        let gpu_block = Rect {
            x: area.x,
            y,
            width: area.width,
            height: per_gpu_rows,
        };

        // ── Row 1: Two gradient bars side by side ──
        let gauges = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .horizontal_margin(1)
            .split(gpu_block);

        // GPU bar: "GPU: ████████  78%" — space between bar and %
        let bar_width = (gauges[0].width as usize).saturating_sub(10);
        let gpu_line = gradient_bar_line(
            "GPU: ",
            gpu.gpu_utilization,
            bar_width,
            &gpu_stops,
            DIM,
        );
        frame.render_widget(Paragraph::new(gpu_line), gauges[0]);

        // MEM bar: "MEM: ████████  41%" — space between bar and %
        let mem_percent = if gpu.mem_total_mib > 0 {
            (gpu.mem_used_mib as f64 / gpu.mem_total_mib as f64 * 100.0) as u32
        } else {
            0
        };
        let bar_width = (gauges[1].width as usize).saturating_sub(10);
        let mem_line = gradient_bar_line(
            "MEM: ",
            mem_percent,
            bar_width,
            &mem_stops,
            DIM,
        );
        frame.render_widget(Paragraph::new(mem_line), gauges[1]);

        // ── Row 2: Info line (auto-collapse, GPU name folds first) ──
        let info_y = y + 1;
        let info_area = Rect {
            x: area.x + 2,
            y: info_y,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        let info_width = info_area.width as usize;

        let temp_color = if gpu.temperature >= 85 {
            RED
        } else if gpu.temperature >= 75 {
            ORANGE
        } else if gpu.temperature >= 60 {
            YELLOW
        } else {
            GREEN
        };
        let pwr_pct = if gpu.power_limit_watts > 0 {
            gpu.power_draw_watts as f64 / gpu.power_limit_watts as f64 * 100.0
        } else {
            0.0
        };
        let pwr_color = if pwr_pct > 90.0 {
            RED
        } else if pwr_pct > 70.0 {
            YELLOW
        } else {
            GREEN
        };

        let sep = " │ ";
        let sep_len = sep.len();

        let temp_seg = format!("{}°C", gpu.temperature);
        let pwr_seg = format!("{}W", gpu.power_draw_watts);
        let fan_seg = format!("{}%", gpu.fan_speed.unwrap_or(0));
        let clk_seg = format!(
            "Clk:{}/{:.0}G",
            gpu.graphics_clock,
            gpu.memory_clock as f64 / 1000.0
        );
        let used_gb = (gpu.mem_used_mib as f64 / 1024.0 * 10.0).round() / 10.0;
        let total_gb = (gpu.mem_total_mib as f64 / 1024.0 * 10.0).round() / 10.0;
        let mem_seg = format!("MEM:{:.1}G/{:.1}G", used_gb, total_gb);

        // GPU name segments — foldable
        let name_prefix = format!("GPU{} ", gpu.index);
        let gpu_name_len = gpu.name.len();
        let full_name_len = name_prefix.len() + gpu_name_len;

        // Calculate widths for each fold level
        let level1 = full_name_len + sep_len * 5 + temp_seg.len() + pwr_seg.len() + fan_seg.len() + clk_seg.len() + mem_seg.len();
        let short_name_len = name_prefix.len();
        let level2 = short_name_len + sep_len * 5 + temp_seg.len() + pwr_seg.len() + fan_seg.len() + clk_seg.len() + mem_seg.len();
        let level3 = short_name_len + sep_len * 2 + temp_seg.len() + pwr_seg.len();

        // Determine what fits and build the display string
        let gpu_display = if level1 <= info_width {
            format!("{}{}", name_prefix, gpu.name)
        } else if level2 <= info_width {
            name_prefix.clone()
        } else {
            name_prefix.clone()
        };

        let info = if level1 <= info_width || level2 <= info_width {
            // Show fan + clk + mem
            Line::from(vec![
                Span::styled(&gpu_display, Style::new().bold().fg(CYAN)),
                Span::raw(sep),
                Span::styled(temp_seg, Style::new().bold().fg(temp_color)),
                Span::raw(sep),
                Span::styled(pwr_seg, Style::new().fg(pwr_color)),
                Span::raw(sep),
                Span::styled(fan_seg, Style::new().fg(GRAY)),
                Span::raw(sep),
                Span::styled(clk_seg, Style::new().fg(PURPLE)),
                Span::raw(sep),
                Span::styled(mem_seg, Style::new().fg(BLUE)),
            ])
        } else if level3 <= info_width {
            Line::from(vec![
                Span::styled(&gpu_display, Style::new().bold().fg(CYAN)),
                Span::raw(sep),
                Span::styled(temp_seg, Style::new().bold().fg(temp_color)),
                Span::raw(sep),
                Span::styled(pwr_seg, Style::new().fg(pwr_color)),
            ])
        } else {
            Line::from(vec![
                Span::styled(&gpu_display, Style::new().bold().fg(CYAN)),
                Span::raw(sep),
                Span::styled(temp_seg, Style::new().bold().fg(temp_color)),
            ])
        };
        frame.render_widget(info, info_area);

        // ── Rows 3+: Timeline chart ──
        let chart_y = info_y + 1;
        if chart_y < gpu_block.y + per_gpu_rows && chart_height > 0 && i < app.gpu_util_history.len() {
            let chart_area = Rect {
                x: area.x + 1,
                y: chart_y,
                width: area.width.saturating_sub(2),
                height: chart_height,
            };
            let hist = &app.gpu_util_history[i];
            if !hist.is_empty() && chart_area.width > 5 {
                let lines = render_timeline(hist, chart_area.width as usize, chart_height as usize, &gpu_stops);
                frame.render_widget(Paragraph::new(Text::from(lines)), chart_area);
            }
        }
    }
}

// ── CPU / MEM Section ──────────────────────────────────────────

fn render_cpu_section(frame: &mut Frame, app: &App, area: Rect, cores_per_row: u16) {
    if area.width < 15 || area.height < 2 {
        return;
    }

    // CPU temp in section title
    let cpu_temp = app.cpu_data.cpu_temp_celsius;
    let temp_color = if cpu_temp >= 85 {
        RED
    } else if cpu_temp >= 75 {
        ORANGE
    } else if cpu_temp >= 60 {
        YELLOW
    } else {
        GREEN
    };
    let title = if cpu_temp > 0 {
        Line::from(vec![
            Span::styled(" CPU / MEM  ", Style::new().bold().fg(CYAN)),
            Span::styled(format!("{}°C ", cpu_temp), Style::new().bold().fg(temp_color)),
        ])
    } else {
        Line::from(Span::styled(" CPU / MEM ", Style::new().bold().fg(CYAN)))
    };
    frame.render_widget(
        title,
        Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width.saturating_sub(2),
            height: 1,
        },
    );

    let inner = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(1),
    };

    let mut y = inner.y;
    let gpu_stops = gpu_gradient_stops();
    let _mem_stops = mem_gradient_stops();

    // ── Total CPU gradient bar ──
    let bar_width = (inner.width as usize).saturating_sub(12);
    let cpu_line = gradient_bar_line(
        "CPU: ",
        app.cpu_data.total_usage as u32,
        bar_width,
        &gpu_stops,
        DIM,
    );
    frame.render_widget(
        Paragraph::new(cpu_line),
        Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        },
    );
    y += 1;

    // ── Per-core compact bars ──
    let cores = &app.cpu_data.cores;
    if !cores.is_empty() && y < inner.y + inner.height - 1 {
        let cores_per_row = cores_per_row as usize;
        for chunk in cores.chunks(cores_per_row) {
            if y >= inner.y + inner.height - 1 {
                break;
            }
            let row_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };

            let cell_width = (row_area.width as usize).saturating_sub(
                chunk.len() as usize - 1,
            ) / chunk.len() as usize;
            for (ci, core) in chunk.iter().enumerate() {
                let cx = row_area.x + (ci as u16) * (cell_width as u16 + 1);
                if cx + cell_width as u16 > row_area.x + row_area.width {
                    break;
                }
                let cell_area = Rect {
                    x: cx,
                    y,
                    width: cell_width as u16,
                    height: 1,
                };

                let bar_w = (cell_area.width as usize).saturating_sub(8);
                let core_label = format!("C{:>2}", core.index);
                if bar_w < 6 {
                    // Narrow bar: single color (gradient would be invisible at this width)
                    let percent = core.usage_percent as u32;
                    let color = gauge_fg(percent).fg.unwrap_or(WHITE);
                    let line = solid_bar_line(&core_label, percent, bar_w, color, DIM);
                    frame.render_widget(Paragraph::new(line), cell_area);
                } else {
                    let core_line = gradient_bar_line(
                        &core_label,
                        core.usage_percent as u32,
                        bar_w,
                        &gpu_stops,
                        DIM,
                    );
                    frame.render_widget(Paragraph::new(core_line), cell_area);
                }
            }
            y += 1;
        }
    }

    // ── MEM breakdown bar (btop style) ──
    if y < inner.y + inner.height {
        let mem = &app.cpu_data.mem;
        if mem.total_mib > 0 {
            let used_pct = (mem.used_mib as f64 / mem.total_mib as f64 * 100.0) as u32;
            let buf_pct = (mem.buffers_mib as f64 / mem.total_mib as f64 * 100.0) as u32;
            let cache_pct = (mem.cached_mib as f64 / mem.total_mib as f64 * 100.0) as u32;

            let bar_width = (inner.width as usize).saturating_sub(40);
            let mut spans = vec![Span::styled("MEM: ", Style::new().bold().fg(WHITE))];

            let used_w = (used_pct as f64 / 100.0 * bar_width as f64).round() as usize;
            let buf_w = (buf_pct as f64 / 100.0 * bar_width as f64).round() as usize;
            let cache_w = (cache_pct as f64 / 100.0 * bar_width as f64).round() as usize;
            let free_w = bar_width.saturating_sub(used_w + buf_w + cache_w);

            // Used segment (smooth gradient)
            for i in 0..used_w {
                let pos = i as f64 / used_w.max(1) as f64;
                let color = sample_gradient(&gpu_stops, pos);
                spans.push(Span::styled("█", Style::new().fg(color)));
            }
            // Buffers segment (smooth green gradient)
            for i in 0..buf_w {
                let pos = i as f64 / buf_w.max(1) as f64;
                let color = lerp_color(GREEN, Color::Rgb(0x00, 0xff, 0x80), pos);
                spans.push(Span::styled("█", Style::new().fg(color)));
            }
            // Cached segment (smooth blue gradient)
            for i in 0..cache_w {
                let pos = i as f64 / cache_w.max(1) as f64;
                let color = lerp_color(BLUE, Color::Rgb(0x00, 0x80, 0xff), pos);
                spans.push(Span::styled("█", Style::new().fg(color)));
            }
            // Free segment (space)
            for _ in 0..free_w {
                spans.push(Span::raw(" "));
            }

            // Legend
            spans.push(Span::styled(
                format!(
                    " U:{}G B:{}G C:{}G F:{}G",
                    (mem.used_mib as f64 / 1024.0 * 10.0).round() / 10.0,
                    (mem.buffers_mib as f64 / 1024.0 * 10.0).round() / 10.0,
                    (mem.cached_mib as f64 / 1024.0 * 10.0).round() / 10.0,
                    (mem.free_mib as f64 / 1024.0 * 10.0).round() / 10.0,
                ),
                Style::new().fg(WHITE),
            ));

            let mem_line = Line::from(spans);
            frame.render_widget(
                Paragraph::new(mem_line),
                Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                },
            );
        }
    }
}

// ── Process Table ──────────────────────────────────────────────

fn render_process_table(frame: &mut Frame, app: &App, area: Rect) {
    if app.processes.is_empty() {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" GPU Processes ")
            .border_style(Style::new().fg(CYAN));
        frame.render_widget(block, area);
        return;
    }

    let gpu_stops = gpu_gradient_stops();

    let sort_name = match app.sort_column {
        SortColumn::Pid => "PID",
        SortColumn::GpuMem => "GPU-MEM",
        SortColumn::SmPercent => "SM%",
        SortColumn::CpuPercent => "CPU%",
    };

    let table = Table::new(
        app.processes.iter().enumerate().map(|(idx, p)| {
            let is_even = idx % 2 == 0;
            let row_fg = if is_even { WHITE } else { GRAY };

            let cpu_mini = mini_bar(p.cpu_percent as u32, &gpu_stops);
            let sm_mini = mini_bar(p.sm_percent, &gpu_stops);

            let sm_style = match p.sm_percent {
                0 => Style::new().fg(DIM),
                _ => gauge_fg(p.sm_percent),
            };
            let mem_str = format!("{}MiB", p.gpu_mem_mib);
            let mem_style = if p.gpu_mem_mib > 1024 {
                Style::new().fg(ORANGE)
            } else {
                Style::new().fg(row_fg)
            };

            let mut cpu_spans = vec![Span::styled(
                format!("{:>5.1} ", p.cpu_percent),
                Style::new().fg(row_fg),
            )];
            cpu_spans.extend(cpu_mini.spans);
            let cpu_cell = Line::from(cpu_spans);

            let mut sm_spans = vec![Span::styled(
                format!("{:>4}% ", p.sm_percent),
                sm_style,
            )];
            sm_spans.extend(sm_mini.spans);
            let sm_cell = Line::from(sm_spans);

            Row::new(vec![
                Cell::from(Span::styled(format!("{}", p.pid), Style::new().fg(row_fg))),
                Cell::from(Span::styled("—", Style::new().fg(DIM))),
                Cell::from(cpu_cell),
                Cell::from(Span::styled(mem_str, mem_style)),
                Cell::from(sm_cell),
                Cell::from(Span::styled(&p.name, Style::new().fg(CYAN))),
            ])
        }),
        [
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Fill(1),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from(Span::styled("PID", Style::new().bold().fg(YELLOW))),
            Cell::from(Span::styled("USER", Style::new().bold().fg(YELLOW))),
            Cell::from(Span::styled("CPU%  ██████", Style::new().bold().fg(YELLOW))),
            Cell::from(Span::styled("GPU-MEM", Style::new().bold().fg(YELLOW))),
            Cell::from(Span::styled("SM%   ██████", Style::new().bold().fg(YELLOW))),
            Cell::from(Span::styled("COMMAND", Style::new().bold().fg(YELLOW))),
        ])
        .bottom_margin(1),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(
                " GPU Processes  [ {} ] ",
                sort_name
            ))
            .title_bottom(format!(
                " {} process{} ",
                app.processes.len(),
                if app.processes.len() == 1 { "" } else { "es" },
            ))
            .border_style(Style::new().fg(CYAN)),
    )
    .row_highlight_style(
        Style::new()
            .bg(Color::Rgb(0x2c, 0x3e, 0x50))
            .bold(),
    )
    .highlight_symbol("▸ ")
    .column_spacing(1);

    let mut state = TableState::default();
    if let Some(idx) = app.selected_process {
        state.select(Some(idx));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

// ── Footer ─────────────────────────────────────────────────────

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app
        .selected_process
        .and_then(|idx| app.processes.get(idx))
        .map(|p| format!("PID:{} ", p.pid))
        .unwrap_or_default();

    let help = Text::from(vec![
        Line::from(vec![
            Span::styled("gputop ", Style::new().bold().fg(CYAN)),
            Span::styled(
                format!(
                    "{} GPU | {} proc | {}",
                    app.gpu_data.len(),
                    app.processes.len(),
                    selected,
                ),
                Style::new().fg(GRAY),
            ),
            Span::raw("  "),
            Span::styled("q", Style::new().bold().fg(RED)),
            Span::styled(":quit ", Style::new().fg(DIM)),
            Span::styled("/", Style::new().bold().fg(YELLOW)),
            Span::styled(":sort ", Style::new().fg(DIM)),
            Span::styled("↑↓", Style::new().bold().fg(WHITE)),
            Span::styled(":nav ", Style::new().fg(DIM)),
            Span::styled("c", Style::new().bold().fg(RED)),
            Span::styled(":INT ", Style::new().fg(DIM)),
            Span::styled("t", Style::new().bold().fg(RED)),
            Span::styled(":TERM ", Style::new().fg(DIM)),
            Span::styled("K", Style::new().bold().fg(RED)),
            Span::styled(":KILL ", Style::new().fg(DIM)),
            Span::styled("e", Style::new().bold().fg(YELLOW)),
            Span::styled(":err", Style::new().fg(DIM)),
        ]),
    ]);

    frame.render_widget(help, area);
}

// ── Color helpers ──────────────────────────────────────────────

fn gauge_fg(val: u32) -> Style {
    if val >= 80 {
        Style::new().fg(RED)
    } else if val >= 50 {
        Style::new().fg(YELLOW)
    } else if val >= 20 {
        Style::new().fg(GREEN)
    } else {
        Style::new().fg(DIM)
    }
}
