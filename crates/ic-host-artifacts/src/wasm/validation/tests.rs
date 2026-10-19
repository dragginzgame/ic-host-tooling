use super::*;
use crate::wasm::{InspectionLimits, inspect};
use std::error::Error as _;

const HEADER: &[u8] = b"\0asm\x01\0\0\0";
// One () -> () function with an empty body.
const FUNCTION: &[u8] = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0\x0a\x04\x01\x02\0\x0b";

#[test]
fn full_validation_accepts_empty_and_complete_modules_at_exact_byte_limits() {
    for bytes in [HEADER, FUNCTION] {
        validate(bytes, bytes.len()).unwrap();
    }
}

#[test]
fn byte_admission_precedes_malformed_input_and_component_checks() {
    for bytes in [HEADER, b"not wasm", b"\0asm\x0d\0\x01\0"] {
        for limit in [0, bytes.len() - 1] {
            assert!(
                matches!(validate(bytes, limit), Err(ValidationError::LimitExceeded { actual, limit: maximum }) if actual == bytes.len() && maximum == limit)
            );
        }
    }
    assert!(matches!(validate(&[], 0), Err(ValidationError::Invalid(_))));
}

#[test]
fn rejects_components_and_malformed_modules_with_owned_diagnostics() {
    let component = b"\0asm\x0d\0\x01\0";
    assert!(matches!(
        validate(component, component.len()),
        Err(ValidationError::UnsupportedEncoding)
    ));
    for bytes in [
        b"not wasm".as_slice(),
        &FUNCTION[..FUNCTION.len() - 1],
        &[HEADER, &[42, 0]].concat(),
    ] {
        let error = validate(bytes, bytes.len()).unwrap_err();
        let ValidationError::Invalid(diagnostic) = &error else {
            panic!("expected invalid module")
        };
        assert_ne!(diagnostic.message(), "");
        assert_ne!(diagnostic.to_string(), "");
        assert!(diagnostic.offset() <= bytes.len() as u64);
        assert!(error.source().unwrap().is::<ParseError>());
    }
}

#[test]
fn validates_indices_types_and_instructions_beyond_structural_inspection() {
    let limits = InspectionLimits {
        module_bytes: 128,
        sections: 8,
        exports: 2,
        custom_sections: 0,
    };
    let mut invalid_instruction = FUNCTION.to_vec();
    *invalid_instruction.last_mut().unwrap() = 0xff;
    // Change () -> () to () -> i32 while leaving its empty body unchanged.
    let invalid_type =
        b"\0asm\x01\0\0\0\x01\x05\x01\x60\0\x01\x7f\x03\x02\x01\0\x0a\x04\x01\x02\0\x0b";
    let invalid_export = [HEADER, &[7, 5, 1, 1, b'x', 0, 99]].concat();
    for bytes in [
        invalid_instruction.as_slice(),
        invalid_type,
        &invalid_export,
    ] {
        inspect(bytes, limits).unwrap();
        assert!(matches!(
            validate(bytes, bytes.len()),
            Err(ValidationError::Invalid(_))
        ));
    }
}

#[test]
fn validates_every_function_body_not_only_the_first() {
    // Two () -> () functions: the first is valid, the second leaves i32 on stack.
    let invalid_second = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x03\x02\0\0\x0a\x09\x02\x02\0\x0b\x04\0\x41\0\x0b";
    assert!(matches!(
        validate(invalid_second, invalid_second.len()),
        Err(ValidationError::Invalid(_))
    ));
}
