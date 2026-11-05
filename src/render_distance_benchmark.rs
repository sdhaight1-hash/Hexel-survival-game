//! Reproducible full-resolution render-distance experiment (not terrain LOD).
//! Each 30-second stage reserves at least 5s to warm up; only settled
//! samples with zero pending sections are used for steady-frame percentiles.
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    time::{Duration, Instant},
};

pub const CSV_PATH: &str = "target/render-distance-benchmark.csv";
const RADII: [i32; 7] = [2, 4, 3, 2, 3, 4, 2];
const STAGE_SECONDS: u64 = 30;
const WARMUP_SECONDS: u64 = 5;

#[derive(Default)]
struct Samples {
    settled_intervals_ms: Vec<f64>,
    moving_samples: usize,
    peak_pending: usize,
    total_added: usize,
    generation_ms: f64,
    meshing_ms: f64,
    upload_bytes: u64,
    distance_m: f64,
    max_tick_ms: f64,
    over_33ms: usize,
}

pub struct RenderDistanceBenchmark {
    started: Instant,
    stage: usize,
    samples: Samples,
    csv: Option<BufWriter<File>>,
}

impl RenderDistanceBenchmark {
    pub fn new(now: Instant) -> Self {
        if let Some(parent) = Path::new(CSV_PATH).parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("RENDER BENCH CSV directory error: {e}");
            }
        }
        let csv = match File::create(CSV_PATH) {
            Ok(file) => {
                let mut out = BufWriter::new(file);
                let _ = writeln!(
                    out,
                    "stage,radius,elapsed_s,settled_samples,moving_samples,mean_tick_fps,tick_p95_ms,tick_p99_ms,max_tick_ms,over33ms,peak_pending,sections_added,generation_cpu_ms,mesh_cpu_ms,uploaded_mib,travel_m,resident_sections,gpu_meshes,gpu_vertices"
                );
                let _ = out.flush();
                Some(out)
            }
            Err(e) => {
                eprintln!("RENDER BENCH cannot write {CSV_PATH}: {e}");
                None
            }
        };
        Self {
            started: now,
            stage: 0,
            samples: Samples::default(),
            csv,
        }
    }

    pub fn radius(&self) -> i32 {
        RADII[self.stage.min(RADII.len() - 1)]
    }

    pub fn introduction(&self) -> String {
        format!(
            "RENDER BENCH START | radius R{} | stage 1/{} | {}s stages, {}s warmup + pending=0 | sequence {:?} | full-res sections (NOT LOD) | CSV {}",
            self.radius(), RADII.len(), STAGE_SECONDS, WARMUP_SECONDS, RADII, CSV_PATH
        )
    }

    /// Called before streaming to apply the next radius on schedule.
    /// Returns messages for the existing F2 / terminal / disk logger.
    pub fn advance(
        &mut self,
        now: Instant,
        resident: usize,
        gpu_meshes: usize,
        vertices: u64,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        if self.stage >= RADII.len() {
            return messages;
        }
        if now.duration_since(self.started) < Duration::from_secs(STAGE_SECONDS) {
            return messages;
        }
        let old_radius = self.radius();
        self.samples.settled_intervals_ms.sort_by(f64::total_cmp);
        let samples = &self.samples.settled_intervals_ms;
        let mean = if samples.is_empty() {
            0.0
        } else {
            samples.len() as f64 * 1000.0 / samples.iter().sum::<f64>()
        };
        let p95 = percentile(samples, 0.95);
        let p99 = percentile(samples, 0.99);
        let s = &self.samples;
        let status = if samples.len() >= 100 && s.moving_samples >= 50 {
            "VALID"
        } else {
            "INSUFFICIENT_MOVEMENT_OR_SETTLING"
        };
        let summary = format!(
            "RENDER BENCH END | stage {}/{} R{} | {} | settled {} frames moving {} | tickFPS {:.1} p95 {:.2}ms p99 {:.2}ms max {:.2}ms >33ms {} | pending peak {} | +{} sections | gen {:.1}ms mesh {:.1}ms upload {:.2}MiB travel {:.1}m | resident {} GPU meshes {} verts {}",
            self.stage + 1, RADII.len(), old_radius, status,
            samples.len(), s.moving_samples, mean, p95, p99, s.max_tick_ms,
            s.over_33ms, s.peak_pending, s.total_added, s.generation_ms,
            s.meshing_ms, s.upload_bytes as f64 / 1_048_576.0,
            s.distance_m, resident, gpu_meshes, vertices
        );
        messages.push(summary);
        if let Some(csv) = self.csv.as_mut() {
            let _ = writeln!(
                csv, "{},{},{},{},{},{:.2},{:.3},{:.3},{:.3},{},{},{},{:.3},{:.3},{:.3},{:.2},{},{},{}",
                self.stage + 1, old_radius, STAGE_SECONDS,
                samples.len(), s.moving_samples, mean, p95, p99, s.max_tick_ms,
                s.over_33ms, s.peak_pending, s.total_added, s.generation_ms,
                s.meshing_ms, s.upload_bytes as f64 / 1_048_576.0,
                s.distance_m, resident, gpu_meshes, vertices
            );
            let _ = csv.flush();
        }
        self.stage += 1;
        self.samples = Samples::default();
        self.started = now;
        if self.stage == RADII.len() {
            messages.push(format!("RENDER BENCH COMPLETE | {} stages | CSV {} | no further radius changes", RADII.len(), CSV_PATH));
        } else {
            messages.push(format!(
                "RENDER BENCH SWITCH | stage {}/{} | R{} -> R{} | target {} resident sections | settle first {}s AND pending=0",
                self.stage + 1, RADII.len(), old_radius, self.radius(),
                5 * (1 + 3 * self.radius() * (self.radius() + 1)), WARMUP_SECONDS
            ));
        }
        messages
    }

    pub fn record_tick(
        &mut self,
        now: Instant,
        interval_ms: f64,
        moving: bool,
        pending: usize,
        sections_added: usize,
        generation_ms: f64,
        meshing_ms: f64,
        uploaded_bytes: u64,
        distance_m: f64,
    ) {
        if self.stage >= RADII.len() {
            return;
        }
        let s = &mut self.samples;
        s.peak_pending = s.peak_pending.max(pending);
        s.total_added += sections_added;
        s.generation_ms += generation_ms;
        s.meshing_ms += meshing_ms;
        s.upload_bytes += uploaded_bytes;
        s.distance_m += distance_m;
        if now.duration_since(self.started) < Duration::from_secs(WARMUP_SECONDS)
            || pending != 0 || !interval_ms.is_finite() || interval_ms < 0.0
        {
            return;
        }
        s.settled_intervals_ms.push(interval_ms);
        s.moving_samples += usize::from(moving);
        s.max_tick_ms = s.max_tick_ms.max(interval_ms);
        s.over_33ms += usize::from(interval_ms > 33.3);
    }
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() as f64 * fraction).ceil() as usize).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_radius_sequence_has_repeated_baselines() {
        assert_eq!(RADII, [2, 4, 3, 2, 3, 4, 2]);
        for radius in RADII {
            assert!((2..=4).contains(&radius));
            assert_eq!(5 * (1 + 3 * radius * (radius + 1)), match radius {
                2 => 95, 3 => 185, 4 => 305, _ => unreachable!()
            });
        }
    }

    #[test]
    fn stage_progression_is_time_based_and_stops_after_seven() {
        let start = Instant::now();
        let mut bench = RenderDistanceBenchmark::new(start);
        assert_eq!(bench.radius(), 2);
        assert!(bench.advance(start + Duration::from_secs(29), 95, 19, 87552).is_empty());
        for index in 0..7 {
            let messages = bench.advance(start + Duration::from_secs(30 * (index + 1)), 95, 19, 87552);
            assert!(!messages.is_empty());
        }
        assert!(bench.advance(start + Duration::from_secs(300), 95, 19, 87552).is_empty());
    }

    #[test]
    fn warmup_and_pending_samples_are_excluded() {
        let start = Instant::now();
        let mut bench = RenderDistanceBenchmark::new(start);
        bench.record_tick(start + Duration::from_secs(1), 100.0, true, 0, 1, 2.0, 3.0, 10, 2.0);
        bench.record_tick(start + Duration::from_secs(10), 100.0, true, 20, 1, 2.0, 3.0, 10, 2.0);
        bench.record_tick(start + Duration::from_secs(11), 16.7, true, 0, 0, 0.0, 0.0, 0, 2.0);
        assert_eq!(bench.samples.settled_intervals_ms, vec![16.7]);
        assert_eq!(bench.samples.peak_pending, 20);
        assert_eq!(bench.samples.total_added, 2);
    }
}
