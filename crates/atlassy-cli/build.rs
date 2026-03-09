use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    let git_commit_sha = run_git(&["rev-parse", "HEAD"], "git rev-parse HEAD");
    if git_commit_sha.len() != 40 || !git_commit_sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
        panic!(
            "`git rev-parse HEAD` returned malformed SHA `{git_commit_sha}`; expected 40 hex characters"
        );
    }

    let git_status = run_git(&["status", "--porcelain"], "git status --porcelain");
    let git_dirty = (!git_status.is_empty()).to_string();

    println!("cargo:rustc-env=GIT_COMMIT_SHA={git_commit_sha}");
    println!("cargo:rustc-env=GIT_DIRTY={git_dirty}");
}

fn run_git(args: &[&str], command_name: &str) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to execute `{command_name}` while embedding provenance: {error}")
        });

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stderr_detail = if stderr.is_empty() {
            "no stderr output".to_string()
        } else {
            stderr
        };
        panic!(
            "`{command_name}` failed while embedding provenance ({status}): {stderr_detail}",
            status = output.status
        );
    }

    String::from_utf8(output.stdout)
        .unwrap_or_else(|error| {
            panic!("`{command_name}` returned non-UTF-8 output while embedding provenance: {error}")
        })
        .trim()
        .to_string()
}
