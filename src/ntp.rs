use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NTP_PORT: u16 = 123;
const NTP_SERVER: &str = "time.google.com";
// NTP epoch is 1900-01-01, Unix epoch is 1970-01-01. Difference is 70 years.
const NTP_TO_UNIX_SECONDS: u64 = 2_208_988_800;

pub fn get_ntp_time() -> Result<SystemTime, String> {
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // SNTP request format (48 bytes):
    // First byte: Leap Indicator (0), Version (4), Mode (3 - Client) -> 0x23
    let mut buf = [0u8; 48];
    buf[0] = 0x23;

    socket
        .send_to(&buf, format!("{}:{}", NTP_SERVER, NTP_PORT))
        .map_err(|e| format!("Failed to send NTP request: {}", e))?;

    let mut recv_buf = [0u8; 48];
    let (size, _) = socket
        .recv_from(&mut recv_buf)
        .map_err(|e| format!("Failed to receive NTP response (network down?): {}", e))?;

    if size != 48 {
        return Err("Invalid NTP response size".to_string());
    }

    // Transmit timestamp is bytes 40-47 (64-bit timestamp)
    // First 32 bits are seconds since 1900-01-01
    let seconds_bytes: [u8; 4] = recv_buf[40..44].try_into().unwrap();
    let ntp_seconds = u32::from_be_bytes(seconds_bytes) as u64;

    if ntp_seconds < NTP_TO_UNIX_SECONDS {
        return Err("Invalid NTP timestamp".to_string());
    }

    let unix_seconds = ntp_seconds - NTP_TO_UNIX_SECONDS;
    Ok(UNIX_EPOCH + Duration::from_secs(unix_seconds))
}
