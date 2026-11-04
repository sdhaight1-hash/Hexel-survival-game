//! First-person one-chunk demo or opt-in 200-chunk material benchmark.

mod debug_menu;
mod materials_p0;
mod materials_v2;
mod mesh;
mod render;
mod texture_codec;
mod texture_pack;

use debug_menu::{hit_test, MenuAction};
use hexel_survival_game::{
    player::{MoveInput, Player},
    world::World,
};
use render::Renderer;
use std::{collections::HashSet, sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

struct Game {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    player: Player,
    pressed: HashSet<KeyCode>,
    mouse_captured: bool,
    menu_open: bool,
    menu_cursor: Option<(f64, f64)>,
    menu_hover: Option<MenuAction>,
    last_frame: Instant,
    benchmark_logging: bool,
    benchmark_window_start: Instant,
    benchmark_frames: u32,
    benchmark_draw_ms_total: f64,
    benchmark_gpu_ms_total: f64,
    benchmark_gpu_samples: u32,
    benchmark_frame_intervals_ms: Vec<f64>,
    benchmark_last_present: Option<Instant>,
}

impl Default for Game {
    fn default() -> Self {
        // A deliberately opt-in resident scene, not streaming or worldgen.
        let world = if std::env::args().any(|arg| arg == "--benchmark-200") {
            World::benchmark(42)
        } else {
            World::demo(42)
        };
        let player = Player::spawn(&world);
        Self {
            window: None,
            renderer: None,
            world,
            player,
            pressed: HashSet::new(),
            mouse_captured: false,
            menu_open: false,
            menu_cursor: None,
            menu_hover: None,
            last_frame: Instant::now(),
            benchmark_logging: false,
            benchmark_window_start: Instant::now(),
            benchmark_frames: 0,
            benchmark_draw_ms_total: 0.0,
            benchmark_gpu_ms_total: 0.0,
            benchmark_gpu_samples: 0,
            benchmark_frame_intervals_ms: Vec::new(),
            benchmark_last_present: None,
        }
    }
}

impl Game {
    fn capture_mouse(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        match window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
        {
            Ok(()) => {
                window.set_cursor_visible(false);
                self.mouse_captured = true;
            }
            Err(error) => eprintln!("Could not capture mouse: {error}"),
        }
    }

    fn release_mouse(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
        self.mouse_captured = false;
        self.pressed.clear();
    }

    fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
        self.menu_hover = None;
        self.menu_cursor = None;
        self.pressed.clear(); // avoid held movement keys leaking out of the menu
        if self.menu_open {
            self.release_mouse();
            println!("F1 diagnostics menu: OPEN (click a row; F1/Esc closes)");
        } else {
            self.capture_mouse();
            println!("F1 diagnostics menu: CLOSED");
        }
    }

    fn reset_benchmark_window(&mut self) {
        self.benchmark_window_start = Instant::now();
        self.benchmark_frames = 0;
        self.benchmark_draw_ms_total = 0.0;
        self.benchmark_gpu_ms_total = 0.0;
        self.benchmark_gpu_samples = 0;
        self.benchmark_frame_intervals_ms.clear();
        self.benchmark_last_present = None;
    }

    /// One action path for F-keys and mouse-selected menu rows.
    fn apply_debug_action(&mut self, action: MenuAction) {
        if action == MenuAction::FrameStats {
            self.benchmark_logging = !self.benchmark_logging;
            self.reset_benchmark_window();
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.set_gpu_profiling(self.benchmark_logging);
            }
            println!(
                "F3 benchmark logging: {}",
                if self.benchmark_logging { "ON" } else { "OFF" }
            );
            return;
        }
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        match action {
            MenuAction::FrameStats => unreachable!(),
            MenuAction::Presentation => match renderer.toggle_present_mode() {
                Some(true) => println!("F5 presentation: UNLOCKED (Immediate, tearing possible)"),
                Some(false) => println!("F5 presentation: VSYNC (FIFO)"),
                None => println!(
                    "F5 unavailable: Immediate presentation not supported in this benchmark."
                ),
            },
            MenuAction::ChunkCulling => match renderer.toggle_chunk_culling() {
                Some(true) => println!("F4 nearby-chunk culling: ON"),
                Some(false) => println!("F4 nearby-chunk culling: OFF (all resident chunks drawn)"),
                None => println!("F4 available only in multi-chunk benchmark mode."),
            },
            MenuAction::UvChecker => {
                let active = renderer.toggle_checker();
                println!("P0 UV checker: {}", if active { "ON" } else { "OFF" });
            }
            MenuAction::UvDensity => {
                println!(
                    "P0 checker scale: {} texels/metre",
                    renderer.cycle_density()
                );
            }
            MenuAction::V2Materials => match renderer.toggle_v2_preview() {
                Some(true) => println!("V2 candidate gallery: ON"),
                Some(false) => println!("V2 candidate gallery: OFF (P0)"),
                None => {
                    println!("V2 gallery unavailable: bake compatible candidate textures first.")
                }
            },
            MenuAction::SurfacePreset => match renderer.cycle_surface_preset() {
                Some(name) => println!("F8 surface preset: {name}"),
                None => println!("F9 unavailable: bake v2 candidate art first."),
            },
            MenuAction::MetalGlints => match renderer.toggle_metallic_glints() {
                Some(true) => println!("F8 copper metallic glints: ON"),
                Some(false) => println!("F8 copper metallic glints: OFF"),
                None => println!("F10 unavailable: bake v2 candidate art first."),
            },
            MenuAction::GrayCopper => match renderer.toggle_grayscale_copper() {
                Some(true) => println!("F11 copper comparison: grayscale mask + shader palette"),
                Some(false) => println!("F11 copper comparison: original full-color candidate"),
                None => {
                    println!("F11 unavailable: enable F8 with a baked native-copper candidate.")
                }
            },
        }
        if self.benchmark_logging {
            self.reset_benchmark_window(); // never mix settings within one benchmark interval
        }
    }

    fn input(&self) -> MoveInput {
        if !self.mouse_captured {
            return MoveInput::default();
        }
        MoveInput {
            forward: self.pressed.contains(&KeyCode::KeyW),
            backward: self.pressed.contains(&KeyCode::KeyS),
            left: self.pressed.contains(&KeyCode::KeyA),
            right: self.pressed.contains(&KeyCode::KeyD),
            jump: self.pressed.contains(&KeyCode::Space),
            sprint: self.pressed.contains(&KeyCode::ControlLeft),
            sneak: self.pressed.contains(&KeyCode::ShiftLeft),
        }
    }
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title(if self.world.is_benchmark() {
                    "Hexel Survival | 200-chunk benchmark | F3 stats | F4 culling"
                } else {
                    "Hexel Survival | 16x16x16 | WASD | Mouse | Space | Ctrl | Shift | Esc"
                })
                .with_inner_size(PhysicalSize::new(1280, 800)),
        ) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("Could not create game window: {error}");
                event_loop.exit();
                return;
            }
        };

        match materials_v2::inspect_default() {
            Ok(None | Some((0, 0))) => println!("V2 materials: no production assets; P0 gallery remains active."),
            Ok(Some((textures, materials))) => println!(
                "V2 materials: validated {textures} texture file(s) and {materials} recipe(s). F8 preview is available when compatible gallery candidates exist."
            ),
            Err(error) => eprintln!("V2 material cache rejected: {error}; P0 gallery remains active."),
        }

        match pollster::block_on(Renderer::new(window.clone(), &self.world)) {
            Ok(renderer) => {
                self.window = Some(window);
                self.renderer = Some(renderer);
                self.capture_mouse();
                self.last_frame = Instant::now();
                println!(
                    "Hexel Survival: {} resident 16³ chunk(s), deterministic demo seed 42.",
                    self.world.chunk_count()
                );
                println!(
                    "WASD move | Mouse look | Space jump | Left Ctrl sprint | Left Shift sneak"
                );
                println!("F1: clickable diagnostics menu (F1/Esc closes); F3–F11 shortcuts remain active.");
                println!(
                    "F5: opt-in VSYNC / UNLOCKED (Immediate) benchmark presentation if available."
                );
                println!("F3: frame p50/p95/p99 + asynchronous GPU render-pass timestamps when supported.");
                println!("Esc release/recapture mouse | Click window to recapture.");
                println!(
                    "Static loaded-chunk boundaries are barriers; streaming/torus wrapping are NOT implemented."
                );
                println!("P0 material gallery faces your initial spawn at r=9..12.");
                println!("F6: toggle six-face UV checker | F7: cycle checker at 64 / 128 / 256 texels/m.");
                println!("F8: toggle optional v2 candidate rock/copper gallery vs P0.");
                println!("F9: cycle surface preview Dry / Wet / Moss / Frost / Combined (F8).");
                println!("F10: toggle native-copper metallic highlights (F8).");
                println!("F11: compare original full-color copper vs shader-colored grayscale copper (F8).");
                println!("F3: toggle 3-second FPS / CPU frame / visible chunk stats.");
                if self.world.is_benchmark() {
                    println!("F4: toggle 112m distance culling vs all 200 resident chunk draws.");
                    println!("Benchmark: static 20x10x1 chunks with repeated visual ore fixtures, NOT geology/streaming.");
                }
            }
            Err(error) => {
                eprintln!("Failed to initialize the first-person renderer: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.release_mouse();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.release_mouse(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if key == KeyCode::Escape {
                        if event.state == ElementState::Pressed && !event.repeat {
                            if self.menu_open {
                                self.toggle_menu();
                            } else if self.mouse_captured {
                                self.release_mouse();
                            } else {
                                self.capture_mouse();
                            }
                        }
                        return;
                    }
                    if key == KeyCode::F1 {
                        if event.state == ElementState::Pressed && !event.repeat {
                            self.toggle_menu();
                        }
                        return;
                    }
                    if event.state == ElementState::Pressed && !event.repeat {
                        if let Some(action) = MenuAction::from_key(key) {
                            self.apply_debug_action(action);
                            return;
                        }
                    }
                    if event.state == ElementState::Pressed {
                        self.pressed.insert(key);
                    } else {
                        self.pressed.remove(&key);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.menu_cursor = Some((position.x, position.y));
                if self.menu_open {
                    if let Some(window) = self.window.as_ref() {
                        let size = window.inner_size();
                        self.menu_hover = hit_test(position.x, position.y, size.width, size.height);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.menu_open {
                    if let (Some((x, y)), Some(window), Some(renderer)) = (
                        self.menu_cursor,
                        self.window.as_ref(),
                        self.renderer.as_ref(),
                    ) {
                        let size = window.inner_size();
                        if let Some(action) = hit_test(x, y, size.width, size.height) {
                            if renderer
                                .menu_snapshot(self.benchmark_logging)
                                .enabled(action)
                            {
                                self.apply_debug_action(action);
                            }
                        }
                    }
                } else if !self.mouse_captured {
                    self.capture_mouse();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let menu = self.menu_open.then(|| {
                        (
                            renderer.menu_snapshot(self.benchmark_logging),
                            self.menu_hover,
                        )
                    });
                    match renderer.draw(&self.player, menu) {
                        Ok(()) => {
                            if self.benchmark_logging {
                                let now = Instant::now();
                                if let Some(previous) = self.benchmark_last_present {
                                    self.benchmark_frame_intervals_ms
                                        .push(now.duration_since(previous).as_secs_f64() * 1000.0);
                                }
                                self.benchmark_last_present = Some(now);
                                self.benchmark_frames += 1;
                                let stats = renderer.stats();
                                self.benchmark_draw_ms_total += f64::from(stats.draw_cpu_ms);
                                if let Some(ms) = stats.gpu_pass_ms {
                                    self.benchmark_gpu_ms_total += f64::from(ms);
                                    self.benchmark_gpu_samples += 1;
                                }
                                let elapsed = self.benchmark_window_start.elapsed().as_secs_f64();
                                if elapsed >= 3.0 {
                                    self.benchmark_frame_intervals_ms.sort_by(f64::total_cmp);
                                    let p50 =
                                        frame_percentile_ms(&self.benchmark_frame_intervals_ms, 50);
                                    let p95 =
                                        frame_percentile_ms(&self.benchmark_frame_intervals_ms, 95);
                                    let p99 =
                                        frame_percentile_ms(&self.benchmark_frame_intervals_ms, 99);
                                    let gpu = if self.benchmark_gpu_samples > 0 {
                                        format!(
                                            "{:.2} ms GPU pass ({} samples)",
                                            self.benchmark_gpu_ms_total
                                                / f64::from(self.benchmark_gpu_samples),
                                            self.benchmark_gpu_samples
                                        )
                                    } else if renderer.gpu_timestamps_supported() {
                                        "GPU pass pending".to_string()
                                    } else {
                                        "GPU pass N/A".to_string()
                                    };
                                    println!(
                                        "BENCH [{} | {}]: {:.1} FPS wall, frame p50/p95/p99 {:.2}/{:.2}/{:.2} ms, {:.2} ms CPU draw incl. acquire/present, {}, {}/{} submitted chunks, {} vertices",
                                        renderer.material_comparison_label(),
                                        renderer.presentation_label(),
                                        f64::from(self.benchmark_frames) / elapsed,
                                        p50,
                                        p95,
                                        p99,
                                        self.benchmark_draw_ms_total / f64::from(self.benchmark_frames),
                                        gpu,
                                        stats.submitted_chunks,
                                        self.world.chunk_count(),
                                        stats.submitted_vertices,
                                    );
                                    self.reset_benchmark_window();
                                }
                            }
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            if let Some(window) = self.window.as_ref() {
                                renderer.resize(window.inner_size());
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(error) => eprintln!("Surface error: {error}"),
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if self.mouse_captured {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.player.look(dx as f32, dy as f32);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.player.step(&self.world, self.input(), dt);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

/// Nearest sampled frame interval at a requested percentile. Input is sorted
/// by the caller; empty windows report zero instead of implying GPU timing.
fn frame_percentile_ms(sorted: &[f64], percentile: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[(sorted.len() * percentile / 100).min(sorted.len() - 1)]
}

#[cfg(test)]
mod benchmark_stats_tests {
    use super::*;

    #[test]
    fn frame_percentiles_use_observed_sorted_intervals() {
        assert_eq!(frame_percentile_ms(&[], 95), 0.0);
        let frames: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(frame_percentile_ms(&frames, 50), 51.0);
        assert_eq!(frame_percentile_ms(&frames, 95), 96.0);
        assert_eq!(frame_percentile_ms(&frames, 99), 100.0);
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Could not initialize window event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    if let Err(error) = event_loop.run_app(&mut Game::default()) {
        eprintln!("Game event loop stopped: {error}");
    }
}

#[cfg(test)]
mod shader_tests {
    #[test]
    fn debug_menu_shader_parses_and_validates() {
        let source = include_str!("debug_menu.wgsl");
        let module =
            naga::front::wgsl::parse_str(source).expect("debug menu shader must parse as WGSL");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("debug menu shader must pass Naga semantic validation");
    }

    #[test]
    fn p2_surface_shader_parses_and_validates() {
        let source = include_str!("world_shader.wgsl");
        let module =
            naga::front::wgsl::parse_str(source).expect("P2.2 world shader must parse as WGSL");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("P2.2 world shader must pass Naga semantic validation");
    }
}
