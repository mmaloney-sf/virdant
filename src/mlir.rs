use std::sync::Arc;
use std::io::Write;

use crate::types::Type;
use crate::hir::*;

type SsaName = String;

impl Package {
    pub fn mlir<F: Write>(&self, writer: &mut F) -> std::io::Result<()> {
        let mut mlir = Mlir {
            writer,
            gensym: 0,
        };

        mlir.mlir_package::<dyn Write>(self)?;
        Ok(())
    }
}

struct Mlir<'a> {
    writer: &'a mut dyn Write,
    gensym: usize,
}

impl<'a> Mlir<'a> {
    fn mlir_package<F: Write + ?Sized>(&mut self, package: &Package) -> std::io::Result<()> {
        for moddef in package.moddefs.values() {
            self.mlir_moddef(moddef)?;
        }
        Ok(())
    }

    fn mlir_moddef(&mut self, moddef: &ModDef) -> std::io::Result<()> {
        writeln!(self.writer, "virdant.module {} {{", moddef.name)?;
        for component in &moddef.components {
            self.mlir_component(component)?;

        }
        for submodule in &moddef.submodules {
            self.mlir_submodule(submodule)?;
        }
        writeln!(self.writer, "}}")?;
        Ok(())
    }

    fn mlir_component(&mut self, component: &Component) -> std::io::Result<()> {
        match component {
            Component::Incoming(name, typ) => {
                write!(self.writer, "    %{name} = virdant.incoming() : ")?;
                self.mlir_type(typ.clone())?;
                writeln!(self.writer)?;
            },
            Component::Outgoing(name, _typ, expr) => {
                let ssa = self.mlir_expr(&expr)?;
                writeln!(self.writer, "    %{name} = virdant.outgoing({ssa})")?;
            },
            Component::Wire(name, typ, expr) => {
                let ssa = self.mlir_expr(&expr)?;
                write!(self.writer, "    %{name} = virdant.wire({ssa}) : ")?;
                self.mlir_type(typ.clone())?;
                writeln!(self.writer)?;
            },
            Component::Reg(name, typ, clk, /* rst, */ expr) => {
                let clock_ssa = self.mlir_expr(clk)?;
                let connect_ssa = self.mlir_expr(&expr)?;
                write!(self.writer, "    %{name} = virdant.reg({clock_ssa}, {connect_ssa}) : ")?;
                self.mlir_type(typ.clone())?;
                writeln!(self.writer)?;
            },
        }
        Ok(())
    }

    fn mlir_submodule(&mut self, submodule: &Submodule) -> std::io::Result<()> {
        writeln!(self.writer, "    %{} = virdant.submodule @{}", submodule.name, submodule.moddef)?;
        Ok(())
    }

    fn mlir_type(&mut self, typ: Arc<Type>) -> std::io::Result<()> {
        match typ.as_ref() {
            Type::Clock => write!(self.writer, "!virdant.clock")?,
            Type::Word(n) => write!(self.writer, "!virdant.word<{n}>")?,
            Type::Vec(typ, n) => {
                write!(self.writer, "!virdant.vec<")?;
                self.mlir_type(typ.clone())?;
                write!(self.writer, ", {n}>")?;
            },
            _ => todo!(),
        }
        Ok(())
    }

    fn mlir_expr(&mut self, expr: &Expr) -> std::io::Result<SsaName> {
        match expr.as_node() {
            ExprNode::Reference(r) => Ok(format!("%{}", r.path())),
            ExprNode::Word(w) => {
                let gs = self.gensym();
                writeln!(self.writer, "    {gs} = virdant.const {{ value = {} }}", w.value())?;
                Ok(gs)
            },
            ExprNode::MethodCall(m) => {
                let gs = self.gensym();
                let subject_ssa = self.mlir_expr(&m.subject())?;
                let mut args_ssa: Vec<SsaName> = vec![];
                self.mlir_expr(&m.subject())?;
                for arg in &m.args() {
                    let arg_ssa = self.mlir_expr(arg)?;
                    args_ssa.push(arg_ssa);
                }
                writeln!(self.writer, "    {gs} = virdant.methodcall({subject_ssa}, {}) {{ method = \"{}\" }}", args_ssa.join(", "), m.method())?;
                Ok(gs)
            },
            _ => {
                let gs = self.gensym();
                writeln!(self.writer, "    {gs} = ...{expr:?}")?;
                Ok(gs)
            },
        }
    }

    fn gensym(&mut self) -> SsaName {
        self.gensym += 1;
        format!("%{}", self.gensym)
    }
}
