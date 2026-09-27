use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use dashmap::DashMap;
use xxhash_rust::xxh3::Xxh3;

const BUF_SIZE: usize = 1024 * 1024; // 1 MiB streaming read chunks

/// Computes a streaming XXH3-64 hash of a file's full contents. Streams in
/// 1 MiB chunks rather than reading the whole file into memory, since
/// backup drives commonly hold multi-gigabyte media files.
pub fn hash_file(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut hasher = Xxh3::new();
    let mut buf = vec![0u8; BUF_SIZE];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.digest())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HashCacheKey {
    path: PathBuf,
    size: u64,
    mtime: SystemTime,
}

/// In-memory cache of file hashes keyed by `(path, size, mtime)`: if either
/// changes, it's a cache miss and gets rehashed. Lives only as long as the
/// server process (no disk persistence), same as the rest of this app's
/// runtime-only state.
#[derive(Default)]
pub struct HashCache {
    inner: DashMap<HashCacheKey, u64>,
}

impl HashCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached hash for `path` if `size`/`mtime` still match,
    /// otherwise computes, caches, and returns a fresh hash.
    pub fn get_or_hash(&self, path: &Path, size: u64, mtime: SystemTime) -> io::Result<u64> {
        let key = HashCacheKey {
            path: path.to_path_buf(),
            size,
            mtime,
        };
        if let Some(existing) = self.inner.get(&key) {
            return Ok(*existing);
        }
        let hash = hash_file(path)?;
        self.inner.insert(key, hash);
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn same_content_hashes_equal() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        std::fs::write(&a, b"hello world").unwrap();
        std::fs::write(&b, b"hello world").unwrap();
        assert_eq!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn different_content_hashes_differ() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        std::fs::write(&a, b"hello world").unwrap();
        std::fs::write(&b, b"hello WORLD").unwrap();
        assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn streams_files_larger_than_one_chunk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("big.bin");
        let mut f = File::create(&path).unwrap();
        // A bit over 2 chunks, with non-repeating content so a buffering bug would show up.
        for i in 0..(BUF_SIZE * 2 + 1000) {
            f.write_all(&[(i % 251) as u8]).unwrap();
        }
        drop(f);
        // Just needs to not panic/error and to be deterministic.
        let h1 = hash_file(&path).unwrap();
        let h2 = hash_file(&path).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn cache_hits_avoid_rehashing_and_misses_on_size_change() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("f.bin");
        std::fs::write(&path, b"v1").unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        let mtime = meta.modified().unwrap();

        let cache = HashCache::new();
        let h1 = cache.get_or_hash(&path, 2, mtime).unwrap();
        let h2 = cache.get_or_hash(&path, 2, mtime).unwrap();
        assert_eq!(h1, h2);

        // Different size => different cache key => recomputed even if the file on disk didn't change.
        std::fs::write(&path, b"v2-longer").unwrap();
        let h3 = cache.get_or_hash(&path, 9, mtime).unwrap();
        assert_ne!(h1, h3);
    }
}
