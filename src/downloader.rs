use crate::settings::LocalData;
use anyhow::Error;
use reqwest::get;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

const URL: &'static str = "http://127.0.0.1:3000";

pub async fn handle_download(data: LocalData, map: String) -> Result<(), Error> {
    let Some(levels_dir) = data.levels_dir else {
        return Err(Error::msg(
            "Set the location of the custom levels folder in your Account page",
        ));
    };

    // Download the ZIP file
    let response = get(format!("{URL}/output/{map}.zip")).await?;
    let bytes = response.bytes().await?;

    if bytes.len() == 0 {
        return Err(Error::msg("Unknown error downloading beatmap!"));
    }

    // Create a cursor for the ZIP bytes
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;

    let map = levels_dir.join(map);
    if fs::exists(map.clone()).unwrap_or(false) {
        return Err(Error::msg("Map already downloaded!"));
    }
    fs::create_dir_all(map.clone())?;
    zip.extract(map.clone())
        // Delete the folder if we screw up
        .map_err(|err| {
            let _ = fs::remove_dir_all(levels_dir.join(map));
            err.into()
        })
}

pub fn handle_remove(data: LocalData, map: String) -> Result<(), Error> {
    let Some(levels_dir) = data.levels_dir else {
        return Err(Error::msg(
            "Set the location of the custom levels folder in your Account page",
        ));
    };

    let map = levels_dir.join(map);
    if !fs::exists(map.clone()).unwrap_or(true) {
        return Err(Error::msg(format!(
            "Map is not downloaded! {}",
            map.to_str().unwrap()
        )));
    }

    fs::remove_dir_all(map)?;
    Ok(())
}
