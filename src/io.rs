use crate::hash::ForensicHasher;
use aligned_vec::{AVec, ConstAlign};
use crossbeam_channel::{Receiver, Sender, bounded};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::OpenOptions;
use std::io::{Read, Result as IoResult, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::Arc;
use std::thread;

type AlignedBuffer = AVec<u8, ConstAlign<4096>>;

struct Chunk {
    data: Arc<AlignedBuffer>,
    len: usize,
}

#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;

pub fn copy_and_hash<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    hasher: ForensicHasher,
    block_size: usize,
) -> IoResult<(String, String)> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let o_direct = nix::libc::O_DIRECT;
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    let o_direct = 0;

    let input = OpenOptions::new()
        .read(true)
        .custom_flags(o_direct)
        .open(&input_path)
        .or_else(|_| OpenOptions::new().read(true).open(&input_path))?;

    let output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(o_direct)
        .open(&output_path)
        .or_else(|_| {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output_path)
        })?;

    #[cfg(target_os = "macos")]
    {
        // On macOS, we use F_NOCACHE to achieve a similar effect to O_DIRECT
        unsafe {
            nix::libc::fcntl(input.as_raw_fd(), nix::libc::F_NOCACHE, 1);
            nix::libc::fcntl(output.as_raw_fd(), nix::libc::F_NOCACHE, 1);
        }
    }

    let mut input = input;
    let mut output = output;

    let total_size = input.metadata().map(|m| m.len()).unwrap_or(0);

    // Pre-allocate buffer pool (16 buffers)
    let (free_tx, free_rx) = bounded::<AlignedBuffer>(16);
    for _ in 0..16 {
        free_tx
            .send(AVec::from_iter(4096, vec![0u8; block_size]))
            .unwrap();
    }

    let (write_tx, write_rx): (Sender<Option<Chunk>>, Receiver<Option<Chunk>>) = bounded(8);
    let (hash_tx, hash_rx): (Sender<Option<Chunk>>, Receiver<Option<Chunk>>) = bounded(8);

    let pb = if total_size > 0 {
        ProgressBar::new(total_size)
    } else {
        ProgressBar::new_spinner()
    };
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

    // --- WRITER THREAD ---
    let writer_handle = thread::spawn(move || -> IoResult<()> {
        while let Ok(Some(chunk)) = write_rx.recv() {
            output.write_all(&chunk.data[..chunk.len])?;
        }
        output.sync_all()?;
        Ok(())
    });

    // --- HASHER THREAD ---
    let hasher_handle = thread::spawn(move || -> (String, String) {
        let mut h = hasher;
        while let Ok(Some(chunk)) = hash_rx.recv() {
            h.update(&chunk.data[..chunk.len]);
        }
        h.finalize()
    });

    // --- MAIN READER LOOP ---
    loop {
        let mut buffer = match free_rx.recv() {
            Ok(b) => b,
            Err(_) => break,
        };

        let bytes_read = match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!(
                    "\n⚠️ WARNING: Read error at offset {}: {}",
                    input.stream_position().unwrap_or(0),
                    e
                );
                eprintln!("Filling block with zeros to preserve forensic alignment...");
                // In forensic mode, we pad the block with zeros to keep the rest of the image aligned
                for byte in buffer.iter_mut() {
                    *byte = 0;
                }
                // Try to skip the bad block
                let _ = input.seek(SeekFrom::Current(block_size as i64));
                block_size
            }
        };

        let chunk_data = Arc::new(buffer);
        let chunk = Chunk {
            data: Arc::clone(&chunk_data),
            len: bytes_read,
        };

        write_tx
            .send(Some(Chunk {
                data: Arc::clone(&chunk_data),
                len: bytes_read,
            }))
            .unwrap();
        hash_tx.send(Some(chunk)).unwrap();

        // Return buffer to pool efficiently
        let free_tx_clone = free_tx.clone();
        thread::spawn(move || {
            while Arc::strong_count(&chunk_data) > 1 {
                thread::yield_now();
            }
            if let Ok(b) = Arc::try_unwrap(chunk_data) {
                let _ = free_tx_clone.send(b);
            }
        });

        pb.inc(bytes_read as u64);
    }

    let _ = write_tx.send(None);
    let _ = hash_tx.send(None);

    pb.finish_with_message("Copy completed");
    writer_handle.join().expect("Writer thread panicked")?;
    let hashes = hasher_handle.join().expect("Hasher thread panicked");

    Ok(hashes)
}
