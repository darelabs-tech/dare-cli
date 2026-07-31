//! Property tests: `CaseSpec` YAML parse never panics on arbitrary bytes.

use proptest::prelude::*;

use crate::CaseSpec;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn yaml_case_parse_no_panic(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        let _ = CaseSpec::try_from_yaml_bytes(&bytes);
    }
}
