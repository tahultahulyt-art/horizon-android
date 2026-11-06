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
/// Actual home page drawing, also used by the local visual harness.
pub fn home(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    installed: bool,
    profile: u32,
    retry: bool,
) {
    let (w, h) = (ui.w, ui.h);
    let x = 24.0;
    let width = w - 48.0;
    let top = 98.0;
    let hero_h = (h * 0.44).clamp(270.0, 340.0);
    gradient(
        ui,
        [x, top, width, hero_h],
        ink(0x160d27),
        ink(0x32144c),
        22.0,
    );
    // Moonlit sky, hand-drawn deterministically. Kept to the right of text.
    let sky = x + width * 0.61;
    let moon = width.min(700.0) * 0.13;
    ui.rounded(
        sky + width * 0.13 - moon * 0.12,
        top + 24.0 - moon * 0.12,
        moon * 1.24,
        moon * 1.24,
        moon,
        ink(0x51316c),
    );
    ui.rounded(
        sky + width * 0.13,
        top + 24.0,
        moon,
        moon,
        moon * 0.5,
        ink(0xc6a1ed),
    );
    for (dx, dy, s) in [(0.2, 0.3, 0.16), (0.58, 0.2, 0.22), (0.4, 0.65, 0.12)] {
        ui.rounded(
            sky + width * 0.13 + moon * dx,
            top + 24.0 + moon * dy,
            moon * s,
            moon * s,
            moon * s * 0.5,
            ink(0xa17ac7),
        );
    }
    for n in 0..26 {
        let sx = sky + ((n * 73 + 17) % 211) as f32 / 211.0 * (width * 0.34);
        let sy = top + 18.0 + ((n * 43 + 9) % 157) as f32 / 157.0 * (hero_h * 0.58);
        ui.rounded(sx, sy, 2.0, 2.0, 1.0, ink(0x8864a9));
    }
    for n in 0..13 {
        let xx = sky + (n as f32) * width * 0.026;
        let hh = 32.0 + ((n * 29) % 53) as f32;
        ui.rounded(xx, top + hero_h - hh, width * 0.04, hh, 2.0, ink(0x140c22));
    }
    ui.border(x, top, width, hero_h, 22.0, BORDER);
    let tx = x + 24.0;
    let font = if w < 600.0 { 30.0 } else { 40.0 };
    label(
        ui,
        font,
        tx,
        top + 25.0,
        TEXT,
        if installed { "Ready to" } else { "Your Java" },
    );
    let word = if installed { "play" } else { "worlds" };
    let prefix = if installed { "Ready to " } else { "Your Java " };
    let end = tx + ui.measure(Face::Medium, font, prefix) + 10.0;
    // Narrow content places the highlighted word on its own line.
    let narrow = w < 600.0;
    label(
        ui,
        font,
        if narrow { tx } else { end },
        top + 25.0 + if narrow { 42.0 } else { 0.0 },
        ACCENT,
        word,
    );
    let shift = if narrow { 40.0 } else { 0.0 };
    ui.rounded(tx, top + 78.0 + shift, 90.0, 3.0, 1.5, ACCENT);
    label(
        ui,
        15.0,
        tx,
        top + 94.0 + shift,
        MUTED,
        &format!("Minecraft 26.3  /  Profile {profile}"),
    );
    if !narrow {
        for (i, line) in [
            "Open a world or join a Java server",
            "Touch setup: Options > Controls",
            "Find packs and .hmod in the tabs",
        ]
        .iter()
        .enumerate()
        {
            let y = top + 134.0 + i as f32 * 25.0;
            ui.rounded(tx, y + 2.0, 13.0, 13.0, 6.5, ACCENT);
            label(ui, 13.0, tx + 21.0, y, MUTED, line);
        }
    }
    button(
        ui,
        targets,
        [
            tx,
            top + hero_h - 65.0,
            (width * 0.44).clamp(190.0, 375.0),
            46.0,
        ],
        if installed {
            "Play"
        } else if retry {
            "Retry installation"
        } else {
            "Download Minecraft 26.3"
        },
        Action::Play,
        true,
    );
    let quick_y = top + hero_h + 18.0;
    let columns = if w < 600.0 { 3 } else { 5 };
    let gap = 12.0;
    let cw = (width - gap * (columns as f32 - 1.0)) / columns as f32;
    for (i, (page, sub)) in [
        (Page::Mods, "Add mods easily"),
        (Page::Shaders, "Enhance graphics"),
        (Page::Packs, "Manage packs"),
        (Page::Settings, "Customize launcher"),
        (Page::Skins, "Change your skin"),
    ]
    .iter()
    .enumerate()
    {
        let xx = x + (i % columns) as f32 * (cw + gap);
        let yy = quick_y + (i / columns) as f32 * 76.0;
        card(ui, [xx, yy, cw, 64.0]);
        icon(ui, [xx + 12.0, yy + 21.0, 20.0, 20.0], i);
        label(ui, 14.0, xx + 42.0, yy + 15.0, TEXT, page.name());
        if cw > 160.0 {
            label(ui, 10.0, xx + 42.0, yy + 37.0, MUTED, sub);
        }
        targets.push(([xx, yy, cw, 64.0], Action::Page(*page)));
    }
    let profile_y = quick_y + if columns == 3 { 164.0 } else { 90.0 };
    if profile_y + 100.0 < h - 65.0 {
        label(ui, 20.0, x, profile_y, ACCENT, "Current Profile");
        card(
            ui,
            [
                x,
                profile_y + 34.0,
                if narrow { width } else { width * 0.73 },
                76.0,
            ],
        );
        ui.rounded(x + 18.0, profile_y + 53.0, 30.0, 30.0, 5.0, ink(0x568f47));
        label(ui, 18.0, x + 64.0, profile_y + 49.0, TEXT, "Minecraft 26.3");
        label(
            ui,
            12.0,
            x + 64.0,
            profile_y + 77.0,
            if installed { GREEN } else { MUTED },
            &format!(
                "Profile {profile}  /  {}",
                if installed {
                    "Ready to play"
                } else {
                    "Not installed"
                }
            ),
        );
        if narrow {
            return;
        }
        let ax = x + width * 0.75;
        let aw = width * 0.25;
        card(ui, [ax, profile_y + 34.0, aw, 76.0]);
        label(ui, 14.0, ax + 14.0, profile_y + 49.0, ACCENT, "Appearance");
        label(
            ui,
            11.0,
            ax + 14.0,
            profile_y + 76.0,
            MUTED,
            "Free moon theme",
        );
        targets.push((
            [ax, profile_y + 34.0, aw, 76.0],
            Action::Page(Page::Settings),
        ));
    }
}
