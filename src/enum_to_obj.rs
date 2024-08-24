use swc_core::common::DUMMY_SP;
use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
    atoms::JsWord
};

pub struct EnumToObjVisitor;

impl VisitMut for EnumToObjVisitor {
    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);

        let mut replacements: Vec<(usize, ModuleItem)> = Vec::new();
        stmts.iter().enumerate().for_each(|(i, stmt)| {
            if i + 1 == stmts.len() { return; }
            let next_stmt = stmts.get(i + 1).unwrap();

            if let Some(var_decls) = as_var_decls(stmt) {
                if let Some((enum_id, enum_stmts)) = var_iife(var_decls, next_stmt) {
                    if let Some(enum_items) = extract_enum(enum_id, enum_stmts) {
                        let obj_decl = build_obj_decl(enum_id, &enum_items);
                        let item = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(obj_decl))));
                        replacements.push((i, item));
                    }
                }
            }

            if let Some(var_decls) = as_export_var_decls(stmt) {
                if let Some((enum_id, enum_stmts)) = var_iife(var_decls, next_stmt) {
                    if let Some(enum_items) = extract_enum(enum_id, enum_stmts) {
                        let obj_decl = build_obj_decl(enum_id, &enum_items);
                        let item = ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                            span: DUMMY_SP,
                            decl: Decl::Var(Box::new(obj_decl)),
                        }));
                        replacements.push((i, item));
                    }
                }
            }
        });

        replacements.iter().enumerate().for_each(|(j, entry)| {
            let (i, item) = entry.clone();
            let index = i - j;
            stmts.remove(index);
            stmts.remove(index);
            stmts.insert(index, item);
        })
    }
}

fn as_var_decls(stmt: &ModuleItem) -> Option<&Box<VarDecl>> {
    stmt.as_stmt()?.as_decl()?.as_var()
}

fn as_export_var_decls(stmt: &ModuleItem) -> Option<&Box<VarDecl>> {
    stmt.as_module_decl()?.as_export_decl()?.decl.as_var()
}

fn var_iife<'a>(var_decls: &'a Box<VarDecl>, next_stmt: &'a ModuleItem) -> Option<(&'a Ident, &'a Vec<Stmt>)> {
    // Find 'var X' pattern in the first statement.
    match var_decls.kind {
        VarDeclKind::Var => (),
        _ => return None,
    };

    let decls = &var_decls.decls;
    if decls.len() != 1 { return None; } // Make sure it contains only 1 declaration

    let decl = decls.get(0)?;
    if let Some(_) = decl.init { return None; } // without init expression.

    // Record the name of the variable
    let id = &decl.name.as_ident()?.id;
    let name = id.sym.to_string();

    // Find an IIFE in the second statement;
    let call_expr = as_call(&*next_stmt.as_stmt()?.as_expr()?.expr)?;
    if call_expr.args.len() != 1 { return None; }

    // X || (X = {})
    let bin = call_expr.args.get(0)?.expr.as_bin()?;
    match bin.op {
        BinaryOp::LogicalOr => (),
        _ => return None,
    };

    let left_ident_name = bin.left.as_ident()?.sym.to_string();
    if left_ident_name != name { return None; }

    // (X = {})
    let right_assign = unwrap_paren(bin.right.as_ref()).as_assign()?;
    if right_assign.right.as_object()?.props.len() != 0 { return None; }
    match right_assign.op {
        AssignOp::Assign => (),
        _ => return None,
    };

    let right_ident_name = right_assign.left.as_ident()?.sym.to_string();
    if right_ident_name != name { return None; }

    let fn_expr = unwrap_paren(call_expr.callee.as_expr()?).as_fn_expr()?;
    if let Some(_) = fn_expr.ident { return None; } // ensure anonymous

    let fn_params = &fn_expr.function.params;
    if fn_params.len() != 1 { return None; }

    let fn_param_name = fn_params.get(0)?.pat.as_ident()?.sym.to_string();
    if fn_param_name != name { return None; }

    match &fn_expr.function.body {
        Some(fn_body) => Some((id, &fn_body.stmts)),
        _ => None,
    }
}

type EnumItems<'a> = Vec<(&'a Lit, &'a Expr)>;

fn extract_enum<'a>(enum_id: &'a Ident, stmts: &'a Vec<Stmt>) -> Option<EnumItems<'a>> {
    let mut items: EnumItems = Vec::new();
    let result = stmts.iter().try_for_each(|stmt| {
        extract_enum_item(enum_id, stmt)?.iter().for_each(|item| {
            items.push(item.clone())
        });

        Some(())
    });

    match result {
        Some(_) => Some(items),
        _ => None,
    }
}

fn extract_enum_item<'a>(enum_id: &Ident, stmt: &'a Stmt) -> Option<EnumItems<'a>> {
    let enum_name = enum_id.sym.to_string();

    let assign = stmt.as_expr()?.expr.as_assign()?;
    if !is_equal_op(assign) { return None; }

    // Promise AssignTarget::Pat(e.g. let [a, b] = [1, 2]) during enum extract is unreachable
    let left_member = match &assign.left {
        AssignTarget::Simple(at) => at.as_member(),
        AssignTarget::Pat(_at) => None
    }?;
    if member_expr_ident_name(left_member)? != enum_name { return None; }

    let left_member_computed = &left_member.prop.as_computed()?.expr;

    // B["a"] = "x";
    if let Some(lit) = left_member_computed.as_lit() {
        return match lit {
            &Lit::Str(_) => Some(vec![(lit, assign.right.as_ref())]),
            _ => None,
        };
    }

    // B[B["a"] = 1] = "a";
    if let Some(inner_assign) = left_member_computed.as_assign() {
        let inner_member_expr = match &inner_assign.left {
            AssignTarget::Simple(at) => at.as_member(),
            AssignTarget::Pat(_at) => None
        }?;
        if member_expr_ident_name(inner_member_expr)? != enum_name { return None; }
        if !is_equal_op(inner_assign) { return None; }

        let inner_member_name = inner_member_expr.prop.as_computed()?.expr.as_lit()?;
        let inner_member_value = inner_assign.right.as_lit()?;

        return match (inner_member_name, inner_member_value) {
            (&Lit::Str(_), &Lit::Num(_)) => Some(vec![
                (inner_member_name, inner_assign.right.as_ref()),
                (inner_member_value, assign.right.as_ref()),
            ]),
            _ => None,
        };
    }

    None
}

fn build_obj_decl(enum_id: &Ident, enum_items: &EnumItems) -> VarDecl {
    VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent { id: enum_id.clone(), type_ann: None }),
            definite: false,
            init: Some(Box::new(build_obj(&enum_items)))
        }],
        ..Default::default()
    }
}

fn build_obj(enum_items: &EnumItems) -> Expr {
    let props = enum_items.iter().map(|&(k, v)| {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: match k {
                Lit::Str(str) => PropName::Str(str.clone()),
                Lit::Num(num) => PropName::Str(Str { span: num.span, raw: num.clone().raw, value: JsWord::from(num.clone().value.to_string()) }),
                _ => todo!(),
            },
            value: Box::new(v.clone())
        })))
    })
    .collect();

    Expr::Object(ObjectLit { span: DUMMY_SP, props })
}

fn is_equal_op(assign: &AssignExpr) -> bool {
    match assign.op {
        AssignOp::Assign => true,
        _ => false,
    }
}

fn member_expr_ident_name(member_expr: &MemberExpr) -> Option<String> {
    Some(member_expr.obj.as_ident()?.sym.to_string())
}

fn as_call(expr: &Expr) -> Option<&CallExpr> {
    match expr {
        Expr::Call(call) => Some(call),
        _ => None,
    }
}

fn unwrap_paren(expr: &Expr) -> &Expr {
    if let Some(paren) = expr.as_paren() {
        paren.expr.as_ref()
    } else {
        expr
    }
}
