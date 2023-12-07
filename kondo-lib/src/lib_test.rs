#[cfg(test)]
mod test {

    use crate::{Project, ProjectType, ScanOptions};
    use std::{io::Write, path::PathBuf};

    // Given test data, clean should remove some files
    #[test]
    fn test_clean() {
        let scan_options: ScanOptions = ScanOptions {
            follow_symlinks: false,
            same_file_system: true,
        };

        let tempdir = create_fake_python_project("test_data".to_string());
        let path = tempdir.path().join("test_data");

        println!("path: {:?}", path);
        println!("tempdir: {:?}", tempdir.path());

        let project_a = Project {
            path,
            project_type: ProjectType::Python,
        };

        assert!(
            project_a.size(&scan_options) > 0,
            "size of project ought to be greater than 0"
        );
        assert!(project_a.path.exists(), "project ought to exist");

        // Run clean and check before and after that file exists and is deleted
        assert!(
            project_a.path.join("__pycache__/cache.data").exists(),
            "cache file ought to exist"
        );
        Project::clean(&project_a);
        assert!(
            !project_a.path.join("__pycache__/cache.data").exists(),
            "cache file should have been deleted"
        );

        assert!(project_a.path.exists(), "project ought to still exist");

        // clean up
        tempdir.close().unwrap();
    }

    // #[ignore = "this is probably "]
    #[test]
    fn test_clean_nested_python_projects() {
        // make alpha project
        let alpha_tmp_dir = create_fake_python_project("alpha".to_string());

        // inside of alpha, make nested project
        let project_nested_dir = create_fake_python_project_in_dir(
            alpha_tmp_dir.path().clone().to_path_buf(),
            "nested".to_string(),
        );

        // Given alpha project
        let project_alpha = Project {
            path: alpha_tmp_dir.into_path(),
            project_type: ProjectType::Python,
        };
        // and nested project
        let project_nested = Project {
            path: project_nested_dir.clone(),
            project_type: ProjectType::Python,
        };

        // Clean!
        Project::clean(&project_alpha);
        Project::clean(&project_nested);
        // Both project dirs exist
        assert!(
            project_alpha.path.exists(),
            "project alpha ought to still exist"
        );
        assert!(
            project_nested_dir.exists(),
            "nested project ought to still exist"
        );

        // Both cache files are gone
        assert!(
            !project_alpha.path.join("__pycache__/cache.data").exists(),
            "cache file of alpha should have been deleted"
        );
        assert!(
            !project_nested_dir.join("__pycache__/cache.data").exists(),
            "cache file of nested project should have been deleted"
        );
    }
    // TODO: this code is duplicated at konod/src/main.rs
    // Given a name, create a new simulated python project in a safe to delete directry
    pub fn create_fake_python_project(name: String) -> tempfile::TempDir {
        // Make a new project in a temporary directory
        let tmp_dir = tempfile::tempdir().unwrap();
        create_fake_python_project_in_dir(tmp_dir.path().to_path_buf(), name);
        tmp_dir
    }

    pub fn create_fake_python_project_in_dir(dir: PathBuf, name: String) -> PathBuf {
        // make a new root in the dir
        let project_dir = dir.join(name);
        std::fs::create_dir_all(&project_dir).unwrap();

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
        project_dir.to_path_buf()
    }
}
