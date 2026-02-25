# 🛡️ oxiddd

**oxiddd** is a high-performance, digital forensics disk imaging tool written in Rust. It is a modern, faster, and more secure alternative to the classic `dc3dd`.

## ✨ Key Features

*   **🚀 Blazing Fast**: Multi-threaded pipeline architecture with zero-copy buffer pooling.
*   **🔌 Direct I/O**: Uses `O_DIRECT` to bypass the OS kernel cache, ensuring stable throughput and direct hardware interaction.
*   **🔒 Forensic Integrity Binding**: Unique hashing method that links disk content, target filename, and precise time into a single cryptographic signature.
*   **🌐 Inalterable NTP Timestamping**: Fetches secure time from Google NTP servers to prevent local system clock tampering.
*   **📊 Dual-Hashing**: Automatically generates both a standard bit-for-bit hash (for Autopsy/EnCase) and a custom forensic binding hash.
*   **📦 Zero Dependencies**: Compiles to a 100% static binary for portable use on incident response live-USB.

## 🛠️ Installation

### Pre-requisites
*   Rust (latest stable)
*   `musl-tools` (for static Linux builds)

### Build from source
```bash
git clone https://github.com/your-username/oxiddd.git
cd oxiddd
cargo build --release
```

### Static build (Incident Response ready)
```bash
./build_static.sh
```

## 🚀 Usage

`oxiddd` supports both standard CLI flags (with auto-completion) and classic `dd` syntax.

### Standard Syntax (Recommended for Tab-completion)
```bash
sudo ./oxiddd --if /dev/sdb --of evidence.dd --hash sha512
```

### Classic DD Syntax
```bash
sudo ./oxiddd if=/dev/sdb of=evidence.dd hash=sha256 bs=8M
```

## 🛡️ Forensic Integrity Algorithm

Unlike standard tools, `oxiddd` generates a signature using:
`SHA256( Disk_Content + Target_Filename + NTP_Timestamp )`

This ensures that if an image is renamed or the metadata is modified, the forensic hash will no longer match, preserving the chain of custody.

## 📄 License

This project is licensed under the **GPL-3.0 License** - see the [LICENSE](LICENSE) file for details (consistent with the original `dc3dd` spirit).

---
*Developed with ❤️ in Rust for the Forensics Community.*
