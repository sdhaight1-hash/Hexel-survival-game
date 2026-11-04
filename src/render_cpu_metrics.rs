//! CPU-only renderer stage diagnostics; does not time GPU execution.
use crate::render::DrawCpuTimings;

#[derive(Default)]
pub struct RenderCpuMetrics {
    moving_ms: Vec<f64>,
    idle_ms: Vec<f64>,
    looking_ms: Vec<f64>,
    no_look_ms: Vec<f64>,
    draw_by_state: [Vec<f64>; 4],
    acquire_by_state: [Vec<f64>; 4],
    peaks: DrawCpuTimings,
}

impl RenderCpuMetrics {
    pub fn record(&mut self, sample: DrawCpuTimings, is_moving: bool, is_looking: bool) {
        if !sample.total_ms.is_finite() || sample.total_ms < 0.0 {
            return;
        }
        if is_moving {
            self.moving_ms.push(sample.total_ms);
        } else {
            self.idle_ms.push(sample.total_ms);
        }
        if is_looking {
            self.looking_ms.push(sample.total_ms);
        } else {
            self.no_look_ms.push(sample.total_ms);
        }
        // 0=idle/still, 1=idle/look, 2=move/still, 3=move/look.
        let state = usize::from(is_moving) * 2 + usize::from(is_looking);
        self.draw_by_state[state].push(sample.total_ms);
        self.acquire_by_state[state].push(sample.acquire_ms);
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
        self.looking_ms.sort_by(f64::total_cmp);
        self.no_look_ms.sort_by(f64::total_cmp);
        for samples in &mut self.draw_by_state {
            samples.sort_by(f64::total_cmp);
        }
        for samples in &mut self.acquire_by_state {
            samples.sort_by(f64::total_cmp);
        }
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
        let look_report = format!(
            "LOOK CPU: Is_Looking samples {}/{}; draw p95 looking {:.2} ms ({}), no-look {:.2} ms ({}); four states [idle+still {}, idle+look {}, move+still {}, move+look {}] draw p95 {:.2}/{:.2}/{:.2}/{:.2} ms; acquire p95 {:.2}/{:.2}/{:.2}/{:.2} ms (CPU call durations).",
            self.looking_ms.len(), moving + idle,
            percentile(&self.looking_ms, 0.95), self.looking_ms.len(),
            percentile(&self.no_look_ms, 0.95), self.no_look_ms.len(),
            self.draw_by_state[0].len(), self.draw_by_state[1].len(),
            self.draw_by_state[2].len(), self.draw_by_state[3].len(),
            percentile(&self.draw_by_state[0], 0.95),
            percentile(&self.draw_by_state[1], 0.95),
            percentile(&self.draw_by_state[2], 0.95),
            percentile(&self.draw_by_state[3], 0.95),
            percentile(&self.acquire_by_state[0], 0.95),
            percentile(&self.acquire_by_state[1], 0.95),
            percentile(&self.acquire_by_state[2], 0.95),
            percentile(&self.acquire_by_state[3], 0.95)
        );
        *self = Self::default();
        format!("{report}\n{look_report}")
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
            false,
        );
        metrics.record(
            DrawCpuTimings {
                total_ms: 16.0,
                present_ms: 11.0,
                ..DrawCpuTimings::default()
            },
            false,
            true,
        );
        let report = metrics.report_and_reset();
        assert!(report.contains("2 successful draws"));
        assert!(report.contains("moving 12.00 ms (1), idle 16.00 ms (1)"));
        assert!(report.contains("acquire 9.00"));
        assert!(report.contains("present 11.00"));
        assert!(report.contains("Is_Looking samples 1/2"));
        assert!(
            report.contains("four states [idle+still 0, idle+look 1, move+still 1, move+look 0]")
        );
        assert!(metrics.report_and_reset().contains("0 successful draws"));
    }

    #[test]
    fn four_states_keep_acquire_percentiles_separate() {
        let mut metrics = RenderCpuMetrics::default();
        for (moving, looking, draw, acquire) in [
            (false, false, 4.0, 2.0),
            (false, true, 12.0, 10.0),
            (true, false, 20.0, 18.0),
            (true, true, 40.0, 38.0),
        ] {
            metrics.record(
                DrawCpuTimings {
                    total_ms: draw,
                    acquire_ms: acquire,
                    ..DrawCpuTimings::default()
                },
                moving,
                looking,
            );
        }
        let report = metrics.report_and_reset();
        assert!(
            report.contains("four states [idle+still 1, idle+look 1, move+still 1, move+look 1]")
        );
        assert!(report.contains("draw p95 4.00/12.00/20.00/40.00 ms"));
        assert!(report.contains("acquire p95 2.00/10.00/18.00/38.00 ms"));
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
            true,
        );
        assert!(metrics.report_and_reset().contains("0 successful draws"));
    }
}
