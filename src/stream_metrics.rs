//! Opt-in, CPU-side streaming diagnostics. No GPU timestamp claims.
#[derive(Default)]
pub struct StreamFrameMetrics {
    frame_intervals_ms: Vec<f64>,
    moving_intervals_ms: Vec<f64>,
    idle_intervals_ms: Vec<f64>,
    pub max_generation_ms: f64,
    pub max_mesh_upload_ms: f64,
    pub max_combined_stream_ms: f64,
    pub stream_updates: usize,
    pub over_16_7_ms: usize,
    pub over_33_3_ms: usize,
}

impl StreamFrameMetrics {
    pub fn record_frame_interval(&mut self, milliseconds: f64, is_moving: bool) {
        if !milliseconds.is_finite() || milliseconds < 0.0 {
            return;
        }
        self.frame_intervals_ms.push(milliseconds);
        if is_moving {
            self.moving_intervals_ms.push(milliseconds);
        } else {
            self.idle_intervals_ms.push(milliseconds);
        }
        if milliseconds > 16.7 {
            self.over_16_7_ms += 1;
        }
        if milliseconds > 33.3 {
            self.over_33_3_ms += 1;
        }
    }

    pub fn record_stream_update(&mut self, generation_ms: f64, mesh_upload_ms: f64) {
        self.stream_updates += 1;
        self.max_generation_ms = self.max_generation_ms.max(generation_ms);
        self.max_mesh_upload_ms = self.max_mesh_upload_ms.max(mesh_upload_ms);
        self.max_combined_stream_ms = self
            .max_combined_stream_ms
            .max(generation_ms + mesh_upload_ms);
    }

    pub fn report_and_reset(&mut self) -> String {
        self.frame_intervals_ms.sort_by(f64::total_cmp);
        self.moving_intervals_ms.sort_by(f64::total_cmp);
        self.idle_intervals_ms.sort_by(f64::total_cmp);
        let frames = self.frame_intervals_ms.len();
        let p50 = percentile(&self.frame_intervals_ms, 0.50);
        let p95 = percentile(&self.frame_intervals_ms, 0.95);
        let p99 = percentile(&self.frame_intervals_ms, 0.99);
        let moving_count = self.moving_intervals_ms.len();
        let moving_p95 = percentile(&self.moving_intervals_ms, 0.95);
        let idle_p95 = percentile(&self.idle_intervals_ms, 0.95);
        let report = format!(
            "STREAM FRAME: {frames} update intervals; interval p50/p95/p99 {:.2}/{:.2}/{:.2} ms; >16.7 ms {} >33.3 ms {}; Is_Moving samples {moving_count}/{frames}; moving p95 {:.2} ms ({} samples), idle p95 {:.2} ms ({} samples); {} stream updates; worst CPU generation {:.2} ms, mesh/buffer {:.2} ms, combined {:.2} ms (single update).",
            p50, p95, p99, self.over_16_7_ms, self.over_33_3_ms,
            moving_p95, moving_count, idle_p95, self.idle_intervals_ms.len(),
            self.stream_updates, self.max_generation_ms, self.max_mesh_upload_ms,
            self.max_combined_stream_ms
        );
        *self = Self::default();
        report
    }
}

// Nearest-rank percentile for diagnostic intervals, not an interpolated statistical estimate.
fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = ((sorted.len() as f64 * fraction).ceil() as usize).saturating_sub(1);
    sorted[rank.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_peak_work_by_single_update_not_three_second_sum() {
        let mut metrics = StreamFrameMetrics::default();
        metrics.record_stream_update(2.0, 6.0);
        metrics.record_stream_update(5.0, 1.0);
        assert_eq!(metrics.stream_updates, 2);
        assert_eq!(metrics.max_generation_ms, 5.0);
        assert_eq!(metrics.max_mesh_upload_ms, 6.0);
        assert_eq!(metrics.max_combined_stream_ms, 8.0);
    }

    #[test]
    fn percentiles_and_reset_are_bounded_per_reporting_window() {
        let mut metrics = StreamFrameMetrics::default();
        for ms in [4.0, 8.0, 16.0, 20.0, 40.0] {
            metrics.record_frame_interval(ms, false);
        }
        let report = metrics.report_and_reset();
        assert!(report.contains("16.00/40.00/40.00 ms"));
        assert!(report.contains(">16.7 ms 2 >33.3 ms 1"));
        assert!(metrics.report_and_reset().contains("0 update intervals"));
    }

    #[test]
    fn movement_samples_are_partitioned_without_mixing_idle_percentiles() {
        let mut metrics = StreamFrameMetrics::default();
        for (ms, moving) in [(4.0, false), (50.0, false), (9.0, true), (12.0, true)] {
            metrics.record_frame_interval(ms, moving);
        }
        let report = metrics.report_and_reset();
        assert!(report.contains("Is_Moving samples 2/4"));
        assert!(report.contains("moving p95 12.00 ms (2 samples)"));
        assert!(report.contains("idle p95 50.00 ms (2 samples)"));
        assert!(metrics.report_and_reset().contains("Is_Moving samples 0/0"));
    }

    #[test]
    fn rejects_invalid_intervals() {
        let mut metrics = StreamFrameMetrics::default();
        metrics.record_frame_interval(f64::NAN, true);
        metrics.record_frame_interval(-1.0, true);
        assert!(metrics.report_and_reset().contains("0 update intervals"));
    }
}
