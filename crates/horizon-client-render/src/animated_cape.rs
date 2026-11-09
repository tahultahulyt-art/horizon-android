//! The Extras page's animated capes (default off, `docs/extras.md`): a clip
//! the player picked, laid out as a vertical strip of 64x32 cape frames and
//! compiled in. While one is on, the local player's cape sprite is a marker
//! whose frames `update_remote_skins` keeps in the atlas and the renderer
//! cycles; only this viewer sees it.

use std::sync::OnceLock;

use crate::hud::decode_bytes;
use crate::skins::process_cape;

/// One animated cape the setting offers. The marker sprite is
/// `capes/anim/{id}`, each frame `capes/anim/{id}/{i}`.
pub struct Choice {
    pub id: &'static str,
    /// The setting button's word for it.
    pub label: &'static str,
    png: &'static [u8],
    pub frames: usize,
}

/// The capes the setting cycles through, in order.
pub const CHOICES: &[Choice] = &[
    Choice {
        id: "anime_boy",
        label: "Anime Boy",
        png: include_bytes!("animated_cape/anime_boy.png"),
        frames: 15,
    },
    Choice {
        id: "obito",
        label: "Obito",
        png: include_bytes!("animated_cape/obito.png"),
        frames: 14,
    },
];

/// A frame shows this long: the clips play at 10 frames per second.
pub const FRAME_SECONDS: f64 = 0.1;

/// The setting's value: 0 is off, else 1-based into `CHOICES`.
pub fn selected(value: u8) -> Option<&'static Choice> {
    value
        .checked_sub(1)
        .and_then(|i| CHOICES.get(usize::from(i)))
}
/// The setting's value as its button shows it.
pub fn label(value: u8) -> &'static str {
    selected(value).map_or("Off", |c| c.label)
}
/// The sprite name `update_remote_skins` gives the local player's cape.
pub fn marker(choice: &Choice) -> String {
    format!("capes/anim/{}", choice.id)
}
/// The sprite name of one of the cape's frames.
pub fn frame_sprite(choice: &Choice, frame: usize) -> String {
    format!("capes/anim/{}/{frame}", choice.id)
}

/// Each frame laid out as a 64x64 cape texture (`skins::process_cape`),
/// decoded once.
pub fn frame_textures(choice: &Choice) -> Option<&'static [Vec<u8>]> {
    static CACHES: [OnceLock<Option<Vec<Vec<u8>>>>; 2] = [OnceLock::new(), OnceLock::new()];
    let index = CHOICES.iter().position(|c| c.id == choice.id)?;
    CACHES[index]
        .get_or_init(|| {
            let (w, h, rgba) = decode_bytes(choice.png).ok()?;
            if w != 64 || h != 32 * choice.frames as u32 {
                return None;
            }
            let mut frames = Vec::with_capacity(choice.frames);
            for f in 0..choice.frames {
                let start = f * 32 * 64 * 4;
                frames.push(process_cape(64, 32, &rgba[start..start + 32 * 64 * 4])?);
            }
            Some(frames)
        })
        .as_ref()
        .map(Vec::as_slice)
}

/// The frame sprite a marker cape shows `now` seconds in; `None` for any
/// other cape (a static cape, another player's).
pub fn frame_for(marker: &str, now: f64) -> Option<String> {
    let id = marker.strip_prefix("capes/anim/")?;
    let choice = CHOICES.iter().find(|c| c.id == id)?;
    let frame = (now / FRAME_SECONDS) as usize % choice.frames;
    Some(frame_sprite(choice, frame))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_decode_to_their_frames() {
        for choice in CHOICES {
            let frames = frame_textures(choice).expect(choice.id);
            assert_eq!(frames.len(), choice.frames);
            assert!(frames.iter().all(|f| f.len() == 64 * 64 * 4));
            // No frame is blank.
            assert!(frames.iter().all(|f| f.iter().any(|&b| b != 0)));
        }
    }

    #[test]
    fn markers_cycle_through_the_frames() {
        let choice = &CHOICES[0];
        let m = marker(choice);
        assert_eq!(
            frame_for(&m, 0.0).as_deref(),
            Some("capes/anim/anime_boy/0")
        );
        assert_eq!(
            frame_for(&m, 0.15).as_deref(),
            Some("capes/anim/anime_boy/1")
        );
        let wrapped = frame_for(&m, choice.frames as f64 * FRAME_SECONDS);
        assert_eq!(wrapped.as_deref(), Some("capes/anim/anime_boy/0"));
        assert_eq!(frame_for("capes/offline", 0.0), None);
        assert_eq!(frame_for("capes/anim/anime_boy/3", 0.0), None);
    }

    #[test]
    fn setting_values_select_a_cape() {
        assert_eq!(label(0), "Off");
        assert_eq!(label(1), "Anime Boy");
        assert_eq!(label(2), "Obito");
        assert_eq!(label(3), "Off");
    }
}
