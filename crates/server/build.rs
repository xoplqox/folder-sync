use std::path::Path;

/// `rust-embed` needs `frontend/dist` to exist at compile time. This does
/// NOT run `npm run build` (that would couple every `cargo build`/`cargo
/// check` to Node being installed and slow down the Rust inner loop) — it
/// only makes sure the directory exists, with a placeholder page if the
/// real frontend hasn't been built yet, and nudges towards `just build`
/// for the real single-binary artifact.
fn main() {
    let dist = Path::new("../../frontend/dist");
    println!("cargo:rerun-if-changed={}", dist.display());

    if dist.join("index.html").exists() {
        return;
    }

    println!(
        "cargo:warning=frontend/dist/index.html not found — embedding a placeholder page. \
         Run `just build` (or `cd frontend && npm ci && npm run build`) for the real UI."
    );

    std::fs::create_dir_all(dist).expect("failed to create frontend/dist placeholder directory");
    std::fs::write(
        dist.join("index.html"),
        "<!doctype html><html><body>frontend not built yet — run `just build`</body></html>",
    )
    .expect("failed to write frontend/dist placeholder index.html");
}
