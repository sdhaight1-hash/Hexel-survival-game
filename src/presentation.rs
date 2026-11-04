//! Opt-in presentation A/B settings. Default remains FIFO with latency 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationOptions {
    pub mode: wgpu::PresentMode,
    pub max_frame_latency: u32,
}

impl Default for PresentationOptions {
    fn default() -> Self {
        Self {
            mode: wgpu::PresentMode::Fifo,
            max_frame_latency: 2,
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
                "--frame-latency" => {
                    let value = args.next().ok_or("--frame-latency needs a value")?;
                    options.max_frame_latency = value.parse::<u32>().map_err(|_| {
                        format!("Invalid --frame-latency '{value}'; use 1, 2 or 3")
                    })?;
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
    }

    #[test]
    fn parses_explicit_ab_settings() {
        let (streamed, settings) = PresentationOptions::parse([
            "--streamed-world",
            "--present-mode",
            "mailbox",
            "--frame-latency",
            "1",
        ].into_iter().map(str::to_owned)).unwrap();
        assert!(streamed);
        assert_eq!(settings.mode, wgpu::PresentMode::Mailbox);
        assert_eq!(settings.max_frame_latency, 1);
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
        ] {
            assert!(PresentationOptions::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
