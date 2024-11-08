use anyhow::Error;
use reqwest::header::ACCEPT;
use self_update::backends::github::ReleaseList;
use self_update::self_replace::self_replace;
use self_update::{ArchiveKind, Download, Extract};
use std::env;
use std::fs::File;
use std::path::PathBuf;

pub fn update() -> Result<(), Error> {
    let releases = ReleaseList::configure()
        .repo_owner("BigBadE")
        .repo_name("BeatBlock-Oneclick")
        .build()?
        .fetch()?;
    println!("found releases:");
    println!("{:#?}\n", releases);

    // get the first available release
    let asset = releases[0]
        .asset_for(&self_update::get_target(), None).unwrap();

    let tmp_dir = tempfile::Builder::new()
        .prefix("self_update")
        .tempdir_in(env::current_dir()?)?;
    let tmp_tarball_path = tmp_dir.path().join(&asset.name);
    let tmp_tarball = File::open(&tmp_tarball_path)?;

    Download::from_url(&asset.download_url)
        .set_header(ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_tarball)?;

    let bin_name = PathBuf::from("self_update_bin");
    Extract::from_source(&tmp_tarball_path)
        .archive(ArchiveKind::Zip)
        .extract_file(&tmp_dir.path(), &bin_name)?;

    let new_exe = tmp_dir.path().join(bin_name);
    self_replace(new_exe)?;

    Ok(())
}