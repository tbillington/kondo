mod common;
use common::with_temp_dir_from;
use std::{fs, process::Command};

use crate::common::with_cache_at;

#[test]
fn test_version() {
    let bin = common::bin();

    let output = Command::new(bin)
        .arg("--")
        .arg("--version")
        .output()
        .expect("failed to execute process");
    assert!(output.status.success());
}

#[test]
fn test_cli_run_kondo_all_in_python_project() {
    let scenario = "scenario_a".to_string();
    with_temp_dir_from(scenario.clone(), |tmpdir| {
        let bin = common::bin();
        println!("tmpdr: {:?}", tmpdir);

        with_cache_at(
            tmpdir
                .clone()
                .join(scenario.clone())
                .join("python-project-a")
                .join("__pycache__")
                .join("1"),
            |cache| {
                // run kondo --all in the temp dir
                let mut cmd = Command::new(bin);

                let cmd_w_args = cmd
                    .arg(tmpdir.join(scenario.clone())) // note the path tmp/scenario-name
                    .arg("--all");
                print!("cmd_w_args: {:?}", cmd_w_args);
                let output = cmd_w_args.output().unwrap();

                assert!(output.status.success(), "failed to run kondo");
                println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            },
        );
    });
}

#[ignore = "failing unexpectedly. should work, but doesn't see the project unless we specify the right top dir as in the test above. I'd expect kondo to recurse into the scenario directory. "]
#[test]
fn test_cli_run_kondo_all_above_project_fails() {
    let scenario = "scenario_a".to_string();
    with_temp_dir_from(scenario.clone(), |tmpdir| {
        let bin = common::bin();
        println!("tmpdr: {:?}", tmpdir.clone());

        with_cache_at(
            tmpdir
                .clone()
                .join(scenario.clone())
                .join("python-project-a")
                .join("__pycache__")
                .join("1"),
            |cache| {
                // run kondo --all in the temp dir
                let mut cmd = Command::new(bin);

                let cmd_w_args = cmd
                    .arg(tmpdir.clone()) // here note the path, just tmpdir
                    .arg("--all");

                print!("cmd_w_args: {:?}", cmd_w_args);
                let output = cmd_w_args.output().unwrap();

                assert!(output.status.success(), "failed to run kondo");
                println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            },
        );
    });
}

#[ignore = "nested projects not working yet"]
#[test]
fn test_cli_run_kondo_scenario_nested_a() {
    let scenario = "scenario_nested_a".to_string();
    with_temp_dir_from(scenario.clone(), |tmpdir| {
        let bin = common::bin();
        println!("tmpdr: {:?}", tmpdir.clone());

        assert!(
            tmpdir
                .clone()
                .join(scenario.clone())
                .join("python-project-a")
                .join("__pycache__")
                .join("1")
                .exists(),
            "cache must exist before running kondo"
        );

        // run kondo --all in the temp dir
        let mut cmd = Command::new(bin);

        let cmd_w_args = cmd.arg(tmpdir.join(scenario.clone())).arg("--all");
        let output = cmd_w_args.output().unwrap();

        assert!(output.status.success(), "failed to run kondo");
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

        assert!(
            !tmpdir
                .clone()
                .join(scenario.clone())
                .join("python-project-a")
                .join("__pycache__")
                .join("1")
                .exists(),
            "cache ought to be deleted after running kondo"
        );

        assert!(
            !tmpdir
                .clone()
                .join(scenario.clone())
                .join("python-project-a")
                .join("sub-project")
                .join("python-project-b")
                .join("__pycache__")
                .join("cache.data")
                .exists(),
            "cache in nested project ought to be deleted after running kondo"
        );
    });
}

#[test]
fn play_a() {
    let scenario = "scenario_a".to_string();
    with_temp_dir_from(scenario.clone(), |path| {
        println!("path: {:?}", path);

        let paths = fs::read_dir(path.clone()).unwrap();
        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }

        assert!(path.exists(), "dir ought to exist");

        let paths = fs::read_dir(path.clone().join(scenario.clone())).unwrap();
        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }
        assert!(path.join(scenario.clone()).exists(), "dir ought to exist");
    });
}

#[test]
#[ignore = "bug: --ignored-dirs that don't exist cause a failure"]
fn non_extant_ignore_dirs_work() {
    let scenario = "scenario_nested_a".to_string();
    with_temp_dir_from(scenario.clone(), |tmpdir| {
        let bin = common::bin();

        // run kondo
        let mut cmd = Command::new(bin);
        let cmd_w_args = cmd
            .arg(tmpdir.join(scenario.clone()))
            .arg(tmpdir.clone())
            .arg("--ignored-dirs=doesnotexist")
            .arg("--all");
        let output = cmd_w_args.output().unwrap();
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        assert!(output.status.success(), "failed to run kondo");
    });
}
