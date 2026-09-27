use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate test fixture drive folders for local development, exercising
    /// identical/missing/differing-content/differing-size/casing scenarios.
    GenFixtures {
        /// Output directory the fixture drive folders are created in.
        /// Wiped and recreated on each run for idempotent regeneration.
        #[arg(long, default_value = "fixtures")]
        out: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenFixtures { out } => gen_fixtures(&out),
    }
}

fn gen_fixtures(out: &Path) -> anyhow::Result<()> {
    if out.exists() {
        fs::remove_dir_all(out)?;
    }
    fs::create_dir_all(out)?;

    gen_daten_1(out)?;
    gen_videos_2(out)?;

    println!("Generated fixtures in {}", out.display());
    println!("Point the app at them with: --scan-root {}", out.display());
    Ok(())
}

fn write_file(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(content)?;
    Ok(())
}

/// Deterministic pseudo-random bytes (xorshift64), so repeated runs produce
/// byte-identical fixtures without depending on the `rand` crate.
fn gen_bytes(seed: u64, size: usize) -> Vec<u8> {
    let mut state = seed.wrapping_add(0x9E3779B97F4A7C15);
    let mut out = Vec::with_capacity(size);
    while out.len() < size {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.extend_from_slice(&state.to_le_bytes());
    }
    out.truncate(size);
    out
}

/// `Daten_1` with 3 clones (a, b, c): identical files, a file missing from
/// one clone, a same-name/same-size-but-different-content conflict (only
/// caught in advanced/hash comparison mode), a differing-size conflict
/// (caught even in basic mode), nested folders with mixed rollup states,
/// and a filename casing mismatch between clones.
fn gen_daten_1(out: &Path) -> anyhow::Result<()> {
    let a = out.join("Daten_1a");
    let b = out.join("Daten_1b");
    let c = out.join("Daten_1c");

    let identical = b"This file is byte-identical across all clones.\n";
    write_file(&a.join("identical.txt"), identical)?;
    write_file(&b.join("identical.txt"), identical)?;
    write_file(&c.join("identical.txt"), identical)?;

    // Present in a and b, missing from c.
    let only_ab = b"Present in clones a and b, missing from c.\n";
    write_file(&a.join("only_in_ab.txt"), only_ab)?;
    write_file(&b.join("only_in_ab.txt"), only_ab)?;

    // Same name and size in all three, but different content: a basic
    // name+size comparison sees this as "in sync"; advanced hash mode
    // must flag it as differing.
    let conflict_size = 256;
    write_file(&a.join("conflict_content.bin"), &gen_bytes(1, conflict_size))?;
    write_file(&b.join("conflict_content.bin"), &gen_bytes(2, conflict_size))?;
    write_file(&c.join("conflict_content.bin"), &gen_bytes(3, conflict_size))?;

    // Same name, different size in every clone: caught even in basic mode.
    write_file(&a.join("size_diff.bin"), &gen_bytes(10, 100))?;
    write_file(&b.join("size_diff.bin"), &gen_bytes(10, 150))?;
    write_file(&c.join("size_diff.bin"), &gen_bytes(10, 200))?;

    // Casing mismatch: exFAT is case-insensitive-but-preserving, so
    // "Photo.JPG" and "photo.jpg" should be matched as the same logical
    // file, missing entirely from c.
    let photo = gen_bytes(42, 512);
    write_file(&a.join("Photo.JPG"), &photo)?;
    write_file(&b.join("photo.jpg"), &photo)?;

    // Nested folders (2 levels) with a mixed rollup state: readme.md is
    // identical everywhere, but notes/note2.txt is missing from c, so
    // docs/ should roll up as "partially missing" for clone c while
    // docs/notes/note1.txt alone stays fully in sync.
    let readme = b"# Docs\nShared across all clones.\n";
    write_file(&a.join("docs/readme.md"), readme)?;
    write_file(&b.join("docs/readme.md"), readme)?;
    write_file(&c.join("docs/readme.md"), readme)?;

    let note1 = b"Note 1: present everywhere.\n";
    write_file(&a.join("docs/notes/note1.txt"), note1)?;
    write_file(&b.join("docs/notes/note1.txt"), note1)?;
    write_file(&c.join("docs/notes/note1.txt"), note1)?;

    let note2 = b"Note 2: missing from clone c.\n";
    write_file(&a.join("docs/notes/note2.txt"), note2)?;
    write_file(&b.join("docs/notes/note2.txt"), note2)?;

    // Another folder: one file only in `a`, one file identical everywhere.
    write_file(&a.join("mixed/a_only.txt"), b"Only in clone a.\n")?;
    let common = b"Common to every clone.\n";
    write_file(&a.join("mixed/common.txt"), common)?;
    write_file(&b.join("mixed/common.txt"), common)?;
    write_file(&c.join("mixed/common.txt"), common)?;

    Ok(())
}

/// `Videos_2` with only 2 clones (a, c — deliberately skipping `b` to prove
/// clone-letter gaps don't break grouping), including a few-MB dummy file
/// to give real progress/hash-timing signal.
fn gen_videos_2(out: &Path) -> anyhow::Result<()> {
    let a = out.join("Videos_2a");
    let c = out.join("Videos_2c");

    let movie1 = gen_bytes(100, 3 * 1024 * 1024);
    write_file(&a.join("movie1.bin"), &movie1)?;
    write_file(&c.join("movie1.bin"), &movie1)?;

    // Only in clone a, missing from c.
    write_file(&a.join("movie2.bin"), &gen_bytes(200, 2 * 1024 * 1024))?;

    Ok(())
}
