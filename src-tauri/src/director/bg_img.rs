use std::path::PathBuf;

use crate::news::NewsMaterial;


pub(crate) fn get_bg_img(_material: &NewsMaterial) -> PathBuf {
    const DEFAULT_BG_IMG: &str = "bg-img-0.png";

    PathBuf::from(".")
        .join("assets")
        .join("bg-img")
        .join(DEFAULT_BG_IMG)
}