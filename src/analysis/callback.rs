use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use log::info;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

use crate::verify::{VerifyResult, verify_with_param_info};

use super::common::{CollectedFunction, collect_function, find_named_functions};

#[derive(Default)]
pub struct ForgeCallbacks {
    is_real_file: bool,
    input_path: Option<PathBuf>,
    found_counterexample: bool,
    verification_error: bool,
}

impl rustc_driver::Callbacks for ForgeCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        self.input_path = config.input.opt_path().map(PathBuf::from);
        self.is_real_file = self.input_path.is_some();
        info!("Source file: {:?}", self.input_path);
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        self.run_analysis(tcx);
        rustc_driver::Compilation::Stop
    }
}

impl ForgeCallbacks {
    fn run_analysis<'tcx>(&mut self, tcx: TyCtxt<'tcx>) {
        if !self.is_real_file {
            return;
        }

        let pairs = find_verification_pairs(tcx);
        if pairs.is_empty() {
            println!("No src/tgt function pairs found.");
            return;
        }

        let mut fir_dump = String::new();
        for VerificationPair {
            suffix,
            source: src_fn,
            target: tgt_fn,
        } in pairs
        {
            if !fir_dump.is_empty() {
                fir_dump.push('\n');
            }
            let pair_name = pair_label(suffix.as_deref());
            println!("Verifying {} refinement...", pair_name);
            if src_fn.body == tgt_fn.body {
                println!("NOTE: source FIR is identical to target FIR.");
            }
            let result = verify_with_param_info(&src_fn.body, &tgt_fn.body, &src_fn.param_info);
            fir_dump.push_str(&format!("// Pair: {pair_name}\n"));
            if src_fn.body == tgt_fn.body {
                fir_dump.push_str("// NOTE: source FIR is identical to target FIR.\n");
            }
            fir_dump.push_str(&crate::verify::report::format_result(
                &result,
                &src_fn.body,
                &src_fn.param_info,
                "src",
                "tgt",
            ));
            fir_dump.push('\n');
            fir_dump.push_str(&crate::ir::text::format_pair(
                suffix.as_deref(),
                &src_fn.body,
                &tgt_fn.body,
            ));
            match result {
                VerifyResult::Verified => {
                    println!(
                        "VERIFIED: {} target is a valid refinement of source.",
                        pair_name
                    );
                }
                VerifyResult::Counterexample(msg) => {
                    self.found_counterexample = true;
                    println!("COUNTEREXAMPLE found for {}:", pair_name);
                    println!("{msg}");
                }
                VerifyResult::Error(msg) => {
                    self.verification_error = true;
                    println!("ERROR in {}: {}", pair_name, msg);
                }
            }
        }

        self.write_fir_dump(&fir_dump);
    }

    fn write_fir_dump(&self, fir_dump: &str) {
        let Some(input_path) = &self.input_path else {
            return;
        };

        let output_path = input_path.with_extension("fir");
        if let Err(err) = fs::write(&output_path, fir_dump) {
            println!(
                "Failed to write Forge IR dump to {}: {}",
                output_path.display(),
                err
            );
        }
    }

    pub fn verification_error(&self) -> bool {
        self.verification_error
    }

    pub fn found_counterexample(&self) -> bool {
        self.found_counterexample
    }
}

struct VerificationPair {
    suffix: Option<String>,
    source: CollectedFunction,
    target: CollectedFunction,
}

fn find_verification_pairs<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<VerificationPair> {
    let mut src_defs: BTreeMap<Option<String>, (String, rustc_hir::def_id::LocalDefId)> =
        BTreeMap::new();
    let mut tgt_defs: BTreeMap<Option<String>, (String, rustc_hir::def_id::LocalDefId)> =
        BTreeMap::new();

    for (name, def_id) in find_named_functions(tcx) {
        if let Some(suffix) = parse_pair_name(&name, "src") {
            src_defs.insert(suffix, (name, def_id));
        } else if let Some(suffix) = parse_pair_name(&name, "tgt") {
            tgt_defs.insert(suffix, (name, def_id));
        }
    }

    let mut pairs = Vec::new();
    for (suffix, (src_name, src_id)) in src_defs {
        if let Some((tgt_name, tgt_id)) = tgt_defs.remove(&suffix) {
            let src_fn = collect_function(tcx, &src_name, src_id);
            let tgt_fn = collect_function(tcx, &tgt_name, tgt_id);
            pairs.push(VerificationPair {
                suffix,
                source: src_fn,
                target: tgt_fn,
            });
        }
    }

    pairs
}

fn parse_pair_name(name: &str, prefix: &str) -> Option<Option<String>> {
    if name == prefix {
        Some(None)
    } else {
        name.strip_prefix(&format!("{prefix}_"))
            .map(|suffix| Some(suffix.to_string()))
    }
}

fn pair_label(suffix: Option<&str>) -> String {
    match suffix {
        Some(suffix) => format!("src_{suffix} -> tgt_{suffix}"),
        None => "src -> tgt".to_string(),
    }
}
