use std::{
    fs,
    path::{Path, PathBuf},
};

use kondo_lib::{clean, scan, Project, ScanOptions};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("kondo-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn options() -> ScanOptions {
    ScanOptions {
        follow_symlinks: false,
        same_file_system: true,
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn platformio_project(dir: &Path) {
    write(
        &dir.join("platformio.ini"),
        "[env:esp32]\nplatform = espressif32\n",
    );
    write(&dir.join("src/main.cpp"), "int main() {}\n");
    write(&dir.join(".pio/build/esp32/firmware.bin"), "firmware");
    write(&dir.join(".pio/libdeps/esp32/lib/lib.h"), "// lib");
}

fn scan_one(root: &Path) -> Project {
    let projects: Vec<Project> = scan(&root, &options()).filter_map(Result::ok).collect();
    assert_eq!(projects.len(), 1, "{projects:?}");
    projects.into_iter().next().unwrap()
}

#[test]
fn scan_detects_platformio_project() {
    let tmp = TempDir::new("pio-detect");
    let dir = tmp.0.join("firmware");
    platformio_project(&dir);

    let project = scan_one(&tmp.0);
    assert_eq!(project.path, dir);
    assert_eq!(project.type_name(), "PlatformIO");
    assert_eq!(project.artifact_dirs(), [".pio"]);
    assert!(project.size(&options()) > 0);
}

#[test]
fn platformio_takes_precedence_over_cmake() {
    let tmp = TempDir::new("pio-cmake");
    let dir = tmp.0.join("firmware");
    platformio_project(&dir);
    write(
        &dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.16)\n",
    );

    assert_eq!(scan_one(&tmp.0).type_name(), "PlatformIO");
}

#[test]
fn project_clean_removes_pio_dir() {
    let tmp = TempDir::new("pio-project-clean");
    platformio_project(&tmp.0);

    scan_one(&tmp.0).clean();
    assert!(!tmp.0.join(".pio").exists());
    assert!(tmp.0.join("platformio.ini").exists());
    assert!(tmp.0.join("src/main.cpp").exists());
}

#[test]
fn clean_path_removes_pio_dir() {
    let tmp = TempDir::new("pio-path-clean");
    platformio_project(&tmp.0);
    write(
        &tmp.0.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.16)\n",
    );

    clean(tmp.0.to_str().unwrap()).unwrap();
    assert!(!tmp.0.join(".pio").exists());
    assert!(tmp.0.join("src/main.cpp").exists());
}
