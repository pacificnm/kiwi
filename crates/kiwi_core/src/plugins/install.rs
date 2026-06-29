//! Build and stage plugins for `~/.config/kiwi/plugins/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use kiwi_plugin_api::{PluginInstallConfig, PluginInstallKind, PluginManifest};

use super::registry::PluginRegistryEntry;

/// Outcome of a successful plugin install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstallResult {
    pub entry: PluginRegistryEntry,
    pub messages: Vec<String>,
}

/// Outcome of removing an installed plugin from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRemoveResult {
    pub name: String,
    pub messages: Vec<String>,
}

/// Progress update emitted during a long-running plugin install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallProgressUpdate {
    pub message: String,
    pub step: u32,
    pub total: u32,
}

struct InstallReporter<'a> {
    callback: Option<&'a mut dyn FnMut(InstallProgressUpdate)>,
    step: u32,
    total: u32,
}

impl<'a> InstallReporter<'a> {
    fn new(callback: Option<&'a mut dyn FnMut(InstallProgressUpdate)>, total: u32) -> Self {
        Self {
            callback,
            step: 0,
            total: total.max(1),
        }
    }

    fn report(&mut self, message: impl Into<String>) {
        self.step = (self.step + 1).min(self.total);
        let update = InstallProgressUpdate {
            message: message.into(),
            step: self.step,
            total: self.total,
        };
        if let Some(callback) = self.callback.as_mut() {
            callback(update);
        }
    }
}

/// Install a plugin from `src_path` into `~/.config/kiwi/plugins/<name>/`.
pub fn install_plugin_from_source(src_path: &Path) -> Result<PluginInstallResult, String> {
    install_plugin_from_source_with_progress(src_path, &mut |_| {})
}

/// Install a plugin and emit progress updates for UI surfaces.
pub fn install_plugin_from_source_with_progress(
    src_path: &Path,
    progress: &mut dyn FnMut(InstallProgressUpdate),
) -> Result<PluginInstallResult, String> {
    install_plugin_from_source_inner(src_path, Some(progress))
}

fn install_plugin_from_source_inner(
    src_path: &Path,
    progress: Option<&mut dyn FnMut(InstallProgressUpdate)>,
) -> Result<PluginInstallResult, String> {
    let src_path = src_path
        .canonicalize()
        .map_err(|e| format!("Invalid plugin path {}: {e}", src_path.display()))?;

    let manifest_path = src_path.join("plugin.toml");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read {}: {e}", manifest_path.display()))?;
    let manifest: PluginManifest = toml::from_str(&manifest_str)
        .map_err(|e| format!("Invalid plugin.toml: {e}"))?;

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())?;
    let plugins_dir = crate::config::default_plugins_directory(Some(&home));
    let dest_dir = plugins_dir.join(&manifest.name);

    if dest_dir.exists() {
        return Err(format!(
            "Plugin directory already exists at {}. Remove it with `kiwi plugin remove {}` or reinstall with `kiwi plugin reinstall <path>`.",
            dest_dir.display(),
            manifest.name
        ));
    }

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create {}: {e}", dest_dir.display()))?;

    let plan = resolve_install_plan(&src_path, &manifest)?;
    let mut reporter = InstallReporter::new(progress, estimate_install_steps(&plan));
    reporter.report(format!("Preparing install for `{}`", manifest.name));

    install_plugin_with_plan(&src_path, &dest_dir, &manifest, &plan, &home, &mut reporter)
}

fn install_plugin_with_plan(
    src_path: &Path,
    dest_dir: &Path,
    manifest: &PluginManifest,
    plan: &InstallPlan,
    home: &Path,
    reporter: &mut InstallReporter<'_>,
) -> Result<PluginInstallResult, String> {
    let mut messages = Vec::new();

    let stage_result = match plan.kind {
        PluginInstallKind::Cargo => {
            build_and_stage_cargo(src_path, dest_dir, manifest, plan, home, reporter)
        }
        PluginInstallKind::Artifact => {
            stage_artifact(src_path, dest_dir, manifest, plan, home, reporter)
        }
        PluginInstallKind::Copy => stage_copy(src_path, dest_dir, plan, reporter),
    };

    if let Err(err) = stage_result {
        let _ = fs::remove_dir_all(dest_dir);
        return Err(err);
    }
    messages.extend(stage_result.unwrap());

    let ext = library_extension();
    let lib_filename = find_library_filename(dest_dir, ext)
        .unwrap_or_else(|| manifest.entry.clone());

    messages.push(format!(
        "Plugin `{}` installed to {}.",
        manifest.name,
        dest_dir.display()
    ));

    Ok(PluginInstallResult {
        entry: PluginRegistryEntry {
            name: manifest.name.clone(),
            display_name: manifest.display_name.clone(),
            version: manifest.version.clone(),
            enabled: true,
            installed_path: dest_dir.to_path_buf(),
            entry: lib_filename,
            source: "local".to_string(),
        },
        messages,
    })
}

fn estimate_install_steps(plan: &InstallPlan) -> u32 {
    let mut steps = 1u32; // prepare
    match plan.kind {
        PluginInstallKind::Cargo => {
            steps += 1; // main build
            steps += 1; // stage library / manifest
            steps += plan.binaries.len() as u32;
            for extra in &plan.extra_packages {
                steps += 1;
                steps += extra.binaries.len() as u32;
            }
        }
        PluginInstallKind::Artifact => {
            steps += 1;
            steps += plan.binaries.len() as u32;
        }
        PluginInstallKind::Copy => {
            steps += plan.files.len() as u32;
        }
    }
    steps.max(1)
}

/// Remove an installed plugin directory and any binaries it registered.
pub fn remove_plugin_from_disk(name: &str) -> Result<PluginRemoveResult, String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())?;
    let install_dir = crate::config::default_plugins_directory(Some(&home)).join(name);
    remove_plugin_at(&install_dir, name, Some(&home))
}

fn remove_plugin_at(
    install_dir: &Path,
    name: &str,
    home: Option<&Path>,
) -> Result<PluginRemoveResult, String> {
    if !install_dir.is_dir() {
        return Err(format!(
            "Plugin `{name}` is not installed at {}.",
            install_dir.display()
        ));
    }

    let mut messages = Vec::new();
    let manifest_path = install_dir.join("plugin.toml");
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = toml::from_str::<PluginManifest>(&content) {
            if let Some(home) = home {
                messages.extend(remove_installed_binaries(&manifest, home)?);
            }
        }
    }

    fs::remove_dir_all(install_dir).map_err(|e| {
        format!(
            "Failed to remove plugin directory {}: {e}",
            install_dir.display()
        )
    })?;
    messages.push(format!(
        "Removed plugin `{name}` from {}.",
        install_dir.display()
    ));

    Ok(PluginRemoveResult {
        name: name.to_string(),
        messages,
    })
}

/// Remove an installed plugin, then install it again from `src_path`.
pub fn reinstall_plugin_from_source(src_path: &Path) -> Result<PluginInstallResult, String> {
    reinstall_plugin_from_source_with_progress(src_path, &mut |_| {})
}

/// Reinstall a plugin and emit progress updates for UI surfaces.
pub fn reinstall_plugin_from_source_with_progress(
    src_path: &Path,
    progress: &mut dyn FnMut(InstallProgressUpdate),
) -> Result<PluginInstallResult, String> {
    let src_path = src_path
        .canonicalize()
        .map_err(|e| format!("Invalid plugin path {}: {e}", src_path.display()))?;

    let manifest_path = src_path.join("plugin.toml");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read {}: {e}", manifest_path.display()))?;
    let manifest: PluginManifest = toml::from_str(&manifest_str)
        .map_err(|e| format!("Invalid plugin.toml: {e}"))?;

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())?;
    let plugins_dir = crate::config::default_plugins_directory(Some(&home));
    let dest_dir = plugins_dir.join(&manifest.name);

    let plan = resolve_install_plan(&src_path, &manifest)?;
    let mut reporter = InstallReporter::new(Some(progress), estimate_install_steps(&plan) + 1);
    reporter.report(format!("Removing existing `{}` install", manifest.name));
    let _ = remove_plugin_from_disk(&manifest.name);
    if dest_dir.exists() {
        let _ = fs::remove_dir_all(&dest_dir);
    }

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create {}: {e}", dest_dir.display()))?;

    install_plugin_with_plan(&src_path, &dest_dir, &manifest, &plan, &home, &mut reporter)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallPlan {
    kind: PluginInstallKind,
    package: Option<String>,
    profile: String,
    library: Option<String>,
    binaries: Vec<String>,
    bin_dir: PathBuf,
    files: Vec<String>,
    extra_packages: Vec<kiwi_plugin_api::PluginExtraPackage>,
}

fn resolve_install_plan(src_path: &Path, manifest: &PluginManifest) -> Result<InstallPlan, String> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let default_bin_dir = crate::config::default_plugin_bin_directory(home.as_deref());

    if let Some(config) = &manifest.install {
        return plan_from_config(src_path, config, home.as_deref(), default_bin_dir);
    }

    if src_path.join("Cargo.toml").is_file() {
        let crate_meta = read_crate_metadata(src_path)?;
        return Ok(InstallPlan {
            kind: PluginInstallKind::Cargo,
            package: Some(crate_meta.package_name),
            profile: "release".to_string(),
            library: crate_meta.library_name,
            binaries: Vec::new(),
            bin_dir: default_bin_dir,
            files: vec!["plugin.toml".to_string()],
            extra_packages: Vec::new(),
        });
    }

    if find_library_in_dir(src_path).is_some() {
        return Ok(InstallPlan {
            kind: PluginInstallKind::Artifact,
            package: None,
            profile: "release".to_string(),
            library: None,
            binaries: Vec::new(),
            bin_dir: default_bin_dir,
            files: vec!["plugin.toml".to_string()],
            extra_packages: Vec::new(),
        });
    }

    Ok(InstallPlan {
        kind: PluginInstallKind::Copy,
        package: None,
        profile: "release".to_string(),
        library: None,
        binaries: Vec::new(),
        bin_dir: default_bin_dir,
        files: vec!["plugin.toml".to_string()],
        extra_packages: Vec::new(),
    })
}

fn plan_from_config(
    src_path: &Path,
    config: &PluginInstallConfig,
    home: Option<&Path>,
    default_bin_dir: PathBuf,
) -> Result<InstallPlan, String> {
    let bin_dir = config
        .bin_dir
        .as_deref()
        .map(|path| crate::config::expand_tilde(path, home))
        .unwrap_or(default_bin_dir);

    let mut plan = InstallPlan {
        kind: config.kind,
        package: config.package.clone(),
        profile: config.profile.clone(),
        library: config.library.clone(),
        binaries: config.binaries.clone(),
        bin_dir,
        files: if config.files.is_empty() {
            vec!["plugin.toml".to_string()]
        } else {
            config.files.clone()
        },
        extra_packages: config.extra_packages.clone(),
    };

    if plan.kind == PluginInstallKind::Cargo {
        if plan.package.is_none() {
            plan.package = Some(read_crate_metadata(src_path)?.package_name);
        }
        if plan.library.is_none() {
            plan.library = read_crate_metadata(src_path)?.library_name;
        }
    }

    if plan.profile != "release" && plan.profile != "debug" {
        return Err(format!(
            "Invalid install.profile `{}` — expected `release` or `debug`",
            plan.profile
        ));
    }

    Ok(plan)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CrateMetadata {
    package_name: String,
    library_name: Option<String>,
}

fn read_crate_metadata(crate_root: &Path) -> Result<CrateMetadata, String> {
    let cargo_path = crate_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| format!("Cannot read {}: {e}", cargo_path.display()))?;
    let table: toml::Table = toml::from_str(&content)
        .map_err(|e| format!("Invalid Cargo.toml at {}: {e}", cargo_path.display()))?;

    let package_name = table
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Cargo.toml at {} is missing [package].name", cargo_path.display()))?
        .to_string();

    let library_name = table
        .get("lib")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    Ok(CrateMetadata {
        package_name,
        library_name,
    })
}

fn build_and_stage_cargo(
    src_path: &Path,
    dest_dir: &Path,
    _manifest: &PluginManifest,
    plan: &InstallPlan,
    home: &Path,
    reporter: &mut InstallReporter<'_>,
) -> Result<Vec<String>, String> {
    let package = plan
        .package
        .as_deref()
        .ok_or_else(|| "install.package is required for cargo plugins".to_string())?;

    let workspace_root = find_workspace_root(src_path)?;
    reporter.report(format!("Building `{package}` ({})…", plan.profile));

    run_cargo_build(&workspace_root, package, &plan.profile)?;
    let target_dir = workspace_root.join("target").join(&plan.profile);
    let mut messages = Vec::new();

    if let Some(library_stem) = plan.library.as_deref() {
        let lib_filename = format!("lib{library_stem}.{ext}", ext = library_extension());
        let built_lib = target_dir.join(&lib_filename);
        if !built_lib.is_file() {
            return Err(format!(
                "Build succeeded but library not found at {}. Check install.library in plugin.toml.",
                built_lib.display()
            ));
        }

        copy_file(&src_path.join("plugin.toml"), &dest_dir.join("plugin.toml"))?;
        copy_file(&built_lib, &dest_dir.join(&lib_filename))?;
        reporter.report(format!("Staged {lib_filename}"));
        messages.push(format!("Staged {lib_filename}"));
    } else {
        copy_file(&src_path.join("plugin.toml"), &dest_dir.join("plugin.toml"))?;
        reporter.report("Copied plugin.toml".to_string());
        messages.push("Copied plugin.toml".to_string());
    }

    messages.extend(install_binaries(&target_dir, plan, home, reporter)?);
    messages.extend(install_extra_packages(
        &workspace_root,
        &plan.extra_packages,
        &plan.profile,
        &plan.bin_dir,
        home,
        reporter,
    )?);
    Ok(messages)
}

fn run_cargo_build(workspace_root: &Path, package: &str, profile: &str) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--package").arg(package);
    if profile == "release" {
        cmd.arg("--release");
    }
    cmd.current_dir(workspace_root);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run cargo build: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo build --package {package} failed:\n{stderr}"));
    }
    Ok(())
}

fn install_extra_packages(
    workspace_root: &Path,
    extras: &[kiwi_plugin_api::PluginExtraPackage],
    profile: &str,
    bin_dir: &Path,
    home: &Path,
    reporter: &mut InstallReporter<'_>,
) -> Result<Vec<String>, String> {
    if extras.is_empty() {
        return Ok(Vec::new());
    }

    let mut messages = Vec::new();
    let target_dir = workspace_root.join("target").join(profile);
    for extra in extras {
        if extra.binaries.is_empty() {
            return Err(format!(
                "install.extra_packages entry for `{}` must list at least one binary",
                extra.package
            ));
        }
        reporter.report(format!("Building `{}` ({profile})…", extra.package));
        run_cargo_build(workspace_root, &extra.package, profile)?;
        let plan = InstallPlan {
            kind: PluginInstallKind::Cargo,
            package: Some(extra.package.clone()),
            profile: profile.to_string(),
            library: None,
            binaries: extra.binaries.clone(),
            bin_dir: bin_dir.to_path_buf(),
            files: Vec::new(),
            extra_packages: Vec::new(),
        };
        messages.extend(install_binaries(&target_dir, &plan, home, reporter)?);
    }
    Ok(messages)
}

fn stage_artifact(
    src_path: &Path,
    dest_dir: &Path,
    _manifest: &PluginManifest,
    plan: &InstallPlan,
    home: &Path,
    reporter: &mut InstallReporter<'_>,
) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();
    copy_file(&src_path.join("plugin.toml"), &dest_dir.join("plugin.toml"))?;

    let ext = library_extension();
    let library = find_library_in_dir(src_path).ok_or_else(|| {
        format!(
            "No pre-built .{ext} library found in {}. Build the plugin first or use install.kind = \"cargo\".",
            src_path.display()
        )
    })?;
    let dest_name = library
        .file_name()
        .ok_or_else(|| "Invalid library filename".to_string())?;
    copy_file(&library, &dest_dir.join(dest_name))?;
    reporter.report(format!("Staged {}", dest_name.to_string_lossy()));
    messages.push(format!(
        "Staged {}",
        dest_name.to_string_lossy()
    ));

    messages.extend(install_binaries(src_path, plan, home, reporter)?);
    Ok(messages)
}

fn stage_copy(
    src_path: &Path,
    dest_dir: &Path,
    plan: &InstallPlan,
    reporter: &mut InstallReporter<'_>,
) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();
    for file in &plan.files {
        let src = src_path.join(file);
        if !src.is_file() {
            return Err(format!("Missing install file {} in {}", file, src_path.display()));
        }
        let dest = dest_dir.join(file);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
        }
        copy_file(&src, &dest)?;
        reporter.report(format!("Copied {file}"));
        messages.push(format!("Copied {file}"));
    }
    Ok(messages)
}

fn remove_installed_binaries(
    manifest: &PluginManifest,
    home: &Path,
) -> Result<Vec<String>, String> {
    let Some(install) = &manifest.install else {
        return Ok(Vec::new());
    };
    if install.binaries.is_empty() {
        return Ok(Vec::new());
    }

    let bin_dir = install
        .bin_dir
        .as_deref()
        .map(|path| crate::config::expand_tilde(path, Some(home)))
        .unwrap_or_else(|| crate::config::default_plugin_bin_directory(Some(home)));

    let mut messages = Vec::new();
    for bin in &install.binaries {
        let path = bin_dir.join(bin);
        if path.is_file() {
            fs::remove_file(&path).map_err(|e| {
                format!("Failed to remove binary {}: {e}", path.display())
            })?;
            messages.push(format!("Removed `{bin}` from {}", bin_dir.display()));
        }
    }
    Ok(messages)
}

fn install_binaries(
    search_dir: &Path,
    plan: &InstallPlan,
    home: &Path,
    reporter: &mut InstallReporter<'_>,
) -> Result<Vec<String>, String> {
    if plan.binaries.is_empty() {
        return Ok(Vec::new());
    }

    fs::create_dir_all(&plan.bin_dir)
        .map_err(|e| format!("Failed to create {}: {e}", plan.bin_dir.display()))?;

    let mut messages = Vec::new();
    for bin in &plan.binaries {
        let src = search_dir.join(bin);
        if !src.is_file() {
            return Err(format!(
                "Binary `{bin}` not found at {} after build.",
                src.display()
            ));
        }
        let dest = plan.bin_dir.join(bin);
        reporter.report(format!("Installing `{bin}` to {}", dest.display()));
        copy_executable(&src, &dest)?;
        messages.push(format!(
            "Installed `{bin}` to {}",
            dest.display()
        ));
    }

    let local_bin = crate::config::default_plugin_bin_directory(Some(home));
    if plan.bin_dir == local_bin {
        messages.push(
            "Ensure ~/.local/bin is on your PATH to run installed agent binaries.".to_string(),
        );
    }

    Ok(messages)
}

fn find_workspace_root(crate_root: &Path) -> Result<PathBuf, String> {
    let mut dir = crate_root.to_path_buf();
    loop {
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.is_file() {
            let content = fs::read_to_string(&cargo_path)
                .map_err(|e| format!("Cannot read {}: {e}", cargo_path.display()))?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(crate_root.to_path_buf())
}

fn find_library_in_dir(dir: &Path) -> Option<PathBuf> {
    let ext = library_extension();
    let mut matches = Vec::new();
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == ext) {
            matches.push(path);
        }
    }
    matches.sort();
    matches.into_iter().next()
}

fn find_library_filename(dir: &Path, ext: &str) -> Option<String> {
    find_library_in_dir(dir).and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| name.ends_with(&format!(".{ext}")))
    })
}

fn copy_file(src: &Path, dest: &Path) -> Result<(), String> {
    fs::copy(src, dest).map_err(|e| {
        format!(
            "Failed to copy {} → {}: {e}",
            src.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn copy_executable(src: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        replace_executable(src, dest)?;
    } else {
        copy_file(src, dest)?;
        set_executable_permissions(dest)?;
    }
    Ok(())
}

#[cfg(unix)]
fn replace_executable(src: &Path, dest: &Path) -> Result<(), String> {
    let parent = dest
        .parent()
        .ok_or_else(|| format!("Invalid destination path {}", dest.display()))?;
    let file_name = dest
        .file_name()
        .ok_or_else(|| format!("Invalid destination path {}", dest.display()))?;
    let tmp = parent.join(format!("{}.new", file_name.to_string_lossy()));
    let old = parent.join(format!("{}.old", file_name.to_string_lossy()));

    let _ = fs::remove_file(&tmp);
    copy_file(src, &tmp)?;
    set_executable_permissions(&tmp)?;

    let _ = fs::remove_file(&old);
    fs::rename(dest, &old).map_err(|e| {
        format!(
            "Failed to rotate {} for update: {e}",
            dest.display()
        )
    })?;
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::rename(&old, dest);
        format!(
            "Failed to install updated binary at {}: {e}",
            dest.display()
        )
    })?;
    let _ = fs::remove_file(&old);
    Ok(())
}

#[cfg(not(unix))]
fn replace_executable(src: &Path, dest: &Path) -> Result<(), String> {
    copy_file(src, dest)?;
    set_executable_permissions(dest)
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
        .map_err(|e| format!("Failed to read permissions for {}: {e}", path.display()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
        .map_err(|e| format!("Failed to set permissions on {}: {e}", path.display()))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn library_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("kiwi-plugin-install-{name}-{id}"))
    }

    #[test]
    fn copy_install_stages_manifest_only() {
        let src = temp_dir("copy-src");
        let dest_parent = temp_dir("copy-dest-parent");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("plugin.toml"),
            r#"
            name = "cursor-agent"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"

            [install]
            kind = "copy"
            "#,
        )
        .expect("manifest");

        let manifest: PluginManifest = toml::from_str(
            &fs::read_to_string(src.join("plugin.toml")).expect("read"),
        )
        .expect("parse");
        let plan = resolve_install_plan(&src, &manifest).expect("plan");
        assert_eq!(plan.kind, PluginInstallKind::Copy);

        let dest = dest_parent.join("cursor-agent");
        fs::create_dir_all(&dest).expect("dest");
        let mut reporter = InstallReporter::new(None, 1);
        let messages = stage_copy(&src, &dest, &plan, &mut reporter).expect("stage");
        assert!(dest.join("plugin.toml").is_file());
        assert!(messages.iter().any(|m| m.contains("plugin.toml")));
    }

    #[test]
    fn artifact_install_requires_library() {
        let src = temp_dir("artifact-src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("plugin.toml"),
            r#"
            name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"

            [install]
            kind = "artifact"
            "#,
        )
        .expect("manifest");
        let dest = temp_dir("artifact-dest");
        fs::create_dir_all(&dest).expect("dest");

        let manifest: PluginManifest = toml::from_str(
            &fs::read_to_string(src.join("plugin.toml")).expect("read"),
        )
        .expect("parse");
        let plan = resolve_install_plan(&src, &manifest).expect("plan");
        let mut reporter = InstallReporter::new(None, 1);
        let err = stage_artifact(&src, &dest, &manifest, &plan, Path::new("/tmp"), &mut reporter)
            .expect_err("missing library");
        assert!(err.contains("No pre-built"));
    }

    #[test]
    fn artifact_install_stages_library_and_manifest() {
        let src = temp_dir("artifact-happy-src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("plugin.toml"),
            r#"
            name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"

            [install]
            kind = "artifact"
            "#,
        )
        .expect("manifest");
        fs::write(src.join("libhello.so"), b"fake").expect("lib");

        let dest = temp_dir("artifact-happy-dest");
        fs::create_dir_all(&dest).expect("dest");
        let manifest: PluginManifest = toml::from_str(
            &fs::read_to_string(src.join("plugin.toml")).expect("read"),
        )
        .expect("parse");
        let plan = resolve_install_plan(&src, &manifest).expect("plan");
        let mut reporter = InstallReporter::new(None, 1);
        stage_artifact(&src, &dest, &manifest, &plan, Path::new("/tmp"), &mut reporter).expect("stage");
        assert!(dest.join("plugin.toml").is_file());
        assert!(dest.join("libhello.so").is_file());
    }

    #[test]
    fn remove_plugin_at_deletes_directory_and_binaries() {
        let root = temp_dir("remove-root");
        let home = root.join("home");
        let bin_dir = home.join(".local/bin");
        fs::create_dir_all(&bin_dir).expect("bin dir");
        fs::write(bin_dir.join("kiwi-ollama"), b"fake bin").expect("bin");

        let install_dir = home.join(".config/kiwi/plugins/ollama");
        fs::create_dir_all(&install_dir).expect("install dir");
        fs::write(
            install_dir.join("plugin.toml"),
            r#"
            name = "ollama"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"

            [install]
            kind = "cargo"
            binaries = ["kiwi-ollama"]
            "#,
        )
        .expect("manifest");
        fs::write(install_dir.join("libkiwi_plugin_ollama.so"), b"fake").expect("lib");

        let result = remove_plugin_at(&install_dir, "ollama", Some(&home)).expect("remove");
        assert_eq!(result.name, "ollama");
        assert!(!install_dir.exists());
        assert!(!bin_dir.join("kiwi-ollama").exists());
        assert!(result.messages.iter().any(|m| m.contains("Removed plugin")));
    }

    #[test]
    fn remove_plugin_at_errors_when_missing() {
        let install_dir = temp_dir("remove-missing");
        let err = remove_plugin_at(&install_dir, "ghost", None).expect_err("missing");
        assert!(err.contains("not installed"));
    }

    #[test]
    fn read_crate_metadata_parses_package_and_lib() {
        let dir = temp_dir("crate-meta");
        fs::create_dir_all(&dir).expect("dir");
        fs::write(
            dir.join("Cargo.toml"),
            r#"
            [package]
            name = "kiwi_plugin_hello"

            [lib]
            name = "kiwi_plugin_hello"
            crate-type = ["cdylib"]
            "#,
        )
        .expect("cargo");
        let meta = read_crate_metadata(&dir).expect("meta");
        assert_eq!(meta.package_name, "kiwi_plugin_hello");
        assert_eq!(meta.library_name.as_deref(), Some("kiwi_plugin_hello"));
    }

    #[test]
    fn replace_executable_updates_running_binary() {
        let root = temp_dir("replace-exec");
        let bin_dir = root.join("bin");
        fs::create_dir_all(&bin_dir).expect("bin dir");
        let dest = bin_dir.join("kiwi-mcp-git");
        fs::write(&dest, b"version-1").expect("dest");
        set_executable_permissions(&dest).expect("chmod");

        let src = root.join("kiwi-mcp-git-new");
        fs::write(&src, b"version-2").expect("src");

        replace_executable(&src, &dest).expect("replace");
        let content = fs::read(&dest).expect("read dest");
        assert_eq!(content, b"version-2");
    }
}
