use sha2::{Digest, Sha256, Sha512};

pub enum HashAlgo {
    Sha256,
    Sha512,
}

pub enum HasherInstance {
    Sha256(Sha256, Sha256), // (Standard, Custom)
    Sha512(Sha512, Sha512), // (Standard, Custom)
}

pub struct ForensicHasher {
    instance: HasherInstance,
    target_filename: String,
    ntp_timestamp: String,
    algo: HashAlgo,
}

impl ForensicHasher {
    pub fn new(algo: HashAlgo, target_filename: String, ntp_timestamp: String) -> Self {
        let instance = match algo {
            HashAlgo::Sha256 => HasherInstance::Sha256(Sha256::new(), Sha256::new()),
            HashAlgo::Sha512 => HasherInstance::Sha512(Sha512::new(), Sha512::new()),
        };
        Self {
            instance,
            target_filename,
            ntp_timestamp,
            algo,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        match &mut self.instance {
            HasherInstance::Sha256(std, custom) => {
                std.update(data);
                custom.update(data);
            }
            HasherInstance::Sha512(std, custom) => {
                std.update(data);
                custom.update(data);
            }
        }
    }

    pub fn finalize(self) -> (String, String) {
        let name_bytes = self.target_filename.as_bytes();
        let ts_bytes = self.ntp_timestamp.as_bytes();

        match self.instance {
            HasherInstance::Sha256(std, mut custom) => {
                let std_hash = hex::encode(std.finalize());
                custom.update(name_bytes);
                custom.update(ts_bytes);
                let custom_hash = hex::encode(custom.finalize());
                (std_hash, custom_hash)
            }
            HasherInstance::Sha512(std, mut custom) => {
                let std_hash = hex::encode(std.finalize());
                custom.update(name_bytes);
                custom.update(ts_bytes);
                let custom_hash = hex::encode(custom.finalize());
                (std_hash, custom_hash)
            }
        }
    }

    pub fn extension(&self) -> &str {
        match self.algo {
            HashAlgo::Sha256 => "sha256",
            HashAlgo::Sha512 => "sha512",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forensic_binding() {
        let data = b"hello forensic world";
        let ts = "2026-02-25T120000Z";

        let mut h1 = ForensicHasher::new(HashAlgo::Sha256, "file1.dd".to_string(), ts.to_string());
        let mut h2 = ForensicHasher::new(HashAlgo::Sha256, "file2.dd".to_string(), ts.to_string());

        h1.update(data);
        h2.update(data);

        let (std1, cust1) = h1.finalize();
        let (std2, cust2) = h2.finalize();

        assert_eq!(std1, std2);
        assert_ne!(cust1, cust2);
    }

    #[test]
    fn test_sha512_logic() {
        let mut h = ForensicHasher::new(HashAlgo::Sha512, "test.dd".to_string(), "ts".to_string());
        h.update(b"abc");
        let (std, _) = h.finalize();
        // Standard SHA512 for "abc" starts with ddaf...
        assert!(std.starts_with("ddaf"));
    }
}
