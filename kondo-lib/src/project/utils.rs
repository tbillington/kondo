use std::path::{Path, PathBuf};

pub(crate) fn filter_paths_exist<'a>(
    root: &'a Path,
    paths: &'a [&str],
) -> impl Iterator<Item = PathBuf> + 'a {
    paths.iter().filter_map(|p| {
        let path = root.join(p);
        path.exists().then_some(path)
    })
}
