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
