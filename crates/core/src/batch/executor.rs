use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use filetime::FileTime;

use crate::batch::action::{ActionKind, ActionStatus, BatchAction};
use crate::batch::progress::{ProgressEvent, ProgressSink};
use crate::batch::{BatchQueue, RunStatus};

const COPY_CHUNK: usize = 8 * 1024 * 1024; // 8 MiB, so large media files show real progress

fn clone_path(scan_root: &Path, group_name: &str, group_number: &str, clone: &str) -> PathBuf {
    scan_root.join(format!("{group_name}_{group_number}{clone}"))
}

/// Sequentially executes queued batch actions against the drives under
/// `scan_root`. Fault-tolerant by design: an individual action's failure
/// (I/O error, permission denied, drive removed mid-run) is recorded on
/// that action and execution continues with the next one — never aborts
/// the whole run. Cancellation is coarse-grained: it stops new actions from
/// starting, but lets any in-flight copy finish.
pub struct BatchExecutor {
    queue: Arc<BatchQueue>,
    cancel_flag: Arc<AtomicBool>,
}

impl BatchExecutor {
    pub fn new(queue: Arc<BatchQueue>) -> Self {
        BatchExecutor {
            queue,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// Runs all currently `Queued` actions. Safe to call again after a
    /// cancel or a completed run: only `Queued` actions are picked up, so
    /// this is exactly what "resume" means — remaining queued items just
    /// get executed on the next call.
    pub fn run(&self, scan_root: &Path, sink: &dyn ProgressSink) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        self.queue.set_run_status(RunStatus::Running);
        sink.emit(ProgressEvent::RunStarted);

        let mut succeeded = 0usize;
        let mut failed = 0usize;

        loop {
            if self.cancel_flag.load(Ordering::SeqCst) {
                self.queue.set_run_status(RunStatus::Cancelled);
                sink.emit(ProgressEvent::RunCancelled);
                return;
            }

            let Some(action) = self.queue.next_queued() else {
                break;
            };

            self.queue.mark_running(action.id);
            sink.emit(ProgressEvent::ActionStarted { action_id: action.id });

            let result = execute_one(scan_root, &action, |done, total| {
                self.queue.set_progress(action.id, done, total);
                sink.emit(ProgressEvent::ActionProgress {
                    action_id: action.id,
                    bytes_done: done,
                    bytes_total: total,
                });
            });

            let (status, error) = match result {
                Ok(()) => {
                    succeeded += 1;
                    (ActionStatus::Done, None)
                }
                Err(msg) => {
                    failed += 1;
                    (ActionStatus::Failed, Some(msg))
                }
            };
            self.queue.mark_finished(action.id, status, error.clone());
            sink.emit(ProgressEvent::ActionFinished {
                action_id: action.id,
                status,
                error,
            });
        }

        self.queue.set_run_status(RunStatus::Completed);
        sink.emit(ProgressEvent::RunCompleted { succeeded, failed });
    }
}

/// Executes a single action against `scan_root`, running every target
/// clone even if an earlier one fails, and aggregating any failures into
/// one message rather than stopping at the first error.
fn execute_one(scan_root: &Path, action: &BatchAction, mut progress: impl FnMut(u64, u64)) -> Result<(), String> {
    match &action.kind {
        ActionKind::SyncFile {
            rel_path,
            source_clone,
            target_clones,
        } => {
            let source_path = clone_path(scan_root, &action.group_name, &action.group_number, source_clone).join(rel_path);
            let metadata = std::fs::metadata(&source_path).map_err(|e| format!("reading source ({source_clone}): {e}"))?;
            let total = metadata.len();
            let mtime = metadata.modified().ok();

            let mut errors = Vec::new();
            for target in target_clones {
                let dest_path = clone_path(scan_root, &action.group_name, &action.group_number, target).join(rel_path);
                if let Err(e) = copy_with_progress(&source_path, &dest_path, total, &mut progress) {
                    errors.push(format!("{target}: {e}"));
                    continue;
                }
                if let Some(mt) = mtime {
                    let _ = filetime::set_file_mtime(&dest_path, FileTime::from_system_time(mt));
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            }
        }
        ActionKind::DeleteFile { rel_path, clones } => {
            let mut errors = Vec::new();
            for clone in clones {
                let path = clone_path(scan_root, &action.group_name, &action.group_number, clone).join(rel_path);
                match std::fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // already gone: not an error
                    Err(e) => errors.push(format!("{clone}: {e}")),
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            }
        }
    }
}

fn copy_with_progress(src: &Path, dest: &Path, total: u64, progress: &mut impl FnMut(u64, u64)) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut reader = std::fs::File::open(src)?;
    let mut writer = std::fs::File::create(dest)?;
    let mut buf = vec![0u8; COPY_CHUNK];
    let mut done = 0u64;
    progress(0, total);
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        done += n as u64;
        progress(done, total);
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::progress::NullSink;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn write(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn sync_file_copies_to_all_targets_and_preserves_mtime() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"hello world");
        std::fs::create_dir_all(dir.path().join("Daten_1b")).unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1c")).unwrap();

        let queue = Arc::new(BatchQueue::new());
        let action = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "f.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into(), "c".into()],
            },
            None,
        );
        let id = action.id;
        queue.add(action);

        let executor = BatchExecutor::new(queue.clone());
        executor.run(dir.path(), &NullSink);

        let run = queue.snapshot();
        let a = run.actions.iter().find(|a| a.id == id).unwrap();
        assert_eq!(a.status, ActionStatus::Done);
        assert_eq!(a.error, None);

        assert_eq!(std::fs::read(dir.path().join("Daten_1b/f.txt")).unwrap(), b"hello world");
        assert_eq!(std::fs::read(dir.path().join("Daten_1c/f.txt")).unwrap(), b"hello world");
    }

    #[test]
    fn delete_file_removes_from_all_listed_clones() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("Daten_1a/f.txt"), b"x");
        write(&dir.path().join("Daten_1b/f.txt"), b"x");

        let queue = Arc::new(BatchQueue::new());
        let action = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::DeleteFile {
                rel_path: "f.txt".into(),
                clones: vec!["a".into(), "b".into()],
            },
            None,
        );
        let id = action.id;
        queue.add(action);

        let executor = BatchExecutor::new(queue.clone());
        executor.run(dir.path(), &NullSink);

        assert_eq!(queue.snapshot().actions.iter().find(|a| a.id == id).unwrap().status, ActionStatus::Done);
        assert!(!dir.path().join("Daten_1a/f.txt").exists());
        assert!(!dir.path().join("Daten_1b/f.txt").exists());
    }

    #[test]
    fn failed_action_does_not_stop_the_run() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1a")).unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1b")).unwrap();
        write(&dir.path().join("Daten_1a/second.txt"), b"ok");

        let queue = Arc::new(BatchQueue::new());
        // First action references a source file that doesn't exist: must fail without blocking the second.
        let failing = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "missing.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into()],
            },
            None,
        );
        let ok_action = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "second.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into()],
            },
            None,
        );
        let (failing_id, ok_id) = (failing.id, ok_action.id);
        queue.add(failing);
        queue.add(ok_action);

        let executor = BatchExecutor::new(queue.clone());
        executor.run(dir.path(), &NullSink);

        let run = queue.snapshot();
        let failed = run.actions.iter().find(|a| a.id == failing_id).unwrap();
        assert_eq!(failed.status, ActionStatus::Failed);
        assert!(failed.error.is_some());

        let ok = run.actions.iter().find(|a| a.id == ok_id).unwrap();
        assert_eq!(ok.status, ActionStatus::Done);
        assert_eq!(std::fs::read(dir.path().join("Daten_1b/second.txt")).unwrap(), b"ok");
    }

    /// A sink that cancels the executor as soon as the first action finishes,
    /// simulating a cancel request arriving mid-run (which in production
    /// comes from a concurrent request while `run()` is blocking a worker
    /// thread) without relying on timing-dependent thread interleaving.
    struct CancelAfterFirstSink {
        executor: Arc<BatchExecutor>,
        first_action_id: Uuid,
    }

    impl ProgressSink for CancelAfterFirstSink {
        fn emit(&self, event: ProgressEvent) {
            if let ProgressEvent::ActionFinished { action_id, .. } = event {
                if action_id == self.first_action_id {
                    self.executor.cancel();
                }
            }
        }
    }

    #[test]
    fn cancel_mid_run_leaves_remaining_actions_queued_for_resume() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1a")).unwrap();
        std::fs::create_dir_all(dir.path().join("Daten_1b")).unwrap();
        write(&dir.path().join("Daten_1a/first.txt"), b"x");
        write(&dir.path().join("Daten_1a/second.txt"), b"y");

        let queue = Arc::new(BatchQueue::new());
        let first = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "first.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into()],
            },
            None,
        );
        let second = BatchAction::new(
            "Daten".into(),
            "1".into(),
            ActionKind::SyncFile {
                rel_path: "second.txt".into(),
                source_clone: "a".into(),
                target_clones: vec!["b".into()],
            },
            None,
        );
        let (first_id, second_id) = (first.id, second.id);
        queue.add(first);
        queue.add(second);

        let executor = Arc::new(BatchExecutor::new(queue.clone()));
        let sink = CancelAfterFirstSink {
            executor: executor.clone(),
            first_action_id: first_id,
        };
        executor.run(dir.path(), &sink);

        let run = queue.snapshot();
        assert_eq!(run.status, RunStatus::Cancelled);
        assert_eq!(run.actions.iter().find(|a| a.id == first_id).unwrap().status, ActionStatus::Done);
        // Cancel took effect before the second action started: still Queued,
        // ready to be picked up by a later run() call ("resume").
        assert_eq!(run.actions.iter().find(|a| a.id == second_id).unwrap().status, ActionStatus::Queued);

        executor.run(dir.path(), &NullSink);
        assert_eq!(queue.snapshot().actions.iter().find(|a| a.id == second_id).unwrap().status, ActionStatus::Done);
    }
}
