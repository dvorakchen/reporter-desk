use std::path::PathBuf;

use crate::news::NewsMaterial;

pub(crate) fn get_bgm(_material: &NewsMaterial) -> PathBuf {
    const DEFAULT_BGM: &str = "general_bgm_01.mp3";

    PathBuf::from(".")
        .join("assets")
        .join("bgm")
        .join(DEFAULT_BGM)
}
