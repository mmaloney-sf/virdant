use crate::common::*;
use crate::ast;
use crate::virdant_error;
use super::*;

#[salsa::query_group(TypeResolutionQStorage)]
pub trait TypeResolutionQ: item_dependency::ItemDependencyQ {
    fn resolve_typ(&self, typ: Arc<ast::Type>, from: PackageId) -> VirdantResult<Type>;

    fn method_sig(&self, typ: Type, method: Ident) -> VirdantResult<MethodSig>;
    fn ctor_sig(&self, typ: Type, ctor: Ident) -> VirdantResult<CtorSig>;

    fn component_typ(&self, component_id: ComponentId) -> VirdantResult<Type>;
}

fn resolve_typ(db: &dyn TypeResolutionQ, typ: Arc<ast::Type>, from: PackageId) -> VirdantResult<Type> {
    Ok(match typ.as_ref() {
        ast::Type::Clock => Type::Clock,
        ast::Type::Word(w) => Type::Word(*w),
        ast::Type::Vec(_, _) => todo!(),
        ast::Type::TypeRef(path) => {
            let typ_args = vec![];

            match db.item(path.clone(), from)? {
                ItemId::ModDef(_) => todo!(),
                ItemId::UnionDef(uniondef) => Type::Union(uniondef, typ_args),
                ItemId::StructDef(structdef) => Type::Struct(structdef, typ_args),
                ItemId::PortDef(_) => todo!(),
            }
        },
    })
}

fn method_sig(_db: &dyn TypeResolutionQ, typ: Type, method: Ident) -> VirdantResult<MethodSig> {
    match typ {
        Type::Word(_n) => {
            if method == "add".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "inc".into() {
                Ok(MethodSig(vec![], typ.clone()))
            } else if method == "sll".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "srl".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "sub".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "and".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "or".into() {
                Ok(MethodSig(vec![typ.clone()], typ.clone()))
            } else if method == "lt".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "lte".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "gt".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "gte".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "eq".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "neq".into() {
                Ok(MethodSig(vec![typ.clone()], Type::Word(1)))
            } else if method == "not".into() {
                Ok(MethodSig(vec![], typ.clone()))
            } else {
                Err(VirdantError::Other(format!("No such method {method} for type {typ}")))
            }
        },
        _ => Err(VirdantError::Other(format!("No such method {method} for type {typ}"))),
    }
}

fn ctor_sig(db: &dyn TypeResolutionQ, typ: Type, ctor: Ident) -> VirdantResult<CtorSig> {
    let uniondef = if let Type::Union(uniondef, _args) = &typ {
        uniondef
    } else {
        return Err(virdant_error!("Type is not a union {typ}"));
    };

    let uniondef_ast = db.uniondef_ast(uniondef.clone())?;

    for ast::Alt(ctor_name, ast_arg_typs) in &uniondef_ast.alts {
        if ctor_name == &ctor {
            let mut arg_typs = vec![];
            for ast_arg_typ in ast_arg_typs {
                let resolved_arg_typ = db.resolve_typ(ast_arg_typ.clone(), uniondef.package())?;
                arg_typs.push(resolved_arg_typ);
            }
            return Ok(CtorSig(arg_typs, typ));
        }
    }

    Err(virdant_error!("Unknown ctor: {ctor} on type {typ}"))
}

fn component_typ(db: &dyn TypeResolutionQ, component_id: ComponentId) -> VirdantResult<Type> {
    let moddef_ast = db.moddef_ast(component_id.moddef()).unwrap();

    for decl in &moddef_ast.decls {
        if let ast::Decl::Component(simplecomponent) = decl {
            if simplecomponent.name == component_id.name() {
                let typ = db.resolve_typ(simplecomponent.typ.clone(), component_id.moddef().package())?;
                return Ok(typ);
            }
        }
    }

    Err(virdant_error!("Unknown component: {component_id} could not resolve type"))
}
