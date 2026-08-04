use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::fs;

use crate::cli::subcommands::file_system::FileSubcommand;
use crate::knot::{Knot, KnotType};
use crate::modes::file::handle_files;

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Creates a unique, isolated directory for each individual test to prevent race conditions.
async fn local_setup() -> Result<(Knot, PathBuf)> {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let test_dir = PathBuf::from(format!("./testing_env_{}", id));

    if test_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&test_dir).await;
    }
    tokio::fs::create_dir_all(&test_dir).await?;

    let knot = Knot::new(KnotType::Local, &test_dir, None).await?;
    Ok((knot, test_dir))
}

async fn cleanup_dir(path: &Path) {
    let _ = tokio::fs::remove_dir_all(path).await;
}

#[tokio::test]
async fn create_and_empty_write_file() -> Result<()> {
    let (knot, dir) = local_setup().await?;
    let file_path = dir.join("test_write.txt");

    // 1. Create and write bytes
    knot.overwrite(&file_path, b"Hello, Rust!").await?;

    // 2. Read it back fully and verify
    let content = knot.read_all(&file_path).await?;
    assert_eq!(content, b"Hello, Rust!");

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn write_into_file_with_offsets() -> Result<()> {
    let (knot, dir) = local_setup().await?;
    let file_path = dir.join("test_offset.txt");

    // Initialize file with "0123456789"
    knot.overwrite(&file_path, b"0123456789").await?;

    // Overwrite in the middle (offset 3) -> "012ABC6789"
    knot.write_at(&file_path, b"ABC", 3).await?;

    let content = knot.read_all(&file_path).await?;
    assert_eq!(content, b"012ABC6789");

    // Edge Case: Write beyond file size -> should write to the end -> "012ABC6789XYZ"
    knot.write_at(&file_path, b"XYZ", 100).await?;

    let content_clamped = knot.read_all(&file_path).await?;
    assert_eq!(content_clamped, b"012ABC6789XYZ");

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn read_at_interval() -> Result<()> {
    let (knot, dir) = local_setup().await?;
    let file_path = dir.join("test_interval.txt");

    knot.overwrite(&file_path, b"abcdefghij").await?;

    // 1. Normal read within bounds (2..6 -> "cdef")
    let res = knot.read_range(&file_path, 2..6).await?;
    assert_eq!(res, b"cdef");

    // 2. Edge Case: Range ends out of bounds (should clamp to file end)
    let res_clamped = knot.read_range(&file_path, 5..100).await?;
    assert_eq!(res_clamped, b"fghij");

    // 3. Edge Case: Start is entirely out of bounds (should return empty vector)
    let res_empty = knot.read_range(&file_path, 50..100).await?;
    assert!(res_empty.is_empty());

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn empty_file() -> Result<()> {
    let (knot, dir) = local_setup().await?;
    let file_path = dir.join("test_empty.txt");

    // Write some dummy data first
    knot.overwrite(&file_path, b"Stale Data").await?;

    // Empty the file
    knot.truncate(&file_path).await?;

    // Ensure file exists but has 0 length
    let content = knot.read_all(&file_path).await?;
    assert!(content.is_empty());

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn rename_and_delete() -> Result<()> {
    let (knot, dir) = local_setup().await?;
    let old_path = dir.join("old.txt");
    let new_path = dir.join("new.txt");

    // 1. Create original file
    knot.create(&old_path).await?;
    assert!(old_path.exists());

    // 2. Rename file
    knot.rename(&old_path, &new_path).await?;
    assert!(!old_path.exists());
    assert!(new_path.exists());

    // 3. Delete file
    knot.delete(vec![new_path.clone()]).await?;
    assert!(!new_path.exists());

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_create_and_empty_write() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let file_path = dir.join("cli_empty_write.txt");

    let raw_data = b"Hello from CLI!";
    let encoded_data = STANDARD.encode(raw_data);

    // Dispatch CLI Command
    let cmd = FileSubcommand::EmptyWrite {
        path: file_path.clone(),
        data: encoded_data,
    };
    handle_files(cmd).await?;

    // Verify on disk
    let disk_content = fs::read(&file_path).await?;
    assert_eq!(disk_content, raw_data);

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_write_with_offset() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let file_path = dir.join("cli_offset.txt");

    fs::write(&file_path, b"0123456789").await?;

    let encoded_payload = STANDARD.encode(b"XYZ");

    let cmd = FileSubcommand::Write {
        path: file_path.clone(),
        data: encoded_payload,
        offset: 3,
    };
    handle_files(cmd).await?;

    let disk_content = fs::read(&file_path).await?;
    assert_eq!(disk_content, b"012XYZ6789");

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_empty_file() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let file_path = dir.join("cli_empty.txt");

    fs::write(&file_path, b"some data").await?;

    let cmd = FileSubcommand::Empty {
        path: file_path.clone(),
    };
    handle_files(cmd).await?;

    let disk_content = fs::read(&file_path).await?;
    assert!(disk_content.is_empty());

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_rename_and_delete() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let old_path = dir.join("cli_old.txt");
    let new_path = dir.join("cli_new.txt");

    let create_cmd = FileSubcommand::Create {
        path: old_path.clone(),
    };
    handle_files(create_cmd).await?;
    assert!(old_path.exists());

    let rename_cmd = FileSubcommand::Rename {
        old_path: old_path.clone(),
        new_path: new_path.clone(),
    };
    handle_files(rename_cmd).await?;
    assert!(!old_path.exists());
    assert!(new_path.exists());

    let delete_cmd = FileSubcommand::Delete {
        path: vec![new_path.clone()],
    };
    handle_files(delete_cmd).await?;
    assert!(!new_path.exists());

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_read_interval_and_full() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let file_path = dir.join("cli_read_test.txt");

    fs::write(&file_path, b"0123456789").await?;

    let interval_cmd = FileSubcommand::ReadInterval {
        path: file_path.clone(),
        start: 2,
        end: 6,
    };
    handle_files(interval_cmd).await?;

    let full_cmd = FileSubcommand::ReadFull {
        path: file_path.clone(),
    };
    handle_files(full_cmd).await?;

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn cli_create_dir_and_dirs() -> Result<()> {
    let (_knot, dir) = local_setup().await?;
    let single_dir = dir.join("single_dir");
    let batch_dirs = vec![dir.join("batch_dir_1/nested"), dir.join("batch_dir_2")];

    let create_dir_cmd = FileSubcommand::CreateDir {
        path: single_dir.clone(),
    };
    handle_files(create_dir_cmd).await?;
    assert!(single_dir.is_dir());

    let create_dirs_cmd = FileSubcommand::CreateDirs {
        path: batch_dirs.clone(),
    };
    handle_files(create_dirs_cmd).await?;
    for d in &batch_dirs {
        assert!(d.is_dir());
    }

    cleanup_dir(&dir).await;
    Ok(())
}

#[tokio::test]
async fn knot_mkdir_and_mkdir_batch() -> Result<()> {
    let (knot, dir) = local_setup().await?;

    let dir1 = dir.join("knot_dir");
    let dir_batch = vec![dir.join("knot_batch_a/sub"), dir.join("knot_batch_b")];

    knot.mkdir(&dir1).await?;
    assert!(dir1.is_dir());

    knot.mkdir_batch(dir_batch.clone()).await?;
    for d in &dir_batch {
        assert!(d.is_dir());
    }

    cleanup_dir(&dir).await;
    Ok(())
}
