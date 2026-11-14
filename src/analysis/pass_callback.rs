use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use log::info;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

use super::common::{
    CollectedFunction, collect_function, collect_mir_function, find_named_functions,
};

pub struct PassVerifyCallbacks {
    historical_pass: bool,
    is_real_file: bool,
    input_path: Option<PathBuf>,
    verify_fns: Option<BTreeSet<String>>,
    functions: BTreeMap<String, CollectedFunction>,
}

impl rustc_driver::Callbacks for PassVerifyCallbacks {
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
        if self.is_real_file {
            self.functions = find_named_functions(tcx)
                .into_iter()
                .filter(|(name, _)| {
                    self.verify_fns
                        .as_ref()
                        .is_none_or(|verify_fns| verify_fns.contains(name))
                })
                .map(|(name, def_id)| {
                    let function = if self.historical_pass {
                        let mut mir = tcx.optimized_mir(def_id).clone();
                        crate::passes::MatchBranchSimplification_124122.run_pass(tcx, &mut mir);
                        collect_mir_function(tcx, &name, &mir)
                    } else {
                        collect_function(tcx, &name, def_id)
                    };
                    (name, function)
                })
                .collect();
        }
        rustc_driver::Compilation::Stop
    }
}

impl PassVerifyCallbacks {
    pub fn new(verify_fns: Option<BTreeSet<String>>, historical_pass: bool) -> Self {
        Self {
            historical_pass,
            is_real_file: false,
            input_path: None,
            verify_fns,
            functions: BTreeMap::new(),
        }
    }

    pub fn into_functions(self) -> (BTreeMap<String, CollectedFunction>, Option<PathBuf>) {
        (self.functions, self.input_path)
    }
}
