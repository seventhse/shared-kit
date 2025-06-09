use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::helper::matcher_group::MatcherGroup;
use crate::helper::{
    file_system::copy_directory_with_replace,
    file_transform_pipe::{filter_file_middleware, replace_file_middleware},
};
use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Response;

use crate::helper::{
    file_system::pre_count_files,
    file_transform_pipe::{FileTransformPipe, copy_file_progress_middleware},
};

pub fn create_file_progress(path: &PathBuf) -> anyhow::Result<ProgressBar> {
    let total_files = pre_count_files(path)?;
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} files | {msg}",
        )
        .unwrap(),
    );

    Ok(pb)
}

pub fn create_download_progress(resp: &Response) -> anyhow::Result<ProgressBar> {
    let total_size =
        resp.content_length().with_context(|| "Failed to get content length from response")?;
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    Ok(pb)
}

pub fn download_file_with_progress(resp: Response, dest_path: &Path) -> anyhow::Result<()> {
    let mut dest_file =
        fs::File::create(dest_path).with_context(|| "Failed to create temp zip file")?;

    let pb = create_download_progress(&resp)?;

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];

    let mut stream = resp;
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        dest_file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");

    Ok(())
}

pub fn copy_directory_with_progress(
    origin: &PathBuf,
    target: &PathBuf,
    matcher_group: Option<Arc<MatcherGroup>>,
) -> anyhow::Result<()> {
    let pb = create_file_progress(origin)?;
    let pb = Arc::new(pb);

    let handle = FileTransformPipe::new()
        .add_option(
            matcher_group.clone().map(|mg| replace_file_middleware(mg.clone(), origin.clone())),
        )
        .add_option(
            matcher_group.clone().map(|mg| filter_file_middleware(mg.clone(), origin.clone())),
        )
        .add(copy_file_progress_middleware(pb.clone(), origin.clone()))
        .finalize();

    copy_directory_with_replace(origin, target, Some(&handle))?;

    let total_files = pb.length().unwrap_or(0);

    info_msg!(
        "✅ Template copied from '{}' to '{}' ({} files)",
        origin.display(),
        target.display(),
        total_files
    );
    pb.finish_with_message(format!(
        "\nTemplate copy complete: '{}' → '{}' ({} files)",
        origin.display(),
        target.display(),
        total_files
    ));

    Ok(())
}
