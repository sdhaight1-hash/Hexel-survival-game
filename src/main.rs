//! Native first-person material demo or opt-in moving streamed-world prototype.

mod benchmark_log;
mod debug_menu;
mod materials_p0;
mod materials_v2;
mod mesh;
mod render;
mod stream_metrics;
mod texture_codec;
mod texture_pack;

use benchmark_log::ConsoleLog;
use debug_menu::{console_copy_hit_test, hit_test, MenuAction};
use hexel_survival_game::{
    player::{MoveInput, Player},
    world::World,
};
use render::Renderer;
use std::{collections::HashSet, sync::Arc, time::Instant};
use stream_metrics::StreamFrameMetrics;
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
    last_frame: Instant,
    last_stream_report: Instant,
    streamed_generated: usize,
    streamed_evicted: usize,
    streamed_meshes: usize,
    streamed_uploaded_bytes: u64,
    streamed_generation_cpu_ms: f64,
    streamed_meshing_cpu_ms: f64,
    stream_metrics: StreamFrameMetrics,
    menu_open: bool,
    console_open: bool,
    menu_cursor: Option<(f64, f64)>,
    menu_hover: Option<MenuAction>,
    frame_stats: bool,
    console_log: ConsoleLog,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Game {
    fn new(streamed: bool) -> Self {
        let mut world = if streamed {
            World::streamed(42)
        } else {
            World::demo(42)
        };
        let player = Player::spawn(&world);
        let initial = world.stream_around(player.feet);
        if streamed {
            println!(
                "STREAM initial: {} resident 16³ sections, {:.1} ms CPU generation; seed 42",
                world.chunk_count(),
                initial.generation_cpu_ms
            );
        }
        let now = Instant::now();
        Self {
            window: None,
            renderer: None,
            world,
            player,
            pressed: HashSet::new(),
            mouse_captured: false,
            last_frame: now,
            last_stream_report: now,
            streamed_generated: 0,
            streamed_evicted: 0,
            streamed_meshes: 0,
            streamed_uploaded_bytes: 0,
            streamed_generation_cpu_ms: 0.0,
            streamed_meshing_cpu_ms: 0.0,
            stream_metrics: StreamFrameMetrics::default(),
            menu_open: false,
            console_open: false,
            menu_cursor: None,
            menu_hover: None,
            frame_stats: false,
            console_log: ConsoleLog::new(),
        }
    }

    fn capture_mouse(&mut self) {
        if self.menu_open || self.console_open {
            return;
        }
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
        self.console_open = false;
        self.menu_hover = None;
        if self.menu_open {
            self.release_mouse();
            self.console_log.record("F1 diagnostics menu: OPEN");
        } else {
            self.console_log.record("F1 diagnostics menu: CLOSED");
            self.capture_mouse();
        }
    }

    fn toggle_console(&mut self) {
        self.console_open = !self.console_open;
        self.menu_open = false;
        self.menu_hover = None;
        if self.console_open {
            self.release_mouse();
            self.console_log
                .record("F2 bottom log: OPEN (Ctrl+C copies full session)");
        } else {
            self.console_log.record("F2 bottom log: CLOSED");
            self.capture_mouse();
        }
    }

    fn copy_console(&mut self) {
        match self.console_log.copy() {
            Ok(()) => self
                .console_log
                .record("F2: full session copied to clipboard"),
            Err(error) => self.console_log.record(format!("F2 copy failed: {error}")),
        }
    }

    fn apply_debug_action(&mut self, action: MenuAction) {
        if action == MenuAction::Console {
            self.toggle_console();
            return;
        }
        if action == MenuAction::FrameStats {
            self.frame_stats = !self.frame_stats;
            self.console_log.record(format!(
                "F3 streaming metrics: {}",
                if self.frame_stats { "ON" } else { "OFF" }
            ));
            return;
        }
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        if !renderer.menu_snapshot(self.frame_stats).enabled(action) {
            return;
        }
        let message = match action {
            MenuAction::UvChecker => format!("F6 UV checker: {}", renderer.toggle_checker()),
            MenuAction::UvDensity => format!(
                "F7 checker scale: {} texels/metre",
                renderer.cycle_density()
            ),
            MenuAction::V2Materials => {
                format!("F8 candidate materials: {:?}", renderer.toggle_v2_preview())
            }
            MenuAction::SurfacePreset => {
                format!("F9 surface preset: {:?}", renderer.cycle_surface_preset())
            }
            MenuAction::MetalGlints => {
                format!("F10 metal glints: {:?}", renderer.toggle_metallic_glints())
            }
            _ => return,
        };
        self.console_log.record(message);
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
                .with_title("Hexel Survival | 16x16x16 | WASD | Mouse | Space | Ctrl | Shift | Esc")
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
                self.console_log.record(benchmark_log::system_report());
                self.console_log.record(format!(
                    "WORLD {} resident sections",
                    self.world.chunk_count()
                ));
                self.console_log
                    .record("F1 menu | F2 console + Ctrl+C | F3 streamed frame metrics");
                if self.world.is_streamed() {
                    println!(
                        "Hexel Survival STREAMED WORLD: {} resident sections; procedural source seed 42.",
                        self.world.chunk_count()
                    );
                    println!(
                        "No static terrain boundary. Q/R/Y streaming ON. No torus wrap, edits, caves, distant LOD or camera rebasing yet."
                    );
                    println!("STREAM prints 3-second resident/generation/mesh/upload counters and per-update CPU peaks plus frame interval percentiles.");
                } else {
                    println!("Hexel Survival: one 16×16×16 chunk, deterministic demo seed 42.");
                    println!(
                        "Chunk edges are temporary barriers; torus wrapping is NOT implemented."
                    );
                    println!("P0 material gallery faces your initial spawn at r=9..12.");
                }
                println!(
                    "WASD move | Mouse look | Space jump | Left Ctrl sprint | Left Shift sneak"
                );
                println!("Esc release/recapture mouse | Click window to recapture.");
                println!("F6: toggle six-face UV checker | F7: cycle checker at 64 / 128 / 256 texels/m.");
                println!("F8: toggle optional v2 candidate rock/copper gallery vs P0.");
                println!("F9: cycle surface preview Dry / Wet / Moss / Frost / Combined (F8).");
                println!("F10: toggle native-copper metallic highlights (F8).");
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
                    if event.state == ElementState::Pressed && !event.repeat {
                        if key == KeyCode::Escape {
                            if self.menu_open {
                                self.toggle_menu();
                            } else if self.console_open {
                                self.toggle_console();
                            } else if self.mouse_captured {
                                self.release_mouse();
                            } else {
                                self.capture_mouse();
                            }
                            return;
                        }
                        if key == KeyCode::F1 {
                            self.toggle_menu();
                            return;
                        }
                        if key == KeyCode::F2 {
                            self.toggle_console();
                            return;
                        }
                        if self.console_open
                            && key == KeyCode::KeyC
                            && (self.pressed.contains(&KeyCode::ControlLeft)
                                || self.pressed.contains(&KeyCode::ControlRight))
                        {
                            self.copy_console();
                            return;
                        }
                        if let Some(action) = MenuAction::from_key(key) {
                            self.apply_debug_action(action);
                            return;
                        }
                    }
                    // Track modifier keys even while the overlay owns input:
                    // Ctrl+C must work in F2 without forwarding movement controls.
                    if self.menu_open || self.console_open {
                        if matches!(key, KeyCode::ControlLeft | KeyCode::ControlRight) {
                            if event.state == ElementState::Pressed {
                                self.pressed.insert(key);
                            } else {
                                self.pressed.remove(&key);
                            }
                        }
                        return;
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
                if self.console_open {
                    if let (Some((x, y)), Some(window)) = (self.menu_cursor, self.window.as_ref()) {
                        let size = window.inner_size();
                        if console_copy_hit_test(x, y, size.width, size.height) {
                            self.copy_console();
                        }
                    }
                } else if self.menu_open {
                    if let (Some((x, y)), Some(window)) = (self.menu_cursor, self.window.as_ref()) {
                        let size = window.inner_size();
                        if let Some(action) = hit_test(x, y, size.width, size.height) {
                            self.apply_debug_action(action);
                        }
                    }
                } else if !self.mouse_captured {
                    self.capture_mouse();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let menu = self
                        .menu_open
                        .then(|| (renderer.menu_snapshot(self.frame_stats), self.menu_hover));
                    let console = self
                        .console_open
                        .then(|| (self.console_log.lines(), self.console_log.generation()));
                    match renderer.draw(&self.player, menu, console) {
                        Ok(()) => {}
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
        if self.world.is_streamed() {
            self.stream_metrics
                .record_frame_interval(f64::from(dt) * 1000.0);
        }
        self.last_frame = now;
        self.player.step(&self.world, self.input(), dt);
        if self.world.is_streamed() {
            let delta = self.world.stream_around(self.player.feet);
            if !delta.added.is_empty() || !delta.removed.is_empty() {
                self.streamed_generated += delta.added.len();
                self.streamed_evicted += delta.removed.len();
                self.streamed_generation_cpu_ms += delta.generation_cpu_ms;
                let mut mesh_upload_ms = 0.0;
                if let Some(renderer) = self.renderer.as_mut() {
                    let result = renderer.sync_stream(&self.world, &delta);
                    self.streamed_meshes += result.meshed_buffers;
                    self.streamed_uploaded_bytes += result.uploaded_bytes;
                    self.streamed_meshing_cpu_ms += result.cpu_ms;
                    mesh_upload_ms = result.cpu_ms;
                }
                self.stream_metrics
                    .record_stream_update(delta.generation_cpu_ms, mesh_upload_ms);
            }
            if now.duration_since(self.last_stream_report).as_secs_f64() >= 3.0 {
                let (gpu_meshes, submitted_vertices) = self
                    .renderer
                    .as_ref()
                    .map_or((0, 0), Renderer::terrain_counts);
                self.console_log.record(format!(
                    "STREAM: {} resident chunks, {} nonempty GPU meshes, {} submitted vertices; +{}/-{} chunks, {} new GPU buffers, {:.2} MiB uploaded; {:.2} ms CPU generation, {:.2} ms CPU mesh/buffer calls (last {:.1}s).",
                    self.world.chunk_count(),
                    gpu_meshes,
                    submitted_vertices,
                    self.streamed_generated,
                    self.streamed_evicted,
                    self.streamed_meshes,
                    self.streamed_uploaded_bytes as f64 / 1_048_576.0,
                    self.streamed_generation_cpu_ms,
                    self.streamed_meshing_cpu_ms,
                    now.duration_since(self.last_stream_report).as_secs_f64(),
                ));
                self.streamed_generated = 0;
                self.streamed_evicted = 0;
                self.streamed_meshes = 0;
                self.streamed_uploaded_bytes = 0;
                self.streamed_generation_cpu_ms = 0.0;
                self.streamed_meshing_cpu_ms = 0.0;
                let report = self.stream_metrics.report_and_reset();
                if self.frame_stats {
                    self.console_log.record(report);
                }
                self.last_stream_report = now;
            }
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Could not initialize window event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let streamed = std::env::args()
        .skip(1)
        .any(|arg| arg == "--streamed-world");
    if let Err(error) = event_loop.run_app(&mut Game::new(streamed)) {
        eprintln!("Game event loop stopped: {error}");
    }
}

#[cfg(test)]
mod shader_tests {
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
