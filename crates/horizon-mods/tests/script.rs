//! Lua script mods (docs/script-mods.md), run in the game's own sandbox
//! through [`TestGame`], as a WebAssembly mod's `sandbox.rs` does.
use horizon_mods::manifest;
use horizon_mods::testing::TestGame;
use horizon_mods::{Output, Status};

const MANIFEST: &str = r#"
id = "cps_counter"
name = "CPS Counter"
version = "1.0.0"
description = "Attack/use clicks per second on the HUD."
authors = ["test"]
api = "0.1"
side = "client"
capabilities = ["game-read", "hud", "chat", "storage"]
"#;

const CPS: &str = r#"
MOD = { commands = {"cps"}, tick = true, chat = true }

function on_tick(t)
  local attack, use = game.cps()
  local p = game.player()
  hud.panel("top-left", {
    "CPS: " .. attack,
    { text = "x " .. string.format("%.1f", p.x), color = 0x55FFFF },
  })
end

function on_chat(text)
  if text == "show" then chat.show("hello from script") end
end

function on_command(name, args)
  if args == "fail" then error("broken on purpose") end
  storage.set("last", args)
  return "stored " .. (storage.get("last") or "?")
end
"#;

fn hmod(name: &str, script: &str) -> std::path::PathBuf {
    let bytes = manifest::write_script(MANIFEST, script).expect("pack the script mod");
    let path = std::env::temp_dir().join(format!("horizon-script-test-{name}-{}.ksmod", std::process::id()));
    std::fs::write(&path, bytes).expect("write the test mod");
    path
}

#[test]
fn script_mod_ticks_panels_chats_and_commands() {
    let path = hmod("cps", CPS);
    let mut game = TestGame::load(&path).expect("the script mod starts");

    game.tick(2);
    let panel = game.panel().expect("on_tick set a panel");
    assert_eq!(panel.lines[0].text, "CPS: 0");
    assert_eq!(panel.lines[1].text, "x 0.5");
    assert_eq!(panel.lines[1].color, 0x55FFFF);

    game.chat("show");
    game.chat("ignore me");
    let chats: Vec<_> = game
        .drain()
        .into_iter()
        .filter_map(|o| match o {
            Output::Chat { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert_eq!(chats, ["hello from script"]);

    assert!(game.command("cps sprint-reset"));
    let out = game.drain();
    assert!(
        matches!(&out[..], [Output::Feedback { text, error: false }] if text == "stored sprint-reset"),
        "feedback for /cps: {out:?}"
    );
    assert_eq!(game.status(), &Status::Running);

    // A script error in a command is red feedback, not a stopped mod.
    assert!(game.command("cps fail"));
    let out = game.drain();
    assert!(
        matches!(&out[..], [Output::Feedback { text, error: true }] if text.contains("broken on purpose")),
        "red feedback for /cps fail: {out:?}"
    );
    assert_eq!(game.status(), &Status::Running);
}

#[test]
fn runaway_script_is_stopped() {
    let path = hmod(
        "loop",
        "MOD = { tick = true }\nfunction on_tick(t) while true do end end\n",
    );
    let mut game = TestGame::load(&path).expect("the looping mod starts");
    game.tick(1);
    assert!(
        matches!(game.status(), Status::Failed(reason) if reason.contains("too long")),
        "an endless loop stops the mod: {:?}",
        game.status()
    );
    // The game goes on: a stopped mod answers nothing.
    assert!(!game.command("cps x"));
}

#[test]
fn script_errors_at_load_say_why() {
    let path = hmod("bad", "MOD = { tick = true }\nthis is not lua\n");
    let err = TestGame::load(&path).err().expect("a broken script cannot start");
    assert!(err.contains("main.lua"), "the error names the file: {err}");
}

#[test]
fn script_mod_may_not_use_undeclared_surfaces() {
    // Capability `command` is not open to scripts in this build.
    let manifest = MANIFEST.replace(
        "capabilities = [\"game-read\", \"hud\", \"chat\", \"storage\"]",
        "capabilities = [\"game-read\", \"hud\", \"command\"]",
    );
    let bytes = manifest::write_script(&manifest, "MOD = {}\n").expect("pack");
    let path = std::env::temp_dir().join(format!("horizon-script-test-caps-{}.ksmod", std::process::id()));
    std::fs::write(&path, bytes).expect("write");
    let err = TestGame::load(&path).err().expect("command is not a script capability");
    assert!(err.contains("command"), "the refused capability is named: {err}");
}

const CK_MANIFEST: &str = include_str!("../../../mods/ck-kits/horizon-mod.toml");
const CK_LUA: &str = include_str!("../../../mods/ck-kits/main.lua");

fn ck_game() -> TestGame {
    let bytes = manifest::write_script(CK_MANIFEST, CK_LUA).expect("pack ck_kits");
    let path = std::env::temp_dir().join(format!("ck-kits-test-{}.hmod", std::process::id()));
    std::fs::write(&path, bytes).expect("write");
    TestGame::load(&path).expect("ck_kits starts")
}

fn feedback(game: &mut TestGame) -> (String, bool) {
    match game.drain().into_iter().next() {
        Some(Output::Feedback { text, error }) => (text, error),
        other => panic!("expected feedback, got {other:?}"),
    }
}

#[test]
fn ck_kits_save_load_roundtrip_in_creative() {
    let mut game = ck_game();
    game.creative = true;
    game.inventory = vec![
        (0, "{id:\"minecraft:diamond_sword\",count:1}".into()),
        (103, "{id:\"minecraft:iron_helmet\",count:1}".into()),
        (150, "{id:\"minecraft:shield\",count:1}".into()),
    ];
    assert!(game.command("ck save PvP Kit"));
    assert_eq!(feedback(&mut game), ("Saved kit 'pvp_kit' (3 stacks)".into(), false));
    assert!(game.command("ck list"));
    assert_eq!(feedback(&mut game), ("Kits: pvp_kit".into(), false));
    game.inventory = vec![];
    assert!(game.command("ck load pvp_kit"));
    let out = game.drain();
    match &out[..] {
        [Output::SetInventory { slots }, Output::Feedback { error: false, .. }] => {
            assert_eq!(slots.len(), 3);
            assert!(slots.iter().any(|(s, i)| *s == 103 && i.contains("iron_helmet")));
        }
        _ => panic!("unexpected output {out:?}"),
    }
    assert!(game.command("ck delete pvp_kit"));
    assert!(!feedback(&mut game).1);
    assert!(game.command("ck load pvp_kit"));
    assert!(feedback(&mut game).1);
}

#[test]
fn ck_kits_save_works_anywhere_load_needs_creative() {
    let mut game = ck_game();
    game.inventory = vec![(0, "{id:\"minecraft:diamond_sword\",count:1}".into())];
    assert!(game.command("ck save a"));
    assert_eq!(feedback(&mut game), ("Saved kit 'a' (1 stacks)".into(), false));
    assert!(game.command("ck list"));
    assert!(!feedback(&mut game).1);
    assert!(game.command("ck load a"));
    let (text, error) = feedback(&mut game);
    assert!(error && text.contains("creative"), "{text}");
    game.operator = true;
    assert!(game.command("ck load a"));
    let out = game.drain();
    assert!(matches!(&out[..], [Output::SetInventory { .. }, Output::Feedback { error: false, .. }]), "{out:?}");
    assert!(game.command("ck delete a"));
    assert!(!feedback(&mut game).1);
}

fn waypoint_mod() -> std::path::PathBuf {
    let bytes = manifest::write_script(
        include_str!("../../../mods/premium-waypoints/horizon-mod.toml"),
        include_str!("../../../mods/premium-waypoints/main.lua"),
    ).unwrap();
    let path = std::env::temp_dir().join(format!("waypoints-{}.hmod", std::process::id()));
    std::fs::write(&path, bytes).unwrap(); path
}

#[test]
fn waypoints_commands_persist_and_separate_worlds_and_dimensions() {
    let path = waypoint_mod();
    let mut game = TestGame::load(&path).unwrap();
    assert!(game.command("wp add home pink"));
    assert_eq!(game.waypoints().len(), 1, "{:?}", game.drain());
    assert_eq!(game.waypoints()[0].color, 0xFF77BB);
    assert!(game.command("wp add mine 10 70 20 cyan"));
    assert_eq!(game.waypoints().len(), 2);
    game.reload().unwrap(); game.tick(1);
    assert_eq!(game.waypoints().len(), 2);
    game.world_id = Some("server-two".into()); game.tick(1);
    assert!(game.waypoints().is_empty());
    assert!(game.command("wp add home green"));
    assert_eq!(game.waypoints().len(), 1);
    game.world_id = Some("world-test".into()); game.tick(1);
    assert_eq!(game.waypoints().len(), 2);
    game.player.as_mut().unwrap().dimension = "minecraft:the_nether".into(); game.tick(1);
    assert!(game.waypoints().is_empty());
    game.player.as_mut().unwrap().dimension = "minecraft:overworld".into(); game.tick(1);
    assert_eq!(game.waypoints().len(), 2);
    for command in ["wp hide home", "wp show home", "wp edit home color orange", "wp edit home coords 1 2 3", "wp edit home rename base", "wp edit base beam off", "wp beam off", "wp list"] {
        assert!(game.command(command));
        let out=game.drain();
        assert!(!out.iter().any(|o| matches!(o, Output::Feedback { error: true, .. })), "{command}: {out:?}");
    }
    let p=game.waypoints();
    assert_eq!(p[0].name, "base");assert_eq!(p[0].position,[1.0,2.0,3.0]);assert!(!p[0].beam);
    assert!(game.command("wp remove base")); assert_eq!(game.waypoints().len(),1);
    game.player=None;game.world_id=None;game.tick(1);assert!(game.waypoints().is_empty());
}

#[test]
fn waypoints_reject_duplicates_bad_coordinates_and_over_limit() {
    let path=waypoint_mod();let mut game=TestGame::load(&path).unwrap();
    for n in 0..32 { game.command(&format!("wp add p{n}")); }
    assert_eq!(game.waypoints().len(),32);
    for cmd in ["wp add more", "wp add p0", "wp edit p0 coords nan 0 0", "wp edit p0 color nope", "wp edit p0 rename p1"] {
        game.drain(); assert!(game.command(cmd));
        assert!(game.drain().iter().any(|o| matches!(o,Output::Feedback {error:true,..})), "{cmd}");
        assert_eq!(game.waypoints().len(),32);
    }
    assert_eq!(game.status(),&Status::Running);
}

const UI_MANIFEST: &str = r#"
id = "ui_menu"
name = "UI Menu"
version = "1.0.0"
description = "A menu on a key."
authors = ["test"]
api = "0.1"
side = "client"
capabilities = ["chat", "ui", "input"]
experimental = ["ui", "input"]
"#;

const UI_MOD: &str = r#"
MOD = { commands = {} }

input.register_key("open", "Open the menu", "key.keyboard.n")
ui.set_settings({
  { type = "button", id = 10, label = "Beams: on" },
})

function on_key(name, down)
  if name == "open" and down then
    ui.open_screen({
      title = "Waypoints",
      widgets = {
        { type = "text", id = 1, text = "Points", color = 0xFFFFFF },
        { type = "list", id = 2, rows = { "a", "b" }, selected = 0 },
        { type = "button", id = 3, label = "Add" },
        { type = "item", id = 4, item = "minecraft:torch", name = "Beam" },
      },
    })
  end
end

function on_ui_select(id, row)
  if id == 3 then
    ui.close_screen()
  elseif id == 2 then
    chat.show("picked " .. row)
  end
end

function on_ui_closed()
  chat.show("closed")
end

function on_ui_settings()
  ui.set_settings({
    { type = "button", id = 10, label = "Beams: off" },
  })
end
"#;

#[test]
fn script_mod_opens_screens_and_answers_picks() {
    let bytes = manifest::write_script(UI_MANIFEST, UI_MOD).expect("pack the ui mod");
    let path = std::env::temp_dir().join(format!("horizon-script-test-ui-{}.ksmod", std::process::id()));
    std::fs::write(&path, bytes).expect("write the test mod");
    let mut game = TestGame::load(&path).expect("the ui mod starts");

    // The settings page got the widgets the script pushed at load.
    let settings = game.settings_widgets();
    assert!(
        matches!(&settings[..], [horizon_mods::UiWidget::Button(b)] if b.id == 10 && b.label == "Beams: on"),
        "settings widgets: {settings:?}"
    );

    // The registered key is there, under the mod's name.
    let keys = game.mod_keys();
    assert!(
        matches!(&keys[..], [(m, k)] if m == "UI Menu" && k.name == "open" && k.key == "key.keyboard.n"),
        "registered key: {keys:?}"
    );
    assert!(game.screen().is_none(), "no screen before the key");

    // Pressing the key opens the screen.
    game.key("key.keyboard.n", true);
    game.key("key.keyboard.n", false);
    let screen = game.screen().expect("on_key opened a screen");
    let _ = game.drain();
    assert_eq!(screen.title, "Waypoints");
    assert_eq!(screen.widgets.len(), 4);
    assert!(
        matches!(&screen.widgets[1], horizon_mods::UiWidget::List(l) if l.id == 2 && l.rows == ["a", "b"]),
        "the list widget: {:?}",
        screen.widgets
    );

    // A row pick runs on_ui_select; a button pick closes the screen and
    // on_ui_closed hears it.
    game.ui_select(2, 1);
    let out = game.drain();
    assert!(
        out.iter().any(|o| matches!(o, Output::Chat { text, .. } if text == "picked 1")),
        "list pick: {out:?}"
    );
    game.ui_select(3, 0);
    assert!(game.screen().is_none(), "the button closed the screen");
    let _ = game.drain();
    // Leaving a screen (Esc/Done) runs on_ui_closed.
    game.ui_closed();
    let out = game.drain();
    assert!(
        out.iter().any(|o| matches!(o, Output::Chat { text, .. } if text == "closed")),
        "on_ui_closed heard the close: {out:?}"
    );
    game.ui_closed();
    let out = game.drain();
    assert!(
        matches!(&out[..], [Output::Chat { text, .. }] if text == "closed"),
        "closed event: {out:?}"
    );

    // Opening the settings page asks the script for fresh widgets.
    game.ui_settings_opened();
    let settings = game.settings_widgets();
    assert!(
        matches!(&settings[..], [horizon_mods::UiWidget::Button(b)] if b.label == "Beams: off"),
        "settings after on_ui_settings: {settings:?}"
    );
}

#[test]
fn script_ui_needs_the_capability() {
    // Without "ui" in the manifest there is no `ui` table at all.
    let bytes = manifest::write_script(MANIFEST, "MOD = {}\nui.open_screen({ title = \"x\", widgets = {} })\n").expect("pack");
    let path = std::env::temp_dir().join(format!("horizon-script-test-noui-{}.ksmod", std::process::id()));
    std::fs::write(&path, bytes).expect("write the test mod");
    let err = TestGame::load(&path).err().expect("ui without the capability cannot start");
    assert!(err.contains("attempt to index nil"), "no ui table without the capability: {err}");
}
