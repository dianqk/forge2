use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let bless = args.iter().any(|arg| arg == "--bless");

    match run(bless) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(err) => {
            eprintln!("forge-test: {err}");
            std::process::exit(1);
        }
    }
}

fn run(bless: bool) -> io::Result<bool> {
    let mut tests = Vec::new();
    for root in ["ui-tests", "regression-tests"] {
        collect_rust_tests(Path::new(root), &mut tests)?;
    }
    tests.sort();

    let forge_bin = find_forge_binary()?;
    let mut success = true;

    for test in tests {
        let snapshot = test.with_extension("fir");
        let expected = match fs::read_to_string(&snapshot) {
            Ok(expected) => Some(expected),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err),
        };
        // Each test must generate a fresh report, never pass using a stale dump.
        if snapshot.exists() {
            fs::remove_file(&snapshot)?;
        }
        let compile_flags = read_flags(&test, "compile-flags")?;
        let forge_flags = read_flags(&test, "forge-flags")?;
        let expected_exit = read_flags(&test, "exit-code")?;
        let expected_exit = match expected_exit.as_slice() {
            [] => None,
            [code] => Some(
                code.parse::<i32>()
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?,
            ),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "expected one exit code",
                ));
            }
        };
        let output_mode = read_flags(&test, "output")?;
        let diagnostics_only = match output_mode.as_slice() {
            [] => false,
            [mode] if mode == "diagnostics" && expected_exit.is_some_and(|code| code != 0) => true,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "output: diagnostics requires an explicit nonzero exit-code",
                ));
            }
        };
        let allow_error = fs::read_to_string(&test)?
            .lines()
            .any(|line| line.trim() == "//@ allow-verifier-error");
        let output = run_forge(&forge_bin, &test, &compile_flags, &forge_flags)?;
        let actual = {
            let report = fs::read_to_string(&snapshot);
            // Keep the expected file until status, FileCheck and report checks pass.
            if let Some(expected) = &expected {
                fs::write(&snapshot, expected)?;
            } else if snapshot.exists() {
                fs::remove_file(&snapshot)?;
            }
            match report {
                Ok(_) if diagnostics_only => {
                    success = false;
                    println!(
                        "FAILED {}: diagnostics-only test unexpectedly generated FIR",
                        test.display()
                    );
                    continue;
                }
                Ok(mut report) => {
                    if !output.stderr.is_empty() {
                        report.push_str("\n// Compiler diagnostics:\n");
                        for line in output.stderr.lines() {
                            report.push_str(&format!("{}\n", format!("// {line}").trim_end()));
                        }
                    }
                    report
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound && diagnostics_only => {
                    let mut report = String::from("// Process diagnostics:\n");
                    for line in output.combined.lines().filter(|line| {
                        !line.starts_with("thread '")
                            && !line.starts_with("note: run with `RUST_BACKTRACE")
                    }) {
                        report.push_str(&format!("{}\n", format!("// {line}").trim_end()));
                    }
                    report
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    success = false;
                    println!("FAILED {}: no FIR report generated", test.display());
                    print_block(&output.combined);
                    continue;
                }
                Err(err) => return Err(err),
            }
        };

        let actual = format!(
            "// Exit status: {}\n\n{actual}",
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".into())
        );

        if !expected_exit.map_or_else(
            || is_expected_exit_status(output.status),
            |code| output.status.code() == Some(code),
        ) {
            success = false;
            println!("FAILED {}", test.display());
            print_exit_failure(output.status, &output.combined);
            continue;
        }

        let checked = run_filecheck(&test, &actual, allow_error)?;
        if !checked.status.success() {
            success = false;
            // Failed assertions must not bless the freshly generated report.
            if let Some(expected) = &expected {
                fs::write(&snapshot, expected)?;
            } else if snapshot.exists() {
                fs::remove_file(&snapshot)?;
            }
            println!("FAILED {} (FileCheck)", test.display());
            print_block(&String::from_utf8_lossy(&checked.stdout));
            print_block(&String::from_utf8_lossy(&checked.stderr));
            continue;
        }

        if bless {
            fs::write(&snapshot, &actual)?;
            println!("blessed {}", snapshot.display());
            continue;
        }

        match expected {
            Some(expected) if expected == actual => println!("ok {}", test.display()),
            Some(expected) => {
                success = false;
                println!("FAILED {}", test.display());
                print_mismatch(&snapshot, &expected, &actual);
            }
            None => {
                success = false;
                println!("FAILED {}", test.display());
                println!("missing snapshot {}", snapshot.display());
                println!("run `cargo run --bin forge-test -- --bless` to create it");
            }
        }
    }

    Ok(success)
}

fn collect_rust_tests(dir: &Path, tests: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_rust_tests(&path, tests)?;
        } else if file_type.is_file() && path.extension() == Some(OsStr::new("rs")) {
            tests.push(path);
        }
    }

    Ok(())
}

fn find_forge_binary() -> io::Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let bin_name = if cfg!(windows) { "forge.exe" } else { "forge" };
    let forge_bin = current_exe
        .parent()
        .map(|dir| dir.join(bin_name))
        .ok_or_else(|| io::Error::other("failed to find executable directory"))?;

    if forge_bin.is_file() {
        Ok(forge_bin)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing forge binary at {}", forge_bin.display()),
        ))
    }
}

struct ForgeOutput {
    status: ExitStatus,
    combined: String,
    stderr: String,
}

fn read_flags(test: &Path, directive: &str) -> io::Result<Vec<String>> {
    let source = fs::read_to_string(test)?;
    let mut flags = Vec::new();

    let prefix = format!("//@ {directive}:");
    for line in source.lines() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let parsed = split_shell_words(rest.trim())
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        flags.extend(parsed);
    }

    Ok(flags)
}

fn run_forge(
    forge_bin: &Path,
    test: &Path,
    compile_flags: &[String],
    forge_flags: &[String],
) -> io::Result<ForgeOutput> {
    let mut command = Command::new(forge_bin);
    command
        .arg(test)
        .args(forge_flags)
        .env("RUST_BACKTRACE", "0");
    // Keep the fixtures on their original edition as rustc defaults evolve.
    if !compile_flags
        .iter()
        .any(|flag| flag == "--edition" || flag.starts_with("--edition="))
    {
        command.arg("--edition=2015");
    }
    if !compile_flags.is_empty() {
        command.env("FORGE_RUSTFLAGS", compile_flags.join(" "));
    }
    let output = command.output()?;

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(ForgeOutput {
        status: output.status,
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        combined: normalize_output(&combined),
    })
}

fn run_filecheck(test: &Path, fir: &str, allow_error: bool) -> io::Result<Output> {
    let executable = env::var_os("FILECHECK").unwrap_or_else(|| "FileCheck".into());
    let flags = read_flags(test, "filecheck-flags")?;
    let mut command = Command::new(executable);
    command.arg(test).args(flags);
    if !allow_error {
        command.arg("--implicit-check-not=// ERROR");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| io::Error::new(err.kind(), format!(
            "could not run FileCheck: {err}; enter `nix develop` or set FILECHECK to its path"
        )))?;
    let written = child.stdin.take().unwrap().write_all(fir.as_bytes());
    let output = child.wait_with_output()?;
    // A malformed check file may make FileCheck exit before it reads stdin.
    if output.status.success() {
        written?;
    }
    Ok(output)
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

fn normalize_output(output: &str) -> String {
    let normalized = output.replace("\r\n", "\n");
    if normalized.ends_with('\n') {
        normalized
    } else {
        format!("{normalized}\n")
    }
}

fn print_mismatch(snapshot: &Path, expected: &str, actual: &str) {
    println!("snapshot {}", snapshot.display());
    println!("--- expected");
    print_block(expected);
    println!("--- actual");
    print_block(actual);
}

fn print_exit_failure(status: ExitStatus, output: &str) {
    println!("forge exited with {}", format_exit_status(status));
    println!("--- output");
    print_block(output);
}

fn format_exit_status(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("status code {code}"),
        None => "signal".to_string(),
    }
}

fn is_expected_exit_status(status: ExitStatus) -> bool {
    matches!(status.code(), Some(0 | 1))
}

fn print_block(text: &str) {
    for line in text.lines() {
        println!("{line}");
    }
    if text.is_empty() {
        println!("<empty>");
    }
}
