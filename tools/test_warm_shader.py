#!/usr/bin/env python3
"""Full native GL activation/draw and recovery tests; no player accounts or game assets.
Requires a working host GL adapter; failure is not skipped. Not a phone benchmark.
"""
from pathlib import Path
import os,subprocess
ROOT=Path(__file__).resolve().parents[1]
h=ROOT/'.local/android/warm-tests'
(h/'src').mkdir(parents=True,exist_ok=True)
(h/'Cargo.toml').write_text('[package]\nname="warm-tests"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nhorizon-client-render={path="../../../crates/horizon-client-render",features=["test-support"]}\nhorizon-shader-api={path="../../../crates/horizon-shader-api"}\nwgpu={version="30.0.1",default-features=false,features=["std","parking_lot","gles","wgsl","naga-ir"]}\npollster="0.4"\nserde_json="1"\n')
(h/'src/main.rs').write_text('fn main(){let i=wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle().with_env());let a=pollster::block_on(i.request_adapter(&Default::default())).unwrap();println!("{:?}",a.get_info());let(d,q)=pollster::block_on(a.request_device(&Default::default())).unwrap();assert!(horizon_client_render::gfx::native_probe::injected_panic_is_recoverable(&d));assert!(horizon_client_render::gfx::native_probe::injected_validation_is_recoverable(&d));let p=horizon_shader_api::Pack::read(std::path::Path::new("crates/horizon-shader-api/packs/horizon_warm_lite")).unwrap();let b=horizon_client_render::gfx::native_probe::run(&d,&q,&p).unwrap();std::fs::write("/downloads/warm-lite-native-gl.rgba",b).unwrap();println!("Real NativeFrame/Sky pipeline/Post draw passed after injected failures");}\n')
(h/'src/lib.rs').write_text('#[path="../../../../crates/horizon-client-render/src/shader_safety.rs"] mod shader_safety;\n#[path="../../../../crates/horizon-client/src/android/warm_shader.rs"] mod warm_shader;\n')
env=dict(os.environ);env['WGPU_BACKEND']='gl';env.setdefault('XDG_RUNTIME_DIR','/tmp')
subprocess.run(['cargo','test','--lib','--manifest-path',str(h/'Cargo.toml')],cwd=ROOT,env=env,check=True)
subprocess.run(['cargo','run','--manifest-path',str(h/'Cargo.toml')],cwd=ROOT,env=env,check=True)
