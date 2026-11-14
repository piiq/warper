//! Git statistics and branch metadata shared by terminal chips and branch selection.
//!
//! Some of this code is adapted from GitHub Desktop, licensed under the MIT license.
//! Copyright (c) GitHub, Inc. See GITHUB-DESKTOP-LICENSE in this directory.

use crate::util::git::{detect_main_branch, run_git_command};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
};
use warp_core::safe_warn;

#[derive(Clone, Debug, PartialEq)]
enum GitFileStatus {
    New,
    Modified,
    Deleted,
    Renamed { old_path: String },
    Copied { old_path: String },
    Untracked,
    Conflicted,
}

impl TryFrom<&str> for GitFileStatus {
    type Error = anyhow::Error;

    fn try_from(status_code: &str) -> Result<Self> {
        match status_code {
            ".M" | "M." | "MM" => Ok(GitFileStatus::Modified),
            ".A" | "A." | "AM" => Ok(GitFileStatus::New),
            ".D" | "D." | "AD" => Ok(GitFileStatus::Deleted),
            _ => Ok(GitFileStatus::Modified), // Default fallback
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DiffStats {
    pub files_changed: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
}

struct GitNumStatMetadata {
    lines_added: usize,
    lines_removed: usize,
}

pub struct GitMetadata;

impl GitMetadata {
    /// Returns the number of lines in a given file. Returns `None` if the file is a binary file
    /// or if it's larger than 20MB.
    async fn num_lines_in_file_if_non_binary(file_path: &Path) -> Result<Option<usize>> {
        const MAX_FILE_SIZE_TO_READ_LINES: u64 = 20 * 1000 * 1000; // 20MB

        // Size of the chunk of file to read to determine whether it's binary.
        const CHUNK_SIZE: usize = 1024;

        let mut file = File::open(file_path)?;
        let mut buffer = vec![0; CHUNK_SIZE];

        let n = file.read(&mut buffer)?;
        buffer.truncate(n);

        if warp_util::file_type::is_buffer_binary(&buffer) {
            return Ok(None);
        }
        file.rewind()?;

        if file.metadata()?.len() > MAX_FILE_SIZE_TO_READ_LINES {
            return Ok(None);
        }

        Self::num_lines_in_file(file).await.map(Some)
    }

    /// Returns the number of lines in a file. This is optimized to avoid loading the entire file into memory.
    async fn num_lines_in_file(file: File) -> Result<usize> {
        // Read in chunks of 64 KBs.
        const CHUNK_SIZE: usize = 1024 * 64;

        let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
        let mut count = 0;
        loop {
            let len = {
                let buf = reader.fill_buf()?;
                if buf.is_empty() {
                    break;
                }
                count += bytecount::count(buf, b'\n');
                buf.len()
            };
            reader.consume(len);
            // Yield so that an attempt to abort this operation is handled.
            futures_lite::future::yield_now().await;
        }
        Ok(count)
    }

    pub async fn diff_metadata_against_head(repo_path: &Path) -> Result<DiffStats> {
        // First, get the list of changed files with their status
        log::debug!(
            "[GIT OPERATION] metadata.rs diff_metadata_against_head git --no-optional-locks status --untracked-files=all --branch --porcelain=2 -z"
        );
        let status_output = run_git_command(
            repo_path,
            &[
                "--no-optional-locks", // Avoid taking locks that might interfere with other git operations
                "status",
                "--untracked-files=all", // Get all untracked files
                "--branch",              // Get branch information
                "--porcelain=2",         // Use porcelain=2 to match git desktop implementation
                "-z",                    // Split output by null characters
            ],
        )
        .await?;

        let changed_files = Self::parse_git_status(&status_output)?;
        let num_stat_metadata = Self::get_diff_metadata_using_numstat(repo_path, "HEAD").await?;

        let mut total_additions = 0;
        let mut total_deletions = 0;

        for (file_path, status) in &changed_files {
            if let Some(metadata) = num_stat_metadata.get(file_path) {
                total_additions += metadata.lines_added;
                total_deletions += metadata.lines_removed;
            } else if matches!(status, GitFileStatus::Untracked) {
                let num_lines =
                    Self::num_lines_in_file_if_non_binary(&repo_path.join(file_path)).await?;
                total_additions += num_lines.unwrap_or(0);
            }
        }

        Ok(DiffStats {
            files_changed: changed_files.len(),
            total_additions,
            total_deletions,
        })
    }

    /// Gets git branches, sorted by commit date (most recent first)
    /// Returns a list of (branch_name, is_main_branch) tuples
    /// Defaults to the most recent 100 branches for performance
    pub async fn get_all_branches(
        repo_path: &Path,
        max_branch_count: Option<usize>,
        include_remotes: bool,
    ) -> Result<Vec<(String, bool)>> {
        let main_branch = match detect_main_branch(repo_path).await {
            Ok(branch) => branch,
            Err(err) => {
                log::warn!("Failed to detect main branch: {err}");
                "origin/main".to_string()
            }
        };
        Self::fetch_branch_list_with_main(
            repo_path,
            &main_branch,
            max_branch_count,
            include_remotes,
        )
        .await
    }

    /// Like [`Self::get_all_branches`] but with a pre-known main branch, skipping
    /// [`detect_main_branch`].
    ///
    /// Use this when the main branch is already cached from a previous call to avoid
    /// the up-to-6 sequential subprocess calls that detection may require.
    pub async fn get_all_branches_with_known_main(
        repo_path: &Path,
        main_branch: &str,
        max_branch_count: Option<usize>,
        include_remotes: bool,
    ) -> Result<Vec<(String, bool)>> {
        Self::fetch_branch_list_with_main(repo_path, main_branch, max_branch_count, include_remotes)
            .await
    }

    /// Shared implementation for [`Self::get_all_branches`] and
    /// [`Self::get_all_branches_with_known_main`]. Runs `git for-each-ref` and
    /// marks each branch as main or not based on the supplied `main_branch` string.
    async fn fetch_branch_list_with_main(
        repo_path: &Path,
        main_branch: &str,
        max_branch_count: Option<usize>,
        include_remotes: bool,
    ) -> Result<Vec<(String, bool)>> {
        let count_arg = format!("--count={}", max_branch_count.unwrap_or(100));

        let mut args = vec![
            "for-each-ref",
            count_arg.as_str(),
            "--sort=-committerdate",
            "--format=%(refname:short)",
            "refs/heads",
        ];

        if include_remotes {
            args.push("refs/remotes");
        }
        // Get branches sorted by commit date, limited to 100 most recent for performance
        // Using git's --count option to limit at the git command level for efficiency
        log::debug!(
            "[GIT OPERATION] metadata.rs get_all_branches git {}",
            args.join(" ")
        );
        let output = run_git_command(repo_path, args.as_slice()).await?;

        let mut branches = Vec::new();

        for branch in output.lines() {
            let branch = branch.trim();
            if branch.is_empty() {
                continue;
            }

            // Skip HEAD pointer and detached HEAD states
            if branch.contains("HEAD") || branch.starts_with("(") {
                continue;
            }

            let is_main =
                branch == main_branch || branch == main_branch.trim_start_matches("origin/");
            branches.push((branch.to_string(), is_main));
        }

        // Remove duplicates while preserving order (most recent first)
        let mut seen = std::collections::HashSet::new();
        branches.retain(|(name, _)| seen.insert(name.clone()));

        if branches.is_empty() {
            safe_warn!(
                safe: ("Git metadata: get_all_branches returned empty list"),
                full: ("Git metadata: get_all_branches returned empty list for repo: {:?}", repo_path)
            );
        }

        Ok(branches)
    }

    /// Returns an iterator over `branches` with main branches (`is_main == true`) first,
    /// then the rest in their existing order.
    ///
    /// `get_all_branches` already returns branches sorted by recency; this helper
    /// promotes the main branch to the front while preserving that recency order
    /// for remaining entries.
    pub fn sort_branches_main_first(
        branches: &[(String, bool)],
    ) -> impl Iterator<Item = &(String, bool)> {
        branches
            .iter()
            .filter(|(_, is_main)| *is_main)
            .chain(branches.iter().filter(|(_, is_main)| !is_main))
    }

    /// Parses git status output to get changed files and their status
    /// This handles porcelain=2 format to match git desktop implementation
    fn parse_git_status(status_output: &str) -> Result<Vec<(PathBuf, GitFileStatus)>> {
        if status_output.is_empty() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        let tokens: Vec<&str> = status_output.split('\0').collect();

        let mut i = 0;
        while i < tokens.len() {
            let token = tokens[i];

            if token.is_empty() {
                i += 1;
                continue;
            }

            // Skip header lines that start with '#'
            if token.starts_with("# ") {
                i += 1;
                continue;
            }

            let entry_kind = token.chars().next().unwrap_or('?');

            match entry_kind {
                '1' => {
                    // Changed entry: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                    // Use splitn to preserve spaces in filenames (the path is the last field).
                    let parts: Vec<&str> = token.splitn(9, ' ').collect();
                    if parts.len() >= 9 {
                        let status_code = parts[1];
                        let path = parts[8];

                        let status = Self::map_status_code(status_code).map_err(|e| {
                            anyhow!(
                                "Invalid status code '{}' for path '{}': {}",
                                status_code,
                                path,
                                e
                            )
                        })?;
                        files.push((PathBuf::from(path), status));
                    } else {
                        log::warn!("Invalid format for changed entry: '{token}' - expected at least 9 parts, got {}", parts.len());
                    }
                }
                '2' => {
                    // Renamed/copied entry: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
                    // Use splitn to preserve spaces in filenames (the path is the last field).
                    let parts: Vec<&str> = token.splitn(10, ' ').collect();
                    if parts.len() >= 10 {
                        let status_code = parts[1];
                        let path = parts[9];

                        // Get the old path from the next token
                        let old_path = if i + 1 < tokens.len() {
                            tokens[i + 1].to_string()
                        } else {
                            log::warn!("Missing old path for renamed/copied entry: '{token}'");
                            String::new()
                        };

                        let status = if status_code.starts_with('R') {
                            GitFileStatus::Renamed { old_path }
                        } else if status_code.starts_with('C') {
                            GitFileStatus::Copied { old_path }
                        } else {
                            Self::map_status_code(status_code).map_err(|e| {
                                anyhow!(
                                    "Invalid status code '{}' for renamed/copied path '{}': {}",
                                    status_code,
                                    path,
                                    e
                                )
                            })?
                        };

                        files.push((PathBuf::from(path), status));
                        i += 1; // Skip the old path token
                    } else {
                        log::warn!("Invalid format for renamed/copied entry: '{token}' - expected at least 10 parts, got {}", parts.len());
                    }
                }
                'u' => {
                    // Unmerged entry: u <xy> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
                    // Use splitn to preserve spaces in filenames (the path is the last field).
                    let parts: Vec<&str> = token.splitn(11, ' ').collect();
                    if parts.len() >= 11 {
                        let path = parts[10];
                        files.push((PathBuf::from(path), GitFileStatus::Conflicted));
                    } else {
                        log::warn!("Invalid format for unmerged entry: '{}' - expected at least 11 parts, got {}", token, parts.len());
                    }
                }
                '?' => {
                    // Untracked entry: ? <path>
                    if token.len() > 2 {
                        let path = &token[2..]; // Skip "? "
                        files.push((PathBuf::from(path), GitFileStatus::Untracked));
                    } else {
                        log::warn!("Invalid format for untracked entry: '{token}' - expected path after '? '");
                    }
                }
                '!' => {
                    // Ignored entry: ! <path>
                    // We skip ignored files
                }
                _ => {
                    // Unknown entry type, skip
                    log::debug!("Unknown git status entry type '{entry_kind}' in token: '{token}'");
                }
            }

            i += 1;
        }

        Ok(files)
    }

    /// Get binary files using git diff --numstat against a specific commit
    async fn get_diff_metadata_using_numstat(
        repo_path: &Path,
        commit: &str,
    ) -> Result<HashMap<PathBuf, GitNumStatMetadata>> {
        log::debug!(
            "[GIT OPERATION] metadata.rs get_diff_metadata_using_numstat git diff --numstat {commit}"
        );
        let numstat_output = match run_git_command(repo_path, &["diff", "--numstat", commit]).await
        {
            Ok(output) => output,
            Err(_) => {
                // If numstat fails, return empty set
                return Ok(HashMap::new());
            }
        };

        let mut diff_metadata = std::collections::HashMap::new();

        for line in numstat_output.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let additions = parts[0];
                let deletions = parts[1];
                let filename = parts[2];

                let metadata = GitNumStatMetadata {
                    lines_added: additions.parse().unwrap_or(0),
                    lines_removed: deletions.parse().unwrap_or(0),
                };

                diff_metadata.insert(PathBuf::from(filename), metadata);
            }
        }

        Ok(diff_metadata)
    }

    /// Maps git status codes to GitFileStatus
    fn map_status_code(status_code: &str) -> Result<GitFileStatus> {
        GitFileStatus::try_from(status_code)
    }
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
