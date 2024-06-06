use crate::{logf, ui::taskdialog};

use serde::{Deserialize, Serialize};
use std::{env, io::Write};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("unable to query releases from GitHub")]
    QueryReleases(reqwest::Error),
    #[error("unable to download release from GitHub")]
    DownloadRelease(reqwest::Error),
    #[error("unable to construct Tokio runtime")]
    TokioRuntime(tokio::io::Error),
    #[error("unable to extract update files")]
    Zip(zip::result::ZipError),
    #[error("update was interrupted by user")]
    Interrupted,
    #[error("failed to remove previous version")]
    RemoveOld,
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubObject {
    sha: String,
    #[serde(alias = "type")]
    g_type: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubResponse {
    #[serde(alias = "ref")]
    g_ref: String,
    node_id: String,
    url: String,
    object: GitHubObject,
}

#[derive(Debug)]
pub struct ReleaseInfo {
    pub version: String,
    pub download_url: String,
}

/// Checks if there are any newer releases available.
///
/// # Errors
///
/// Returns an `Err` if no network connection is available or if GitHub
/// returned an unexpected response that could not be parsed.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn check() -> Option<ReleaseInfo> {
    let old_exe_path = env::current_exe().unwrap().with_extension("exe.old");

    if old_exe_path.exists() && std::fs::remove_file(old_exe_path).is_err() {
        logf!("ERROR: Unable to remove previous version of xterminate (is the file still in use?)");

        taskdialog::TaskDialog::new()
            .set_icon(taskdialog::TaskDialogIcon::ErrorIcon)
            .set_title("Update check failed")
            .set_heading("Could not check for updates")
            .set_content(
                "Update check could not start because xterminate was unable \
                to remove an outdated version of itself. Ensure there are not \
                multiple instances of xterminate running and try again.",
            )
            .display()
            .result();

        return None;
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let current_major = current_version[0..1].parse::<usize>().unwrap();
    let current_minor = current_version[2..3].parse::<usize>().unwrap();
    let current_patch = current_version[4..5].parse::<usize>().unwrap();

    match query_latest() {
        Ok(latest) => {
            if let Some(latest) = latest {
                let latest_major = latest.version[0..1].parse::<usize>().unwrap();
                let latest_minor = latest.version[2..3].parse::<usize>().unwrap();
                let latest_patch = latest.version[4..5].parse::<usize>().unwrap();

                if latest_major > current_major
                    || (latest_major == current_major && latest_minor > current_minor)
                    || (latest_major == current_major
                        && latest_minor == current_minor
                        && latest_patch > current_patch)
                {
                    return Some(latest);
                }
            }

            None
        }

        Err(err) => {
            logf!("ERROR: Unable to check for updates: {err}");

            taskdialog::TaskDialog::new()
                .set_icon(taskdialog::TaskDialogIcon::ErrorIcon)
                .set_title("Update failed")
                .set_heading("Could not update xterminate")
                .set_content(format!(
                    "Could not check for updates due to the following error: \n\n{err}"
                ))
                .display()
                .result();

            None
        }
    }
}

/// Downlooads and installs the specified release of xterminate.
/// This method will show [`TaskDialog`]s fto display download progress
/// and error messages if the update is unsuccessful.
#[allow(clippy::missing_panics_doc)]
pub async fn update(release: ReleaseInfo) {
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    let dialog = taskdialog::TaskDialog::new()
        .set_title("Updating xterminate")
        .set_heading("Downloading update")
        .set_content("Hold on for a moment while xterminate downloads the new update!")
        .add_button(taskdialog::TaskDialogAction::Cancel)
        .add_button(taskdialog::TaskDialogAction::Ok)
        .set_progress(progress.clone(), 0, 100)
        .display();

    logf!("Downloading latest release...");
    let file = download(release, progress.clone()).await;

    if let Err(err) = file {
        logf!("ERROR: Failed to download update: {err}");

        taskdialog::TaskDialog::new()
            .set_icon(taskdialog::TaskDialogIcon::ErrorIcon)
            .set_title("Update failed")
            .set_heading("Could not update xterminate")
            .set_content(format!(
                "Sorry, downloading the update failed due to the following error: \n\n{err}"
            ))
            .display()
            .result();

        return;
    }

    logf!("Installing update...");
    install(file.unwrap());

    let dialog_result = dialog.close();
    let progress = dialog_result.progress.unwrap();

    if progress < 100 {
        logf!("Update progress was < 100 - most likely cancelled by the user");

        // User dismissed the update dialog while the update was in progress
        taskdialog::TaskDialog::new()
            .set_icon(taskdialog::TaskDialogIcon::InformationIcon)
            .set_title("Update aborted")
            .set_heading("The update was cancelled by the user")
            .set_content("Maybe another time?")
            .display()
            .result();

        return;
    }

    logf!("Updated successfully");

    taskdialog::TaskDialog::new()
        .set_title("Update success")
        .set_heading("Update complete!")
        .set_content(
            "The update was installed successfully and will \
                    be applied next time xterminate restarts.",
        )
        .display()
        .result();
}

/// Attempts to download the archive associated with specified release.
///
/// # Arguments
///
/// * `progress` - The variable used to write download progress to.
///                The progress value will be 0 through 100 with
///                100 indicating a fully completed download.
async fn download(
    release: ReleaseInfo,
    progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
) -> Result<std::fs::File, UpdateError> {
    let release_url = format!(
        "https://github.com/imxela/xterminate/releases/download/v{}/xterminate-portable.zip",
        release.version
    );

    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "user-agent",
        format!("xterminate/{current_version}").parse().unwrap(),
    );

    let api_response = client.get(release_url).headers(headers).send();
    let mut response = api_response.await.map_err(UpdateError::DownloadRelease)?;

    let length = response.content_length().unwrap();
    let mut written = 0;

    let mut file = tempfile::tempfile().unwrap();

    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).unwrap();
        written += chunk.len();

        // Map download progress to a 0-100 range to feed to the progress bar
        let written_mapped = written * 100 / usize::try_from(length).unwrap();

        progress.store(
            written_mapped.try_into().unwrap(),
            std::sync::atomic::Ordering::SeqCst,
        );
    }

    Ok(file)
}

fn install(file: std::fs::File) {
    std::fs::rename(
        env::current_exe().unwrap(),
        env::current_exe()
            .unwrap()
            .with_file_name("xterminate.exe.old"),
    )
    .unwrap();

    let mut archive = zip::read::ZipArchive::new(file).unwrap();
    archive.extract(env::current_dir().unwrap()).unwrap();
}

/// Returns a vector containing all GitHub releases for xterminate
///
/// # Errors
///
/// Returns an `Err` if reqwest was unable to query GitHub or if
/// GitHub returned an expected response.
#[allow(clippy::missing_panics_doc)]
pub fn query_all() -> Result<Vec<ReleaseInfo>, UpdateError> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::blocking::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "user-agent",
        format!("xterminate/{current_version}").parse().unwrap(),
    );

    let api_url = "https://api.github.com/repos/imxela/xterminate/git/refs/tags".to_owned();
    let api_response = client
        .get(api_url)
        .headers(headers)
        .send()
        .map_err(UpdateError::QueryReleases)?;

    let result = api_response
        .json::<Vec<GitHubResponse>>().map_err(UpdateError::QueryReleases)?
        .into_iter()
        .map(|v| {
            // Slice at the last forward-slash so we only have '/vMajor.Minor.Patch'
            // and then move the slice another 2 positions so there's only
            // the version number left.
            let version = v.g_ref[v.g_ref.rfind('/').unwrap() + 2..].to_owned();

            ReleaseInfo {
                version: version.clone(),
                download_url: format!("https://github.com/imxela/xterminate/releases/download/v{version}/xterminate-setup.exe"),
            }
        })
        .collect::<Vec<ReleaseInfo>>();

    Ok(result)
}

/// # Errors
///
/// Returns `Err` if [`query_releases()`] return an `Err`.
#[allow(clippy::missing_panics_doc)]
pub fn query_latest() -> Result<Option<ReleaseInfo>, UpdateError> {
    let releases = query_all()?;

    let mut latest_major = 0;
    let mut latest_minor = 0;
    let mut latest_patch = 0;
    let mut latest = None;

    for release in releases {
        let release_major = release.version[0..1].parse::<usize>().unwrap();
        let release_minor = release.version[2..3].parse::<usize>().unwrap();
        let release_patch = release.version[4..5].parse::<usize>().unwrap();

        if release_major > latest_major
            || (release_major == latest_major && release_minor > latest_minor)
            || (release_major == latest_major
                && release_minor == latest_minor
                && release_patch > latest_patch)
        {
            latest_major = release_major;
            latest_minor = release_minor;
            latest_patch = release_patch;
            latest = Some(release);
        }
    }

    Ok(latest)
}
