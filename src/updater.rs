use anyhow::Error;
use reqwest::header::{HeaderMap, ACCEPT, USER_AGENT};
use reqwest::Client;
use self_update::backends::github::ReleaseList;
use self_update::self_replace::self_replace;
use std::fs::File;
use std::io::{Cursor, Read};
use std::ops::Deref;
use std::{env, io};
use zip::ZipArchive;

pub async fn update() -> Result<(), Error> {
    let releases = tokio::task::spawn_blocking(|| ReleaseList::configure()
        .repo_owner("BigBadE")
        .repo_name("BeatBlock-Oneclick")
        .build()?
        .fetch()).await??;

    let target = match env::consts::OS {
        "linux" => match env::consts::ARCH {
            "x86_64" => "linux",
            _ => return Err(Error::msg("unsupported ARCH")),
        },
        "macos" => match env::consts::ARCH {
            "x86_64" => "macos-x86",
            "arm" => "macos-arm",
            _ => return Err(Error::msg("unsupported ARCH")),
        },
        "windows" => match env::consts::ARCH {
            "x86_64" => "windows",
            _ => return Err(Error::msg("unsupported ARCH")),
        }
        _ => return Err(Error::msg("unsupported OS")),
    };
    // get the first available release
    let asset = releases[0]
        .asset_for(&target, None)
        .unwrap();

    let tmp_dir = tempfile::Builder::new()
        .prefix("self_update")
        .tempdir_in(env::current_dir()?)?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "BigBadE-BeatBlock-Oneclick".parse()?);
    headers.insert(ACCEPT, "application/octet-stream".parse()?);
    let response = Client::builder().default_headers(headers).build()?.get(asset.download_url).send().await?;
    let bytes = response.bytes().await?;

    println!("{}", String::from_utf8_lossy(bytes.clone().into_iter().collect::<Vec<_>>().deref()));

    // Create a cursor for the ZIP bytes
    let mut cursor = Cursor::new(bytes);
    let new_exe = tmp_dir.path().join("self_update_bin");
    io::copy(&mut cursor, &mut File::create(&new_exe)?)?;
    self_replace(new_exe)?;

    Ok(())
}
