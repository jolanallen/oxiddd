mod hash;
mod io;
mod ntp;

use crate::hash::HashAlgo;
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

    for (key, value) in args.kv_args {
        match key.as_str() {
            "if" => input = value,
            "of" => output = value,
            "hash" => hash_str = value.to_lowercase(),
            "bs" => bs_str = value,
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

    let base_output = Path::new(&output);
    let original_ext = base_output
        .extension()
        .unwrap_or_default()
        .to_string_lossy();
    let out_ext = if original_ext.is_empty() {
        "dd".to_string()
    } else {
        original_ext.into_owned()
    };

    let out_file_path = append_timestamp_to_path(&output, &timestamp_str, &out_ext);
    let out_filename_only = out_file_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let hasher = hash::ForensicHasher::new(algo, out_filename_only.clone(), timestamp_str.clone());
    let hash_ext = hasher.extension().to_string();
    let hash_file_path = append_timestamp_to_path(&output, &timestamp_str, &hash_ext);

    println!("NTP Timestamp: {}", timestamp_str);
    println!("Source:        {}", input);
    println!("Destination:   {}", out_file_path.display());
    println!("Hash File:     {}", hash_file_path.display());
    println!("Algorithm:     {}", hash_str.to_uppercase());

    let (std_hash, custom_hash) = match io::copy_and_hash(&input, &out_file_path, hasher, bs) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error during copy: {}", e);
            std::process::exit(1);
        }
    };

    println!("Standard Hash (Content Only): {}", std_hash);
    println!("Custom Forensic Hash:        {}", custom_hash);

    let hash_content = format!(
        "--- STANDARD HASH (Bit-for-bit content) ---
{}: {}

--- CUSTOM FORENSIC HASH (Binding) ---
Method: {}(Content + FileName + Timestamp)
Hash:   {}

--- METADATA ---
Target FileName: {}
NTP Timestamp:   {}
",
        hash_str.to_uppercase(),
        std_hash,
        hash_str.to_uppercase(),
        custom_hash,
        out_filename_only,
        timestamp_str
    );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_appending() {
        let ts = "2026-02-25";
        let result = append_timestamp_to_path("evidence.dd", ts, "dd");
        assert_eq!(result.to_str().unwrap(), "evidence_2026-02-25.dd");

        let result_hash = append_timestamp_to_path("/tmp/case1", ts, "sha256");
        assert_eq!(
            result_hash.to_str().unwrap(),
            "/tmp/case1_2026-02-25.sha256"
        );
    }

    #[test]
    fn test_kv_parsing() {
        let (k, v) = parse_key_val("if=/dev/sda").unwrap();
        assert_eq!(k, "if");
        assert_eq!(v, "/dev/sda");

        assert!(parse_key_val("invalid_arg").is_err());
    }
}
