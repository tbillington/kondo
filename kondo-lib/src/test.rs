use std::{io::Write, path::PathBuf};

pub(crate) fn id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ID: AtomicUsize = AtomicUsize::new(0);
    ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default)]
pub(crate) struct TestDirectoryBuilder {
    files: Vec<(String, String)>,
    artifacts: Vec<(String, String)>,
}

impl TestDirectoryBuilder {
    pub(crate) fn build(self) -> Result<TestDirectory, std::io::Error> {
        let root = std::env::temp_dir().join(format!("kondo-test-{}", id()));
        let _ = std::fs::remove_dir_all(&root);

        let create_files = |files: Vec<(String, String)>| -> Result<Vec<PathBuf>, std::io::Error> {
            files
                .into_iter()
                .map(|(f, content)| -> Result<PathBuf, std::io::Error> {
                    let f = root.join(f);
                    std::fs::create_dir_all(f.parent().unwrap())?;
                    std::fs::File::create(&f)?.write_all(content.as_bytes())?;
                    Ok(f)
                })
                .collect::<Result<Vec<_>, _>>()
        };

        let files = create_files(self.files)?;

        let artifacts = create_files(self.artifacts)?;

        Ok(TestDirectory {
            root,
            files,
            artifacts,
        })
    }

    pub(crate) fn file(mut self, path: &str) -> Self {
        self.files.push((path.to_string(), String::new()));
        self
    }

    pub(crate) fn file_content(mut self, path: &str, content: &str) -> Self {
        self.files.push((path.to_string(), content.to_string()));
        self
    }

    pub(crate) fn artifact(mut self, path: &str) -> Self {
        self.artifacts.push((path.to_string(), "test".to_string()));
        self
    }
}

pub(crate) struct TestDirectory {
    pub(crate) root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) artifacts: Vec<PathBuf>,
}

impl TestDirectory {
    // pub(crate) fn artifact_bytes(&self) -> usize {
    //     self.artifacts
    //         .iter()
    //         .map(|p| std::fs::metadata(p).unwrap().len() as usize)
    //         .sum()
    // }

    pub(crate) fn fully_clean(&self) -> bool {
        self.artifacts.iter().all(|p| !p.exists())
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.root) {
            eprintln!("failed cleaning up TestDirectory {:?}: {}", self.root, err);
        }
    }
}
