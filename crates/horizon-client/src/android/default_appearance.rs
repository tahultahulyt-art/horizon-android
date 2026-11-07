//! Local user-requested appearance; never uploads or changes an online profile.
use crate::skins::{
    Model,
    library::{Kind, Library},
};
use std::path::Path;
const SKIN: &[u8] = include_bytes!("assets/risiya/skin.png");
const CAPE: &[u8] = include_bytes!("assets/risiya/cape.png");
pub fn seed(root: &Path) -> Result<bool, String> {
    let marker = root.join("risiya-default-appearance-v1");
    if marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(false);
    }
    let mut library = Library::load(root)?;
    // An existing selection, including an explicit hide/default choice, wins.
    if !library.selections.contains_key("offline:risiya") {
        let skin = library.import(root, "risiya skin", Kind::Skin, SKIN)?;
        let cape = library.import(root, "risiya cape", Kind::Cape, CAPE)?;
        library.select(root, "offline:risiya", &skin, Kind::Skin)?;
        library.select(root, "offline:risiya", &cape, Kind::Cape)?;
        library.selections.get_mut("offline:risiya").unwrap().model = Model::Classic;
        library.save(root)?;
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requested_images_validate_and_seed_without_resetting_later_choice() {
        let root = std::env::temp_dir().join(format!("risiya-appearance-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            crate::skins::read_skin(SKIN).unwrap().dimensions(),
            (64, 64)
        );
        assert_eq!(
            crate::skins::read_cape(CAPE).unwrap().dimensions(),
            (64, 32)
        );
        assert_eq!(seed(&root), Ok(true));
        let mut lib = Library::load(&root).unwrap();
        let selection = &lib.selections["offline:risiya"];
        assert!(selection.skin.is_some() && selection.cape.is_some());
        assert_eq!(selection.model, Model::Classic);
        lib.selections.get_mut("offline:risiya").unwrap().hide_cape = true;
        lib.save(&root).unwrap();
        assert_eq!(seed(&root), Ok(false));
        assert!(Library::load(&root).unwrap().selections["offline:risiya"].hide_cape);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn existing_risiya_and_other_users_appearances_are_preserved() {
        let root =
            std::env::temp_dir().join(format!("risiya-appearance-existing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut lib = Library::default();
        lib.selections
            .insert("offline:risiya".into(), Default::default());
        lib.selections
            .insert("online-id".into(), Default::default());
        lib.save(&root).unwrap();
        assert_eq!(seed(&root), Ok(true));
        let lib = Library::load(&root).unwrap();
        assert!(lib.entries.is_empty());
        assert_eq!(lib.selections.len(), 2);
        assert!(lib.selections["offline:risiya"].skin.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
