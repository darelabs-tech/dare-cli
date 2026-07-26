//! Minimal stub repo for dare bench sample-ok fixture.

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_passes() {
        assert_eq!(add(1, 1), 2);
    }

    #[test]
    fn sample_fails_then_passes() {
        assert_eq!(add(2, 2), 4);
    }
}
