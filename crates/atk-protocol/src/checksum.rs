/* Checksum algorithms for ATK protocol packets. */

/* Calculates an additive 8-bit checksum over the given payload slice. */
pub fn calculate_sum8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/*
 * Calculates the Compx / ATK / VXE checksum:
 * (0x55 - sum(bytes[0..n])) & 0xFF
 */
pub fn calculate_compx_checksum(data: &[u8]) -> u8 {
    let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    0x55u8.wrapping_sub(sum)
}

/* Calculates an XOR 8-bit checksum over the given payload slice. */
pub fn calculate_xor8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

/* Validates that the checksum at the specified index matches `calculate_sum8` over `0..checksum_index`. */
pub fn verify_sum8(packet: &[u8], checksum_index: usize) -> bool {
    if checksum_index >= packet.len() {
        return false;
    }
    calculate_sum8(&packet[..checksum_index]) == packet[checksum_index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum8() {
        let data = [0x01, 0x02, 0x03, 0x04];
        assert_eq!(calculate_sum8(&data), 10);
    }

    #[test]
    fn test_xor8() {
        let data = [0xAA, 0x55, 0xFF];
        assert_eq!(calculate_xor8(&data), 0xAA ^ 0x55 ^ 0xFF);
    }

    #[test]
    fn test_compx_checksum() {
        let mut packet = [0u8; 17];
        packet[0] = 0x08;
        packet[1] = 0x04;
        let csum = calculate_compx_checksum(&packet[..16]);
        packet[16] = csum;
        /* The sum of all 17 bytes should equal 0x55 */
        let total: u8 = packet.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        assert_eq!(total, 0x55);
    }
}

