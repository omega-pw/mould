use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args([
            "log",
            "--date=format:%Y-%m-%d %H:%M:%S %z",
            "--pretty=format:Commit: %H %nDate: %cd %nRef: %D",
            "-n",
            "1",
        ])
        .output()
        .expect("Failed to get version infomation by executing process 'git log'!");
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Failed to get environment variable 'CARGO_MANIFEST_DIR'!");
    let version_file = format!("{}/version.txt", cargo_manifest_dir);
    if output.status.success() {
        fs::write(version_file, &output.stdout).expect("Write version.txt failed!");
    } else {
        panic!("Failed to get version infomation by executing process 'git log'!");
    }
}
