use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub bin_path: PathBuf,
    pub source: Source,
    #[serde(default)]
    pub size_bytes: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Category>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shadowed_by: Vec<PathBuf>,
}

impl Tool {
    pub fn new(name: impl Into<String>, bin_path: impl Into<PathBuf>, source: Source) -> Self {
        Self {
            name: name.into(),
            bin_path: bin_path.into(),
            source,
            size_bytes: 0,
            version: None,
            install_path: None,
            symlink_target: None,
            installed_at: None,
            package_id: None,
            homepage: None,
            category: None,
            aliases: Vec::new(),
            shadowed_by: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    Homebrew { tap: String, kind: BrewKind },
    Npm { scope: Option<String> },
    Bun,
    Cargo,
    Rustup { toolchain: String },
    Go,
    Gh,
    Pip { interpreter: PathBuf },
    AppBundle { app_path: PathBuf },
    Manual { dir: PathBuf },
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Homebrew {
                kind: BrewKind::Formula,
                ..
            } => "brew",
            Source::Homebrew {
                kind: BrewKind::Cask,
                ..
            } => "cask",
            Source::Npm { .. } => "npm",
            Source::Bun => "bun",
            Source::Cargo => "cargo",
            Source::Rustup { .. } => "rustup",
            Source::Go => "go",
            Source::Gh => "gh-ext",
            Source::Pip { .. } => "pip",
            Source::AppBundle { .. } => "app",
            Source::Manual { .. } => "manual",
        }
    }

    pub fn is_manual(&self) -> bool {
        matches!(self, Source::Manual { .. })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrewKind {
    Formula,
    Cask,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Ai,
    DevTool,
    Container,
    Lang,
    Other,
}

/// Accumulates install roots and binary paths claimed by managed scanners,
/// so the orphan detector can tell what's already accounted for.
#[derive(Debug, Default, Clone)]
pub struct ClaimedPaths {
    bin_paths: HashSet<PathBuf>,
    install_roots: Vec<PathBuf>,
}

impl ClaimedPaths {
    pub fn add_bin(&mut self, p: impl Into<PathBuf>) {
        self.bin_paths.insert(p.into());
    }

    pub fn add_install_root(&mut self, p: impl Into<PathBuf>) {
        self.install_roots.push(p.into());
    }

    pub fn claims_bin(&self, p: &Path) -> bool {
        self.bin_paths.contains(p)
    }

    pub fn claims_canonical(&self, canonical: &Path) -> bool {
        self.install_roots
            .iter()
            .any(|root| canonical.starts_with(root))
    }

    pub fn into_parts(self) -> ClaimedPartsOwned {
        ClaimedPartsOwned {
            bin_paths: self.bin_paths,
            install_roots: self.install_roots,
        }
    }
}

pub struct ClaimedPartsOwned {
    pub bin_paths: HashSet<PathBuf>,
    pub install_roots: Vec<PathBuf>,
}
