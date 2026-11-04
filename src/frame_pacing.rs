//! Opt-in redraw pacing with a monotonic deadline; never sleeps inside callbacks.
use std::time::{Duration, Instant};

pub struct FramePacer {
    period: Option<Duration>,
    next: Instant,
    last_draw: Option<Instant>,
    draw_intervals_ms: Vec<f64>,
    successful_draws: usize,
    late_ticks: usize,
}

impl FramePacer {
    pub fn new(cap: Option<u32>, now: Instant) -> Self {
        Self {
            period: cap.map(|fps| Duration::from_secs_f64(1.0 / f64::from(fps))),
            next: now,
            last_draw: None,
            draw_intervals_ms: Vec::new(),
            successful_draws: 0,
            late_ticks: 0,
        }
    }

    /// Returns the next wake deadline when capped, or None for the existing Poll path.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.period.map(|_| self.next)
    }

    /// Only advances the world and requests a draw at the configured deadline.
    /// Missed deadlines are counted, never replayed as catch-up redraw bursts.
    pub fn tick(&mut self, now: Instant) -> bool {
        let Some(period) = self.period else {
            return true;
        };
        if now < self.next {
            return false;
        }
        if now > self.next + period {
            self.late_ticks += 1;
        }
        self.next = now + period;
        true
    }

    pub fn record_draw(&mut self, now: Instant) {
        if let Some(previous) = self.last_draw {
            self.draw_intervals_ms
                .push(now.duration_since(previous).as_secs_f64() * 1000.0);
        }
        self.last_draw = Some(now);
        self.successful_draws += 1;
    }

    pub fn report_and_reset(&mut self, elapsed: Duration) -> String {
        self.draw_intervals_ms.sort_by(f64::total_cmp);
        let n = self.draw_intervals_ms.len();
        let p95 = if n == 0 {
            0.0
        } else {
            let idx = ((n as f64 * 0.95).ceil() as usize).saturating_sub(1);
            self.draw_intervals_ms[idx]
        };
        let fps = if elapsed.is_zero() {
            0.0
        } else {
            self.successful_draws as f64 / elapsed.as_secs_f64()
        };
        let report = format!(
            "PACING: mode {}; {} successful draws / {:.2}s ({:.1} draws/s); successful draw interval p95 {:.2} ms ({} interval samples); missed pacing deadlines {} (CPU/winit observation, not display refresh).",
            self.period.map_or("uncapped".to_string(), |p| format!("capped {:.0} FPS", 1.0 / p.as_secs_f64())),
            self.successful_draws, elapsed.as_secs_f64(), fps, p95, n, self.late_ticks
        );
        self.last_draw = None;
        self.draw_intervals_ms.clear();
        self.successful_draws = 0;
        self.late_ticks = 0;
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_skips_early_events_and_never_catches_up_in_bursts() {
        let origin = Instant::now();
        let mut pacer = FramePacer::new(Some(60), origin);
        assert!(pacer.tick(origin));
        assert!(!pacer.tick(origin + Duration::from_millis(2)));
        assert!(pacer.tick(origin + Duration::from_millis(17)));
        assert!(!pacer.tick(origin + Duration::from_millis(18)));
        assert!(pacer.tick(origin + Duration::from_millis(100)));
        assert!(!pacer.tick(origin + Duration::from_millis(101)));
        assert!(pacer
            .report_and_reset(Duration::from_secs(1))
            .contains("missed pacing deadlines 1"));
    }

    #[test]
    fn uncapped_does_not_block_legacy_event_loop() {
        let origin = Instant::now();
        let mut pacer = FramePacer::new(None, origin);
        assert_eq!(pacer.next_deadline(), None);
        assert!(pacer.tick(origin));
        assert!(pacer.tick(origin));
    }

    #[test]
    fn successful_draw_intervals_reset_per_window() {
        let origin = Instant::now();
        let mut pacer = FramePacer::new(Some(120), origin);
        pacer.record_draw(origin);
        pacer.record_draw(origin + Duration::from_millis(9));
        let report = pacer.report_and_reset(Duration::from_secs(1));
        assert!(report.contains("2 successful draws"));
        assert!(report.contains("p95 9.00 ms (1 interval samples)"));
        assert!(pacer
            .report_and_reset(Duration::from_secs(1))
            .contains("0 successful draws"));
    }
}
