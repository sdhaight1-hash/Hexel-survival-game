//! CPU-only renderer stage diagnostics; does not time GPU execution.
use crate::render::DrawCpuTimings;

#[derive(Default)]
pub struct RenderCpuMetrics {
    moving_ms: Vec<f64>,
    idle_ms: Vec<f64>,
    peaks: DrawCpuTimings,
}

impl RenderCpuMetrics {
    pub fn record(&mut self, sample: DrawCpuTimings, is_moving: bool) {
        if !sample.total_ms.is_finite() || sample.total_ms < 0.0 {
            return;
        }
        if is_moving {
            self.moving_ms.push(sample.total_ms);
        } else {
            self.idle_ms.push(sample.total_ms);
        }
        self.peaks.total_ms = self.peaks.total_ms.max(sample.total_ms);
        self.peaks.prepare_ms = self.peaks.prepare_ms.max(sample.prepare_ms);
        self.peaks.acquire_ms = self.peaks.acquire_ms.max(sample.acquire_ms);
        self.peaks.encode_ms = self.peaks.encode_ms.max(sample.encode_ms);
        self.peaks.submit_ms = self.peaks.submit_ms.max(sample.submit_ms);
        self.peaks.present_ms = self.peaks.present_ms.max(sample.present_ms);
    }

    pub fn report_and_reset(&mut self) -> String {
        self.moving_ms.sort_by(f64::total_cmp);
        self.idle_ms.sort_by(f64::total_cmp);
        let moving = self.moving_ms.len();
        let idle = self.idle_ms.len();
        let report = format!(
            "RENDER CPU: {} successful draws; draw p95 moving {:.2} ms ({}), idle {:.2} ms ({}); peak draw total {:.2} ms; independent CPU peaks prepare {:.2}, acquire {:.2}, encode {:.2}, submit {:.2}, present {:.2} ms (not GPU timings).",
            moving + idle,
            percentile(&self.moving_ms, 0.95), moving,
            percentile(&self.idle_ms, 0.95), idle,
            self.peaks.total_ms,
            self.peaks.prepare_ms, self.peaks.acquire_ms, self.peaks.encode_ms,
            self.peaks.submit_ms, self.peaks.present_ms
        );
        *self = Self::default();
        report
    }
}

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
    fn moving_and_idle_draws_are_separate_and_peaks_reset() {
        let mut metrics = RenderCpuMetrics::default();
        metrics.record(
            DrawCpuTimings {
                total_ms: 12.0,
                acquire_ms: 9.0,
                ..DrawCpuTimings::default()
            },
            true,
        );
        metrics.record(
            DrawCpuTimings {
                total_ms: 16.0,
                present_ms: 11.0,
                ..DrawCpuTimings::default()
            },
            false,
        );
        let report = metrics.report_and_reset();
        assert!(report.contains("2 successful draws"));
        assert!(report.contains("moving 12.00 ms (1), idle 16.00 ms (1)"));
        assert!(report.contains("acquire 9.00"));
        assert!(report.contains("present 11.00"));
        assert!(metrics.report_and_reset().contains("0 successful draws"));
    }

    #[test]
    fn invalid_samples_do_not_pollute_stats() {
        let mut metrics = RenderCpuMetrics::default();
        metrics.record(
            DrawCpuTimings {
                total_ms: f64::NAN,
                ..DrawCpuTimings::default()
            },
            true,
        );
        assert!(metrics.report_and_reset().contains("0 successful draws"));
    }
}
