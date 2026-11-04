//! One native 16×16×16 chunk viewed through a first-person camera.

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
}

impl Default for Game {
    fn default() -> Self {
        let world = World::demo(42);
        let player = Player::spawn(&world);
        Self {
            window: None,
            renderer: None,
            world,
            player,
            pressed: HashSet::new(),
            mouse_captured: false,
            last_frame: Instant::now(),
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
                "V2 materials: validated {textures} texture file(s) and {materials} recipe(s); P0 gallery remains active until GPU binding is implemented."
            ),
            Err(error) => eprintln!("V2 material cache rejected: {error}; P0 gallery remains active."),
        }

        match pollster::block_on(Renderer::new(window.clone(), &self.world)) {
            Ok(renderer) => {
                self.window = Some(window);
                self.renderer = Some(renderer);
                self.capture_mouse();
                self.last_frame = Instant::now();
                println!("Hexel Survival: one 16×16×16 chunk, deterministic demo seed 42.");
                println!(
                    "WASD move | Mouse look | Space jump | Left Ctrl sprint | Left Shift sneak"
                );
                println!("Esc release/recapture mouse | Click window to recapture.");
                println!("Chunk edges are temporary barriers; torus wrapping is NOT implemented.");
                println!("P0 material gallery faces your initial spawn at r=9..12.");
                println!("F6: toggle six-face UV checker | F7: cycle checker at 64 / 128 / 256 texels/m.");
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
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Could not initialize window event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    if let Err(error) = event_loop.run_app(&mut Game::default()) {
        eprintln!("Game event loop stopped: {error}");
    }
}
