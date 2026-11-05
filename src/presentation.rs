//! Opt-in presentation A/B settings. Default remains FIFO with latency 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationOptions {
    pub mode: wgpu::PresentMode,
    pub max_frame_latency: u32,
    pub fps_cap: Option<u32>,
    /// Maximum new CPU sections and matching GPU meshes per streamed-world update.
    pub stream_budget: Option<usize>,
    /// Experimental distant spawn in existing unwrapped i32 axial coordinates.
    pub spawn_cell: Option<(i32, i32)>,
    /// Explicit opt-in periodic torus runtime with global i64 spawn.
    pub torus_spawn: Option<(i64, i64)>,
    /// Opt-in, reproducible R2/R3/R4 full-resolution distance benchmark.
    pub render_benchmark: bool,
    /// Opt-in visual-only coarse distant surface underlay.
    pub distant_terrain: bool,
}

impl Default for PresentationOptions {
    fn default() -> Self {
        Self {
            mode: wgpu::PresentMode::Fifo,
            max_frame_latency: 2,
            fps_cap: None,
            stream_budget: Some(1),
            spawn_cell: None,
            torus_spawn: None,
            render_benchmark: false,
            distant_terrain: false,
        }
    }
}

impl PresentationOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<(bool, Self), String> {
        let mut streamed = false;
        let mut spawn_q = None;
        let mut spawn_r = None;
        let mut torus_q = None;
        let mut torus_r = None;
        let mut torus_mode = false;
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--streamed-world" => streamed = true,
                "--render-benchmark" => options.render_benchmark = true,
                "--distant-terrain" => options.distant_terrain = true,
                "--torus-test-ne" => {
                    streamed = true;
                    torus_mode = true;
                }
                "--torus-spawn-q" => {
                    let value = args.next().ok_or("--torus-spawn-q needs i64")?;
                    torus_q = Some(
                        value
                            .parse::<i64>()
                            .map_err(|_| "Invalid --torus-spawn-q".to_string())?,
                    );
                }
                "--torus-spawn-r" => {
                    let value = args.next().ok_or("--torus-spawn-r needs i64")?;
                    torus_r = Some(
                        value
                            .parse::<i64>()
                            .map_err(|_| "Invalid --torus-spawn-r".to_string())?,
                    );
                }
                "--spawn-q" => {
                    let value = args.next().ok_or("--spawn-q needs a signed integer")?;
                    spawn_q = Some(value.parse::<i32>().map_err(|_| {
                        format!("Invalid --spawn-q '{value}'; use an i32 axial coordinate")
                    })?);
                }
                "--spawn-r" => {
                    let value = args.next().ok_or("--spawn-r needs a signed integer")?;
                    spawn_r = Some(value.parse::<i32>().map_err(|_| {
                        format!("Invalid --spawn-r '{value}'; use an i32 axial coordinate")
                    })?);
                }
                "--present-mode" => {
                    let value = args.next().ok_or("--present-mode needs a value")?;
                    options.mode = match value.as_str() {
                        "fifo" => wgpu::PresentMode::Fifo,
                        "fifo-relaxed" => wgpu::PresentMode::FifoRelaxed,
                        "mailbox" => wgpu::PresentMode::Mailbox,
                        "immediate" => wgpu::PresentMode::Immediate,
                        _ => {
                            return Err(format!(
                                "Unknown presentation mode '{value}'; use fifo, fifo-relaxed, mailbox or immediate"
                            ));
                        }
                    };
                }
                "--stream-budget" => {
                    let value = args.next().ok_or("--stream-budget needs a value")?;
                    options.stream_budget = match value.as_str() {
                        "1" => Some(1),
                        "2" => Some(2),
                        "4" => Some(4),
                        "unbounded" => None,
                        _ => {
                            return Err(format!(
                                "Invalid --stream-budget '{value}'; use 1, 2, 4 or unbounded"
                            ))
                        }
                    };
                }
                "--fps-cap" => {
                    let value = args.next().ok_or("--fps-cap needs a value")?;
                    options.fps_cap = match value.as_str() {
                        "60" => Some(60),
                        "120" => Some(120),
                        _ => return Err(format!("Invalid --fps-cap '{value}'; use 60 or 120")),
                    };
                }
                "--frame-latency" => {
                    let value = args.next().ok_or("--frame-latency needs a value")?;
                    options.max_frame_latency = value
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid --frame-latency '{value}'; use 1, 2 or 3"))?;
                    if !(1..=3).contains(&options.max_frame_latency) {
                        return Err(format!("Invalid --frame-latency '{value}'; use 1, 2 or 3"));
                    }
                }
                _ => return Err(format!("Unknown argument '{arg}'")),
            }
        }
        options.spawn_cell = match (spawn_q, spawn_r) {
            (None, None) => None,
            (Some(q), Some(r)) if streamed => {
                // Preserve headroom for streamed windows and 16³ mesh neighbors.
                const LIMIT: u32 = (i32::MAX - 128) as u32;
                if q.unsigned_abs() > LIMIT || r.unsigned_abs() > LIMIT {
                    return Err(
                        "Distant spawn must stay within the safe i32 coordinate range".into(),
                    );
                }
                Some((q, r))
            }
            (Some(_), Some(_)) => return Err("--spawn-q/--spawn-r require --streamed-world".into()),
            _ => return Err("--spawn-q and --spawn-r must be provided together".into()),
        };
        if torus_mode {
            if options.spawn_cell.is_some() {
                return Err("--torus-test-ne cannot combine with --spawn-q/--spawn-r".into());
            }
            options.torus_spawn = match (torus_q, torus_r) {
                (None, None) => Some((432_776, 1_182_375)),
                (Some(q), Some(r))
                    if q.unsigned_abs() < 9_000_000_000_000_u64
                        && r.unsigned_abs() < 9_000_000_000_000_u64 =>
                {
                    Some((q, r))
                }
                (Some(_), Some(_)) => return Err("Torus spawn is outside supported range".into()),
                _ => return Err("Both --torus-spawn-q and --torus-spawn-r are required".into()),
            };
        } else if torus_q.is_some() || torus_r.is_some() {
            return Err("Torus spawn coordinates require --torus-test-ne".into());
        }
        if options.distant_terrain && !torus_mode {
            return Err("--distant-terrain requires --torus-test-ne".into());
        }
        if options.render_benchmark && !torus_mode {
            return Err("--render-benchmark requires --torus-test-ne".into());
        }
        Ok((streamed, options))
    }

    /// Unsupported modes never reach surface.configure; use the known-safe FIFO baseline.
    pub fn resolve_mode(self, supported: &[wgpu::PresentMode]) -> wgpu::PresentMode {
        if supported.contains(&self.mode) {
            self.mode
        } else {
            wgpu::PresentMode::Fifo
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keeps_existing_presentation() {
        let (streamed, settings) = PresentationOptions::parse(Vec::<String>::new()).unwrap();
        assert!(!streamed);
        assert_eq!(settings, PresentationOptions::default());
        assert_eq!(settings.mode, wgpu::PresentMode::Fifo);
        assert_eq!(settings.max_frame_latency, 2);
        assert_eq!(settings.fps_cap, None);
        assert_eq!(settings.stream_budget, Some(1));
    }

    #[test]
    fn streamed_world_defaults_to_one_section_and_allows_unbounded_override() {
        let (streamed, settings) =
            PresentationOptions::parse(["--streamed-world".to_owned()]).unwrap();
        assert!(streamed);
        assert_eq!(settings.stream_budget, Some(1));
        assert_eq!(settings.mode, wgpu::PresentMode::Fifo);
        assert_eq!(settings.max_frame_latency, 2);
        assert_eq!(settings.fps_cap, None);
        let (_, unlimited) = PresentationOptions::parse(
            ["--streamed-world", "--stream-budget", "unbounded"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(unlimited.stream_budget, None);
    }

    #[test]
    fn opt_in_distant_spawn_requires_complete_streamed_coordinates() {
        let (streamed, options) = PresentationOptions::parse(
            [
                "--streamed-world",
                "--spawn-q",
                "1023984",
                "--spawn-r",
                "-256",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert!(streamed);
        assert_eq!(options.spawn_cell, Some((1_023_984, -256)));
        assert_eq!(options.stream_budget, Some(1));
        for args in [
            vec!["--spawn-q", "10", "--spawn-r", "20"],
            vec!["--streamed-world", "--spawn-q", "10"],
            vec!["--streamed-world", "--spawn-q", "abc", "--spawn-r", "0"],
            vec![
                "--streamed-world",
                "--spawn-q",
                "-2147483648",
                "--spawn-r",
                "0",
            ],
        ] {
            assert!(PresentationOptions::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn distant_terrain_is_opt_in_and_requires_periodic_world() {
        assert!(PresentationOptions::parse(["--distant-terrain".to_string()]).is_err());
        let (_, options) = PresentationOptions::parse(
            ["--torus-test-ne", "--distant-terrain"]
                .into_iter().map(str::to_owned)
        ).unwrap();
        assert!(options.distant_terrain);
        assert!(!PresentationOptions::default().distant_terrain);
    }

    #[test]
    fn render_benchmark_is_opt_in_and_requires_periodic_world() {
        assert!(PresentationOptions::parse(["--render-benchmark".to_string()]).is_err());
        let (_, settings) = PresentationOptions::parse(
            ["--torus-test-ne", "--render-benchmark"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        assert!(settings.render_benchmark);
        assert_eq!(settings.torus_spawn, Some((432_776, 1_182_375)));
        assert!(!PresentationOptions::default().render_benchmark);
    }

    #[test]
    fn arbitrary_global_torus_spawn_is_opt_in_and_paired() {
        let parse =
            |args: &[&str]| PresentationOptions::parse(args.iter().map(|s| (*s).to_owned()));
        let (streamed, ne) = parse(&["--torus-test-ne"]).unwrap();
        assert!(streamed);
        assert_eq!(ne.torus_spawn, Some((432_776, 1_182_375)));
        let (_, far) = parse(&[
            "--torus-test-ne",
            "--torus-spawn-q",
            "-8192000000",
            "--torus-spawn-r",
            "1182400",
        ])
        .unwrap();
        assert_eq!(far.torus_spawn, Some((-8_192_000_000, 1_182_400)));
        assert!(parse(&["--torus-test-ne", "--torus-spawn-q", "12"]).is_err());
        assert!(parse(&["--torus-spawn-q", "1", "--torus-spawn-r", "2"]).is_err());
        assert!(parse(&["--torus-test-ne", "--spawn-q", "10", "--spawn-r", "20"]).is_err());
    }

    #[test]
    fn parses_explicit_ab_settings() {
        let (streamed, settings) = PresentationOptions::parse(
            [
                "--streamed-world",
                "--present-mode",
                "mailbox",
                "--frame-latency",
                "1",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert!(streamed);
        assert_eq!(settings.mode, wgpu::PresentMode::Mailbox);
        assert_eq!(settings.max_frame_latency, 1);
    }

    #[test]
    fn accepts_opt_in_60_and_120_caps() {
        for (fps, expected) in [("60", Some(60)), ("120", Some(120))] {
            let (_, settings) =
                PresentationOptions::parse(["--fps-cap", fps].into_iter().map(str::to_owned))
                    .unwrap();
            assert_eq!(settings.fps_cap, expected);
        }
    }

    #[test]
    fn parses_opt_in_stream_budgets() {
        for (size, expected) in [
            ("1", Some(1)),
            ("2", Some(2)),
            ("4", Some(4)),
            ("unbounded", None),
        ] {
            let (_, settings) = PresentationOptions::parse(
                ["--stream-budget", size].into_iter().map(str::to_owned),
            )
            .unwrap();
            assert_eq!(settings.stream_budget, expected);
        }
    }

    #[test]
    fn unsupported_mode_falls_back_to_fifo() {
        let settings = PresentationOptions {
            mode: wgpu::PresentMode::Immediate,
            ..PresentationOptions::default()
        };
        assert_eq!(
            settings.resolve_mode(&[wgpu::PresentMode::Fifo, wgpu::PresentMode::Mailbox]),
            wgpu::PresentMode::Fifo
        );
        assert_eq!(
            settings.resolve_mode(&[wgpu::PresentMode::Fifo, wgpu::PresentMode::Immediate]),
            wgpu::PresentMode::Immediate
        );
    }

    #[test]
    fn rejects_invalid_modes_and_latencies() {
        for args in [
            vec!["--present-mode", "unknown"],
            vec!["--present-mode"],
            vec!["--frame-latency", "0"],
            vec!["--frame-latency", "4"],
            vec!["--frame-latency", "invalid"],
            vec!["--frame-latency"],
            vec!["--fps-cap"],
            vec!["--fps-cap", "0"],
            vec!["--fps-cap", "30"],
            vec!["--fps-cap", "garbage"],
            vec!["--stream-budget"],
            vec!["--stream-budget", "0"],
            vec!["--stream-budget", "9"],
            vec!["--stream-budget", "garbage"],
        ] {
            assert!(PresentationOptions::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
