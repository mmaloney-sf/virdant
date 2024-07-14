use crate::{ast, common::*};
use super::*;

#[salsa::query_group(TypeResolutionQStorage)]
pub trait TypeResolutionQ: item_resolution::ItemResolutionQ {
    fn typ(&self, path: Path, typ_args: Vec<TypeArg>, from: Package) -> VirdantResult<Type>;
}

fn typ(db: &dyn TypeResolutionQ, path: Path, typ_args: Vec<TypeArg>, from: Package) -> VirdantResult<Type> {
    if path == "Word".into() {
        if typ_args.len() != 1 {
            return Err(VirdantError::Other(format!("Word takes exactly 1 type argument, but given: {typ_args:?}")));
        }

        if let TypeArg::Nat(w) = &typ_args[0] {
            Ok(Type::Word(*w))
        } else {
            Err(VirdantError::Other(format!("Word takes exactly 1 type argument, but given: {typ_args:?}")))
        }
    } else if path == "Clock".into() {
        if typ_args.len() != 0 {
            return Err(VirdantError::Other(format!("Clock takes exactly 0 type arguments, but given: {typ_args:?}")));
        }

        Ok(Type::Clock)
    } else {
        if typ_args.len() != 0 {
            todo!()
        }

        Ok(match db.item(path, from)? {
            Item::Package(_) => todo!(),
            Item::ModDef(_) => todo!(),
            Item::UnionDef(uniondef) => Type::Union(uniondef, typ_args),
            Item::StructDef(structdef) => Type::Struct(structdef, typ_args),
            Item::PortDef(_) => todo!(),
        })
    }
}
