use crate::common::*;
use crate::ast;
use super::*;

#[salsa::query_group(TypeResolutionQStorage)]
pub trait TypeResolutionQ: item_resolution::ItemResolutionQ {
    fn typ(&self, typ: Arc<ast::Type>, from: Package) -> VirdantResult<Type>;
}

fn typ(db: &dyn TypeResolutionQ, typ: Arc<ast::Type>, from: Package) -> VirdantResult<Type> {
    Ok(match typ.as_ref() {
        ast::Type::Clock => Type::Clock,
        ast::Type::Word(w) => Type::Word(*w),
        ast::Type::Vec(_, _) => todo!(),
        ast::Type::TypeRef(path) => {
            let typ_args = vec![];

            match db.item(path.clone(), from)? {
                Item::Package(_) => todo!(),
                Item::ModDef(_) => todo!(),
                Item::UnionDef(uniondef) => Type::Union(uniondef, typ_args),
                Item::StructDef(structdef) => Type::Struct(structdef, typ_args),
                Item::PortDef(_) => todo!(),
            }
        },
    })
}
