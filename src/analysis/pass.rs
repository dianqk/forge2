// Keep in sync with rustc_mir_transform::run_optimization_passes for the pinned nightly.
pub const PASSES: &[&str] = &[
    "CheckAlignment",
    "CheckNull",
    "CheckEnums",
    "LowerSliceLenCalls",
    "InstSimplify-before-inline",
    "ForceInline",
    "Inline",
    "RemoveStorageMarkers",
    "RemoveZsts",
    "RemoveUnneededDrops",
    "UnreachableEnumBranching",
    "UnreachablePropagation",
    "SimplifyCfg-after-unreachable-enum-branching",
    "MultipleReturnTerminators",
    "InstSimplify-after-simplifycfg",
    "SimplifyConstCondition-after-inst-simplify",
    "ReferencePropagation",
    "ScalarReplacementOfAggregates",
    "SimplifyLocals-before-const-prop",
    "DeadStoreElimination-initial",
    "GVN",
    "SimplifyLocals-after-value-numbering",
    "SsaRangePropagation",
    "MatchBranchSimplification",
    "DataflowConstProp",
    "SingleUseConsts",
    "SimplifyConstCondition-after-const-prop",
    "JumpThreading",
    "EarlyOtherwiseBranch",
    "SimplifyComparisonIntegral",
    "SimplifyConstCondition-final",
    "RemoveNoopLandingPads",
    "SimplifyCfg-final",
    "StripDebugInfo",
    "CopyProp",
    "DeadStoreElimination-final",
    "DestinationPropagation",
    "TailCopyToMove",
    "MoveElimination",
    "SimplifyLocals-final",
    "MultipleReturnTerminators",
    "AddCallGuards",
    "ReorderBasicBlocks",
    "ReorderLocals",
];

pub fn mir_enable_passes_before(target_pass: &str) -> Result<String, String> {
    let idx = unique_pass_index(target_pass)?;
    Ok(render_pass_overrides(idx, false))
}

pub fn mir_enable_passes_after(target_pass: &str) -> Result<String, String> {
    let idx = unique_pass_index(target_pass)?;
    Ok(render_pass_overrides(idx, true))
}

fn unique_pass_index(target_pass: &str) -> Result<usize, String> {
    let matches = PASSES
        .iter()
        .enumerate()
        .filter_map(|(idx, pass)| (*pass == target_pass).then_some(idx))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [idx] => Ok(*idx),
        [] => Err(format!("unknown MIR pass `{target_pass}`")),
        _ => Err(format!(
            "ambiguous MIR pass `{target_pass}` appears multiple times in the pipeline"
        )),
    }
}

fn render_pass_overrides(target_idx: usize, include_target: bool) -> String {
    PASSES
        .iter()
        .enumerate()
        .map(|(idx, pass)| {
            let enabled = idx < target_idx || (include_target && idx == target_idx);
            format!("{}{pass}", if enabled { "+" } else { "-" })
        })
        .collect::<Vec<_>>()
        .join(",")
}
