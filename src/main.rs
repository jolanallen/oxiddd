mod hash;
mod io;
mod ntp;

use crate::hash::{ForensicHasher, HashAlgo};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// oxiddd - A modern, high-performance forensic disk imaging tool in Rust.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file (Alternative to if=)
    #[arg(long = "if", value_name = "FILE")]
    input_flag: Option<String>,

    /// Output file (Alternative to of=)
    #[arg(long = "of", value_name = "FILE")]
    output_flag: Option<String>,

    /// Hash algorithm (Alternative to hash=)
    #[arg(long = "hash", value_name = "ALGO")]
    hash_flag: Option<String>,

    /// Block size (Alternative to bs=)
    #[arg(long = "bs", value_name = "SIZE")]
    bs_flag: Option<String>,

    /// Verify output after acquisition
    #[arg(long = "verify", short = 'v')]
    verify: bool,

    /// Create both a Master copy and a Working copy
    #[arg(long = "working-copy", short = 'w')]
    working_copy: bool,

    /// Positional arguments in key=value format (e.g., if=/dev/sdb)
    #[arg(value_parser = parse_key_val)]
    kv_args: Vec<(String, String)>,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() {
    let args = Args::parse();

    let mut input = args.input_flag.unwrap_or_default();
    let mut output = args.output_flag.unwrap_or_default();
    let mut hash_str = args.hash_flag.unwrap_or_else(|| "sha256".to_string());
    let mut bs_str = args.bs_flag.unwrap_or_else(|| "4M".to_string());
    let mut verify = args.verify;
    let mut working_copy = args.working_copy;

    for (key, value) in args.kv_args {
        match key.as_str() {
            "if" => input = value,
            "of" => output = value,
            "hash" => hash_str = value.to_lowercase(),
            "bs" => bs_str = value,
            "verify" => verify = value == "true" || value == "1",
            "working-copy" => working_copy = value == "true" || value == "1",
            _ => eprintln!("Warning: unknown argument {}={}", key, value),
        }
    }

    if input.is_empty() || output.is_empty() {
        eprintln!("Error: input (if=) and output (of=) are required.");
        std::process::exit(1);
    }

    let multiplier = if bs_str.ends_with('K') {
        1024
    } else if bs_str.ends_with('M') {
        1024 * 1024
    } else if bs_str.ends_with('G') {
        1024 * 1024 * 1024
    } else {
        1
    };
    let num_part = bs_str.trim_end_matches(|c: char| !c.is_numeric());
    let bs = num_part.parse::<usize>().unwrap_or(4194304) * multiplier;

    let algo = match hash_str.as_str() {
        "sha256" => HashAlgo::Sha256,
        "sha512" => HashAlgo::Sha512,
        _ => {
            eprintln!("Error: unsupported hash algorithm '{}'.", hash_str);
            std::process::exit(1);
        }
    };

    println!("oxiddd - Forensic Disk Copy Tool");
    println!("Fetching secure timestamp from Google NTP...");

    let ntp_time = match ntp::get_ntp_time() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("CRITICAL ERROR: Failed to get secure NTP time: {}", e);
            std::process::exit(1);
        }
    };

    let dt: DateTime<Utc> = ntp_time.into();
    let timestamp_str = dt.format("%Y-%m-%dT%H%M%SZ").to_string();

    let base_path = Path::new(&output);
    let original_ext = base_path.extension().unwrap_or_default().to_string_lossy();
    let out_ext = if original_ext.is_empty() {
        "dd".to_string()
    } else {
        original_ext.into_owned()
    };

    let mut out_file_paths = Vec::new();
    let mut hashers = Vec::new();
    let mut names_only = Vec::new();

    if working_copy {
        let master_path = append_prefix_and_timestamp(&output, "master", &timestamp_str, &out_ext);
        let working_path =
            append_prefix_and_timestamp(&output, "working", &timestamp_str, &out_ext);

        let master_name = master_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let working_name = working_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        hashers.push(ForensicHasher::new(
            algo,
            master_name.clone(),
            timestamp_str.clone(),
        ));
        hashers.push(ForensicHasher::new(
            algo,
            working_name.clone(),
            timestamp_str.clone(),
        ));

        out_file_paths.push(master_path);
        out_file_paths.push(working_path);
        names_only.push(master_name);
        names_only.push(working_name);
    } else {
        let path = append_timestamp_to_path(&output, &timestamp_str, &out_ext);
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        hashers.push(ForensicHasher::new(
            algo,
            name.clone(),
            timestamp_str.clone(),
        ));
        out_file_paths.push(path);
        names_only.push(name);
    }

    let hash_ext = match algo {
        HashAlgo::Sha256 => "sha256",
        HashAlgo::Sha512 => "sha512",
    };
    let hash_file_path = append_timestamp_to_path(&output, &timestamp_str, hash_ext);

    println!("NTP Timestamp: {}", timestamp_str);
    println!("Source:        {}", input);
    for (i, p) in out_file_paths.iter().enumerate() {
        let label = if working_copy && i == 0 {
            "Master Copy:"
        } else if working_copy && i == 1 {
            "Working Copy:"
        } else {
            "Destination:"
        };
        println!("{:<14} {}", label, p.display());
    }
    println!("Hash File:     {}", hash_file_path.display());
    println!("Algorithm:     {}", hash_str.to_uppercase());

    let (std_hash, forensic_hashes) =
        match io::copy_and_hash(&input, out_file_paths.clone(), hashers, bs) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Error during copy: {}", e);
                std::process::exit(1);
            }
        };

    println!("Standard Hash (Content Only): {}", std_hash);
    for (i, h) in forensic_hashes.iter().enumerate() {
        let label = if working_copy && i == 0 {
            "Master Binding Hash:"
        } else if working_copy && i == 1 {
            "Working Binding Hash:"
        } else {
            "Custom Forensic Hash:"
        };
        println!("{:<22} {}", label, h);
    }

    if verify {
        for (i, path) in out_file_paths.iter().enumerate() {
            let label = if working_copy && i == 0 {
                "MASTER"
            } else {
                "WORKING"
            };
            println!("\nStarting verification of {} copy...", label);
            match io::verify_file(path, algo, bs) {
                Ok(v_hash) => {
                    if v_hash == std_hash {
                        println!("✅ {} VERIFICATION SUCCESSFUL: Hashes match.", label);
                    } else {
                        eprintln!("\n❌ CRITICAL ERROR: {} VERIFICATION FAILED!", label);
                        eprintln!("Expected Hash: {}", std_hash);
                        eprintln!("Actual Hash:   {}", v_hash);
                        std::process::exit(2);
                    }
                }
                Err(e) => {
                    eprintln!("Error during verification of {}: {}", label, e);
                    std::process::exit(1);
                }
            }
        }
    }

    let mut hash_content = format!(
        "--- STANDARD HASH (Bit-for-bit content) ---
{}: {}

--- FORENSIC BINDING HASHES ---
",
        hash_str.to_uppercase(),
        std_hash
    );

    for (i, h) in forensic_hashes.iter().enumerate() {
        let role = if working_copy && i == 0 {
            "MASTER"
        } else if working_copy && i == 1 {
            "WORKING"
        } else {
            "DEFAULT"
        };
        hash_content.push_str(&format!(
            "Role:     {}
FileName: {}
Method:   {}(Content + FileName + Timestamp)
Hash:     {}

",
            role,
            names_only[i],
            hash_str.to_uppercase(),
            h
        ));
    }

    hash_content.push_str(&format!(
        "--- METADATA ---
NTP Timestamp:   {}
--- VERIFICATION ---
Status: {}
",
        timestamp_str,
        if verify {
            "Verified (Success)"
        } else {
            "Not performed"
        }
    ));

    if let Err(e) = fs::write(&hash_file_path, hash_content) {
        eprintln!("Failed to write hash file: {}", e);
        std::process::exit(1);
    }

    println!("Forensic copy completed successfully.");
}

fn append_timestamp_to_path(base_path: &str, timestamp: &str, ext: &str) -> PathBuf {
    let path = Path::new(base_path);
    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let mut new_name = file_stem.into_owned();
    new_name.push('_');
    new_name.push_str(timestamp);
    new_name.push('.');
    new_name.push_str(ext);
    if parent.as_os_str().is_empty() {
        PathBuf::from(new_name)
    } else {
        parent.join(new_name)
    }
}

fn append_prefix_and_timestamp(
    base_path: &str,
    prefix: &str,
    timestamp: &str,
    ext: &str,
) -> PathBuf {
    let path = Path::new(base_path);
    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let new_name = format!("{}_{}_{}.{}", prefix, file_stem, timestamp, ext);
    if parent.as_os_str().is_empty() {
        PathBuf::from(new_name)
    } else {
        parent.join(new_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_appending() {
        let ts = "2026-02-25";
        let result = append_timestamp_to_path("evidence.dd", ts, "dd");
        assert_eq!(result.to_str().unwrap(), "evidence_2026-02-25.dd");
    }

    #[test]
    fn test_prefix_appending() {
        let ts = "2026-02-25";
        let result = append_prefix_and_timestamp("evidence.dd", "master", ts, "dd");
        assert_eq!(result.to_str().unwrap(), "master_evidence_2026-02-25.dd");
    }

    #[test]
    fn test_kv_parsing() {
        let (k, v) = parse_key_val("if=/dev/sda").unwrap();
        assert_eq!(k, "if");
        assert_eq!(v, "/dev/sda");
    }
}
