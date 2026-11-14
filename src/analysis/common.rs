use std::collections::BTreeMap;

use rustc_hir::def::DefKind;
use rustc_middle::mir;
use rustc_middle::ty::TyCtxt;

use crate::ir::inst::{
    ForgeAssume, ForgeBody, ForgeConst, ForgeOperand, ForgePlace, ForgeStmt, VarId,
};
use crate::translate::translate_body;
use crate::verify::{ParamDisplay, ParamInfo};

pub struct CollectedFunction {
    pub body: ForgeBody,
    pub param_info: Vec<ParamInfo>,
}

pub fn find_named_functions<'tcx>(
    tcx: TyCtxt<'tcx>,
) -> BTreeMap<String, rustc_hir::def_id::LocalDefId> {
    let mut functions = BTreeMap::new();

    for item_def_id in tcx.hir_body_owners() {
        let kind = tcx.def_kind(item_def_id);
        if kind != DefKind::Fn && kind != DefKind::AssocFn {
            continue;
        }

        let name = tcx.item_name(item_def_id.to_def_id()).to_string();
        functions.insert(name, item_def_id);
    }

    functions
}

pub fn collect_function<'tcx>(
    tcx: TyCtxt<'tcx>,
    name: &str,
    def_id: rustc_hir::def_id::LocalDefId,
) -> CollectedFunction {
    let mir = tcx.optimized_mir(def_id);
    collect_mir_function(tcx, name, mir)
}

pub fn collect_mir_function<'tcx>(
    tcx: TyCtxt<'tcx>,
    name: &str,
    mir: &mir::Body<'tcx>,
) -> CollectedFunction {
    let body = match translate_body(tcx, mir) {
        Ok(body) => body,
        Err(err) => unimplemented!("Failed to translate {name} to Forge IR:\n{err}"),
    };
    let body = inject_param_assumes(tcx, mir, body);
    let param_info = build_param_info(tcx, mir);

    CollectedFunction { body, param_info }
}

pub fn build_param_info<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) -> Vec<ParamInfo> {
    let mut param_info = Vec::new();

    for (local, local_decl) in body
        .local_decls
        .iter_enumerated()
        .skip(1)
        .take(body.arg_count)
    {
        let name =
            build_param_name(body, local).unwrap_or_else(|| format!("param_{}", local.as_u32()));
        let display = build_param_display(tcx, local_decl.ty).unwrap_or(ParamDisplay::Raw);
        param_info.push(ParamInfo {
            var_id: VarId(local.as_u32()),
            name,
            display,
        });
    }

    param_info
}

fn inject_param_assumes<'tcx>(
    tcx: TyCtxt<'tcx>,
    mir_body: &mir::Body<'tcx>,
    mut body: ForgeBody,
) -> ForgeBody {
    let mut assumes = Vec::new();

    for (local, local_decl) in mir_body
        .local_decls
        .iter_enumerated()
        .skip(1)
        .take(mir_body.arg_count)
    {
        let place = ForgePlace {
            base: VarId(local.as_u32()),
            projections: vec![],
        };
        build_param_assumes(tcx, local_decl.ty, place, &mut assumes);
    }

    if let Some(entry) = body.blocks.first_mut() {
        entry.stmts.splice(0..0, assumes);
    }

    body
}

fn build_param_assumes<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
    place: ForgePlace,
    assumes: &mut Vec<ForgeStmt>,
) {
    match ty.kind() {
        rustc_middle::ty::TyKind::Bool => assumes.push(ForgeStmt::Assume(ForgeAssume::OneOf {
            operand: ForgeOperand::Place(place),
            allowed: vec![
                ForgeConst::Int(
                    0,
                    crate::ir::ty::ForgeTy::Int(
                        crate::ir::ty::IntWidth::I8,
                        crate::ir::ty::IntKind::Bool,
                    ),
                ),
                ForgeConst::Int(
                    1,
                    crate::ir::ty::ForgeTy::Int(
                        crate::ir::ty::IntWidth::I8,
                        crate::ir::ty::IntKind::Bool,
                    ),
                ),
            ],
        })),
        rustc_middle::ty::TyKind::Adt(adt_def, _) if adt_def.is_enum() => {
            if adt_def
                .variants()
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                return;
            }

            let Some(discr_ty) = crate::translate::translate_ty_with_tcx(tcx, ty) else {
                return;
            };
            let allowed = adt_def
                .variants()
                .iter_enumerated()
                .map(|(idx, _)| {
                    let discr = adt_def.discriminant_for_variant(tcx, idx).val;
                    let value = if discr_ty.is_signed() {
                        let shift = 128 - discr_ty.bits();
                        ((discr as i128) << shift) >> shift
                    } else {
                        discr as i128
                    };
                    ForgeConst::Int(value, discr_ty.clone())
                })
                .collect();

            assumes.push(ForgeStmt::Assume(ForgeAssume::OneOf {
                operand: ForgeOperand::Place(place),
                allowed,
            }));
        }
        rustc_middle::ty::TyKind::Tuple(fields) => {
            for (index, field_ty) in fields.iter().enumerate() {
                let mut field_place = place.clone();
                field_place
                    .projections
                    .push(crate::ir::inst::ForgeProjection::Field(index as u32));
                build_param_assumes(tcx, field_ty, field_place, assumes);
            }
        }
        rustc_middle::ty::TyKind::Array(inner, len) => {
            let Some(len) = len.try_to_target_usize(tcx) else {
                return;
            };
            for index in 0..len {
                let mut field_place = place.clone();
                field_place
                    .projections
                    .push(crate::ir::inst::ForgeProjection::Field(index as u32));
                build_param_assumes(tcx, *inner, field_place, assumes);
            }
        }
        rustc_middle::ty::TyKind::Ref(_, inner, mutability)
            if *mutability == rustc_middle::ty::Mutability::Mut =>
        {
            let mut deref_place = place;
            deref_place
                .projections
                .push(crate::ir::inst::ForgeProjection::Deref);
            build_param_assumes(tcx, *inner, deref_place, assumes);
        }
        rustc_middle::ty::TyKind::Adt(adt_def, args) if !adt_def.is_union() => {
            for (index, field_def) in adt_def.all_fields().enumerate() {
                let mut field_place = place.clone();
                field_place
                    .projections
                    .push(crate::ir::inst::ForgeProjection::Field(index as u32));
                build_param_assumes(
                    tcx,
                    field_def.ty(tcx, *args).skip_norm_wip(),
                    field_place,
                    assumes,
                );
            }
        }
        _ => {}
    }
}

fn build_param_name<'tcx>(body: &mir::Body<'tcx>, local: mir::Local) -> Option<String> {
    let argument_index = u16::try_from(local.as_u32()).ok()?;
    body.var_debug_info
        .iter()
        .find(|info| info.argument_index == Some(argument_index))
        .map(|info| info.name.to_string())
}

fn build_param_display<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
) -> Option<ParamDisplay> {
    match ty.kind() {
        rustc_middle::ty::TyKind::Char => Some(ParamDisplay::Char),
        rustc_middle::ty::TyKind::Adt(adt_def, _) if adt_def.is_enum() => {
            if adt_def
                .variants()
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                return None;
            }

            let enum_name = tcx.item_name(adt_def.did()).to_string();
            let variants = adt_def
                .variants()
                .iter_enumerated()
                .map(|(idx, variant)| {
                    let discr = adt_def.discriminant_for_variant(tcx, idx).val;
                    let name = format!("{enum_name}::{}", variant.name);
                    (discr, name)
                })
                .collect();

            Some(ParamDisplay::Enum { variants })
        }
        _ => None,
    }
}
