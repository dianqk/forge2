use super::{ParamDisplay, ParamInfo, VerifyResult};
use crate::ir::inst::ForgeBody;

pub fn format_result(
    result: &VerifyResult,
    body: &ForgeBody,
    param_info: &[ParamInfo],
    source: &str,
    target: &str,
) -> String {
    let summary = match result {
        VerifyResult::Verified => format!("VERIFIED: {target} refines {source}."),
        VerifyResult::Counterexample(msg) | VerifyResult::Error(msg) => {
            let mut message = msg
                .replace("src returns:", &format!("{source} returns:"))
                .replace("tgt returns:", &format!("{target} returns:"));
            for param in param_info {
                if let ParamDisplay::Enum { variants } = &param.display {
                    let ty = &body
                        .locals
                        .iter()
                        .find(|(var, _)| *var == param.var_id)
                        .unwrap()
                        .1;
                    for (bits, variant) in variants {
                        let value = if ty.is_signed() {
                            crate::ir::ty::sign_extend(*bits, ty.bits()).to_string()
                        } else {
                            bits.to_string()
                        };
                        message = message.replace(
                            &format!("  {} = {variant}\n", param.name),
                            &format!("  {} = {variant} ({value})\n", param.name),
                        );
                    }
                }
                message = message.replace(
                    &format!("  {} = ", param.name),
                    &format!("  _{} ({}) = ", param.var_id.0, param.name),
                );
            }
            let label = if matches!(result, VerifyResult::Counterexample(_)) {
                "COUNTEREXAMPLE"
            } else {
                "ERROR"
            };
            format!("{label}\n{message}")
        }
    };
    let mut report = String::new();
    for line in summary.lines() {
        report.push_str("//");
        if !line.is_empty() {
            report.push(' ');
            report.push_str(line.trim_end());
        }
        report.push('\n');
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::inst::VarId;
    use crate::ir::ty::{ForgeTy, IntKind, IntWidth};

    fn body() -> ForgeBody {
        ForgeBody {
            locals: vec![
                (VarId(0), ForgeTy::Int(IntWidth::I16, IntKind::Signed)),
                (VarId(1), ForgeTy::Int(IntWidth::I8, IntKind::Signed)),
            ],
            param_count: 1,
            blocks: vec![],
        }
    }

    #[test]
    fn maps_counterexample_to_fir_and_signed_enum_value() {
        let result = VerifyResult::Counterexample(
            "Return value mismatch:\n  i = Input::A\n  src returns: 255\n  tgt returns: -1\n"
                .into(),
        );
        let params = [ParamInfo {
            var_id: VarId(1),
            name: "i".into(),
            display: ParamDisplay::Enum {
                variants: vec![(255, "Input::A".into())],
            },
        }];
        assert_eq!(
            format_result(&result, &body(), &params, "before", "after"),
            "// COUNTEREXAMPLE\n// Return value mismatch:\n//   _1 (i) = Input::A (-1)\n//   before returns: 255\n//   after returns: -1\n",
        );
    }

    #[test]
    fn preserves_unsigned_enum_values() {
        let mut body = body();
        body.locals[1].1 = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let result = VerifyResult::Counterexample("  i = Input::A\n".into());
        let params = [ParamInfo {
            var_id: VarId(1),
            name: "i".into(),
            display: ParamDisplay::Enum {
                variants: vec![(255, "Input::A".into())],
            },
        }];
        assert_eq!(
            format_result(&result, &body, &params, "src", "tgt"),
            "// COUNTEREXAMPLE\n//   _1 (i) = Input::A (255)\n"
        );
    }

    #[test]
    fn solver_error_is_distinct_from_a_counterexample() {
        assert_eq!(
            format_result(
                &VerifyResult::Error("timeout\n\n".into()),
                &body(),
                &[],
                "before",
                "after"
            ),
            "// ERROR\n// timeout\n//\n"
        );
    }
}
