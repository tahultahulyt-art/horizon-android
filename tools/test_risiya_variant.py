#!/usr/bin/env python3
"""One-time Risiya content plus native shader/GLES translation checks; no user data."""
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
h=ROOT/'.local/android/risiya-tests'
(h/'src').mkdir(parents=True,exist_ok=True)
(h/'Cargo.toml').write_text('[package]\nname="risiya-tests"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nhorizon-protocol={path="../../../crates/horizon-protocol"}\nhorizon-world={path="../../../crates/horizon-world"}\nhorizon-shader-api={path="../../../crates/horizon-shader-api"}\nnaga={version="30.0.1",features=["wgsl-in","glsl-out"]}\n')
(h/'src/lib.rs').write_text('#[path="../../../../crates/horizon-client/src/android/risiya_content.rs"] mod content;\n#[test] fn shader_check_and_gles() {\nlet p=horizon_shader_api::Pack::read(std::path::Path::new("../../../crates/horizon-client/src/android/assets/risiya/light")).unwrap();\nlet report=horizon_shader_api::check(&p).unwrap();for stage in &report.stages {\nfor ep in &stage.module.entry_points {\nlet options=naga::back::glsl::Options::default();\nlet pipeline=naga::back::glsl::PipelineOptions{shader_stage:ep.stage,entry_point:ep.name.clone(),multiview:None};let mut glsl=String::new();\nnaga::back::glsl::Writer::new(&mut glsl,&stage.module,&stage.info,&options,&pipeline,Default::default()).unwrap().write().unwrap();assert!(glsl.contains("#version 310 es"));\n}\n}\n\n}\n')
subprocess.run(['cargo','test','--manifest-path',str(h/'Cargo.toml')],cwd=ROOT,check=True)
