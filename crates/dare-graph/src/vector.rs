//! Float32 little-endian vector encoding (ADR-006 / TS Float32Array).

/// Encode `f32` values as little-endian bytes (identical layout to TS `Float32Array` buffer).
pub fn serialize_f32_le(vector: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vector.len() * 4);
    for &v in vector {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// Decode little-endian f32 blob. Returns `None` if empty or length not divisible by 4.
pub fn deserialize_f32_le(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let arr: [u8; 4] = chunk.try_into().ok()?;
        out.push(f32::from_le_bytes(arr));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_reject_bad_len() {
        let v = vec![1.0_f32, -2.5, 0.0];
        let bytes = serialize_f32_le(&v);
        assert_eq!(bytes.len(), 12);
        assert_eq!(deserialize_f32_le(&bytes).as_deref(), Some(v.as_slice()));
        assert!(deserialize_f32_le(&[]).is_none());
        assert!(deserialize_f32_le(&[1, 2, 3]).is_none());
    }

    #[test]
    fn known_le_bytes() {
        let bytes = serialize_f32_le(&[1.0]);
        assert_eq!(bytes, [0x00, 0x00, 0x80, 0x3f]);
    }
}
