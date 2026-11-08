# Kusobo Loader v1

Kusobo is an Android management layer for Horizon's existing sandboxed .hmod WebAssembly API 0.1 plus Lua script mods (.ksmod/.hmod holding main.lua; docs/script-mods.md). It is not a separate Java runtime. Fabric, Forge and NeoForge .jar mods do not run here. Source-only .hnmod bundles need a separate native port/build.

Lua sample installs the Kusobo CPS Counter script mod (OFF until enabled). Lua template exports kusobo-script-template.zip to Configuration with a ready manifest, main.lua and GUIDE.txt: edit on the phone, zip, rename to .ksmod, import. No computer needed.

The script resources live in script/: main.lua and kusobo_cps.hmod.toml are the source; kusobo_cps.ksmod and kusobo-script-template.zip are packed from them (zip with horizon-mod.toml at the root). Re-pack after edits and refresh SHA256SUMS.

Open Mods > Kusobo Loader. Import a supported .hmod, then open Installed to enable or disable it. Changes apply on the next Play. New imports and the sample start OFF. Invalid imports report their error in the launcher status. Runtime traps, missing capabilities and incompatible APIs appear in game at Options > Mods. A trapped mod is stopped by the existing runtime.

Wasm sample adds Kusobo Coordinates without replacing a file or changing the enabled list. Enable it in Installed, then Play. /coords hides or shows the HUD; /where prints coordinates to your own chat. It reads position/world time and can draw HUD and local chat. No network or filesystem access is granted to the sample. It does not change worlds.

Export template writes kusobo-template.zip into Configuration without replacing an existing file. Open Files > Configuration to copy it out. This is a source template, NOT an importable mod. Building Rust/WebAssembly requires a Rust development environment. Phone-only compilation is not part of v1.

## Build sample/template on a development computer

Install Rust, then `rustup target add wasm32-unknown-unknown`.
Install the packager: `cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod`.
In the extracted template, run `cargo horizon-mod build`. The .hmod is written under target/horizon-mod. Run `cargo horizon-mod test` for the panel/command tests. The API belongs to Horizon and is not renamed. MIT attribution is retained.

In this repository the equivalent source is mods/kusobo/coordinates. Its SDK dependency points to the matching local Horizon SDK. From that directory use the same build/test commands. The checked-in APK resource was built from that source using Rust 1.95.0, wit-bindgen 0.60.0 and wit-component 0.254.2. See SHA256SUMS for packaged resource hashes.
