use std::path::{Component, Path, PathBuf};

use ignore::gitignore::Gitignore;

pub fn is_git_internal_path(path: &Path) -> bool {
    path.components().any(|component| {
        if let Component::Normal(name) = component {
            name == ".git"
        } else {
            false
        }
    })
}

/// Returns true if a path matches any of the gitignores.
///
/// For example, if the directory `/target` is ignored:
/// - If `check_ancestors` is true, then `/target/debug` will match.
/// - If `check_ancestors` is false, then `/target/debug` will not match.
pub fn matches_gitignores(
    path: &Path,
    is_dir: bool,
    gitignores: &[Gitignore],
    check_ancestors: bool,
) -> bool {
    gitignores.iter().any(|gitignore| {
        if let Ok(relative_path) = path.strip_prefix(gitignore.path()) {
            // `matched_path_or_any_parents` panics if the path has a root.
            // If not on windows, we allow paths with a root if the gitignore path is empty (since this denotes a global gitignore).
            if relative_path.has_root() && (cfg!(windows) || gitignore.path() != Path::new("")) {
                return false;
            }

            if check_ancestors {
                gitignore
                    .matched_path_or_any_parents(relative_path, is_dir)
                    .is_ignore()
            } else {
                gitignore.matched(relative_path, is_dir).is_ignore()
            }
        } else {
            false
        }
    })
}

/// Returns the path components after `.git` in a git-internal path,
/// skipping the worktree indirection (`.git/worktrees/<name>/…`) if present.
/// Returns `None` if the path has no `.git` component or nothing follows it.
fn git_suffix_components(path: &Path) -> Option<Vec<Component<'_>>> {
    let components: Vec<_> = path.components().collect();
    let git_index = components.iter().position(|c| c.as_os_str() == ".git")?;

    let after_git = &components[git_index + 1..];
    if after_git.is_empty() {
        return None;
    }

    // For worktrees the layout is `.git/worktrees/<name>/…`.
    // Skip the `worktrees/<name>` prefix so callers see the same
    // logical structure as a normal repo.
    if after_git.first().map(|c| c.as_os_str()) == Some(std::ffi::OsStr::new("worktrees"))
        && after_git.len() >= 3
    {
        // after_git[0] = "worktrees", [1] = <name>, [2..] = actual content
        return Some(after_git[2..].to_vec());
    }

    Some(after_git.to_vec())
}

/// Given a path like `.../repo/.git/worktrees/foo/HEAD`, returns
/// `.../repo/.git/worktrees/foo`. Returns `None` for non-worktree paths.
pub(crate) fn extract_worktree_git_dir(path: &Path) -> Option<PathBuf> {
    let components: Vec<_> = path.components().collect();
    let git_index = components.iter().position(|c| c.as_os_str() == ".git")?;
    let after_git = &components[git_index + 1..];
    if after_git.len() >= 3
        && after_git
            .first()
            .map(|c| c.as_os_str() == "worktrees")
            .unwrap_or(false)
    {
        // Rebuild: everything up to and including .git/worktrees/<name>
        Some(components[..git_index + 3].iter().collect())
    } else {
        None
    }
}

/// Returns `true` for shared ref paths that live directly in the common
/// `.git` directory and should be broadcast to all repos sharing it.
/// Currently this means `.git/refs/heads/*` (not under `.git/worktrees/`).
pub(crate) fn is_shared_git_ref(path: &Path) -> bool {
    if extract_worktree_git_dir(path).is_some() {
        return false;
    }
    let components: Vec<_> = path.components().collect();
    let Some(git_index) = components.iter().position(|c| c.as_os_str() == ".git") else {
        return false;
    };
    let after_git = &components[git_index + 1..];
    after_git
        .first()
        .map(|c| c.as_os_str() == "refs")
        .unwrap_or(false)
        && after_git
            .get(1)
            .map(|c| c.as_os_str() == "heads")
            .unwrap_or(false)
}

/// Returns true for `.git/HEAD` and `.git/refs/heads/*`
/// (and their worktree equivalents `.git/worktrees/*/HEAD`, etc.).
pub(crate) fn is_commit_related_git_file(path: &Path) -> bool {
    let Some(suffix) = git_suffix_components(path) else {
        return false;
    };
    match suffix.first().map(|c| c.as_os_str()) {
        Some(name) if name == "HEAD" => true,
        Some(name) if name == "refs" => {
            suffix.get(1).map(|c| c.as_os_str()) == Some(std::ffi::OsStr::new("heads"))
        }
        _ => false,
    }
}

/// Returns true for `.git/index.lock`
/// (and its worktree equivalent `.git/worktrees/*/index.lock`).
pub(crate) fn is_index_lock_file(path: &Path) -> bool {
    let Some(suffix) = git_suffix_components(path) else {
        return false;
    };
    suffix.len() == 1 && suffix[0].as_os_str() == "index.lock"
}

/// Determines if a git-related path should be ignored by the filesystem watcher.
///
/// Uses an allowlist approach: only commit-related files (HEAD, refs/heads/*)
/// and the index lock file are allowed through. Everything else inside `.git/`
/// is ignored.
pub fn should_ignore_git_path(path: &Path) -> bool {
    if !is_git_internal_path(path) {
        return false; // Not a git path, don't ignore
    }
    // Ignore everything inside .git/ except the allowlisted patterns.
    !is_commit_related_git_file(path) && !is_index_lock_file(path)
}

pub fn gitignores_for_directory(directory_path: &Path) -> Vec<Gitignore> {
    let mut gitignores = Vec::new();
    let gitignore_path = directory_path.join(".gitignore");
    if gitignore_path.exists() {
        let (gitignore, _) = Gitignore::new(&gitignore_path);
        gitignores.push(gitignore);
    }
    let (global_gitignore, _) = Gitignore::global();
    if !global_gitignore.is_empty() {
        gitignores.push(global_gitignore);
    }
    gitignores
}

#[cfg(test)]
#[path = "git_paths_tests.rs"]
mod tests;
