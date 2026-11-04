//! Bounded benchmark history and a copyable session log.
//! All performance aggregation is derived from reported samples, never
//! from renderer/world state or synthesized GPU timings.

use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

/// Portable CPU/core fields with Windows-only physical RAM reporting.
/// This is machine metadata, not a claim about runtime GPU VRAM use.
pub fn system_report() -> String {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "N/A".into());
    let mut cpu = std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown CPU".into());
    #[cfg(windows)]
    {
        // Standard Windows registry API exposed by built-in reg.exe; this
        // is a one-time startup lookup, never called from the render loop.
        if let Ok(output) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
                "/v",
                "ProcessorNameString",
            ])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = text.lines().find(|line| line.contains("REG_SZ")) {
                    if let Some((_, name)) = line.split_once("REG_SZ") {
                        if !name.trim().is_empty() {
                            cpu = name.trim().into();
                        }
                    }
                }
            }
        }
    }
    let ram = physical_ram_report();
    format!(
        "SYSTEM OS {} {} | CPU {} | logical threads {} | RAM {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        cpu,
        threads,
        ram
    )
}

#[cfg(windows)]
fn physical_ram_report() -> String {
    #[repr(C)]
    #[derive(Default)]
    struct MemoryStatus {
        length: u32,
        load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_pagefile: u64,
        avail_pagefile: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    }
    let mut status = MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        ..MemoryStatus::default()
    };
    // SAFETY: the C ABI struct is repr(C), sized as required, and its
    // pointer remains valid for the synchronous OS call.
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        let gib = 1024.0 * 1024.0 * 1024.0;
        format!(
            "{:.1} GiB physical ({:.1} GiB available at startup)",
            status.total_phys as f64 / gib,
            status.avail_phys as f64 / gib
        )
    } else {
        "N/A (Windows memory query failed)".into()
    }
}

#[cfg(not(windows))]
fn physical_ram_report() -> String {
    "N/A (Windows-only physical RAM query)".into()
}

const MAX_ONSCREEN_LOG_LINES: usize = 250;
pub const LOG_FILE: &str = "target/benchmark-session.log";

#[derive(Clone, Copy, Debug, Default)]
pub struct BenchSample {
    pub seconds: f64,
    pub frames: u64,
    pub cpu_draw_ms_total: f64,
    pub gpu_pass_ms_total: f64,
    pub gpu_samples: u64,
}

impl BenchSample {
    pub fn add(&mut self, other: Self) {
        self.seconds += other.seconds;
        self.frames += other.frames;
        self.cpu_draw_ms_total += other.cpu_draw_ms_total;
        self.gpu_pass_ms_total += other.gpu_pass_ms_total;
        self.gpu_samples += other.gpu_samples;
    }

    pub fn fps(self) -> f64 {
        if self.seconds > 0.0 {
            self.frames as f64 / self.seconds
        } else {
            0.0
        }
    }

    pub fn cpu_ms(self) -> f64 {
        if self.frames > 0 {
            self.cpu_draw_ms_total / self.frames as f64
        } else {
            0.0
        }
    }

    pub fn gpu_ms(self) -> Option<f64> {
        (self.gpu_samples > 0).then(|| self.gpu_pass_ms_total / self.gpu_samples as f64)
    }
}

#[derive(Default)]
pub struct BenchHistory {
    recent: VecDeque<BenchSample>,
    trial: BenchSample,
}

impl BenchHistory {
    pub fn clear(&mut self) {
        self.recent.clear();
        self.trial = BenchSample::default();
    }

    pub fn record_window(&mut self, sample: BenchSample) {
        self.trial.add(sample);
        self.recent.push_back(sample);
        // 20 x approximately 3s samples plus wiggle room for short windows.
        while self.recent.len() > 24 {
            self.recent.pop_front();
        }
    }

    /// Full, unbounded trial since the last F3 start or benchmark setting
    /// change. Only O(1) aggregate state is retained for this mean.
    pub fn trial(&self) -> BenchSample {
        self.trial
    }

    /// Aggregate complete recent 3-second windows; window endpoints make the
    /// range approximate. Always report the measured span as well.
    pub fn recent(&self, target_seconds: f64) -> BenchSample {
        let mut sample = BenchSample::default();
        for entry in self.recent.iter().rev() {
            if sample.seconds >= target_seconds {
                break;
            }
            sample.add(*entry);
        }
        sample
    }
}

pub struct ConsoleLog {
    lines: VecDeque<String>,
    generation: u64,
    file: Option<BufWriter<File>>,
}

impl ConsoleLog {
    pub fn new() -> Self {
        let path = Path::new(LOG_FILE);
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("Could not create benchmark log directory: {error}");
            }
        }
        let file = match File::create(path) {
            Ok(file) => Some(BufWriter::new(file)),
            Err(error) => {
                eprintln!("Could not create {LOG_FILE}: {error}");
                None
            }
        };
        Self {
            lines: VecDeque::new(),
            generation: 0,
            file,
        }
    }

    pub fn record(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{line}");
            let _ = file.flush(); // file is usable while the game is running
        }
        self.lines.push_back(line);
        if self.lines.len() > MAX_ONSCREEN_LOG_LINES {
            self.lines.pop_front();
        }
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn text(&self) -> String {
        let mut out = self.lines.iter().cloned().collect::<Vec<_>>().join("\r\n");
        out.push_str("\r\n");
        out
    }

    /// Copy the complete on-disk session to Windows' built-in clipboard
    /// command, not only the 250 lines retained by the on-screen console.
    pub fn copy(&self) -> Result<(), String> {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use std::process::{Command, Stdio};
            let mut child = Command::new("clip.exe")
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| error.to_string())?;
            let mut stdin = child
                .stdin
                .take()
                .ok_or("Clipboard input pipe unavailable")?;
            let complete_log = fs::read_to_string(LOG_FILE).unwrap_or_else(|_| self.text());
            stdin
                .write_all(complete_log.as_bytes())
                .map_err(|error| error.to_string())?;
            drop(stdin);
            let status = child.wait().map_err(|error| error.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("Windows clipboard command returned {status}"))
            }
        }
        #[cfg(not(windows))]
        {
            Err(format!(
                "Clipboard copy is Windows-only; open {LOG_FILE} instead"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_uses_time_and_sample_weighted_means() {
        let mut history = BenchHistory::default();
        history.record_window(BenchSample {
            seconds: 3.0,
            frames: 300,
            cpu_draw_ms_total: 120.0,
            gpu_pass_ms_total: 2.0,
            gpu_samples: 10,
        });
        history.record_window(BenchSample {
            seconds: 3.0,
            frames: 600,
            cpu_draw_ms_total: 180.0,
            gpu_pass_ms_total: 6.0,
            gpu_samples: 20,
        });
        let all = history.trial();
        assert_eq!(all.fps(), 150.0);
        assert!((all.cpu_ms() - 1.0 / 3.0).abs() < 1e-10);
        assert!((all.gpu_ms().unwrap() - 8.0 / 30.0).abs() < 1e-10);
        history.clear();
        assert_eq!(history.trial().fps(), 0.0);
        assert_eq!(history.trial().gpu_ms(), None);
    }

    #[test]
    fn rolling_history_caps_memory_and_uses_recent_windows() {
        let mut history = BenchHistory::default();
        for i in 0..100 {
            history.record_window(BenchSample {
                seconds: 3.0,
                frames: (i + 1) * 30,
                ..BenchSample::default()
            });
        }
        let latest = history.recent(30.0);
        assert_eq!(latest.seconds, 30.0);
        assert_eq!(latest.frames, (91..=100).sum::<u64>() * 30);
        assert_eq!(history.recent(60.0).seconds, 60.0);
        assert_eq!(history.trial().seconds, 300.0);
        assert!(history.recent.len() <= 24);
    }
}
