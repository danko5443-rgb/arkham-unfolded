//! The three authored symbols used by this experiment, from one verified font.

use std::{io::Cursor, path::Path};

use bevy::prelude::{Assets, Font, Handle};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub(super) enum Symbol {
    Wild,
    Curse,
    Willpower,
}

impl Symbol {
    pub(super) fn glyph(self) -> &'static str {
        match self {
            Self::Wild => "j",
            Self::Curse => "y",
            Self::Willpower => "f",
        }
    }
}

#[derive(Deserialize)]
struct Manifest {
    path: String,
    sha256: String,
    bytes: usize,
    block_offset: usize,
    block_length: usize,
    decoded_length: usize,
    font_offset: usize,
    font_length: usize,
    font_sha256: String,
}

pub(super) fn load(root: Option<&Path>, fonts: &mut Assets<Font>) -> Option<Handle<Font>> {
    let manifest: Manifest =
        serde_json::from_str(include_str!("../../docs/experiments/symbol-art.json")).ok()?;
    let path = root?.join(&manifest.path);
    if std::fs::metadata(&path).ok()?.len() != manifest.bytes as u64 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let font = decode(&bytes, &manifest)?;
    Some(fonts.add(Font::from_bytes(font)))
}

fn decode(bytes: &[u8], manifest: &Manifest) -> Option<Vec<u8>> {
    if bytes.len() != manifest.bytes || format!("{:x}", Sha256::digest(bytes)) != manifest.sha256 {
        return None;
    }

    // These offsets belong only to the exact bundle above. Unity omits the
    // eight-byte output length from its LZMA block; restore it for LZMA-alone.
    let block = bytes.get(manifest.block_offset..manifest.block_offset + manifest.block_length)?;
    let mut lzma = Vec::with_capacity(block.len() + 8);
    lzma.extend_from_slice(block.get(..5)?);
    lzma.extend_from_slice(&(manifest.decoded_length as u64).to_le_bytes());
    lzma.extend_from_slice(block.get(5..)?);
    let mut decoded = Vec::with_capacity(manifest.decoded_length);
    lzma_rs::lzma_decompress(&mut Cursor::new(lzma), &mut decoded).ok()?;
    if decoded.len() != manifest.decoded_length {
        return None;
    }
    let font = decoded.get(manifest.font_offset..manifest.font_offset + manifest.font_length)?;
    if format!("{:x}", Sha256::digest(font)) != manifest.font_sha256 {
        return None;
    }
    Some(font.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_and_changed_artwork_leave_font_assets_empty() {
        let mut fonts = Assets::default();
        assert!(load(None, &mut fonts).is_none());
        let root = tempfile::tempdir().unwrap();
        assert!(load(Some(root.path()), &mut fonts).is_none());

        let manifest: Manifest =
            serde_json::from_str(include_str!("../../docs/experiments/symbol-art.json")).unwrap();
        let path = root.path().join(&manifest.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // Matching size is insufficient: a changed bundle must fail identity.
        std::fs::write(path, vec![0; manifest.bytes]).unwrap();
        assert!(load(Some(root.path()), &mut fonts).is_none());
        assert!(fonts.is_empty());
    }

    #[test]
    fn archived_font_decodes_when_the_local_reference_is_available() {
        let root = Path::new(super::super::ARCHIVE);
        let manifest: Manifest =
            serde_json::from_str(include_str!("../../docs/experiments/symbol-art.json")).unwrap();
        if !root.join(&manifest.path).exists() {
            eprintln!("External symbol font is absent; real-asset check skipped.");
            return;
        }
        let mut fonts = Assets::default();
        let handle = load(Some(root), &mut fonts).expect("the archived reference font must decode");
        let font = fonts.get(&handle).unwrap();
        assert_eq!(font.data.len(), 144020);
        assert_eq!(&font.data.as_ref()[..4], &[0, 1, 0, 0]);
        assert_eq!(
            format!("{:x}", Sha256::digest(font.data.as_ref())),
            "186176d7243c72d272a50c24e833fb77d983e7c4c6c99bc64f89c8322dbd8187"
        );
    }
}
