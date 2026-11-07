#!/usr/bin/env python3
"""Original animation math/state regression tests; outputs pose matrices for a labelled fixture preview.
Not an Android performance test or screenshot of the game.
"""
from pathlib import Path
import subprocess,os
root=Path(__file__).resolve().parents[1]
h=root/'.local/android/risiya-animation-tests';(h/'src').mkdir(parents=True,exist_ok=True)
(h/'Cargo.toml').write_text('[package]\nname="risiya-animation-tests"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\nglam="0.30"\nserde_json="1"\n')
(h/'src/lib.rs').write_text('#[path="../../../../crates/horizon-client/src/risiya_animation.rs"] pub mod motion;\n')
(h/'src/main.rs').write_text('''use risiya_animation_tests::motion::*;
fn main(){let mut frames=Vec::new();for kind in ["sword","tool","light","inspect","switch"] {for i in 0..61 {let p=i as f32/60.0;let style=match kind {"tool"=>Style::Tool,"light"=>Style::Light,_=>Style::Sword};let t=transform(Motion{mode:2,style,left:false,attack:if kind=="inspect"||kind=="switch" {0.0}else{p},time:0.0,speed:0.0,sprint:false,vertical:0.0,landing:0.0,inspect:if kind=="inspect"{p}else{0.0}});frames.push(serde_json::json!({"kind":kind,"frame":i,"matrix":t.to_cols_array(),"equip":if kind=="switch"{equip(2,(p*std::f32::consts::PI).sin())}else{0.0}}));}}println!("{}",serde_json::to_string(&frames).unwrap());}''')
subprocess.run(['cargo','test','--manifest-path',str(h/'Cargo.toml')],cwd=root,check=True)
with open('/downloads/risiya-animation-poses.json','w')as f:subprocess.run(['cargo','run','--quiet','--manifest-path',str(h/'Cargo.toml')],cwd=root,stdout=f,check=True)
