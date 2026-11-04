//! Minimal, clickable developer menu for the existing F-key diagnostics.
//! GPU overlay geometry and hit tests share the same pixel-space layout.
//! This intentionally avoids a production-game UI framework for a small testbed.

use bytemuck::{Pod, Zeroable};
use std::collections::VecDeque;
use winit::keyboard::KeyCode;

const PANEL_WIDTH: f32 = 478.0;
const PANEL_HEIGHT: f32 = 575.0;
const PANEL_MARGIN: f32 = 18.0;
const ROW_TOP: f32 = 76.0;
const ROW_HEIGHT: f32 = 43.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Console,
    FrameStats,
    ChunkCulling,
    Presentation,
    UvChecker,
    UvDensity,
    V2Materials,
    SurfacePreset,
    MetalGlints,
    GrayCopper,
}

pub const ACTIONS: [MenuAction; 10] = [
    MenuAction::Console,
    MenuAction::FrameStats,
    MenuAction::ChunkCulling,
    MenuAction::Presentation,
    MenuAction::UvChecker,
    MenuAction::UvDensity,
    MenuAction::V2Materials,
    MenuAction::SurfacePreset,
    MenuAction::MetalGlints,
    MenuAction::GrayCopper,
];

impl MenuAction {
    pub fn from_key(key: KeyCode) -> Option<Self> {
        Some(match key {
            KeyCode::F2 => Self::Console,
            KeyCode::F3 => Self::FrameStats,
            KeyCode::F4 => Self::ChunkCulling,
            KeyCode::F5 => Self::Presentation,
            KeyCode::F6 => Self::UvChecker,
            KeyCode::F7 => Self::UvDensity,
            KeyCode::F8 => Self::V2Materials,
            KeyCode::F9 => Self::SurfacePreset,
            KeyCode::F10 => Self::MetalGlints,
            KeyCode::F11 => Self::GrayCopper,
            _ => return None,
        })
    }

    fn label(self) -> &'static str {
        match self {
            Self::Console => "F2  LOG CONSOLE",
            Self::FrameStats => "F3  FRAME STATS",
            Self::ChunkCulling => "F4  CHUNK CULLING",
            Self::Presentation => "F5  PRESENTATION",
            Self::UvChecker => "F6  UV CHECKER",
            Self::UvDensity => "F7  UV DENSITY",
            Self::V2Materials => "F8  V2 MATERIALS",
            Self::SurfacePreset => "F9  SURFACE MODE",
            Self::MetalGlints => "F10 METAL GLINTS",
            Self::GrayCopper => "F11 GRAY COPPER",
        }
    }
}

/// The renderer and Game remain the single source of truth.
/// The menu only reads their current values and dispatches existing actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuSnapshot {
    pub console_open: bool,
    pub frame_stats: bool,
    pub chunk_culling: Option<bool>,
    pub unlocked_present: Option<bool>,
    pub gpu_timestamps: bool,
    pub uv_checker: bool,
    pub uv_density: u32,
    pub v2_materials: Option<bool>,
    pub surface: Option<&'static str>,
    pub metal_glints: Option<bool>,
    pub gray_copper: Option<bool>,
}

impl MenuSnapshot {
    pub fn enabled(self, action: MenuAction) -> bool {
        match action {
            MenuAction::Console | MenuAction::FrameStats | MenuAction::UvChecker | MenuAction::UvDensity => true,
            MenuAction::ChunkCulling => self.chunk_culling.is_some(),
            MenuAction::Presentation => self.unlocked_present.is_some(),
            MenuAction::V2Materials => self.v2_materials.is_some(),
            MenuAction::SurfacePreset => self.surface.is_some(),
            MenuAction::MetalGlints => self.metal_glints.is_some(),
            MenuAction::GrayCopper => self.gray_copper.is_some(),
        }
    }

    fn status(self, action: MenuAction) -> String {
        fn on_off(value: bool) -> &'static str {
            if value {
                "ON"
            } else {
                "OFF"
            }
        }
        match action {
            MenuAction::Console => if self.console_open { "HIDE" } else { "SHOW" }.into(),
            MenuAction::FrameStats => on_off(self.frame_stats).into(),
            MenuAction::ChunkCulling => self.chunk_culling.map_or("N/A", on_off).into(),
            MenuAction::Presentation => match self.unlocked_present {
                Some(true) => "UNLOCKED",
                Some(false) => "VSYNC",
                None => "N/A",
            }
            .into(),
            MenuAction::UvChecker => on_off(self.uv_checker).into(),
            MenuAction::UvDensity => format!("{} PX/M", self.uv_density),
            MenuAction::V2Materials => self.v2_materials.map_or("NO ART", on_off).into(),
            MenuAction::SurfacePreset => match self.surface {
                None => "NO ART".into(),
                Some(name) => {
                    let short = match name {
                        "Moss / lichen" => "MOSS",
                        "Combined" => "MIX",
                        other => other,
                    };
                    if self.v2_materials == Some(false) {
                        format!("{} SET", short.to_uppercase())
                    } else {
                        short.to_uppercase()
                    }
                }
            },
            MenuAction::MetalGlints => match self.metal_glints {
                None => "NO ART".into(),
                Some(value) if self.v2_materials == Some(false) => {
                    format!("{} SET", on_off(value))
                }
                Some(value) => on_off(value).into(),
            },
            MenuAction::GrayCopper => match self.gray_copper {
                Some(true) => "GRAY",
                Some(false) => "COLOR",
                None if self.v2_materials.is_some() => "NEEDS F8",
                None => "NO ART",
            }
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MenuVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl MenuVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

fn menu_scale(width: u32, height: u32) -> f32 {
    (width as f32 / 520.0)
        .min(height as f32 / 620.0)
        .min(1.0)
        .max(0.01)
}

fn layout(width: u32, height: u32) -> (f32, f32, f32) {
    let scale = menu_scale(width, height);
    (PANEL_MARGIN * scale, PANEL_MARGIN * scale, scale)
}

/// Physical-pixel cursor coordinates; matches winit's CursorMoved position.
pub fn hit_test(x: f64, y: f64, width: u32, height: u32) -> Option<MenuAction> {
    if width == 0 || height == 0 {
        return None;
    }
    let (left, top, scale) = layout(width, height);
    let local_x = x as f32 / scale - left / scale;
    let local_y = y as f32 / scale - top / scale;
    if !(12.0..PANEL_WIDTH - 12.0).contains(&local_x)
        || !(ROW_TOP..ROW_TOP + ROW_HEIGHT * ACTIONS.len() as f32).contains(&local_y)
    {
        return None;
    }
    let row = ((local_y - ROW_TOP) / ROW_HEIGHT) as usize;
    Some(ACTIONS[row])
}

fn rect(
    out: &mut Vec<MenuVertex>,
    size: (f32, f32),
    xy: (f32, f32),
    wh: (f32, f32),
    color: [f32; 4],
) {
    let (width, height) = size;
    let (x, y) = xy;
    let (w, h) = wh;
    let ndc = |px: f32, py: f32| [2.0 * px / width - 1.0, 1.0 - 2.0 * py / height];
    let a = ndc(x, y);
    let b = ndc(x + w, y);
    let c = ndc(x + w, y + h);
    let d = ndc(x, y + h);
    for position in [a, b, c, a, c, d] {
        out.push(MenuVertex { position, color });
    }
}

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 30, 1, 1, 17, 14],
        '6' => [6, 8, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 2, 12],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '=' => [0, 31, 0, 31, 0, 0, 0],
        '%' => [17, 2, 4, 8, 17, 0, 0],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        ',' => [0, 0, 0, 0, 4, 4, 8],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '|' => [4, 4, 4, 4, 4, 4, 4],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        '#' => [10, 31, 10, 10, 31, 10, 0],
        _ => [0; 7],
    }
}

fn write_text(
    out: &mut Vec<MenuVertex>,
    size: (f32, f32),
    x: f32,
    y: f32,
    pixel: f32,
    label: &str,
    color: [f32; 4],
) {
    for (i, ch) in label.chars().enumerate() {
        let x0 = x + i as f32 * pixel * 6.0;
        for (row, bits) in glyph(ch.to_ascii_uppercase()).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    rect(
                        out,
                        size,
                        (x0 + col as f32 * pixel, y + row as f32 * pixel),
                        (pixel, pixel),
                        color,
                    );
                }
            }
        }
    }
}

/// Generate triangles only when menu state/size/hover changes; caller caches
/// the resulting GPU buffer, rather than allocating every rendered frame.
pub fn geometry(
    width: u32,
    height: u32,
    snapshot: MenuSnapshot,
    hovered: Option<MenuAction>,
) -> Vec<MenuVertex> {
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(18_000);
    let size = (width as f32, height as f32);
    let (left, top, scale) = layout(width, height);
    let ink = [0.89, 0.85, 0.75, 1.0];
    let dim = [0.58, 0.59, 0.58, 1.0];
    let gold = [0.91, 0.71, 0.37, 1.0];
    rect(
        &mut out,
        size,
        (left, top),
        (PANEL_WIDTH * scale, PANEL_HEIGHT * scale),
        [0.065, 0.077, 0.081, 0.96],
    );
    rect(
        &mut out,
        size,
        (left, top),
        (PANEL_WIDTH * scale, 4.0 * scale),
        gold,
    );
    write_text(
        &mut out,
        size,
        left + 23.0 * scale,
        top + 18.0 * scale,
        2.3 * scale,
        "HEXAL DIAGNOSTICS",
        ink,
    );
    write_text(
        &mut out,
        size,
        left + 23.0 * scale,
        top + 49.0 * scale,
        1.7 * scale,
        "CLICK ROWS OR USE HOTKEYS",
        dim,
    );

    for (index, action) in ACTIONS.iter().copied().enumerate() {
        let y = top + (ROW_TOP + ROW_HEIGHT * index as f32) * scale;
        let enabled = snapshot.enabled(action);
        let hovered = hovered == Some(action) && enabled;
        rect(
            &mut out,
            size,
            (left + 12.0 * scale, y),
            ((PANEL_WIDTH - 24.0) * scale, (ROW_HEIGHT - 3.0) * scale),
            if hovered {
                [0.24, 0.23, 0.18, 0.96]
            } else if index % 2 == 0 {
                [0.115, 0.13, 0.13, 0.92]
            } else {
                [0.085, 0.10, 0.105, 0.92]
            },
        );
        write_text(
            &mut out,
            size,
            left + 23.0 * scale,
            y + 11.0 * scale,
            2.0 * scale,
            action.label(),
            if enabled { ink } else { dim },
        );
        write_text(
            &mut out,
            size,
            left + 331.0 * scale,
            y + 11.0 * scale,
            2.0 * scale,
            &snapshot.status(action),
            if enabled { gold } else { dim },
        );
    }
    write_text(
        &mut out,
        size,
        left + 22.0 * scale,
        top + 523.0 * scale,
        1.7 * scale,
        "F1 OR ESC TO RETURN TO WORLD",
        dim,
    );
    write_text(
        &mut out,
        size,
        left + 22.0 * scale,
        top + 548.0 * scale,
        1.55 * scale,
        if snapshot.gpu_timestamps {
            "GPU PASS TIMER AVAILABLE"
        } else {
            "GPU PASS TIMER N/A"
        },
        dim,
    );
    out
}

const CONSOLE_MARGIN: f32 = 12.0;
const CONSOLE_HEIGHT: f32 = 270.0;
const CONSOLE_ROW_STEP: f32 = 15.0;
const CONSOLE_CHAR_PIXEL: f32 = 1.55;

fn console_bounds(width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
    if width < 100 || height < 100 {
        return None;
    }
    let margin = CONSOLE_MARGIN.min(width as f32 * 0.02);
    let w = width as f32 - 2.0 * margin;
    let h = CONSOLE_HEIGHT.min(height as f32 * 0.45);
    Some((margin, height as f32 - h - margin, w, h))
}

fn copy_button(width: u32, height: u32) -> Option<(f32, f32, f32, f32)> {
    let (x, y, w, _) = console_bounds(width, height)?;
    Some((x + w - 189.0, y + 8.0, 174.0, 31.0))
}

/// The fixed button occupies the same pixels as the generated overlay.
pub fn console_copy_hit_test(x: f64, y: f64, width: u32, height: u32) -> bool {
    let Some((left, top, w, h)) = copy_button(width, height) else {
        return false;
    };
    (left..left + w).contains(&(x as f32))
        && (top..top + h).contains(&(y as f32))
}

/// Return the most recent wrapped rows, but never truncate the copyable
/// full-length log lines stored in the underlying ConsoleLog.
fn console_tail_rows(lines: &VecDeque<String>, max_chars: usize, count: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        for segment in chars.chunks(max_chars.max(1)) {
            rows.push(segment.iter().collect());
        }
    }
    rows.into_iter().rev().take(count).collect::<Vec<_>>()
        .into_iter().rev().collect()
}

/// Draw the F2 bottom-docked running log using the *existing* HUD shader
/// and bitmap font. Upload only when new messages or viewport size change.
pub fn console_geometry(
    width: u32,
    height: u32,
    lines: &VecDeque<String>,
) -> Vec<MenuVertex> {
    let Some((x, y, w, h)) = console_bounds(width, height) else {
        return Vec::new();
    };
    let size = (width as f32, height as f32);
    let mut out = Vec::new();
    let ink = [0.89, 0.86, 0.78, 1.0];
    let gold = [0.91, 0.71, 0.37, 1.0];
    let dim = [0.61, 0.64, 0.62, 1.0];
    rect(&mut out, size, (x, y), (w, h), [0.05, 0.065, 0.07, 0.95]);
    rect(&mut out, size, (x, y), (w, 3.0), gold);
    write_text(&mut out, size, x + 15.0, y + 13.0, 2.0, "BENCHMARK RUNNING LOG", ink);
    if let Some((cx, cy, cw, ch)) = copy_button(width, height) {
        rect(&mut out, size, (cx, cy), (cw, ch), [0.22, 0.24, 0.20, 0.96]);
        write_text(&mut out, size, cx + 9.0, cy + 8.0, 1.8, "COPY LOG CTRL+C", gold);
    }

    let line_height = CONSOLE_ROW_STEP;
    let max_rows = ((h - 72.0) / line_height).floor().max(1.0) as usize;
    let max_chars = ((w - 34.0) / (6.0 * CONSOLE_CHAR_PIXEL))
        .floor().max(10.0) as usize;
    let visible_rows = console_tail_rows(lines, max_chars, max_rows);
    for (index, text) in visible_rows.iter().enumerate() {
        let label_color = if text.starts_with("BENCH") || text.starts_with("AVG") {
            ink
        } else {
            dim
        };
        write_text(
            &mut out, size, x + 15.0, y + 46.0 + index as f32 * line_height,
            CONSOLE_CHAR_PIXEL, text, label_color,
        );
    }
    write_text(
        &mut out, size, x + 15.0, y + h - 20.0, 1.4,
        "F2 CLOSE   CTRL+C COPY   FULL LOG: TARGET/BENCHMARK-SESSION.LOG",
        dim,
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> MenuSnapshot {
        MenuSnapshot {
            console_open: false,
            frame_stats: false,
            chunk_culling: Some(true),
            unlocked_present: Some(false),
            gpu_timestamps: true,
            uv_checker: false,
            uv_density: 128,
            v2_materials: Some(false),
            surface: Some("Dry"),
            metal_glints: Some(true),
            gray_copper: None,
        }
    }

    #[test]
    fn ten_hotkeys_map_to_distinct_actions() {
        for (key, action) in [
            (KeyCode::F2, MenuAction::Console),
            (KeyCode::F3, MenuAction::FrameStats),
            (KeyCode::F4, MenuAction::ChunkCulling),
            (KeyCode::F5, MenuAction::Presentation),
            (KeyCode::F6, MenuAction::UvChecker),
            (KeyCode::F7, MenuAction::UvDensity),
            (KeyCode::F8, MenuAction::V2Materials),
            (KeyCode::F9, MenuAction::SurfacePreset),
            (KeyCode::F10, MenuAction::MetalGlints),
            (KeyCode::F11, MenuAction::GrayCopper),
        ] {
            assert_eq!(MenuAction::from_key(key), Some(action));
        }
        assert_eq!(MenuAction::from_key(KeyCode::F1), None);
    }

    #[test]
    fn hit_boxes_correspond_to_rendered_rows() {
        let (left, top, scale) = layout(1280, 800);
        for (index, action) in ACTIONS.iter().copied().enumerate() {
            let row_y = top + (ROW_TOP + ROW_HEIGHT * (index as f32 + 0.5)) * scale;
            assert_eq!(
                hit_test((left + 40.0) as f64, row_y as f64, 1280, 800),
                Some(action)
            );
        }
        assert_eq!(hit_test(2.0, 2.0, 1280, 800), None);
        assert_eq!(hit_test(100.0, 50.0, 1280, 800), None);
        assert_eq!(hit_test(100.0, 100.0, 0, 800), None);
    }

    #[test]
    fn console_copy_hitbox_and_wrapped_tail_are_consistent() {
        let (x, y, w, h) = copy_button(1280, 800).unwrap();
        assert!(console_copy_hit_test((x + w / 2.0) as f64,
            (y + h / 2.0) as f64, 1280, 800));
        assert!(!console_copy_hit_test(0.0, 0.0, 1280, 800));
        assert!(!console_copy_hit_test(10.0, 10.0, 0, 0));
        let lines = VecDeque::from(["1234567890123".to_string(), "tail".to_string()]);
        assert_eq!(
            console_tail_rows(&lines, 5, 3),
            vec!["123", "tail"]
        );
        let vertices = console_geometry(1280, 800, &lines);
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 6, 0);
        assert!(vertices.iter().all(|v| v.position.iter().all(|n| n.is_finite())));
    }

    #[test]
    fn disabled_rows_are_reported_without_mutating_game_state() {
        let s = state();
        assert!(!s.enabled(MenuAction::GrayCopper));
        assert_eq!(s.status(MenuAction::GrayCopper), "NEEDS F8");
        assert_eq!(s.status(MenuAction::UvDensity), "128 PX/M");
        assert_eq!(s.status(MenuAction::Presentation), "VSYNC");
        assert_eq!(
            MenuSnapshot {
                unlocked_present: None,
                ..s
            }
            .status(MenuAction::Presentation),
            "N/A"
        );
        assert_eq!(s.status(MenuAction::SurfacePreset), "DRY SET");
        assert_eq!(s.status(MenuAction::MetalGlints), "ON SET");
        assert_eq!(
            MenuSnapshot {
                v2_materials: Some(true),
                ..s
            }
            .status(MenuAction::SurfacePreset),
            "DRY"
        );
        assert!(!MenuSnapshot {
            chunk_culling: None,
            ..s
        }
        .enabled(MenuAction::ChunkCulling));
        let vertices = geometry(1280, 800, s, Some(MenuAction::FrameStats));
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 6, 0);
        assert!(vertices
            .iter()
            .all(|v| v.position.iter().all(|x| x.is_finite())));
    }
}
