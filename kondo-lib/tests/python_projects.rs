use kondo_lib::{clean, scan, Project, ScanOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const PYTHON_ARTIFACT_DIRS: [&str; 7] = [
    ".mypy_cache",
    ".nox",
    ".pytest_cache",
    ".ruff_cache",
    ".tox",
    "__pycache__",
    "__pypackages__",
];
const PYTHON_MARKERS: [(&str, &str); 3] = [
    ("uv.lock", "version = 1\n"),
    ("pyproject.toml", "[tool.uv]\nmanaged = true\n"),
    ("pyproject.toml", "[project]\nname = \"example\"\n"),
];
const CACHE_FILES: [(&str, &[u8]); 7] = [
    (".mypy_cache", b"mypy"),
    (".nox", b"nox-cache"),
    (".pytest_cache", b"pytest-cache"),
    (".ruff_cache", b"ruff"),
    (".tox", b"tox-cache"),
    ("__pycache__", b"bytecode"),
    ("__pypackages__", b"packages"),
];

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_nanos();
        for attempt in 0..100 {
            let sequence = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "kondo-python-{}-{timestamp}-{sequence}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory {path:?}: {error}"),
            }
        }
        panic!("could not create a unique fixture directory");
    }

    fn write(&self, relative: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(path, contents).expect("write fixture file");
    }

    fn directory(&self, relative: impl AsRef<Path>) {
        fs::create_dir_all(self.path.join(relative)).expect("create fixture directory");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!("error removing test fixture {:?}: {error}", self.path);
        }
    }
}

fn options() -> ScanOptions {
    ScanOptions {
        follow_symlinks: false,
        same_file_system: false,
    }
}

fn projects(path: &Path) -> Vec<Project> {
    scan(&path, &options())
        .map(|result| match result {
            Ok(project) => project,
            Err(_) => panic!("scan fixture without filesystem errors"),
        })
        .collect()
}

fn assert_python_project(found: &[Project], path: &Path) {
    assert_eq!(found.len(), 1, "one project should be emitted");
    assert_eq!(found[0].type_name(), "Python");
    assert_eq!(found[0].path, path);
    assert_eq!(found[0].artifact_dirs(), PYTHON_ARTIFACT_DIRS);
}

fn populate(fixture: &Fixture, marker: &str, marker_contents: &str) -> u64 {
    fixture.write(marker, marker_contents);
    fixture.write("src/main.py", b"print('kept')\n");
    fixture.write(".venv/pyvenv.cfg", b"home = /usr/bin\n");
    CACHE_FILES
        .into_iter()
        .map(|(directory, contents)| {
            fixture.write(Path::new(directory).join("entry.bin"), contents);
            contents.len() as u64
        })
        .sum()
}

fn assert_removed(fixture: &Fixture) {
    for directory in PYTHON_ARTIFACT_DIRS {
        assert!(
            !fixture.path.join(directory).exists(),
            "{directory} remains"
        );
    }
}

fn assert_preserved(fixture: &Fixture, marker: &str, contents: &str) {
    assert_eq!(
        fs::read(fixture.path.join(marker)).unwrap(),
        contents.as_bytes()
    );
    assert_eq!(
        fs::read(fixture.path.join("src/main.py")).unwrap(),
        b"print('kept')\n"
    );
    assert_eq!(
        fs::read(fixture.path.join(".venv/pyvenv.cfg")).unwrap(),
        b"home = /usr/bin\n"
    );
}

#[test]
fn uv_markers_detect_python_projects_without_root_python_files() {
    for (marker, contents) in PYTHON_MARKERS {
        let fixture = Fixture::new();
        populate(&fixture, marker, contents);
        assert_python_project(&projects(&fixture.path), &fixture.path);
    }

    let fixture = Fixture::new();
    fixture.write("uv.lock", "version = 1\n");
    fixture.write("pyproject.toml", "[tool.uv]\nmanaged = true\n");
    fixture.write("main.py", b"print('legacy')\n");
    assert_python_project(&projects(&fixture.path), &fixture.path);
}

#[test]
fn python_caches_are_sized_and_removed_by_both_cleanup_apis() {
    for (marker, contents) in PYTHON_MARKERS {
        let fixture = Fixture::new();
        let expected_size = populate(&fixture, marker, contents);
        let found = projects(&fixture.path);
        assert_python_project(&found, &fixture.path);
        let project = &found[0];
        assert_eq!(project.size(&options()), expected_size);
        assert_eq!(project.size_dirs(&options()).artifact_size, expected_size);
        project.clean();
        assert_removed(&fixture);
        assert_eq!(project.size(&options()), 0);
        assert_preserved(&fixture, marker, contents);
    }

    for (marker, contents) in PYTHON_MARKERS {
        let fixture = Fixture::new();
        populate(&fixture, marker, contents);
        clean(fixture.path.to_str().unwrap()).expect("public clean succeeds");
        assert_removed(&fixture);
        assert_preserved(&fixture, marker, contents);
    }
}

#[test]
fn legacy_python_files_still_scan() {
    let fixture = Fixture::new();
    fixture.write("main.py", b"print('kept')\n");
    assert_python_project(&projects(&fixture.path), &fixture.path);
}

#[test]
fn marker_directories_are_not_projects_or_clean_targets() {
    for marker in ["uv.lock", "pyproject.toml"] {
        let fixture = Fixture::new();
        fixture.directory(marker);
        fixture.write(".pytest_cache/entry.bin", b"cache");

        assert!(projects(&fixture.path).is_empty());
        clean(fixture.path.to_str().unwrap()).expect("public clean succeeds");
        assert!(fixture.path.join(".pytest_cache").exists());
    }
}
