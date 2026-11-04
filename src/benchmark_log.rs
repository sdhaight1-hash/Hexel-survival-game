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

