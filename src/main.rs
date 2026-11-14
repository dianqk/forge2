#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::process;

use forge::verify::{VerifyResult, set_z3_timeout_ms, verify_with_param_info, z3_timeout_ms};
use forge::{
    CollectedFunction, ForgeCallbacks, PassVerifyCallbacks, mir_enable_passes_after,
    mir_enable_passes_before,
};

enum Mode {
    Normal,
    VerifyPass(String),
    VerifyPasses(Vec<String>),
}

struct Config {
    mode: Mode,
    verify_fns: Option<BTreeSet<String>>,
    verbose: bool,
    z3_timeout_ms: u32,
    rustc_args: Vec<String>,
}

fn main() {
    pretty_env_logger::init();

    let mut config = parse_args();
    set_z3_timeout_ms(config.z3_timeout_ms);
    if let Some(flags) = env::var_os("FORGE_RUSTFLAGS") {
        let flags = flags.into_string().unwrap();
        let extra_args = split_shell_words(&flags)
            .unwrap_or_else(|err| panic!("failed to parse FORGE_RUSTFLAGS: {err}"));
        config.rustc_args.extend(extra_args);
    }

    match config.mode {
        Mode::Normal => run_normal_mode(config.rustc_args, config.verbose),
        Mode::VerifyPass(pass_name) => run_verify_pass_mode(
            config.rustc_args,
            &pass_name,
            config.verify_fns.as_ref(),
            config.verbose,
        ),
        Mode::VerifyPasses(pass_names) => {
            for pass_name in pass_names {
                run_verify_pass_mode(
                    config.rustc_args.clone(),
                    &pass_name,
                    config.verify_fns.as_ref(),
                    config.verbose,
                );
            }
        }
    }
}

fn parse_args() -> Config {
    let mut mode = Mode::Normal;
    let mut verify_fns = None;
    let mut verbose = false;
    let mut z3_timeout_ms = env::var("FORGE_Z3_TIMEOUT_MS")
        .ok()
        .map(|value| parse_timeout_ms(&value, "FORGE_Z3_TIMEOUT_MS"))
        .unwrap_or_else(z3_timeout_ms);
    let mut rustc_args = Vec::new();

    let mut args = env::args_os().map(|arg| arg.into_string().unwrap());
    let argv0 = args.next().unwrap();
    rustc_args.push(argv0);

    while let Some(arg) = args.next() {
        if arg == "--verify-pass" {
            let pass_name = args
                .next()
                .unwrap_or_else(|| panic!("missing pass name after --verify-pass"));
            mode = Mode::VerifyPass(pass_name);
            continue;
        }
        if arg == "--verify-passes" {
            let pass_names = args
                .next()
                .unwrap_or_else(|| panic!("missing pass list after --verify-passes"));
            mode = Mode::VerifyPasses(parse_verify_passes(&pass_names));
            continue;
        }
        if arg == "--verify-fns" {
            let names = args
                .next()
                .unwrap_or_else(|| panic!("missing function list after --verify-fns"));
            verify_fns = Some(parse_verify_fns(&names));
            continue;
        }
        if arg == "--verbose" {
            verbose = true;
            continue;
        }
        if arg == "--z3-timeout-ms" {
            let value = args
                .next()
                .unwrap_or_else(|| panic!("missing timeout after --z3-timeout-ms"));
            z3_timeout_ms = parse_timeout_ms(&value, "--z3-timeout-ms");
            continue;
        }
        rustc_args.push(arg);
    }

    Config {
        mode,
        verify_fns,
        verbose,
        z3_timeout_ms,
        rustc_args,
    }
}

fn run_normal_mode(rustc_args: Vec<String>, verbose: bool) {
    if verbose {
        print_rustc_args("rustc", &rustc_args);
    }
    let mut callbacks = ForgeCallbacks::default();
    rustc_driver::compiler_entrypoint(&rustc_args, &mut callbacks);
    if callbacks.verification_error() {
        process::exit(2);
    }
    if callbacks.found_counterexample() {
        process::exit(1);
    }
}

fn run_verify_pass_mode(
    rustc_args: Vec<String>,
    pass_name: &str,
    verify_fns: Option<&BTreeSet<String>>,
    verbose: bool,
) {
    let historical_pass = pass_name == forge::passes::HISTORICAL_PASS;
    let cutoff_pass = if historical_pass {
        "MatchBranchSimplification"
    } else {
        pass_name
    };
    let before_overrides =
        mir_enable_passes_before(cutoff_pass).unwrap_or_else(|err| panic!("{err}"));
    let after_overrides = if historical_pass {
        before_overrides.clone()
    } else {
        mir_enable_passes_after(pass_name).unwrap_or_else(|err| panic!("{err}"))
    };

    let (before_functions, input_path) = collect_functions_for_pass_cutoff(
        &rustc_args,
        &before_overrides,
        verify_fns,
        "before",
        false,
        verbose,
    );
    let (after_functions, _) = collect_functions_for_pass_cutoff(
        &rustc_args,
        &after_overrides,
        verify_fns,
        "after",
        historical_pass,
        verbose,
    );

    if before_functions.is_empty() || after_functions.is_empty() {
        println!("No functions found.");
        return;
    }

    let function_names = before_functions
        .keys()
        .filter(|name| after_functions.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();

    if function_names.is_empty() {
        println!("No functions found in both pass states for verification.");
        return;
    }

    let mut report = format!("// Pass: {pass_name}\n");
    let mut found_counterexample = false;
    let mut verification_error = false;
    for name in function_names {
        let before = &before_functions[&name];
        let after = &after_functions[&name];
        println!("Verifying {} after {}...", name, pass_name);
        if before.body == after.body {
            println!("NOTE: source FIR is identical to target FIR.");
        }
        let result = verify_with_param_info(&before.body, &after.body, &before.param_info);
        report.push_str(&format!("\n// Function: {name}\n"));
        report.push_str(&forge::verify::report::format_result(
            &result,
            &before.body,
            &before.param_info,
            "before",
            "after",
        ));
        report.push('\n');
        report.push_str(&forge::ir::text::format_pass(
            &name,
            &before.body,
            &after.body,
        ));
        match result {
            VerifyResult::Verified => {
                println!(
                    "VERIFIED: {} after {} is a valid refinement of before.",
                    name, pass_name
                );
            }
            VerifyResult::Counterexample(msg) => {
                println!("COUNTEREXAMPLE found for {} after {}:", name, pass_name);
                if historical_pass {
                    println!("  after: locally copied {pass_name} applied to the before MIR");
                } else {
                    println!("  before -Zmir-enable-passes={before_overrides}");
                    println!("  after  -Zmir-enable-passes={after_overrides}");
                }
                println!("{msg}");
                found_counterexample = true;
            }
            VerifyResult::Error(msg) => {
                println!("ERROR in {} after {}: {}", name, pass_name, msg);
                verification_error = true;
            }
        }
    }
    if let Some(input_path) = input_path {
        let output_path = input_path.with_extension("fir");
        std::fs::write(&output_path, report)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_path.display()));
    }
    if found_counterexample {
        process::exit(1);
    }
    if verification_error {
        process::exit(2);
    }
}

fn collect_functions_for_pass_cutoff(
    rustc_args: &[String],
    pass_overrides: &str,
    verify_fns: Option<&BTreeSet<String>>,
    label: &str,
    historical_pass: bool,
    verbose: bool,
) -> (
    BTreeMap<String, CollectedFunction>,
    Option<std::path::PathBuf>,
) {
    let mut args = rustc_args.to_vec();
    if !pass_overrides.is_empty() {
        args.push(format!("-Zmir-enable-passes={pass_overrides}"));
    }
    if verbose {
        print_rustc_args(&format!("rustc ({label})"), &args);
    }

    let mut callbacks = PassVerifyCallbacks::new(verify_fns.cloned(), historical_pass);
    rustc_driver::compiler_entrypoint(&args, &mut callbacks);
    callbacks.into_functions()
}

fn parse_timeout_ms(value: &str, source: &str) -> u32 {
    value
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("invalid Z3 timeout from {source}: {value}"))
}

fn print_rustc_args(label: &str, args: &[String]) {
    println!("{label}: {}", format_args(args));
}

fn format_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_escape(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_escape(arg: &str) -> String {
    if !arg.is_empty()
        && arg
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/' | '.' | '=' | ':'))
    {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

fn split_shell_words(input: &str) -> Result<Vec<String>, &'static str> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote = None;

    while let Some(ch) = chars.next() {
        match quote {
            Some(active_quote) if ch == active_quote => {
                quote = None;
            }
            Some(_) if ch == '\\' => {
                let Some(escaped) = chars.next() else {
                    return Err("trailing escape");
                };
                current.push(escaped);
            }
            Some(_) => current.push(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
            }
            None if ch == '\\' => {
                let Some(escaped) = chars.next() else {
                    return Err("trailing escape");
                };
                current.push(escaped);
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err("unterminated quote");
    }
    if !current.is_empty() {
        words.push(current);
    }

    Ok(words)
}

fn parse_verify_fns(input: &str) -> BTreeSet<String> {
    let names = input
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    assert!(
        !names.is_empty(),
        "--verify-fns requires at least one function name"
    );

    names
}

fn parse_verify_passes(input: &str) -> Vec<String> {
    let pass_names = input
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    assert!(
        !pass_names.is_empty(),
        "--verify-passes requires at least one pass name"
    );

    pass_names
}
