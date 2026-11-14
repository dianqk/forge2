use std::fs;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

fn run_forge(source: &str) -> Output {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "forge-cli-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("input.rs");
    fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg("--edition=2024")
        .arg(input)
        .output()
        .unwrap();
    fs::remove_dir_all(directory).unwrap();
    output
}

#[test]
fn user_bswap_is_not_an_intrinsic() {
    let output = run_forge(include_str!("inputs/bswap-shadow.rs"));
    assert!(!output.status.success());
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostics.contains("unsupported call terminator: callee=bswap"),
        "{diagnostics}"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("VERIFIED"));
}

#[test]
fn partial_width_unions_do_not_use_independent_field_storage() {
    let output = run_forge(
        r#"
        #![crate_type="lib"]
        #[repr(C)] pub union U { pub a: u8, pub b: u16 }
        pub fn src(mut u: U) -> u16 { u.a = 9; unsafe { u.b } }
        pub fn tgt(u: U) -> u16 { unsafe { u.b } }
    "#,
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported local type"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("VERIFIED"));
}

#[test]
fn normal_mode_returns_error_status_for_verification_errors() {
    let output = run_forge(
        r#"
        #![crate_type="lib"]
        pub fn src(x: u8) -> u8 { x }
        pub fn tgt(x: u8) -> u16 { x as u16 }
    "#,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("ERROR in src -> tgt: return type mismatch")
    );
}

#[test]
fn normal_mode_distinguishes_verified_and_counterexample_statuses() {
    for (value, expected, label) in [(0, 0, "VERIFIED:"), (1, 1, "COUNTEREXAMPLE")] {
        let output = run_forge(&format!(
            r#"
            #![crate_type="lib"]
            pub fn src() -> u8 {{ 0 }}
            pub fn tgt() -> u8 {{ {value} }}
        "#
        ));
        assert_eq!(output.status.code(), Some(expected));
        assert!(String::from_utf8_lossy(&output.stdout).contains(label));
    }
}

#[test]
fn verifier_error_takes_precedence_in_mixed_batches() {
    let output = run_forge(
        r#"
        #![crate_type="lib"]
        pub fn src_a() -> u8 { 0 }
        pub fn tgt_a() -> u8 { 1 }
        pub fn src_b(x: u8) -> u8 { x }
        pub fn tgt_b(x: u8) -> u16 { x as u16 }
        pub fn src_c() -> u8 { 0 }
        pub fn tgt_c() -> u8 { 0 }
    "#,
    );
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("COUNTEREXAMPLE found for src_a -> tgt_a"));
    assert!(stdout.contains("ERROR in src_b -> tgt_b"));
    assert!(stdout.contains("VERIFIED: src_c -> tgt_c"));
}
