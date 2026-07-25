//! Float32 little-endian vector encoding (ADR-006 / TS Float32Array) + cosine.

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

/// Cosine similarity over equal-length finite vectors.
///
/// Returns `0.0` on length mismatch, zero-norm, or non-finite inputs/results (never NaN).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let xf = f64::from(x);
        let yf = f64::from(y);
        if !xf.is_finite() || !yf.is_finite() {
            return 0.0;
        }
        dot += xf * yf;
        norm_a += xf * xf;
        norm_b += yf * yf;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 || !denom.is_finite() {
        return 0.0;
    }
    let score = dot / denom;
    if score.is_finite() {
        score
    } else {
        0.0
    }
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

    #[test]
    fn cosine_zero_norm() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [1.0_f32, 2.0, 3.0];
        let s = cosine_similarity(&a, &b);
        assert_eq!(s, 0.0);
        assert!(s.is_finite());
        let s2 = cosine_similarity(&b, &a);
        assert_eq!(s2, 0.0);
    }

    #[test]
    fn cosine_len_mismatch() {
        let a = [1.0_f32, 0.0];
        let b = [1.0_f32, 0.0, 0.0];
        let s = cosine_similarity(&a, &b);
        assert_eq!(s, 0.0);
        assert!(!s.is_nan());
    }

    #[test]
    fn cosine_orthogonal_ish() {
        let a = [1.0_f32, 0.0];
        let b = [0.0_f32, 1.0];
        let s = cosine_similarity(&a, &b);
        assert!((s - 0.0).abs() < 1e-12);
        assert!(s.is_finite());
        let parallel = cosine_similarity(&[1.0, 2.0], &[2.0, 4.0]);
        assert!((parallel - 1.0).abs() < 1e-6);
    }
}
