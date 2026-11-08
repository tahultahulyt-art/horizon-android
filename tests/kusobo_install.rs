#[path = "../crates/horizon-client/src/android/kusobo.rs"]
mod kusobo;
use kusobo::Resource;
#[test]
fn install_off_and_keep_existing() {
    let p = std::env::temp_dir().join(format!("kusobo-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    let msg = kusobo::install(&p, Resource::Sample).unwrap();
    assert!(msg.contains("OFF"));
    assert!(!p.join("horizon-mods.txt").exists());
    let f = p.join("horizon-mods/kusobo_coordinates.hmod");
    assert!(std::fs::metadata(&f).unwrap().len() > 1000);
    std::fs::write(&f, b"user data").unwrap();
    assert!(kusobo::install(&p, Resource::Sample).is_err());
    assert_eq!(std::fs::read(&f).unwrap(), b"user data");
    kusobo::install(&p, Resource::Template).unwrap();
    assert!(p.join("kusobo-template.zip").exists());
    let msg = kusobo::install(&p, Resource::ScriptSample).unwrap();
    assert!(msg.contains("OFF"));
    assert!(p.join("horizon-mods/kusobo_cps.ksmod").exists());
    kusobo::install(&p, Resource::ScriptTemplate).unwrap();
    assert!(p.join("kusobo-script-template.zip").exists());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn refuses_redirected_destination() {
    let p = std::env::temp_dir().join(format!("kusobo-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir(&p).unwrap();
    std::os::unix::fs::symlink(std::env::temp_dir(), p.join("horizon-mods")).unwrap();
    assert!(
        kusobo::install(&p, Resource::Sample)
            .unwrap_err()
            .contains("redirected")
    );
    std::fs::remove_dir_all(p).unwrap();
}
