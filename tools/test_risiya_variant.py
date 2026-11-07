#!/usr/bin/env python3
"""One-time Risiya content plus native shader/GLES translation checks; no user data."""
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
h=ROOT/'.local/android/risiya-tests'
(h/'src').mkdir(parents=True,exist_ok=True)
(h/'Cargo.toml').write_text('[package]\nname="risiya-tests"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nhorizon-protocol={path="../../../crates/horizon-protocol"}\nhorizon-world={path="../../../crates/horizon-world"}\nhorizon-shader-api={path="../../../crates/horizon-shader-api"}\nnaga={version="30.0.1",features=["wgsl-in","glsl-out"]}\n')
(h/'src/lib.rs').write_text('#[path="../../../../crates/horizon-client/src/android/risiya_content.rs"] mod content;\n#[test] fn shader_check_and_gles() {\nlet p=horizon_shader_api::Pack::read(std::path::Path::new("../../../tests/fixtures/risiya-light-removed")).unwrap();\nlet report=horizon_shader_api::check(&p).unwrap();for stage in &report.stages {\nfor ep in &stage.module.entry_points {\nlet options=naga::back::glsl::Options::default();\nlet pipeline=naga::back::glsl::PipelineOptions{shader_stage:ep.stage,entry_point:ep.name.clone(),multiview:None};let mut glsl=String::new();\nnaga::back::glsl::Writer::new(&mut glsl,&stage.module,&stage.info,&options,&pipeline,Default::default()).unwrap().write().unwrap();assert!(glsl.contains("#version 310 es"));\n}\n}\n\n}\n')

# Real option codecs; unrelated key mapping definitions are not needed for these values.
with (h/'Cargo.toml').open('a') as f:
    f.write('horizon-core={path="../../../crates/horizon-core"}\nserde_json="1"\n')
source = r'''
#[path="../../../../crates/horizon-client/src/android/fresh_game_settings.rs"] mod fresh_game_settings;
#[path="../../../../crates/horizon-client-render/src/java_text.rs"] mod java_text;
#[path="../../../../crates/horizon-client-render/src/options_fixes.rs"] mod options_fixes;
#[path="../../../../crates/horizon-client-render/src/options_file.rs"] mod options_file;
#[path="../../../../crates/horizon-client-input/src/key_names.rs"] mod key_names;
mod keybinds {pub const MAPPINGS:&[(&str,&str,&str)]=&[("key.jump","key.keyboard.space","movement")];}
mod settings {pub const SOUND_SOURCES:[&str;11]=["master","music","record","weather","block","hostile","neutral","player","ambient","voice","ui"];}
#[test] fn requested_screenshot_values_survive_real_option_codecs() {
let f=options_file::OptionsFile::parse(fresh_game_settings::OPTIONS);
assert_eq!(f.name("preferredGraphicsBackend"),"opengl");assert_eq!(f.name("graphicsPreset"),"custom");assert_eq!(f.name("renderClouds"),"fast");assert_eq!(f.name("inactivityFpsLimit"),"afk");
for (key,value) in [("renderDistance",12),("simulationDistance",12),("biomeBlendRadius",2),("particles",0),("mipmapLevels",4),("menuBackgroundBlurriness",5),("cloudRange",128),("textureFiltering",0),("maxAnisotropyBit",2),("weatherRadius",10),("attackIndicator",1),("maxFps",260),("guiScale",0)] {assert_eq!(f.int(key),value,"{key}");}
for key in ["ao","entityShadows","cutoutLeaves","showAutosaveIndicator","vignette"]{assert!(f.bool(key),"{key}");}
for key in ["improvedTransparency","enableVsync","fullscreen","exclusiveFullscreen","showSubtitles","directionalAudio"]{assert!(!f.bool(key),"{key}");}
assert_eq!(f.double("gamma"),1.0);assert_eq!(f.double("entityDistanceScaling"),1.0);assert_eq!(f.double("chunkSectionFadeInTime"),0.75);
for source in settings::SOUND_SOURCES {assert_eq!(f.double(&format!("soundCategory_{source}")),if source=="music"{0.0}else{1.0},"{source}");}
}
'''
table=(ROOT/'crates/horizon-client-input/src/keybinds.rs').read_text()
table=table[table.index('pub const MAPPINGS:'):table.index('];')+2]
source=source.replace('pub const MAPPINGS:&[(&str,&str,&str)]=&[("key.jump","key.keyboard.space","movement")];',table)
with (h/'src/lib.rs').open('a') as f: f.write(source)
subprocess.run(['cargo','test','--manifest-path',str(h/'Cargo.toml')],cwd=ROOT,check=True)
