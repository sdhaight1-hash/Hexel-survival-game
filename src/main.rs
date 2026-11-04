//! Native first-person material demo or opt-in moving streamed-world prototype.

mod materials_p0;
mod materials_v2;
mod mesh;
mod render;
mod texture_codec;
mod texture_pack;

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
    last_frame: Instant,
    last_stream_report: Instant,
    streamed_generated: usize,
    streamed_evicted: usize,
    streamed_meshes: usize,
    streamed_uploaded_bytes: u64,
    streamed_generation_cpu_ms: f64,
    streamed_meshing_cpu_ms: f64,
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
        }
    }

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
                if self.world.is_streamed() {
                    println!(
                        "Hexel Survival STREAMED WORLD: {} resident sections; procedural source seed 42.",
                        self.world.chunk_count()
                    );
                    println!(
                        "No static terrain boundary. Q/R/Y streaming ON. No torus wrap, edits, caves, distant LOD or camera rebasing yet."
                    );
                    println!("STREAM prints 3-second resident/generation/mesh/upload counters.");
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
                    if key == KeyCode::Escape {
                        if event.state == ElementState::Pressed && !event.repeat {
                            if self.mouse_captured {
                                self.release_mouse();
                            } else {
                                self.capture_mouse();
                            }
                        }
                        return;
                    }
                    if event.state == ElementState::Pressed && !event.repeat {
                        if let Some(renderer) = self.renderer.as_mut() {
                            match key {
                                KeyCode::F6 => {
                                    let active = renderer.toggle_checker();
                                    println!(
                                        "P0 UV checker: {}",
                                        if active { "ON" } else { "OFF" }
                                    );
                                    return;
                                }
                                KeyCode::F7 => {
                                    println!(
                                        "P0 checker scale: {} texels/metre",
                                        renderer.cycle_density()
                                    );
                                    return;
                                }
                                KeyCode::F8 => {
                                    match renderer.toggle_v2_preview() {
                                        Some(true) => println!("V2 candidate gallery: ON"),
                                        Some(false) => println!("V2 candidate gallery: OFF (P0)"),
                                        None => println!("V2 gallery unavailable: bake compatible candidate textures first."),
                                    }
                                    return;
                                }
                                KeyCode::F9 => {
                                    match renderer.cycle_surface_preset() {
                                        Some(name) => println!("F8 surface preset: {name}"),
                                        None => {
                                            println!("F9 unavailable: bake v2 candidate art first.")
                                        }
                                    }
                                    return;
                                }
                                KeyCode::F10 => {
                                    match renderer.toggle_metallic_glints() {
                                        Some(true) => println!("F8 copper metallic glints: ON"),
                                        Some(false) => println!("F8 copper metallic glints: OFF"),
                                        None => println!(
                                            "F10 unavailable: bake v2 candidate art first."
                                        ),
                                    }
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                    if event.state == ElementState::Pressed {
                        self.pressed.insert(key);
                    } else {
                        self.pressed.remove(&key);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if !self.mouse_captured {
                    self.capture_mouse();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.draw(&self.player) {
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
        self.last_frame = now;
        self.player.step(&self.world, self.input(), dt);
        if self.world.is_streamed() {
            let delta = self.world.stream_around(self.player.feet);
            if !delta.added.is_empty() || !delta.removed.is_empty() {
                self.streamed_generated += delta.added.len();
                self.streamed_evicted += delta.removed.len();
                self.streamed_generation_cpu_ms += delta.generation_cpu_ms;
                if let Some(renderer) = self.renderer.as_mut() {
                    let result = renderer.sync_stream(&self.world, &delta);
                    self.streamed_meshes += result.meshed_buffers;
                    self.streamed_uploaded_bytes += result.uploaded_bytes;
                    self.streamed_meshing_cpu_ms += result.cpu_ms;
                }
            }
            if now.duration_since(self.last_stream_report).as_secs_f64() >= 3.0 {
                let (gpu_meshes, submitted_vertices) = self
                    .renderer
                    .as_ref()
                    .map_or((0, 0), Renderer::terrain_counts);
                println!(
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
                );
                self.streamed_generated = 0;
                self.streamed_evicted = 0;
                self.streamed_meshes = 0;
                self.streamed_uploaded_bytes = 0;
                self.streamed_generation_cpu_ms = 0.0;
                self.streamed_meshing_cpu_ms = 0.0;
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
