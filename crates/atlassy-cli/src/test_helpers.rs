use super::*;

pub(super) fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

pub(super) fn execute_batch_from_manifest_file(
    manifest_path: &Path,
    artifacts_dir: &Path,
) -> Result<BatchReport, DynError> {
    execute_batch_from_manifest_file_with_backend(
        manifest_path,
        artifacts_dir,
        RuntimeBackend::Stub,
    )
}
