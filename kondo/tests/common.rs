use std::{env, path::PathBuf};

// Enables us to get a handle on the recently build kondo binary.
// When running tests, our executable is in the target directory.
// This returns the PathBuf to the kondo binary in the same target directory.
pub fn bin() -> PathBuf {
    let current_exe = env::current_exe().unwrap();
    let parent = current_exe.parent().unwrap();

    println!("root: {:?}", current_exe);
    let path = parent.join("../kondo");

    if !path.is_file() {
        panic!("kondo binary not found at {:?}", path);
    }
    path
}

// Enables us to get a temporary directory that contains a copy of test data
// specified by `scenario`.
pub fn with_temp_dir_from<F>(scenario: String, f: F)
where
    F: FnOnce(PathBuf),
{
    let tmp_dir = get_copy_of_test_data_as_temp_dir(scenario);
    f(tmp_dir.path().to_path_buf());
    tmp_dir.close().unwrap();
}

pub fn with_cache_at<F>(cache: PathBuf, f: F)
where
    F: FnOnce(PathBuf),
{
    assert!(cache.exists(), "cache must exist before running kondo");
    f(cache.clone());
    assert!(!cache.exists(), "cache must be deleted after running kondo");
}

// Given `scenario` in the `test_data` directory, this returns a temporary
// directory that contains a copy of the scenario's data.
pub fn get_copy_of_test_data_as_temp_dir(scenario: String) -> tempfile::TempDir {
    extern crate fs_extra;
    let options = fs_extra::dir::CopyOptions::new();

    let project_directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_directory = project_directory.join("../test_data");
    let scenario_directory = data_directory.join(scenario.clone());

    if !scenario_directory.exists() {
        panic!(
            "scenario {:?} does not exist: {:?}. Create it in src or you have a typo",
            scenario.clone().to_string(),
            scenario_directory
        );
    }
    let from_paths = vec![scenario_directory];

    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    fs_extra::copy_items(&from_paths, &tmp_dir, &options).unwrap();

    tmp_dir
}
