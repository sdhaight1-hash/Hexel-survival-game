//! Native first-person material demo or opt-in moving streamed-world prototype.

mod benchmark_log;
mod debug_menu;
mod frame_pacing;
mod materials_p0;
mod materials_v2;
mod mesh;
mod presentation;
mod render;
mod render_cpu_metrics;
mod render_distance_benchmark;
mod seed_lod;
mod stream_metrics;
mod texture_codec;
mod texture_pack;

use benchmark_log::ConsoleLog;
use debug_menu::{console_copy_hit_test, hit_test, MenuAction};
use frame_pacing::FramePacer;
use hexel_survival_game::{
    hex_torus_candidate::Axial64,
    player::{MoveInput, Player},
    torus_frame::{self, CrossingCounts, GlobalCell},
    world::{CellPos, World},
};
use presentation::PresentationOptions;
use render::Renderer;
use render_cpu_metrics::RenderCpuMetrics;
use render_distance_benchmark::RenderDistanceBenchmark;
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
    presentation: PresentationOptions,
    frame_pacer: FramePacer,
    player: Player,
    pressed: HashSet<KeyCode>,
    mouse_captured: bool,
    pending_mouse_dx: f64,
    pending_mouse_dy: f64,
    is_looking: bool,
    last_frame: Instant,
    last_stream_report: Instant,
    streamed_generated: usize,
    streamed_evicted: usize,
    streamed_meshes: usize,
    streamed_uploaded_bytes: u64,
    streamed_generation_cpu_ms: f64,
    streamed_meshing_cpu_ms: f64,
    stream_pending: usize,
    stream_peak_pending: usize,
    stream_backlog_since: Option<Instant>,
    stream_completed_backlogs: usize,
    stream_longest_backlog_ms: f64,
    stream_metrics: StreamFrameMetrics,
    render_cpu_metrics: RenderCpuMetrics,
    menu_open: bool,
    console_open: bool,
    menu_cursor: Option<(f64, f64)>,
    menu_hover: Option<MenuAction>,
    frame_stats: bool,
    console_log: ConsoleLog,
    previous_torus: Option<GlobalCell>,
    torus_crossings: CrossingCounts,
    distance_benchmark: Option<RenderDistanceBenchmark>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(false, PresentationOptions::default())
    }
}

impl Game {
    fn new(streamed: bool, presentation: PresentationOptions) -> Self {
        let (mut world, torus_position) = if let Some((q, r)) = presentation.torus_spawn {
            let (world, local) = World::torus(42, Axial64::new(q, r));
            (world, Some(local))
        } else if streamed {
            (World::streamed(42), None)
        } else {
            (World::demo(42), None)
        };
        let mut player = Player::spawn(&world);
        if let Some(local) = torus_position {
            let x = local.x as f32;
            let z = local.y as f32;
            let ground = world.support_top(x, z).expect("periodic terrain surface");
            player.feet = glam::Vec3::new(x, ground, z);
            println!("TORUS global spawn: canonical {:?}, frame {:?}; periodic terrain, local player f32", world.torus_cell(x, z).expect("torus identity").canonical, world.torus_frame().expect("torus frame").origin);
        }
        if let Some((q, r)) = presentation.spawn_cell {
            let center = CellPos::new(q, r, 0).center();
            let ground = world
                .support_top(center.x, center.z)
                .expect("streamed terrain has a deterministic surface");
            player.feet = glam::Vec3::new(center.x, ground, center.z);
            println!(
                "STREAM distant spawn: requested axial ({q}, {r}), approximate feet ({:.2}, {:.2}, {:.2}); unwrapped prototype terrain (NO torus seam), global f32 precision",
                player.feet.x, player.feet.y, player.feet.z,
            );
        }
        let initial = world.stream_around(player.feet);
        if streamed {
            println!(
                "STREAM initial: {} resident 16³ sections, {:.1} ms CPU generation; seed 42",
                world.chunk_count(),
                initial.generation_cpu_ms
            );
        }
        let now = Instant::now();
        let previous_torus = world.torus_cell(player.feet.x, player.feet.z);
        let distance_benchmark = presentation
            .render_benchmark
            .then(|| RenderDistanceBenchmark::new(now));
        let mut console_log = ConsoleLog::new();
        if let Some(benchmark) = distance_benchmark.as_ref() {
            console_log.record(benchmark.introduction());
        }
        Self {
            window: None,
            renderer: None,
            world,
            presentation,
            frame_pacer: FramePacer::new(presentation.fps_cap, now),
            player,
            pressed: HashSet::new(),
            mouse_captured: false,
            pending_mouse_dx: 0.0,
            pending_mouse_dy: 0.0,
            is_looking: false,
            last_frame: now,
            last_stream_report: now,
            streamed_generated: 0,
            streamed_evicted: 0,
            streamed_meshes: 0,
            streamed_uploaded_bytes: 0,
            streamed_generation_cpu_ms: 0.0,
            streamed_meshing_cpu_ms: 0.0,
            stream_pending: 0,
            stream_peak_pending: 0,
            stream_backlog_since: None,
            stream_completed_backlogs: 0,
            stream_longest_backlog_ms: 0.0,
            stream_metrics: StreamFrameMetrics::default(),
            render_cpu_metrics: RenderCpuMetrics::default(),
            menu_open: false,
            console_open: false,
            menu_cursor: None,
            menu_hover: None,
            frame_stats: false,
            console_log,
            previous_torus,
            torus_crossings: CrossingCounts::default(),
            distance_benchmark,
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
        self.pending_mouse_dx = 0.0;
        self.pending_mouse_dy = 0.0;
        self.is_looking = false;
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

    // Input intent, not physical velocity: sprint/sneak modifiers alone do not move.
    fn is_moving(&self) -> bool {
        let input = self.input();
        input.forward || input.backward || input.left || input.right || input.jump
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

        match pollster::block_on(Renderer::new(
            window.clone(),
            &self.world,
            self.presentation,
        )) {
            Ok(renderer) => {
                self.console_log.record(renderer.presentation_report());
                if self.world.is_streamed() {
                    self.console_log.record(format!(
                        "STREAM BUDGET: {:?} new section(s) per tick (None = original unbounded); initial 95-section bootstrap unchanged.",
                        self.presentation.stream_budget
                    ));
                }
                self.window = Some(window);
                self.renderer = Some(renderer);
                self.capture_mouse();
                self.last_frame = Instant::now();
                self.frame_pacer = FramePacer::new(self.presentation.fps_cap, self.last_frame);
                self.console_log.record(format!(
                    "PACING: configured {:?} FPS cap; event loop {}.",
                    self.presentation.fps_cap,
                    if self.presentation.fps_cap.is_some() {
                        "WaitUntil"
                    } else {
                        "Poll"
                    }
                ));
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
                println!(
                    "F12: toggle magenta resident-section perimeter markers (diagnostic only)."
                );
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
                        if key == KeyCode::F12 {
                            if let Some(renderer) = self.renderer.as_mut() {
                                let enabled = renderer.toggle_boundary_markers(
                                    &self.world,
                                    self.player.eye_position().y,
                                );
                                self.console_log.record(format!(
                                    "F12 RESIDENT BOUNDARY MARKERS: {} | magenta columns at resident perimeter (not procedural world edge)",
                                    if enabled { "ON" } else { "OFF" }
                                ));
                            }
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
                let is_moving = self.is_moving();
                let is_looking = self.is_looking;
                if let Some(renderer) = self.renderer.as_mut() {
                    let menu = self
                        .menu_open
                        .then(|| (renderer.menu_snapshot(self.frame_stats), self.menu_hover));
                    let console = self
                        .console_open
                        .then(|| (self.console_log.lines(), self.console_log.generation()));
                    let torus_hud = self.world.torus_cell(self.player.feet.x, self.player.feet.z).map(|position| {
                        let f = self.player.forward();
                        let facing = if f.x.abs() > f.z.abs() {
                            if f.x > 0.0 { "E" } else { "W" }
                        } else if f.z > 0.0 { "N" } else { "S" };
                        let (east_x, north_z) = torus_frame::nearest_seams(self.world.torus_frame().expect("torus frame"));
                        format!(
                            "TORUS FACE {} YAW {:.0} CANON Q {} R {} SECTOR E {} N {} | EAST {:.1}M NORTH {:.1}M WRAPS E{} W{} N{} S{}",
                            facing, self.player.yaw.to_degrees().rem_euclid(360.0),
                            position.canonical.q, position.canonical.r,
                            position.period.east, position.period.north,
                            east_x - self.player.feet.x, north_z - self.player.feet.z,
                            self.torus_crossings.east, self.torus_crossings.west,
                            self.torus_crossings.north, self.torus_crossings.south
                        )
                    });
                    if let Some(report) = renderer.refresh_distant_terrain(&self.world, self.player.feet) {
                        self.console_log.record(report);
                    }
                    match renderer.draw(&self.player, menu, console, torus_hud.as_deref()) {
                        Ok(timings) => {
                            self.frame_pacer.record_draw(Instant::now());
                            if self.world.is_streamed() {
                                self.render_cpu_metrics
                                    .record(timings, is_moving, is_looking);
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
                if dx.is_finite() && dy.is_finite() {
                    self.pending_mouse_dx += dx.abs();
                    self.pending_mouse_dy += dy.abs();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if !self.frame_pacer.tick(now) {
            if let Some(deadline) = self.frame_pacer.next_deadline() {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
            return;
        }
        if let Some(deadline) = self.frame_pacer.next_deadline() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        }
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        let is_moving = self.is_moving();
        // Sum relative mouse motion between updates, not just cursor capture.
        // A 1-pixel threshold avoids flagging tiny event noise.
        self.is_looking =
            self.mouse_captured && self.pending_mouse_dx + self.pending_mouse_dy >= 1.0;
        self.pending_mouse_dx = 0.0;
        self.pending_mouse_dy = 0.0;
        let is_looking = self.is_looking;
        if self.world.is_streamed() {
            self.stream_metrics
                .record_frame_interval(f64::from(dt) * 1000.0, is_moving);
        }
        self.last_frame = now;
        if let Some(benchmark) = self.distance_benchmark.as_mut() {
            let (meshes, vertices) = self
                .renderer
                .as_ref()
                .map_or((0, 0), Renderer::terrain_counts);
            for message in benchmark.advance(now, self.world.chunk_count(), meshes, vertices) {
                self.console_log.record(message);
            }
            self.world.set_horizontal_radius(benchmark.radius());
        }
        let feet_before = self.player.feet;
        self.player.step(&self.world, self.input(), dt);
        let displacement = self.player.feet - feet_before;
        let travelled_m = f64::from(displacement.x.hypot(displacement.z));
        if self.world.torus_frame().is_some() {
            if let Some(shift) = self.world.rebase_torus(&mut self.player.feet) {
                let replacement = self.world.stream_around(self.player.feet);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.rebuild_torus_world(&self.world);
                    renderer.invalidate_distant_terrain();
                    renderer.refresh_boundary_markers(&self.world, self.player.eye_position().y);
                }
                self.console_log.record(format!(
                    "TORUS REBASE | axial shift ({},{}) local metres ({:.2},{:.2}) | new global origin {:?} | rebuilt {} resident sections",
                    shift.axial_q, shift.axial_r, shift.metres_x, shift.metres_z,
                    self.world.torus_frame().expect("torus frame").origin,
                    replacement.added.len()
                ));
            }
            let current = self
                .world
                .torus_cell(self.player.feet.x, self.player.feet.z)
                .expect("torus position");
            if let Some(before) = self.previous_torus {
                if let Some(message) =
                    torus_frame::crossing_report(before, current, &mut self.torus_crossings)
                {
                    self.console_log.record(message);
                }
            }
            self.previous_torus = Some(current);
        }
        if self.world.is_streamed() {
            let delta = self.world.stream_around_budgeted(
                self.player.feet,
                self.presentation.stream_budget.unwrap_or(usize::MAX),
            );
            self.stream_pending = delta.pending;
            self.stream_peak_pending = self.stream_peak_pending.max(delta.pending);
            if delta.pending > 0 {
                self.stream_backlog_since.get_or_insert(now);
            } else if let Some(since) = self.stream_backlog_since.take() {
                self.stream_completed_backlogs += 1;
                self.stream_longest_backlog_ms = self
                    .stream_longest_backlog_ms
                    .max(now.duration_since(since).as_secs_f64() * 1000.0);
            }
            let mut benchmark_mesh_ms = 0.0;
            let mut benchmark_uploaded_bytes = 0u64;
            if !delta.added.is_empty() || !delta.removed.is_empty() {
                self.streamed_generated += delta.added.len();
                self.streamed_evicted += delta.removed.len();
                self.streamed_generation_cpu_ms += delta.generation_cpu_ms;
                let mut mesh_upload_ms = 0.0;
                if let Some(renderer) = self.renderer.as_mut() {
                    let result = renderer.sync_stream(&self.world, &delta);
                    renderer.refresh_boundary_markers(&self.world, self.player.eye_position().y);
                    self.streamed_meshes += result.meshed_buffers;
                    self.streamed_uploaded_bytes += result.uploaded_bytes;
                    self.streamed_meshing_cpu_ms += result.cpu_ms;
                    mesh_upload_ms = result.cpu_ms;
                    benchmark_mesh_ms = result.cpu_ms;
                    benchmark_uploaded_bytes = result.uploaded_bytes;
                }
                self.stream_metrics
                    .record_stream_update(delta.generation_cpu_ms, mesh_upload_ms);
            }
            if let Some(benchmark) = self.distance_benchmark.as_mut() {
                benchmark.record_tick(
                    now,
                    f64::from(dt) * 1000.0,
                    is_moving,
                    delta.pending,
                    delta.added.len(),
                    delta.generation_cpu_ms,
                    benchmark_mesh_ms,
                    benchmark_uploaded_bytes,
                    travelled_m,
                );
            }
            if now.duration_since(self.last_stream_report).as_secs_f64() >= 3.0 {
                let (gpu_meshes, submitted_vertices) = self
                    .renderer
                    .as_ref()
                    .map_or((0, 0), Renderer::terrain_counts);
                self.console_log.record(format!(
                    "STREAM: radius R{} | {} resident chunks, {} nonempty GPU meshes, {} submitted vertices; +{}/-{} chunks, {} new GPU buffers, {:.2} MiB uploaded; {:.2} ms CPU generation, {:.2} ms CPU mesh/buffer calls (last {:.1}s); Is_Moving={is_moving}; Is_Looking={is_looking} (latest update); pending {} section(s), budget {:?} per tick; peak pending {} (last window), drained {} backlog episode(s), longest drain {:.2} ms, active backlog age {:.2} ms.",
                    self.world.horizontal_radius(),
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
                    self.stream_pending,
                    self.presentation.stream_budget,
                    self.stream_peak_pending,
                    self.stream_completed_backlogs,
                    self.stream_longest_backlog_ms,
                    self.stream_backlog_since
                        .map_or(0.0, |since| now.duration_since(since).as_secs_f64() * 1000.0),
                ));
                self.streamed_generated = 0;
                self.streamed_evicted = 0;
                self.streamed_meshes = 0;
                self.streamed_uploaded_bytes = 0;
                self.streamed_generation_cpu_ms = 0.0;
                self.streamed_meshing_cpu_ms = 0.0;
                self.stream_peak_pending = 0;
                self.stream_completed_backlogs = 0;
                self.stream_longest_backlog_ms = 0.0;
                let report = self.stream_metrics.report_and_reset();
                let draw_report = self.render_cpu_metrics.report_and_reset();
                let pacing_report = self
                    .frame_pacer
                    .report_and_reset(now.duration_since(self.last_stream_report));
                if self.frame_stats {
                    self.console_log.record(report);
                    self.console_log.record(draw_report);
                    self.console_log.record(pacing_report);
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
    let (streamed, presentation) = match PresentationOptions::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Invalid arguments: {error}");
            return;
        }
    };
    event_loop.set_control_flow(if presentation.fps_cap.is_some() {
        ControlFlow::WaitUntil(Instant::now())
    } else {
        ControlFlow::Poll
    });
    if let Err(error) = event_loop.run_app(&mut Game::new(streamed, presentation)) {
        eprintln!("Game event loop stopped: {error}");
    }
}

#[cfg(test)]
mod shader_tests {
    use crate::presentation::PresentationOptions;
    use hexel_survival_game::world::CellPos;

    #[test]
    fn opt_in_distant_spawn_streams_target_window_without_changing_default() {
        let settings = PresentationOptions {
            spawn_cell: Some((65_536, -8_192)),
            ..PresentationOptions::default()
        };
        let game = super::Game::new(true, settings);
        let expected = CellPos::new(65_536, -8_192, 0).center();
        assert!((game.player.feet.x - expected.x).abs() < 0.01);
        assert!((game.player.feet.z - expected.z).abs() < 0.01);
        assert_eq!(game.world.chunk_count(), 95);
        let center = CellPos::at_world_xz(
            game.player.feet.x,
            game.player.feet.z,
            game.player.feet.y.floor() as i32,
        );
        assert!(game.world.chunk(center.chunk_and_local().0).is_some());
        let default = super::Game::new(true, PresentationOptions::default());
        assert!(default.player.feet.x < 100.0);
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
