use swc_core::common::{DUMMY_SP, Span, Spanned};
use swc_core::common::collections::AHashMap;
use swc_core::ecma::{
    ast::*,
    atoms::JsWord,
    visit::{VisitMut, VisitMutWith, noop_visit_mut_type},
    utils::ExprFactory,
};

pub struct EnumToObjVisitor;

impl VisitMut for EnumToObjVisitor {
    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);

        let mut replacements: Vec<(usize, ModuleItem)> = Vec::new();
        stmts.iter_mut().enumerate().for_each(|(i, stmt)| {
            if let Some(e) = ts_enum_decl(stmt) {
                let item = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(handle_enum(e)))));
                replacements.push((i, item));
            }

            if let Some(e) = export_ts_enum(stmt) {
                let item = ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    span: DUMMY_SP,
                    decl: Decl::Var(Box::new(handle_enum(e)))
                }));

                replacements.push((i, item));
            }
        });

        replacements.iter().enumerate().for_each(|(_, entry)| {
            let (i, item) = entry.clone();
            stmts.remove(i);
            stmts.insert(i, item);
        })
    }
}

fn ts_enum_decl(stmt: &mut ModuleItem) -> Option<&Box<TsEnumDecl>> {
    Some(stmt.as_mut_stmt()?.as_decl()?.as_ts_enum()?)
}

fn export_ts_enum(stmt: &mut ModuleItem) -> Option<&Box<TsEnumDecl>> {
    Some(stmt.as_mut_module_decl()?.as_mut_export_decl()?.decl.as_ts_enum()?)
}

/// Value does not contain TsLit::Bool
type EnumValues = AHashMap<JsWord, Option<TsLit>>;

fn handle_enum(e: &Box<TsEnumDecl>) -> VarDecl {
    /// Called only for enums.
    ///
    /// If both of the default value and the initialization is None, this
    /// method returns [Err].
    fn compute(
        e: &TsEnumDecl,
        span: Span,
        values: &mut EnumValues,
        default: Option<i64>,
        init: Option<&Expr>,
    ) -> Result<TsLit, ()> {
        fn compute_bin(
            e: &TsEnumDecl,
            span: Span,
            values: &mut EnumValues,
            expr: &BinExpr,
        ) -> Result<TsLit, ()> {
            let l = compute(e, span, values, None, Some(&expr.left))?;
            let r = compute(e, span, values, None, Some(&expr.right))?;

            Ok(match (l, r) {
                (
                    TsLit::Number(Number { value: l, .. }),
                    TsLit::Number(Number { value: r, .. }),
                ) => {
                    TsLit::Number(Number {
                        span,
                        value: match expr.op {
                            op!(bin, "+") => l + r,
                            op!(bin, "-") => l - r,
                            op!("*") => l * r,
                            op!("/") => l / r,

                            // TODO
                            op!("&") => ((l.trunc() as i32) & (r.trunc() as i32)) as _,
                            op!("|") => ((l.trunc() as i32) | (r.trunc() as i32)) as _,
                            op!("^") => ((l.trunc() as i32) ^ (r.trunc() as i32)) as _,
                            op!("<<") => (l.trunc() as i32).wrapping_shl(r.trunc() as u32) as _,
                            op!(">>") => (l.trunc() as i32).wrapping_shr(r.trunc() as u32) as _,
                            // TODO: Verify this
                            op!(">>>") => {
                                (l.trunc() as u32).wrapping_shr(r.trunc() as u32) as _
                            }

                            _ => return Err(()),
                        },
                        raw: None,
                    })
                }
                (TsLit::Str(l), TsLit::Str(r)) if expr.op == op!(bin, "+") => {
                    let value = format!("{}{}", l.value, r.value);

                    TsLit::Str(Str {
                        span,
                        raw: None,
                        value: value.into(),
                    })
                }
                (TsLit::Number(l), TsLit::Str(r)) if expr.op == op!(bin, "+") => {
                    let value = format!("{}{}", l.value, r.value);

                    TsLit::Str(Str {
                        span,
                        raw: None,
                        value: value.into(),
                    })
                }
                (TsLit::Str(l), TsLit::Number(r)) if expr.op == op!(bin, "+") => {
                    let value = format!("{}{}", l.value, r.value);

                    TsLit::Str(Str {
                        span,
                        raw: None,
                        value: value.into(),
                    })
                }
                _ => return Err(()),
            })
        }

        if let Some(expr) = init {
            match expr {
                Expr::Lit(Lit::Str(s)) => return Ok(TsLit::Str(s.clone())),
                Expr::Lit(Lit::Num(s)) => return Ok(TsLit::Number(s.clone())),
                Expr::Bin(ref bin) => return compute_bin(e, span, values, bin),
                Expr::Paren(ref paren) => {
                    return compute(e, span, values, default, Some(&paren.expr))
                }

                Expr::Ident(ref id) => {
                    if let Some(Some(v)) = values.get(&id.sym) {
                        return Ok(v.clone());
                    }
                    return Err(());
                }
                Expr::Unary(ref expr) => {
                    let v = compute(e, span, values, None, Some(&expr.arg))?;
                    match v {
                        TsLit::BigInt(BigInt { .. }) => {}
                        TsLit::Number(Number { value: v, .. }) => {
                            return Ok(TsLit::Number(Number {
                                span,
                                value: match expr.op {
                                    op!(unary, "+") => v,
                                    op!(unary, "-") => -v,
                                    op!("!") => {
                                        if v == 0.0f64 {
                                            0.0
                                        } else {
                                            1.0
                                        }
                                    }
                                    op!("~") => (!(v as i32)) as f64,
                                    _ => return Err(()),
                                },
                                raw: None,
                            }))
                        }
                        TsLit::Str(_) => {}
                        TsLit::Bool(_) => {}
                        TsLit::Tpl(_) => {}
                    }
                }

                Expr::Tpl(ref t) if t.exprs.is_empty() => {
                    if let Some(v) = &t.quasis[0].cooked {
                        return Ok(TsLit::Str(Str {
                            span,
                            raw: None,
                            value: JsWord::from(&**v),
                        }));
                    }
                }

                _ => {}
            }
        } else if let Some(value) = default {
            return Ok(TsLit::Number(Number {
                span,
                value: value as _,
                raw: None,
            }));
        }

        Err(())
    }

    let id = e.id.clone();

    let mut default = 0;
    let mut values = Default::default();
    let members = e
        .members
        .clone()
        .into_iter()
        .map(|m| -> Result<_, ()> {
            let id_span = m.id.span();
            let val = compute(&e, id_span, &mut values, Some(default), m.init.as_deref())
                .map(|val| {
                    if let TsLit::Number(ref n) = val {
                        default = n.value as i64 + 1;
                    }
                    values.insert(m.id.as_ref().clone(), Some(val.clone()));

                    match val {
                        TsLit::Number(v) => Expr::Lit(Lit::Num(v)),
                        TsLit::Str(v) => Expr::Lit(Lit::Str(v)),
                        TsLit::Bool(v) => Expr::Lit(Lit::Bool(v)),
                        TsLit::Tpl(v) => {
                            let value = v.quasis.into_iter().next().unwrap().raw;

                            Expr::Lit(Lit::Str(Str {
                                span: v.span,
                                raw: None,
                                value: JsWord::from(&*value),
                            }))
                        }
                        TsLit::BigInt(v) => Expr::Lit(Lit::BigInt(v)),
                    }
                })
                .or_else(|err| match &m.init {
                    None => Err(err),
                    Some(v) => {
                        let mut v = *v.clone();
                        let mut visitor = EnumValuesVisitor {
                            previous: &values,
                            ident: &id,
                        };
                        visitor.visit_mut_expr(&mut v);

                        values.insert(m.id.as_ref().clone(), None);

                        Ok(v)
                    }
                })?;

            Ok((m, val))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|_| panic!("invalid value for enum is detected"));

    let a = members
        .into_iter()
        .map(|(m, val)| {
            let name = match m.id {
                TsEnumMemberId::Str(s) => s,
                TsEnumMemberId::Ident(i) => Str {
                    span: i.span,
                    raw: None,
                    value: i.sym,
                },
            };

            KeyValueProp {
                key: PropName::Str(name),
                value: Box::new(val)
            }
        });

    let (sym, ctx) = e.id.to_id();
    VarDecl {
        span: e.span,
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Ident::new(sym, DUMMY_SP.with_ctxt(ctx)).into(),
            definite: false,
            init: Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: a
                    .map(|prop| {
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(prop)))
                    })
                    .collect()
            })))
        }]
    }
}

struct EnumValuesVisitor<'a> {
    ident: &'a Ident,
    previous: &'a EnumValues,
}

impl VisitMut for EnumValuesVisitor<'_> {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Ident(ident) if self.previous.contains_key(&ident.sym) => {
                *expr = self.ident.clone().make_member(ident.clone());
            }
            Expr::Member(MemberExpr {
                obj,
                // prop,
                ..
            }) => {
                if let Expr::Ident(ident) = &**obj {
                    if self.previous.get(&ident.sym).is_some() {
                        **obj = self.ident.clone().make_member(ident.clone());
                    }
                }
            }
            _ => expr.visit_mut_children_with(self),
        }
    }
}
