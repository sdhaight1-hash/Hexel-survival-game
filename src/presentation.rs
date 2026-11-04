//! Opt-in presentation A/B settings. Default remains FIFO with latency 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationOptions {
    pub mode: wgpu::PresentMode,
    pub max_frame_latency: u32,
    pub fps_cap: Option<u32>,
    /// Maximum new CPU sections and matching GPU meshes per streamed-world update.
    pub stream_budget: Option<usize>,
}

impl Default for PresentationOptions {
    fn default() -> Self {
        Self {
            mode: wgpu::PresentMode::Fifo,
            max_frame_latency: 2,
            fps_cap: None,
            stream_budget: Some(1),
        }
    }
}

impl PresentationOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<(bool, Self), String> {
        let mut streamed = false;
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--streamed-world" => streamed = true,
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
                            return Err(format!("Invalid --stream-budget '{value}'; use 1, 2, 4 or unbounded"))
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
        for (size, expected) in [("1", Some(1)), ("2", Some(2)), ("4", Some(4)), ("unbounded", None)] {
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
