use crate::common::*;
use crate::ast;
use crate::virdant_error;
use super::*;

#[salsa::query_group(LayoutQStorage)]
pub trait LayoutQ: typecheck::TypecheckQ {
    fn bitwidth(&self, typ: Type) -> VirdantResult<Width>;

    fn union_ctor_tag(&self, typ: Type, ctor: Ident) -> VirdantResult<u64>;
    fn union_layout(&self, typ: Type) -> VirdantResult<UnionLayout>;
}

fn bitwidth(db: &dyn LayoutQ, typ: Type) -> VirdantResult<Width> {
    match typ.clone() {
        Type::Clock => Ok(1),
        Type::Bool => Ok(1),
        Type::Word(n) => Ok(n.into()),
        Type::Struct(_structdef, _typ_args) => todo!(),
        Type::Union(uniondef, _typ_args) => {
            let uniondef_ast = db.uniondef_ast(uniondef.clone())?;

            let mut payload_width = 0;
            for ast::Alt(_ctor, ast_arg_typs) in &uniondef_ast.alts {
                let mut resolved_arg_typs = vec![];
                for ast_arg_typ in ast_arg_typs {
                    let resolved_typ = db.resolve_typ(ast_arg_typ.clone(), uniondef.package())?;
                    resolved_arg_typs.push(resolved_typ);
                }

                let mut width = 0;
                for resolved_arg_typ in &resolved_arg_typs {
                    width += db.bitwidth(resolved_arg_typ.clone())?;
                }

                if width > payload_width {
                    payload_width = width;
                }
            }
            //let tag_width = db.alttypedef_tag_bitwidth(typ.clone())?;
            let layout = db.union_layout(typ.clone())?;
            let width = layout.tag_width + payload_width;
            Ok(width)
        },
    }
}

fn union_layout(db: &dyn LayoutQ, typ: Type) -> VirdantResult<UnionLayout> {
    let uniondef = if let Type::Union(uniondef, _typ_args) = typ {
        uniondef
    } else {
        todo!()
    };
    let alttypedef_ast = db.uniondef_ast(uniondef.clone())?;
    let tag_width = clog2(alttypedef_ast.alts.len() as u64);

    let mut slots_by_ctor: Vec<(Ident, CtorSlots)> = vec![];
    for ast::Alt(ctor, arg_typs) in &alttypedef_ast.alts {
        let mut slots = CtorSlots::default();
        for arg_typ in arg_typs {
            let resolved_arg_typ = db.resolve_typ(arg_typ.clone(), uniondef.package())?;
            let arg_typ_bitwidth = db.bitwidth(resolved_arg_typ)?;
            slots.add(arg_typ_bitwidth);
        }
        slots_by_ctor.push((ctor.clone(), slots))
    }

    let slots_by_ctor = slots_by_ctor.into_iter().collect();

    let layout = UnionLayout {
        tag_width,
        slots: slots_by_ctor,
    };
    Ok(layout)
}

fn union_ctor_tag(db: &dyn LayoutQ, typ: Type, ctor: Ident) -> VirdantResult<u64> {
    let uniondef = if let Type::Union(uniondef, _typ_args) = typ {
        uniondef
    } else {
        todo!()
    };

    let alttypedef_ast = db.uniondef_ast(uniondef)?;
    for (tag, ast::Alt(ctor_name, _)) in alttypedef_ast.alts.iter().enumerate() {
        if ctor_name == &ctor {
            return Ok(tag.try_into().unwrap());
        }
    }
    Err(virdant_error!("Unknown ctor: {ctor}"))
}

fn clog2(n: u64) -> u64 {
    let mut result = 0;
    while n > (1 << result) {
        result += 1;
    }
    result
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct UnionLayout {
    tag_width: Width,
    slots: Vec<(Ident, CtorSlots)>,
}

impl UnionLayout {
    pub fn width(&self) -> Width {
        self.tag_width + self.payload_width()
    }

    pub fn payload_width(&self) -> Width {
        let mut payload_width = 0;
        for ctor in self.slots.iter().map(|(ctor, _)| ctor) {
            let ctor_payload_width = self.ctor_payload_width(ctor.clone());
            if ctor_payload_width > payload_width {
                payload_width = ctor_payload_width;
            }
        }
        payload_width
    }

    pub fn tag_width(&self) -> Width {
        self.tag_width
    }

    pub fn tag_for(&self, ctor: Ident) -> Tag {
        for (tag, (ctor_name, _slots)) in self.slots.iter().enumerate() {
            if ctor_name == &ctor {
                return tag as Tag;
            }
        }

        panic!("No ctor found: {ctor}")
    }

    pub fn ctor_slots(&self, ctor: Ident) -> Vec<(Offset, Width)> {
        for (_ctor_name, slots) in &self.slots {
            let mut results = vec![];
            for i in 0..slots.0.len() {
                results.push(self.ctor_slot(ctor.clone(), i));
            }
        return results;
        }
        panic!("No ctor found: {ctor}")
    }

    pub fn ctor_slot(&self, ctor: Ident, slot: usize) -> (Offset, Width) {
        for (ctor_name, slots) in &self.slots {
            if ctor_name == &ctor {
                let mut offset = self.tag_width;
                for i in 0..slot {
                    offset += slots.0[i];
                }
                return (offset, slots.0[slot]);
            }
        }
        panic!("No ctor found: {ctor}")
    }

    pub fn ctor_payload_width(&self, ctor: Ident) -> Width {
        for (ctor_name, slots) in &self.slots {
            if ctor_name == &ctor {
                return slots.width();
            }
        }
        panic!("No ctor found: {ctor}")
    }

    pub fn ctor_width(&self, ctor: Ident) -> Width {
        for (ctor_name, slots) in &self.slots {
            if ctor_name == &ctor {
                return self.tag_width + slots.width();
            }
        }
        panic!("No ctor found: {ctor}")
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CtorSlots(Vec<Width>);

impl CtorSlots {
    fn add(&mut self, width: Width) {
        self.0.push(width)
    }

    fn width(&self) -> Width {
        self.0.iter().sum()
    }
}
