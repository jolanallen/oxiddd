use crate::hash::{ForensicHasher, HashAlgo};
use aligned_vec::{AVec, ConstAlign};
use crossbeam_channel::{Receiver, Sender, bounded};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::Digest;
use std::fs::OpenOptions;
use std::io::{Read, Result as IoResult, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

type AlignedBuffer = AVec<u8, ConstAlign<4096>>;

struct Chunk {
    data: Arc<AlignedBuffer>,
    len: usize,
}

#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;

pub fn copy_and_hash<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_paths: Vec<P2>,
    hashers: Vec<ForensicHasher>,
    block_size: usize,
) -> IoResult<(String, Vec<String>)> {
    // --- Configuration des Flags Direct I/O ---
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let flags = nix::libc::O_DIRECT;
    #[cfg(windows)]
    let flags = 0x20000000 | 0x80000000; // FILE_FLAG_NO_BUFFERING + FILE_FLAG_WRITE_THROUGH
    #[cfg(not(any(target_os = "linux", target_os = "android", windows)))]
    let flags = 0;

    let mut input_opts = OpenOptions::new();
    input_opts.read(true);
    #[cfg(any(unix, windows))]
    input_opts.custom_flags(flags);

    let input = input_opts
        .open(&input_path)
        .or_else(|_| OpenOptions::new().read(true).open(&input_path))?;

    let mut outputs = Vec::new();
    for path in &output_paths {
        let mut out_opts = OpenOptions::new();
        out_opts.write(true).create(true).truncate(true);
        #[cfg(any(unix, windows))]
        out_opts.custom_flags(flags);

        let out = out_opts.open(path).or_else(|_| {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
        })?;

        #[cfg(target_os = "macos")]
        unsafe {
            nix::libc::fcntl(out.as_raw_fd(), nix::libc::F_NOCACHE, 1);
        }

        outputs.push(out);
    }

    #[cfg(target_os = "macos")]
    unsafe {
        nix::libc::fcntl(input.as_raw_fd(), nix::libc::F_NOCACHE, 1);
    }

    let mut input = input;
    let total_size = input.metadata().map(|m| m.len()).unwrap_or(0);

    // Pre-allocate buffer pool (16 buffers)
    let (free_tx, free_rx) = bounded::<AlignedBuffer>(16);
    for _ in 0..16 {
        free_tx
            .send(AVec::from_iter(4096, vec![0u8; block_size]))
            .unwrap();
    }

    // Un canal par sortie (writer) et un canal pour le hasher
    let mut write_txs = Vec::new();
    let mut writer_handles = Vec::new();

    for mut output in outputs {
        let (tx, rx): (Sender<Option<Chunk>>, Receiver<Option<Chunk>>) = bounded(8);
        write_txs.push(tx);
        let handle = thread::spawn(move || -> IoResult<()> {
            while let Ok(Some(chunk)) = rx.recv() {
                output.write_all(&chunk.data[..chunk.len])?;
            }
            output.sync_all()?;
            Ok(())
        });
        writer_handles.push(handle);
    }

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

    // --- HASHER THREAD ---
    let hasher_handle = thread::spawn(move || -> (String, Vec<String>) {
        let mut h_list = hashers;
        while let Ok(Some(chunk)) = hash_rx.recv() {
            for h in &mut h_list {
                h.update(&chunk.data[..chunk.len]);
            }
        }

        let mut std_hash = String::new();
        let mut forensic_hashes = Vec::new();

        for (i, h) in h_list.into_iter().enumerate() {
            let (std, forensic) = h.finalize();
            if i == 0 {
                std_hash = std;
            }
            forensic_hashes.push(forensic);
        }
        (std_hash, forensic_hashes)
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
                for byte in buffer.iter_mut() {
                    *byte = 0;
                }
                let _ = input.seek(SeekFrom::Current(block_size as i64));
                block_size
            }
        };

        let chunk_data = Arc::new(buffer);

        for tx in &write_txs {
            tx.send(Some(Chunk {
                data: Arc::clone(&chunk_data),
                len: bytes_read,
            }))
            .unwrap();
        }

        hash_tx
            .send(Some(Chunk {
                data: Arc::clone(&chunk_data),
                len: bytes_read,
            }))
            .unwrap();

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

    for tx in write_txs {
        let _ = tx.send(None);
    }
    let _ = hash_tx.send(None);

    pb.finish_with_message("Copy completed");

    for handle in writer_handles {
        handle.join().expect("Writer thread panicked")?;
    }

    let hashes = hasher_handle.join().expect("Hasher thread panicked");

    Ok(hashes)
}

pub fn verify_file<P: AsRef<Path>>(path: P, algo: HashAlgo, block_size: usize) -> IoResult<String> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let flags = nix::libc::O_DIRECT;
    #[cfg(windows)]
    let flags = 0x20000000; // FILE_FLAG_NO_BUFFERING
    #[cfg(not(any(target_os = "linux", target_os = "android", windows)))]
    let flags = 0;

    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(any(unix, windows))]
    opts.custom_flags(flags);

    let input = opts
        .open(&path)
        .or_else(|_| OpenOptions::new().read(true).open(&path))?;

    #[cfg(target_os = "macos")]
    unsafe {
        nix::libc::fcntl(input.as_raw_fd(), nix::libc::F_NOCACHE, 1);
    }

    let mut input = input;
    let total_size = input.metadata().map(|m| m.len()).unwrap_or(0);

    let pb = if total_size > 0 {
        ProgressBar::new(total_size)
    } else {
        ProgressBar::new_spinner()
    };
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [VÉRIFICATION] [{elapsed_precise}] [{wide_bar:.magenta/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
        .unwrap()
        .progress_chars("#>-"));

    let mut buffer = AlignedBuffer::from_iter(4096, vec![0u8; block_size]);

    let final_hash = match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                match input.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        sha2::Digest::update(&mut hasher, &buffer[..n]);
                        pb.inc(n as u64);
                    }
                    Err(e) => return Err(e),
                }
            }
            hex::encode(hasher.finalize())
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            loop {
                match input.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        sha2::Digest::update(&mut hasher, &buffer[..n]);
                        pb.inc(n as u64);
                    }
                    Err(e) => return Err(e),
                }
            }
            hex::encode(hasher.finalize())
        }
    };

    pb.finish_with_message("Vérification terminée");
    Ok(final_hash)
}
