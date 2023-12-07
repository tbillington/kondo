#[cfg(test)]
mod test {
    use crate::{discover, prepare_directories, DiscoverData};
    use kondo_lib::ScanOptions;
    use std::io::Write;

    #[test]
    fn test_prepare_directories_that_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test_directory");

        // Given a directory that exists
        std::fs::create_dir(&path).unwrap();

        // When we prepare the directories
        let dirs = prepare_directories(vec![path.clone()]).unwrap();

        // Then we ought to get back the same directory
        assert_eq!(dirs[0], path);

        // clean up
        tempdir.close().unwrap();
    }
    #[test]
    fn test_prepare_directories_that_do_not_exist() {
        let tempdir = tempfile::tempdir().unwrap(); // is created
        let path = tempdir.path().join("test_directory"); // is not created

        // Given a directory that DOES NOT exist
        // When we prepare the directories
        let dirs = prepare_directories(vec![path.clone()]).unwrap();

        // Then there is not directory
        assert_eq!(dirs.len(), 0);

        // clean up
        tempdir.close().unwrap();
    }

    #[test]
    fn test_discover() {
        let tempdir = create_fake_python_project("test_data".to_string());

        // and basic setup
        let scan_options: ScanOptions = ScanOptions {
            follow_symlinks: false,
            same_file_system: true,
        };
        let project_min_age = 0;
        let (result_sender, result_recv) = std::sync::mpsc::sync_channel::<DiscoverData>(5);
        let ignored_dirs = vec![];

        // discover ...
        discover(
            vec![tempdir.path().join("test_data").to_path_buf()],
            &scan_options,
            project_min_age,
            result_sender,
            &ignored_dirs,
        );

        let count = result_recv.try_iter().count();

        // ought to find the right number of projects
        assert_eq!(count, 1);

        // clean up
        tempdir.close().unwrap();
    }

    #[ignore = "does discover not recurse? this test is running discover a level above the project. it doesn't work. above does, at the same level."]
    #[test]
    fn test_discover_broken() {
        let tempdir = create_fake_python_project("test_data".to_string());

        // and basic setup
        let scan_options: ScanOptions = ScanOptions {
            follow_symlinks: false,
            same_file_system: true,
        };
        let project_min_age = 0;
        let (result_sender, result_recv) = std::sync::mpsc::sync_channel::<DiscoverData>(5);
        let ignored_dirs = vec![];

        // discover ...
        println!("discover in {:?}", tempdir.path());
        discover(
            vec![tempdir.path().to_path_buf()],
            &scan_options,
            project_min_age,
            result_sender,
            &ignored_dirs,
        );

        let count = result_recv.try_iter().count();

        // ought to find the right number of projects
        assert_eq!(count, 1);

        // clean up
        tempdir.close().unwrap();
    }

    // TODO: this code is duplicated at kondo-lib/src/lib_test.rs
    // Given a name, create a new simulated python project in a safe to delete directry
    pub fn create_fake_python_project(name: String) -> tempfile::TempDir {
        // Make a new project in a temporary directory
        let tmp_dir = tempfile::tempdir().unwrap();

        // make a new root in the tmp dir
        let project_dir = tmp_dir.path().join(name);
        std::fs::create_dir(&project_dir).unwrap();

        // Must have a directory to hold the project.
        let cache_dir = project_dir.join("__pycache__");
        std::fs::create_dir(&cache_dir).unwrap();

        // Must have data in the cache to delete
        let mut data_file = std::fs::File::create(cache_dir.join("cache.data")).unwrap();
        data_file.write_all(b"#oodles of cache')\n").unwrap();
        let mut data_file_b = std::fs::File::create(cache_dir.join("other.cache")).unwrap();
        data_file_b.write_all(b"#oodles of cache')\n").unwrap();

        // and a file of type .py to signal we're a python project
        let mut python_file = std::fs::File::create(project_dir.join("main.py")).unwrap();
        python_file
            .write_all(b"#!/bin/python\n\nprint('Hello, world!')\n")
            .unwrap();

        tmp_dir
    }
}
