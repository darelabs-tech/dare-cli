//! Property tests: `SafeRelativePath` never panics on arbitrary input.

use dare_core::SafeRelativePath;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn safe_relative_path_no_panic(s in "\\PC*") {
        let _ = SafeRelativePath::new(&s);
    }
}
