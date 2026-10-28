use std::{collections::HashSet, env, fs};

use zed_extension_api::{
    self as zed,
    http_client::{HttpMethod, HttpRequest},
    serde_json,
    settings::LspSettings,
    Architecture, DownloadedFileType, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result,
};

const SERVER_ID: &str = "svelte-language-server";
const SERVER_NAME: &str = "svelte-language-server-rs";
const SERVER_REPO: &str = "hanielu/svelte-language-server-rs";
const SERVER_DIR: &str = "server";
const TSRS_DIR: &str = "tsrs";
const TS_PLUGIN_PACKAGE_NAME: &str = "typescript-svelte-plugin";

struct SvelteExtension {
    /// Paths (relative to the work directory) resolved in this extension process, so each
    /// language server start does not ask GitHub or npm again.
    server_binary: Option<String>,
    tsrs_dir: Option<String>,
    installed_packages: HashSet<String>,
}

impl SvelteExtension {
    /// The newest svelte-language-server-rs release for this platform, downloaded once.
    fn server_binary(&mut self, id: &LanguageServerId) -> Result<String> {
        if let Some(path) = self.server_binary.as_ref().filter(|path| is_file(path)) {
            return Ok(path.clone());
        }
        let (os, arch) = zed::current_platform();
        let target = match (os, arch) {
            (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
            (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
            (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc",
            _ => return Err(format!("{SERVER_NAME} has no release for {os:?} {arch:?}")),
        };
        let binary_in = |version: &str| {
            format!(
                "{SERVER_DIR}/{version}/{SERVER_NAME}-{version}-{target}/{SERVER_NAME}{}",
                exe_suffix(os)
            )
        };

        zed::set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = match zed::latest_github_release(
            SERVER_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(release) => release,
            // Offline or rate-limited: keep the last download.
            Err(error) => {
                let binary = only_version(SERVER_DIR)
                    .map(|version| binary_in(&version))
                    .filter(|path| is_file(path))
                    .ok_or(error)?;
                self.server_binary = Some(binary.clone());
                return Ok(binary);
            }
        };

        let binary = binary_in(&release.version);
        if !is_file(&binary) {
            let (extension, file_type) = match os {
                Os::Windows => ("zip", DownloadedFileType::Zip),
                _ => ("tar.gz", DownloadedFileType::GzipTar),
            };
            let asset_name = format!("{SERVER_NAME}-{}-{target}.{extension}", release.version);
            let asset = release
                .assets
                .iter()
                .find(|asset| asset.name == asset_name)
                .ok_or_else(|| format!("release {} has no {asset_name}", release.version))?;
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &format!("{SERVER_DIR}/{}", release.version),
                file_type,
            )
            .map_err(|error| format!("downloading {asset_name}: {error}"))?;
            zed::make_file_executable(&binary)?;
            remove_other_versions(SERVER_DIR, &release.version);
        }
        self.server_binary = Some(binary.clone());
        Ok(binary)
    }

    /// The directory of a tsrs for the server to type check with, unless the project or the
    /// user's `PATH` already has one (the server finds those itself and prefers the project's).
    fn tsrs_dir(&mut self, id: &LanguageServerId, worktree: &zed::Worktree) -> Option<String> {
        if worktree.which("tsrs").is_some()
            || worktree.read_text_file("node_modules/.bin/tsrs").is_ok()
        {
            return None;
        }
        let (os, _) = zed::current_platform();
        let binary_name = format!("tsrs{}", exe_suffix(os));
        if let Some(dir) = self
            .tsrs_dir
            .as_ref()
            .filter(|dir| is_file(&format!("{dir}/{binary_name}")))
        {
            return Some(dir.clone());
        }
        match download_tsrs(id, &binary_name) {
            Ok(dir) => {
                self.tsrs_dir = Some(dir.clone());
                Some(dir)
            }
            Err(error) => {
                eprintln!("{SERVER_NAME}: no tsrs for type checking: {error}");
                None
            }
        }
    }

    fn install_package_if_needed(&mut self, id: &LanguageServerId, package: &str) -> Result<()> {
        let installed_version = zed::npm_package_installed_version(package)?;
        if installed_version.is_some() && self.installed_packages.contains(package) {
            return Ok(());
        }
        zed::set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let latest_version = zed::npm_package_latest_version(package)?;
        if installed_version.as_ref() != Some(&latest_version) {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Downloading,
            );
            // A failed update must not take down a working install.
            if let Err(error) = zed::npm_install_package(package, &latest_version) {
                if installed_version.is_none() {
                    return Err(error);
                }
            }
        }
        self.installed_packages.insert(package.into());
        Ok(())
    }
}

impl zed::Extension for SvelteExtension {
    fn new() -> Self {
        Self {
            server_binary: None,
            tsrs_dir: None,
            installed_packages: HashSet::new(),
        }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let (os, _) = zed::current_platform();
        let binary = LspSettings::for_worktree(SERVER_ID, worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let mut env = worktree.shell_env();
        if let Some(dir) = self.tsrs_dir(id, worktree) {
            let dir = absolute(&dir)?;
            match env
                .iter_mut()
                .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
            {
                // Last on PATH: a tsrs the user installs later still comes first.
                Some((_, path)) => {
                    path.push(if os == Os::Windows { ';' } else { ':' });
                    path.push_str(&dir);
                }
                None => set_env(
                    &mut env,
                    "SVELTE_LANGUAGE_SERVER_RS_TSRS",
                    format!("{dir}/tsrs{}", exe_suffix(os)),
                ),
            }
        }
        if let Some(vars) = binary.as_ref().and_then(|binary| binary.env.clone()) {
            for (key, value) in vars {
                set_env(&mut env, &key, value);
            }
        }
        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| vec!["--stdio".into()]);
        let command = match binary.and_then(|binary| binary.path) {
            Some(path) => path,
            None => match worktree.which(SERVER_NAME) {
                Some(path) => path,
                None => absolute(&self.server_binary(id)?)?,
            },
        };
        Ok(zed::Command { command, args, env })
    }

    fn language_server_additional_workspace_configuration(
        &mut self,
        id: &LanguageServerId,
        target_id: &LanguageServerId,
        _: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // Lets vtsls resolve `.svelte` imports in `.ts` files, as the official extension does.
        if target_id.as_ref() != "vtsls" {
            return Ok(None);
        }
        self.install_package_if_needed(id, TS_PLUGIN_PACKAGE_NAME)?;
        Ok(Some(serde_json::json!({
            "vtsls": {
                "tsserver": {
                    "globalPlugins": [{
                        "name": TS_PLUGIN_PACKAGE_NAME,
                        "location": absolute(&format!("node_modules/{TS_PLUGIN_PACKAGE_NAME}"))?,
                        "enableForWorkspaceTypeScriptVersions": true
                    }]
                }
            }
        })))
    }
}

/// The newest `@tsrs/<platform>` package from npm, unpacked without Node; returns the directory
/// holding the binary.
fn download_tsrs(id: &LanguageServerId, binary_name: &str) -> Result<String> {
    let (os, arch) = zed::current_platform();
    let package = match (os, arch) {
        (Os::Mac, Architecture::Aarch64) => "darwin-arm64",
        (Os::Mac, Architecture::X8664) => "darwin-x64",
        (Os::Linux, Architecture::Aarch64) => "linux-arm64-gnu",
        (Os::Linux, Architecture::X8664) => "linux-x64-gnu",
        (Os::Windows, Architecture::X8664) => "win32-x64",
        _ => return Err(format!("tsrs has no build for {os:?} {arch:?}")),
    };
    let dir_in = |version: &str| format!("{TSRS_DIR}/{version}/package");
    let latest = HttpRequest::builder()
        .method(HttpMethod::Get)
        .url(format!("https://registry.npmjs.org/@tsrs/{package}/latest"))
        .build()?
        .fetch()
        .and_then(|response| {
            serde_json::from_slice::<serde_json::Value>(&response.body)
                .map_err(|error| error.to_string())
        });
    let latest = match latest {
        Ok(latest) => latest,
        Err(error) => {
            return only_version(TSRS_DIR)
                .map(|version| dir_in(&version))
                .filter(|dir| is_file(&format!("{dir}/{binary_name}")))
                .ok_or(error);
        }
    };
    let (Some(version), Some(tarball)) = (
        latest["version"].as_str(),
        latest["dist"]["tarball"].as_str(),
    ) else {
        return Err(format!("unexpected npm metadata for @tsrs/{package}"));
    };
    let dir = dir_in(version);
    let binary = format!("{dir}/{binary_name}");
    if !is_file(&binary) {
        zed::set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::Downloading,
        );
        zed::download_file(
            tarball,
            &format!("{TSRS_DIR}/{version}"),
            DownloadedFileType::GzipTar,
        )
        .map_err(|error| format!("downloading @tsrs/{package}@{version}: {error}"))?;
        zed::make_file_executable(&binary)?;
        remove_other_versions(TSRS_DIR, version);
    }
    Ok(dir)
}

fn exe_suffix(os: Os) -> &'static str {
    if os == Os::Windows {
        ".exe"
    } else {
        ""
    }
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

fn absolute(path: &str) -> Result<String> {
    let dir = env::current_dir().map_err(|error| error.to_string())?;
    Ok(dir.join(path).to_string_lossy().into_owned())
}

/// The version directory under `dir` when there is exactly one, as after every download.
fn only_version(dir: &str) -> Option<String> {
    let mut versions = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok());
    let version = versions.next()?;
    versions.next().is_none().then_some(version)
}

fn remove_other_versions(dir: &str, keep: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_str() != Some(keep) {
            fs::remove_dir_all(entry.path()).ok();
        }
    }
}

fn set_env(env: &mut Vec<(String, String)>, key: &str, value: String) {
    match env.iter_mut().find(|(name, _)| name == key) {
        Some((_, existing)) => *existing = value,
        None => env.push((key.to_string(), value)),
    }
}

zed::register_extension!(SvelteExtension);
