//! Small adapter for the MirPatch API that moved into rustc's private pass crate.
use rustc_middle::mir::*;
use rustc_middle::ty::Ty;
use rustc_span::Span;

pub(super) struct MirPatch<'tcx> {
    next_local: usize,
    locals: Vec<LocalDecl<'tcx>>,
    statements: Vec<(Location, StatementKind<'tcx>)>,
    terminators: Vec<(BasicBlock, TerminatorKind<'tcx>)>,
}

impl<'tcx> MirPatch<'tcx> {
    pub fn new(body: &Body<'tcx>) -> Self {
        Self {
            next_local: body.local_decls.len(),
            locals: Vec::new(),
            statements: Vec::new(),
            terminators: Vec::new(),
        }
    }

    pub fn new_temp(&mut self, ty: Ty<'tcx>, span: Span) -> Local {
        let local = Local::from_usize(self.next_local);
        self.next_local += 1;
        self.locals.push(LocalDecl::new(ty, span));
        local
    }

    pub fn add_statement(&mut self, location: Location, kind: StatementKind<'tcx>) {
        self.statements.push((location, kind));
    }

    pub fn add_assign(&mut self, location: Location, place: Place<'tcx>, value: Rvalue<'tcx>) {
        self.add_statement(location, StatementKind::Assign(Box::new((place, value))));
    }

    pub fn patch_terminator(&mut self, block: BasicBlock, kind: TerminatorKind<'tcx>) {
        self.terminators.push((block, kind));
    }

    pub fn apply(self, body: &mut Body<'tcx>) {
        body.local_decls.extend(self.locals);
        for (block, kind) in self.terminators {
            body.basic_blocks_mut()[block].terminator_mut().kind = kind;
        }
        // Insert from back to front so locations keep referring to the original body.
        // Reversing the insertion order also preserves order at an identical location.
        let mut statements = self.statements;
        statements.sort_by_key(|(location, _)| (location.block, location.statement_index));
        for (location, kind) in statements.into_iter().rev() {
            let source_info = *body.source_info(location);
            body.basic_blocks_mut()[location.block]
                .statements
                .insert(location.statement_index, Statement::new(source_info, kind));
        }
    }
}
