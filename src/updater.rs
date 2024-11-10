use anyhow::{Context, Error};
use reqwest::header::{HeaderMap, ACCEPT, USER_AGENT};
use reqwest::Client;
use self_update::backends::github::ReleaseList;
use self_update::self_replace::self_replace;
use self_update::version::bump_is_greater;
use std::fs::File;
use std::io::Cursor;
use std::ops::Deref;
use std::{env, io};

pub async fn update() -> Result<(), Error> {
    let releases = tokio::task::spawn_blocking(|| ReleaseList::configure()
        .repo_owner("BigBadE")
        .repo_name("BeatBlock-Oneclick")
        .build()?
        .fetch()).await??;

    // get the first available release
    let asset = releases[0]
        .asset_for(&format!("{}-{}", env::consts::OS, env::consts::ARCH.replace("x86_64", "x64")), None)
        .context("Your OS and architecture is not supported! Please file an issue!")?;

    if !bump_is_greater(env!("CARGO_PKG_VERSION"),
                        asset.name.split("-").nth(2).unwrap())? {
        return Ok(());
    }

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
