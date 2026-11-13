//! Launcher-only purple theme. No game renderer or preferences are changed.
use super::pages::Action;
use crate::catalog::Page;
use horizon_client::ui_theme::{Face, Ui};

pub const fn ink(rgb: u32) -> u32 {
    0xff000000 | (rgb & 255) << 16 | (rgb & 0xff00) | (rgb >> 16)
}
pub const BG: u32 = ink(0x07060d);
pub const PANEL: u32 = ink(0x181226);
pub const BORDER: u32 = ink(0x38234f);
pub const TEXT: u32 = ink(0xf5f1ff);
pub const MUTED: u32 = ink(0xaaa3b8);
pub const ACCENT: u32 = ink(0x913dff);
pub const GREEN: u32 = ink(0x22e875);

fn mix(a: u32, b: u32, t: f32) -> u32 {
    let mut out = 0xff000000;
    for shift in [0, 8, 16] {
        let x = ((a >> shift) & 255) as f32;
        let y = ((b >> shift) & 255) as f32;
        out |= ((x + (y - x) * t).round() as u32) << shift;
    }
    out
}
fn gradient(ui: &mut Ui, r: [f32; 4], left: u32, right: u32, radius: f32) {
    let [x, y, w, h] = r;
    let start = ui.batch.vertices.len();
    ui.rounded(x, y, w, h, radius, left);
    let scale = ui.batch.scale / ui.density;
    for v in &mut ui.batch.vertices[start..] {
        let t = ((v.pos[0] * scale - x) / w).clamp(0.0, 1.0);
        v.color = mix(left, right, t);
    }
}
fn label(ui: &mut Ui, size: f32, x: f32, y: f32, color: u32, text: &str) {
    ui.text(Face::Medium, size, x, y, color, text);
}
fn card(ui: &mut Ui, r: [f32; 4]) {
    gradient(ui, r, ink(0x1d1530), ink(0x120e1e), 16.0);
    ui.border(r[0], r[1], r[2], r[3], 16.0, BORDER);
}
pub fn button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    text: &str,
    action: Action,
    active: bool,
) {
    if active {
        ui.rounded(
            r[0] - 2.0,
            r[1] - 2.0,
            r[2] + 4.0,
            r[3] + 4.0,
            14.0,
            ink(0x2b104a),
        );
        gradient(ui, r, ink(0x7731ff), ink(0xa258ff), 12.0);
    } else {
        card(ui, r);
    }
    let font = if r[2] < 90.0 { 13.0 } else { 16.0 };
    let text = ui.fit_text(Face::Medium, font, text, r[2] - 20.0);
    let x = r[0] + (r[2] - ui.measure(Face::Medium, font, &text)) * 0.5;
    label(ui, font, x, r[1] + (r[3] - font) * 0.5 - 1.0, TEXT, &text);
    targets.push((r, action));
}
/// Draw the full-screen shell; return the content inset in dp.
pub fn shell(ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>, page: Page, account: &str) -> f32 {
    let (w, h) = (ui.w, ui.h);
    let side = if w < 700.0 {
        92.0
    } else {
        (w * 0.145).clamp(145.0, 205.0)
    };
    ui.rounded(0.0, 0.0, w, h, 0.0, BG);
    // Original vector artwork and glows, no downloaded/copyrighted art.
    ui.rounded(w - 360.0, -160.0, 500.0, 420.0, 210.0, ink(0x11091d));
    ui.rounded(0.0, h - 160.0, 300.0, 300.0, 150.0, ink(0x10091a));
    ui.rounded(0.0, 0.0, side, h, 0.0, ink(0x0b0814));
    ui.rounded(side - 1.0, 0.0, 1.0, h, 0.0, BORDER);
    gradient(
        ui,
        [20.0, 20.0, 42.0, 42.0],
        ink(0x9b5cff),
        ink(0x5415d9),
        12.0,
    );
    label(ui, 30.0, 29.0, 23.0, TEXT, "H");
    if side > 100.0 {
        label(ui, 14.0, 74.0, 24.0, TEXT, "Horizon");
        label(ui, 12.0, 74.0, 44.0, ACCENT, "Android");
    }
    let step = ((h - 180.0) / 8.0).clamp(38.0, 56.0);
    for (i, p) in Page::ALL.iter().enumerate() {
        let y = 88.0 + i as f32 * step;
        let r = [12.0, y, side - 24.0, step - 6.0];
        if *p == page {
            gradient(ui, r, ink(0x7d38ff), ink(0x5520c7), 12.0);
        }
        let size = if side < 100.0 { 12.0 } else { 15.0 };
        let name = p.name();
        label(
            ui,
            size,
            if side < 100.0 { 20.0 } else { 48.0 },
            y + (step - size) * 0.5 - 3.0,
            if *p == page { TEXT } else { MUTED },
            name,
        );
        if side >= 100.0 {
            icon(ui, [24.0, y + 12.0, 14.0, 14.0], i);
        }
        targets.push((r, Action::Page(*p)));
    }
    if h > 570.0 {
        card(ui, [12.0, h - 90.0, side - 24.0, 70.0]);
        label(ui, 13.0, 22.0, h - 76.0, ACCENT, "Appearance");
        if side > 100.0 {
            label(ui, 11.0, 22.0, h - 53.0, MUTED, "Free moon theme");
        }
        targets.push((
            [12.0, h - 90.0, side - 24.0, 70.0],
            Action::Page(Page::Settings),
        ));
    }
    let x = side + 24.0;
    label(
        ui,
        if w < 700.0 { 20.0 } else { 27.0 },
        x,
        22.0,
        TEXT,
        "Horizon",
    );
    let hx = x
        + 8.0
        + ui.measure(
            Face::Medium,
            if w < 700.0 { 20.0 } else { 27.0 },
            "Horizon ",
        );
    label(
        ui,
        if w < 700.0 { 20.0 } else { 27.0 },
        hx,
        22.0,
        ACCENT,
        "Android",
    );
    label(ui, 12.0, x, 57.0, MUTED, "0.7.7  /  Minecraft Launcher");
    if w - side > 600.0 {
        let aw = (ui.measure(Face::Medium, 13.0, account) + 40.0).min(300.0);
        card(ui, [w - aw - 124.0, 24.0, aw, 40.0]);
        ui.rounded(w - aw - 110.0, 39.0, 8.0, 8.0, 4.0, GREEN);
        let name = ui.fit_text(Face::Medium, 13.0, account, aw - 36.0);
        label(ui, 13.0, w - aw - 94.0, 35.0, TEXT, &name);
        targets.push(([w - aw - 124.0, 24.0, aw, 40.0], Action::Account));
        button(
            ui,
            targets,
            [w - 110.0, 24.0, 40.0, 40.0],
            "S",
            Action::Page(Page::Settings),
            false,
        );
        button(
            ui,
            targets,
            [w - 60.0, 24.0, 40.0, 40.0],
            "i",
            Action::Logs,
            false,
        );
    } else {
        targets.push(([x, 18.0, w - side - 40.0, 56.0], Action::Account));
    }
    side
}
fn icon(ui: &mut Ui, r: [f32; 4], i: usize) {
    let [x, y, w, h] = r;
    ui.rounded(
        x,
        y,
        w,
        h,
        if i == 3 || i == 6 { w * 0.5 } else { 3.0 },
        ink(0xb879ff),
    );
    ui.rounded(
        x + w * 0.3,
        y + h * 0.3,
        w * 0.4,
        h * 0.4,
        1.0,
        ink(0x442065),
    );
}
/// Proper AARRGGBB from 0xRRGGBB (the `ink` helper above keeps its historic
/// channel order for the sidebar shell it was tuned for).
pub const fn rgb(c: u32) -> u32 {
    0xff000000 | c
}
/// Same colour with a new alpha (0.0 - 1.0).
pub fn with_alpha(c: u32, a: f32) -> u32 {
    (c & 0x00ff_ffff) | (((a.clamp(0.0, 1.0)) * 255.0).round() as u32) << 24
}
fn amix(a: u32, b: u32, t: f32) -> u32 {
    let mut out = 0u32;
    for shift in [0u32, 8, 16, 24] {
        let x = ((a >> shift) & 255) as f32;
        let y = ((b >> shift) & 255) as f32;
        out |= ((x + (y - x) * t).round() as u32) << shift;
    }
    out
}
/// Vertical gradient fill, top colour at the rect's top.
fn vgradient(ui: &mut Ui, r: [f32; 4], top: u32, bottom: u32, radius: f32) {
    let [x, y, w, h] = r;
    let start = ui.batch.vertices.len();
    ui.rounded(x, y, w, h, radius, top);
    let scale = ui.batch.scale / ui.density;
    for v in &mut ui.batch.vertices[start..] {
        let t = ((v.pos[1] * scale - y) / h).clamp(0.0, 1.0);
        v.color = amix(top, bottom, t);
    }
}
#[derive(Clone, Copy)]
enum Icon {
    Cube,
    Camera,
    Layers,
    Person,
    Folder,
    Gear,
}
/// Small original line/fill icons drawn only from rounded primitives.
fn draw_icon(ui: &mut Ui, icon: Icon, r: [f32; 4]) {
    let [x, y, w, _] = r;
    let c = rgb(0xcfc6ff);
    let deep = rgb(0x442065);
    match icon {
        Icon::Cube => {
            ui.border(x, y, w, w, w * 0.22, c);
            ui.rounded(x + w * 0.28, y + w * 0.28, w * 0.44, w * 0.44, w * 0.12, deep);
            ui.border(x + w * 0.28, y + w * 0.28, w * 0.44, w * 0.44, w * 0.12, c);
        }
        Icon::Camera => {
            ui.rounded(x, y + w * 0.2, w, w * 0.68, w * 0.16, deep);
            ui.border(x, y + w * 0.2, w, w * 0.68, w * 0.16, c);
            ui.rounded(x + w * 0.3, y + w * 0.36, w * 0.4, w * 0.4, w * 0.2, c);
            ui.rounded(x + w * 0.42, y + w * 0.48, w * 0.16, w * 0.16, w * 0.08, deep);
            ui.rounded(x + w * 0.08, y + w * 0.06, w * 0.3, w * 0.16, w * 0.06, c);
        }
        Icon::Layers => {
            ui.rounded(x, y, w, w * 0.26, w * 0.08, c);
            ui.rounded(x + w * 0.08, y + w * 0.36, w * 0.84, w * 0.26, w * 0.08, rgb(0x9d8cff));
            ui.rounded(x + w * 0.16, y + w * 0.72, w * 0.68, w * 0.26, w * 0.08, deep);
        }
        Icon::Person => {
            ui.rounded(x + w * 0.3, y, w * 0.4, w * 0.4, w * 0.2, c);
            ui.rounded(x + w * 0.06, y + w * 0.52, w * 0.88, w * 0.46, w * 0.22, c);
        }
        Icon::Folder => {
            ui.rounded(x, y + w * 0.16, w, w * 0.72, w * 0.14, c);
            ui.rounded(x, y + w * 0.16, w * 0.44, w * 0.34, w * 0.12, c);
            ui.rounded(x + w * 0.06, y + w * 0.32, w * 0.88, w * 0.5, w * 0.12, deep);
        }
        Icon::Gear => {
            ui.rounded(x + w * 0.18, y + w * 0.18, w * 0.64, w * 0.64, w * 0.32, c);
            ui.rounded(x + w * 0.38, y + w * 0.38, w * 0.24, w * 0.24, w * 0.12, deep);
            for (dx, dy) in [(0.44, 0.0), (0.44, 0.86), (0.0, 0.44), (0.86, 0.44)] {
                ui.rounded(x + w * dx, y + w * dy, w * 0.14, w * 0.14, w * 0.07, c);
            }
        }
    }
}
/// The mockup's square header buttons: icon over a tiny caps label.
fn icon_button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    text: &str,
    icon: Icon,
    action: Action,
) {
    card(ui, r);
    let iw = 24.0;
    draw_icon(
        ui,
        icon,
        [r[0] + (r[2] - iw) * 0.5, r[1] + 9.0, iw, iw],
    );
    let s = 9.0;
    let text = ui.fit_text(Face::Medium, s, text, r[2] - 8.0);
    let tw = ui.measure(Face::Medium, s, &text);
    label(ui, s, r[0] + (r[2] - tw) * 0.5, r[1] + r[3] - 17.0, MUTED, &text);
    targets.push((r, action));
}
/// An original 8x8 pixel face for the profile chip.
fn pixel_head(ui: &mut Ui, x: f32, y: f32, size: f32) {
    const MAP: [&str; 8] = [
        "HHHHHHHH", "HHHHHHHH", "HSSSSSSH", "SSSSSSSS", "SWESSEWS", "SSSSSSSS", "SSMMMMSS",
        "SSBSSBSS",
    ];
    let cell = size / 8.0;
    for (row, line) in MAP.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let color = match ch {
                'H' => rgb(0x3b2a1c),
                'S' => rgb(0xc08a63),
                'W' => rgb(0xffffff),
                'E' => rgb(0x3b6fd6),
                'M' => rgb(0x6b3f2a),
                _ => rgb(0xa9764f),
            };
            ui.rounded(
                x + col as f32 * cell,
                y + row as f32 * cell,
                cell,
                cell,
                0.0,
                color,
            );
        }
    }
    ui.border(x, y, size, size, 2.0, BORDER);
}
fn profile_chip(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    name: &str,
    online: bool,
) {
    card(ui, r);
    let head = (r[3] - 18.0).min(40.0);
    pixel_head(ui, r[0] + 9.0, r[1] + (r[3] - head) * 0.5, head);
    let tx = r[0] + head + 19.0;
    let name = ui.fit_text(Face::Medium, 14.0, name, r[2] - head - 30.0);
    label(ui, 14.0, tx, r[1] + 10.0, TEXT, &name);
    let pill_w = 56.0;
    let pill_y = r[1] + r[3] - 21.0;
    let (pill_border, pill_text, word) = if online {
        (rgb(0x22d3ee), rgb(0x8be9f5), "ONLINE")
    } else {
        (BORDER, MUTED, "OFFLINE")
    };
    ui.rounded(tx, pill_y, pill_w, 15.0, 7.5, with_alpha(pill_border, 0.16));
    ui.border(tx, pill_y, pill_w, 15.0, 7.5, pill_border);
    let tw = ui.measure(Face::Medium, 8.5, word);
    label(ui, 8.5, tx + (pill_w - tw) * 0.5, pill_y + 3.0, pill_text, word);
    targets.push((r, Action::Account));
}
/// Brand block: HORIZON wordmark, subtitle and the sunset badge.
fn brand(ui: &mut Ui, x: f32, y: f32) {
    ui.text(Face::SemiBold, 30.0, x, y, TEXT, "HORIZON");
    label(ui, 10.5, x + 2.0, y + 38.0, MUTED, "MINECRAFT CLIENT");
    let bx = x + ui.measure(Face::SemiBold, 30.0, "HORIZON") + 16.0;
    gradient(ui, [bx, y + 3.0, 52.0, 52.0], rgb(0x2a1b4e), rgb(0x170f30), 12.0);
    ui.border(bx, y + 3.0, 52.0, 52.0, 12.0, BORDER);
    let cx = bx + 26.0;
    let cy = y + 3.0 + 22.0;
    ui.rounded(cx - 10.0, cy - 14.0, 20.0, 20.0, 10.0, rgb(0xc98bff));
    for (i, (bw, bc)) in [
        (34.0, rgb(0xc9b8ff)),
        (26.0, rgb(0x9d8cff)),
        (16.0, rgb(0x7b6cff)),
    ]
    .iter()
    .enumerate()
    {
        ui.rounded(
            cx - bw * 0.5,
            cy + 3.0 + i as f32 * 6.5,
            *bw,
            3.0,
            1.5,
            *bc,
        );
    }
}
/// Tiny deterministic LCG so the scene is stable frame to frame.
struct Rnd(u32);
impl Rnd {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(16807) % 2147483647;
        self.0 as f32 / 2147483647.0
    }
}
fn tree(ui: &mut Ui, x: f32, ground: f32, s: f32, rnd: &mut Rnd) {
    ui.rounded(x, ground - 30.0 * s, 7.0 * s, 30.0 * s, 0.0, rgb(0x0f0a24));
    ui.rounded(
        x - 14.0 * s,
        ground - 56.0 * s,
        36.0 * s,
        28.0 * s,
        2.0,
        rgb(0x1c1248),
    );
    ui.rounded(
        x - 6.0 * s,
        ground - 72.0 * s,
        20.0 * s,
        18.0 * s,
        2.0,
        rgb(0x1c1248),
    );
    for _ in 0..10 {
        let dx = (rnd.next() * 34.0 - 14.0) * s;
        let dy = (rnd.next() * 50.0 - 72.0) * s;
        let c = if rnd.next() < 0.5 {
            rgb(0xa67bff)
        } else {
            rgb(0x7b5cff)
        };
        ui.rounded(x + dx, ground + dy, 2.0, 2.0, 0.0, c);
    }
}
/// The aurora banner: original pixel-art night scene, drawn procedurally.
fn banner(ui: &mut Ui, r: [f32; 4]) {
    let [x, y, w, h] = r;
    vgradient(ui, r, rgb(0x120a3a), rgb(0x4a2385), 14.0);
    let mut rnd = Rnd(7);
    // Aurora columns.
    for i in 0..5 {
        let cx = x + w * (0.16 + i as f32 * 0.17);
        let sw = w * 0.055;
        let col = if i % 2 == 0 { 0x22d3ee } else { 0x8b5cf6 };
        vgradient(
            ui,
            [cx - sw * 0.5, y + 8.0, sw, h * 0.6],
            with_alpha(rgb(col), 0.0),
            with_alpha(rgb(col), 0.22),
            sw * 0.5,
        );
    }
    // Stars.
    let stars = ((w * h) / 11000.0) as i32;
    for _ in 0..stars.clamp(30, 80) {
        let sx = x + 8.0 + rnd.next() * (w - 16.0);
        let sy = y + 8.0 + rnd.next() * h * 0.5;
        let a = 0.3 + rnd.next() * 0.7;
        ui.rounded(sx, sy, 2.0, 2.0, 1.0, with_alpha(rgb(0xffffff), a));
    }
    // Far ridge.
    let step = 18.0;
    let count = (w / step) as i32;
    for n in 0..=count {
        let xx = x + n as f32 * step;
        let t = n as f32;
        let hh = h * (0.24 + 0.18 * ((t * 0.35).sin() * 0.6 + (t * 0.13).cos() * 0.4).abs());
        ui.rounded(xx, y + h - hh - 12.0, step + 1.0, hh, 0.0, rgb(0x2c1a5e));
    }
    // Trees before the ground band so their trunks root into it.
    let ground = y + h - 26.0;
    tree(ui, x + w * 0.07, ground, 1.1, &mut rnd);
    tree(ui, x + w * 0.17, ground, 0.8, &mut rnd);
    tree(ui, x + w * 0.86, ground, 1.3, &mut rnd);
    tree(ui, x + w * 0.95, ground, 0.9, &mut rnd);
    // Player silhouette in the middle.
    let px = x + w * 0.5;
    ui.rounded(px - 5.0, ground - 46.0, 10.0, 10.0, 1.0, rgb(0x07040f));
    ui.rounded(px - 8.0, ground - 36.0, 16.0, 20.0, 1.0, rgb(0x07040f));
    ui.rounded(px - 8.0, ground - 16.0, 7.0, 16.0, 0.0, rgb(0x07040f));
    ui.rounded(px + 1.0, ground - 16.0, 7.0, 16.0, 0.0, rgb(0x07040f));
    // Terraced ground band, overdrawn sideways so the banner corners stay round.
    ui.rounded(x - 2.0, y + h - 26.0, w + 4.0, 28.0, 13.0, rgb(0x170f38));
    ui.rounded(x - 2.0, y + h - 26.0, w + 4.0, 3.0, 1.5, rgb(0x241a52));
    // Glowing blocks on the ground.
    for i in 0..6 {
        let gx = x + w * (0.12 + i as f32 * 0.15) + rnd.next() * 14.0;
        let gy = y + h - 34.0 - rnd.next() * 6.0;
        ui.rounded(gx - 2.0, gy - 2.0, 8.0, 8.0, 2.0, with_alpha(rgb(0x2cd4ee), 0.35));
        ui.rounded(gx, gy, 4.0, 4.0, 1.0, rgb(0xbff6ff));
    }
    ui.border(x, y, w, h, 14.0, rgb(0x7a4fd0));
    // Centered headline.
    let small = "THE AURORA UPDATE";
    let big = "WELCOME BACK";
    let big_size = (w * 0.032).clamp(30.0, 46.0);
    let small_size = 13.0;
    let big_w = ui.measure(Face::SemiBold, big_size, big);
    let small_w = ui.measure(Face::Medium, small_size, small);
    let big_y = y + h * 0.5 - big_size * 0.5;
    ui.text(
        Face::Medium,
        small_size,
        x + (w - small_w) * 0.5,
        big_y - small_size - 12.0,
        rgb(0xc9c3e6),
        small,
    );
    ui.text(
        Face::SemiBold,
        big_size,
        x + (w - big_w) * 0.5 + 2.0,
        big_y + 2.0,
        with_alpha(rgb(0x000000), 0.45),
        big,
    );
    ui.text(
        Face::SemiBold,
        big_size,
        x + (w - big_w) * 0.5,
        big_y,
        TEXT,
        big,
    );
}
/// One of the three feature cards (Mods / Shaders / Packs).
fn feature_card(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    title: &str,
    line1: &str,
    line2: &str,
    icon: Icon,
    tint: u32,
    action: Action,
) {
    card(ui, r);
    ui.border(r[0], r[1], r[2], r[3], 16.0, tint);
    let iw = (r[3] * 0.34).clamp(40.0, 58.0);
    draw_icon(ui, icon, [r[0] + r[2] - iw - 16.0, r[1] + 14.0, iw, iw]);
    let title_size = (r[3] * 0.15).clamp(17.0, 22.0);
    label(ui, title_size, r[0] + 18.0, r[1] + r[3] - 66.0, TEXT, title);
    label(ui, 11.5, r[0] + 18.0, r[1] + r[3] - 42.0, rgb(0xd6d2ee), line1);
    label(ui, 11.5, r[0] + 18.0, r[1] + r[3] - 26.0, rgb(0xd6d2ee), line2);
    targets.push((r, action));
}
/// The big gradient Play card; the subtitle always names the real game version.
fn play_card(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    r: [f32; 4],
    installed: bool,
    retry: bool,
) {
    gradient(ui, r, rgb(0x8b2cf5), rgb(0x22d3ee), 14.0);
    ui.border(r[0], r[1], r[2], r[3], 14.0, rgb(0xbfa8ff));
    let size = (r[3] * 0.32).clamp(30.0, 52.0);
    let tw = ui.measure(Face::SemiBold, size, "PLAY");
    ui.text(
        Face::SemiBold,
        size,
        r[0] + (r[2] - tw) * 0.5,
        r[1] + r[3] * 0.5 - size * 0.62,
        rgb(0xffffff),
        "PLAY",
    );
    let sub = if installed {
        "MINECRAFT 26.3"
    } else if retry {
        "TAP TO RETRY INSTALL"
    } else {
        "TAP TO DOWNLOAD 26.3"
    };
    let sw = ui.measure(Face::Medium, 12.0, sub);
    ui.text(
        Face::Medium,
        12.0,
        r[0] + (r[2] - sw) * 0.5,
        r[1] + r[3] * 0.5 + size * 0.28,
        rgb(0xe6ecff),
        sub,
    );
    targets.push((r, Action::Play));
}
/// The dashboard home: the reference design, full-screen without the sidebar.
pub fn home(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    installed: bool,
    retry: bool,
    account: &str,
    online: bool,
) {
    let (w, h) = (ui.w, ui.h);
    let pad = (w * 0.022).clamp(14.0, 32.0);
    // Backdrop: deep navy with soft violet and cyan glows.
    ui.rounded(0.0, 0.0, w, h, 0.0, rgb(0x0b0a1a));
    ui.rounded(w - 420.0, -180.0, 560.0, 460.0, 230.0, rgb(0x151029));
    ui.rounded(-140.0, h - 200.0, 420.0, 360.0, 180.0, rgb(0x0f1a2b));

    // Header.
    let head_h = 60.0;
    brand(ui, pad, pad);
    let btn = 58.0;
    let gap = 12.0;
    let mut rx = w - pad - btn;
    icon_button(
        ui,
        targets,
        [rx, pad, btn, head_h],
        "SETTINGS",
        Icon::Gear,
        Action::Page(Page::Settings),
    );
    rx -= btn + gap;
    icon_button(
        ui,
        targets,
        [rx, pad, btn, head_h],
        "SAVES",
        Icon::Folder,
        Action::ComingSoon("Saves"),
    );
    rx -= btn + gap;
    icon_button(
        ui,
        targets,
        [rx, pad, btn, head_h],
        "SKINS",
        Icon::Person,
        Action::Page(Page::Skins),
    );
    let name = if account.trim().is_empty() {
        "Player"
    } else {
        account.trim()
    };
    let chip_w = (ui.measure(Face::Medium, 14.0, name) + 108.0).clamp(170.0, 250.0);
    rx -= chip_w + gap;
    profile_chip(ui, targets, [rx, pad, chip_w, head_h], name, online);

    // Banner + cards share the middle; the footer keeps its own row.
    let banner_y = pad + head_h + 14.0;
    let footer_h = 26.0;
    let cards_h = ((h - banner_y - footer_h - pad - 44.0) * 0.44).clamp(118.0, 170.0);
    let banner_h = (h - banner_y - cards_h - footer_h - pad - 28.0).max(140.0);
    banner(ui, [pad, banner_y, w - pad * 2.0, banner_h]);

    let cards_y = banner_y + banner_h + 14.0;
    let cw = (w - pad * 2.0 - gap * 3.0) / 4.36;
    let pw = cw * 1.36;
    feature_card(
        ui,
        targets,
        [pad, cards_y, cw, cards_h],
        "MODS",
        "Enhance gameplay with",
        "community modifications.",
        Icon::Cube,
        rgb(0x7a4fd0),
        Action::Page(Page::Mods),
    );
    feature_card(
        ui,
        targets,
        [pad + cw + gap, cards_y, cw, cards_h],
        "SHADERS",
        "Transform visuals with",
        "realistic lighting and fx.",
        Icon::Camera,
        rgb(0x5a64d0),
        Action::Page(Page::Shaders),
    );
    feature_card(
        ui,
        targets,
        [pad + (cw + gap) * 2.0, cards_y, cw, cards_h],
        "PACKS",
        "Manage resource and",
        "texture packs.",
        Icon::Layers,
        rgb(0x4a6ab0),
        Action::Page(Page::Packs),
    );
    play_card(
        ui,
        targets,
        [pad + (cw + gap) * 3.0, cards_y, pw, cards_h],
        installed,
        retry,
    );

    // Footer: real status and launcher version, plus the manager pages the
    // sidebar used to hold so nothing existing becomes unreachable.
    let fy = h - pad - 12.0;
    let (dot, word, word_c) = if installed {
        (rgb(0x34d399), "ALL SYSTEMS READY", rgb(0x9ef0cd))
    } else {
        (rgb(0xf5b83d), "SETUP NEEDED", rgb(0xf5cf8a))
    };
    ui.rounded(pad, fy + 3.0, 8.0, 8.0, 4.0, dot);
    label(ui, 11.0, pad + 16.0, fy, word_c, word);
    let sep_x = pad + 16.0 + ui.measure(Face::Medium, 11.0, word) + 14.0;
    ui.rounded(sep_x, fy + 1.0, 1.0, 12.0, 0.0, BORDER);
    label(ui, 11.0, sep_x + 14.0, fy, MUTED, "VERSION 0.7.7");
    for (text, page, right) in [
        ("FILES", Page::Files, true),
        ("INSTALLED", Page::Installed, false),
    ] {
        let tw = ui.measure(Face::Medium, 11.0, text);
        let tx = if right {
            w - pad - tw
        } else {
            w - pad - tw - ui.measure(Face::Medium, 11.0, "FILES") - 32.0
        };
        label(ui, 11.0, tx, fy, MUTED, text);
        targets.push(([tx - 8.0, fy - 9.0, tw + 16.0, 30.0], Action::Page(page)));
    }
        }
