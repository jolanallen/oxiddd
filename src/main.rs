mod discovery;
mod hash;
mod io;
mod ntp;

use crate::hash::{ForensicHasher, HashAlgo};
use chrono::{DateTime, Utc};
use clap::Parser;
use inquire::{Confirm, Select, Text};
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

fn print_banner() {
    println!(
        r#"
  ____           _       _      _     _ 
 / __ \         (_)     | |    | |   | |
| |  | |__  __  _   __| |  __| | __| |
| |  | |\ \/ / | | / _` | / _` |/ _` |
| |__| | >  <  | || (_| || (_| | (_| |
 \____/ /_/\_\ |_| \__,_| \__,_|\__,_|_|
 
 Forensic Acquisition Tool v0.3.0
 Author: Jolan Allen | Integrity is the priority.
"#
    );
}

fn run_interactive_mode() -> (String, String, String, String, bool, bool) {
    print_banner();
    println!("--- MODE INTERACTIF ---\n");

    let devices = discovery::list_block_devices();
    let input = if !devices.is_empty() {
        let device = Select::new("Sélectionnez le disque source (if) :", devices)
            .prompt()
            .unwrap();
        device.path // Note: on real systems, we'd need to resolve the raw device path
    } else {
        Text::new("Aucun disque détecté automatiquement. Entrez le chemin source :")
            .with_placeholder("/dev/sdb")
            .prompt()
            .unwrap()
    };

    let output = Text::new("Entrez le chemin/nom de destination (of) :")
        .with_placeholder("acquisition")
        .prompt()
        .unwrap();

    let hash_algo = Select::new(
        "Choisissez l'algorithme de hachage :",
        vec!["sha256", "sha512"],
    )
    .prompt()
    .unwrap();

    let bs = Text::new("Taille de bloc (bs) :")
        .with_default("4M")
        .prompt()
        .unwrap();

    let verify = Confirm::new("Activer la vérification post-écriture ?")
        .with_default(true)
        .prompt()
        .unwrap();

    let working_copy = Confirm::new("Créer une double copie (Master + Working) ?")
        .with_default(false)
        .prompt()
        .unwrap();

    println!("\n--- RÉSUMÉ DE L'OPÉRATION ---");
    println!("Source :      {}", input);
    println!("Destination : {}", output);
    println!("Algorithme :  {}", hash_algo.to_uppercase());
    println!("Vérification: {}", if verify { "OUI" } else { "NON" });
    println!("Double Copie: {}", if working_copy { "OUI" } else { "NON" });

    if !Confirm::new("Confirmer le lancement de l'acquisition ?")
        .with_default(false)
        .prompt()
        .unwrap()
    {
        println!("Opération annulée.");
        std::process::exit(0);
    }

    (
        input,
        output,
        hash_algo.to_string(),
        bs,
        verify,
        working_copy,
    )
}

fn main() {
    let args = Args::parse();

    let is_interactive = args.input_flag.is_none()
        && args.output_flag.is_none()
        && args.kv_args.is_empty()
        && !args.verify
        && !args.working_copy;

    // Détecter si on doit lancer le mode interactif
    let (mut input, mut output, mut hash_str, mut bs_str, mut verify, mut working_copy) =
        if is_interactive {
            run_interactive_mode()
        } else {
            (
                args.input_flag.clone().unwrap_or_default(),
                args.output_flag.clone().unwrap_or_default(),
                args.hash_flag
                    .clone()
                    .unwrap_or_else(|| "sha256".to_string()),
                args.bs_flag.clone().unwrap_or_else(|| "4M".to_string()),
                args.verify,
                args.working_copy,
            )
        };

    for (key, value) in &args.kv_args {
        match key.as_str() {
            "if" => input = value.clone(),
            "of" => output = value.clone(),
            "hash" => hash_str = value.to_lowercase(),
            "bs" => bs_str = value.clone(),
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

    if !is_interactive {
        print_banner();
    }

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
