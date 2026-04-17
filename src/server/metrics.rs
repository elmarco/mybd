use axum::response::Html;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

/// Per-endpoint timing stats.
#[derive(Debug, Clone)]
struct EndpointStats {
    count: u64,
    total: Duration,
    min: Duration,
    max: Duration,
    /// Last 100 durations for p50/p95/p99.
    recent: Vec<Duration>,
}

impl EndpointStats {
    fn new() -> Self {
        Self {
            count: 0,
            total: Duration::ZERO,
            min: Duration::MAX,
            max: Duration::ZERO,
            recent: Vec::with_capacity(100),
        }
    }

    fn record(&mut self, d: Duration) {
        self.count += 1;
        self.total += d;
        if d < self.min {
            self.min = d;
        }
        if d > self.max {
            self.max = d;
        }
        if self.recent.len() >= 100 {
            self.recent.remove(0);
        }
        self.recent.push(d);
    }

    fn avg(&self) -> Duration {
        if self.count == 0 {
            Duration::ZERO
        } else {
            self.total / self.count as u32
        }
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.recent.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted = self.recent.clone();
        sorted.sort();
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }
}

static METRICS: LazyLock<Mutex<HashMap<String, EndpointStats>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Record a request duration for a given endpoint.
pub fn record(endpoint: &str, duration: Duration) {
    let mut map = METRICS.lock().unwrap();
    map.entry(endpoint.to_string())
        .or_insert_with(EndpointStats::new)
        .record(duration);
}

fn fmt_dur(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.0}µs", d.as_micros())
    } else if ms < 1000.0 {
        format!("{ms:.1}ms")
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

/// Axum handler: renders an HTML page with per-endpoint metrics.
pub async fn metrics_handler() -> Html<String> {
    let _ = *START_TIME; // ensure initialized
    let uptime = START_TIME.elapsed();
    let map = METRICS.lock().unwrap();

    let mut entries: Vec<_> = map.iter().collect();
    // Sort by total time descending (biggest bottlenecks first)
    entries.sort_by(|a, b| b.1.total.cmp(&a.1.total));

    let mut rows = String::new();
    let mut total_requests: u64 = 0;

    for (name, stats) in &entries {
        total_requests += stats.count;
        rows.push_str(&format!(
            "<tr>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;font-family:monospace;font-size:13px'>{name}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{count}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{avg}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{min}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{p50}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{p95}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{p99}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{max}</td>\
                <td style='padding:6px 12px;border-bottom:1px solid #eee;text-align:right'>{total}</td>\
            </tr>",
            name = name,
            count = stats.count,
            avg = fmt_dur(stats.avg()),
            min = fmt_dur(stats.min),
            p50 = fmt_dur(stats.percentile(50.0)),
            p95 = fmt_dur(stats.percentile(95.0)),
            p99 = fmt_dur(stats.percentile(99.0)),
            max = fmt_dur(stats.max),
            total = fmt_dur(stats.total),
        ));
    }

    let uptime_str = {
        let secs = uptime.as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>mybd metrics</title><meta charset="utf-8"></head>
<body style="font-family:system-ui;max-width:1200px;margin:40px auto;padding:0 20px;background:#fafafa">
<h1 style="font-size:24px;font-weight:600">mybd server metrics</h1>
<p style="color:#666;font-size:14px">Uptime: {uptime} · Total requests: {total_requests} · Sorted by total time (biggest bottleneck first)</p>
<p style="color:#888;font-size:12px">Percentiles computed from last 100 requests per endpoint. <a href="" onclick="location.reload();return false">Refresh</a></p>
<table style="width:100%;border-collapse:collapse;background:#fff;border:1px solid #ddd;border-radius:8px;overflow:hidden;margin-top:16px">
<thead><tr style="background:#f5f5f5">
    <th style="padding:8px 12px;text-align:left;border-bottom:2px solid #ddd">Endpoint</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">Count</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">Avg</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">Min</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">p50</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">p95</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">p99</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">Max</th>
    <th style="padding:8px 12px;text-align:right;border-bottom:2px solid #ddd">Total</th>
</tr></thead>
<tbody>{rows}</tbody>
</table>
</body></html>"#,
        uptime = uptime_str,
        total_requests = total_requests,
        rows = rows,
    );

    Html(html)
}
