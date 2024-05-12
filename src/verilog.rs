use std::io::Write;

use crate::common::*;
use crate::types::Type;
use crate::hir::*;
use crate::db::*;
use crate::ast;

type SsaName = String;

impl Db {
    pub fn verilog<F: Write>(&self, writer: &mut F) -> VirdantResult<()> {
        let mut verilog = Verilog {
            writer,
            db: self,
            gensym: 0,
        };

        verilog.verilog_package()?;
        Ok(())
    }
}

struct Verilog<'a> {
    writer: &'a mut dyn Write,
    db: &'a Db,
    gensym: usize,
}

impl<'a> Verilog<'a> {
    fn verilog_package(&mut self) -> VirdantResult<()> {
        let package = self.db.package_hir()?;
        for moddef in package.moddefs.values() {
            self.verilog_moddef(moddef)?;
        }
        Ok(())
    }

    fn verilog_moddef(&mut self, moddef: &ModDef) -> VirdantResult<()> {
        writeln!(self.writer, "module {}(", moddef.name)?;
        let ports = moddef.ports();
        for (i, port) in ports.iter().enumerate() {
            let is_last = i + 1 == ports.len();
            match port {
                Component::Incoming(..) => self.verilog_port(port, is_last)?,
                Component::Outgoing(..) => self.verilog_port(port, is_last)?,
                _ => (),
            }
        }
        writeln!(self.writer, ");")?;

        for submodule in &moddef.submodules {
            self.verilog_submodule(moddef, submodule)?;
        }

        for component in &moddef.components {
            self.verilog_component(component)?;
        }

        writeln!(self.writer, "endmodule")?;
        writeln!(self.writer)?;
        Ok(())
    }

    fn verilog_port(&mut self, component: &Component, is_last_port: bool) -> VirdantResult<()> {
        match component {
            Component::Incoming(name, typ) => {
                if let Type::Word(1) = typ.as_ref() {
                    write!(self.writer, "    input  wire            {name}")?;
                } else if let Type::Word(n) = typ.as_ref() {
                    let max_bit = n - 1;
                    let width_str = format!("[{max_bit}:0]");
                    let padded_width_str = format!("{width_str: >8}");
                    write!(self.writer, "    input  wire  {padded_width_str} {name}")?;
                } else if let Type::Clock = typ.as_ref() {
                    write!(self.writer, "    input  wire            {name}")?;
                } else {
                    todo!()
                }
            },
            Component::Outgoing(name, typ, _expr) => {
                if let Type::Word(1) = typ.as_ref() {
                    write!(self.writer, "    output wire            {name}")?;
                } else if let Type::Word(n) = typ.as_ref() {
                    let max_bit = n - 1;
                    let width_str = format!("[{max_bit}:0]");
                    let padded_width_str = format!("{width_str: >8}");
                    write!(self.writer, "    output wire {padded_width_str} {name}")?;
                } else if let Type::Clock = typ.as_ref() {
                    write!(self.writer, "    output wire          {name}")?;
                } else {
                    todo!()
                }
            },
            _ => panic!(),
        }

        if is_last_port {
            writeln!(self.writer)?;
        } else {
            writeln!(self.writer, ",")?;
        }

        Ok(())
    }

    fn verilog_component(&mut self, component: &Component) -> VirdantResult<()> {
        match component {
            Component::Incoming(_name, _typ) => (),
            Component::Outgoing(name, _typ, expr) => {
                let ssa = self.verilog_expr(&expr)?;
                writeln!(self.writer, "    assign {name} = {ssa};")?;
            },
            Component::Wire(name, _typ, expr) => {
                let ssa = self.verilog_expr(&expr)?;
                writeln!(self.writer, "    wire [31:0] {name};")?;
                writeln!(self.writer, "    assign {name} = {ssa};")?;
            },
            Component::Reg(name, typ, clk, /* rst, */ expr) => {
                //let clock_ssa = self.verilog_expr(clk)?;
                let connect_ssa = self.verilog_expr(&expr)?;
                let width = if let Type::Word(n) = typ.as_ref() { n } else { panic!() };
                let max_bit = width - 1;
                writeln!(self.writer, "    reg  [{max_bit}:0] {name};")?;
                writeln!(self.writer, "    always @(posedge {clk}) begin")?;
                writeln!(self.writer, "        {name} <= {connect_ssa};")?;
                writeln!(self.writer, "    end")?;
            },
        }
        Ok(())
    }

    fn verilog_submodule(&mut self, moddef: &ModDef, submodule: &Submodule) -> VirdantResult<()> {
        let mut ports = vec![];

        let moddef_hir = self.db.moddef_hir(submodule.moddef.clone())?;
        for component in &moddef_hir.components {
            match component {
                Component::Outgoing(name, _typ, _expr) => ports.push(name.clone()),
                Component::Incoming(name, _typ) => ports.push(name.clone()),
                _ => (),
            }
        }

        for Connect(path, _connect_type, expr) in &self.db.moddef_submodule_connects_typed(moddef.name.clone(), submodule.name.clone())? {
            let gs = self.verilog_expr(&expr)?;
            let parts = path.parts();
            let name = parts[1];
            writeln!(self.writer, "    assign __TEMP_{sm}_{name} = {gs};", sm = submodule.name)?;
        }

//        let width = todo!(); // TODO
        for port in &ports {
            writeln!(self.writer, "    wire [31:0] __TEMP_{sm}_{port};", sm = submodule.name)?;
        }

        writeln!(self.writer, "    {} {}(", submodule.moddef, submodule.name)?;
        for (i, port) in ports.iter().enumerate() {
            let last_port = i + 1 == ports.len();
            write!(self.writer, "        .{port}(__TEMP_{sm}_{port})", sm = submodule.name)?;
            if last_port {
                writeln!(self.writer)?;
            } else {
                writeln!(self.writer, ",")?;
            }
        }
        writeln!(self.writer, "    );")?;
        Ok(())
    }

    /*
    fn verilog_type(&mut self, typ: Arc<Type>) -> VirdantResult<()> {
        match typ.as_ref() {
            Type::Clock => write!(self.writer, "!virdant.clock")?,
            Type::Word(n) => write!(self.writer, "!virdant.word<{n}>")?,
            Type::Vec(typ, n) => {
                write!(self.writer, "!virdant.vec<")?;
                self.verilog_type(typ.clone())?;
                write!(self.writer, ", {n}>")?;
            },
            _ => todo!(),
        }
        Ok(())
    }
    */

    fn verilog_expr(&mut self, expr: &Expr) -> VirdantResult<SsaName> {
        match expr.as_node() {
            ExprNode::Reference(r) if r.path().is_local()  => Ok(format!("{}", r.path())),
            ExprNode::Reference(r) => {
                let parts = r.path().parts();
                let sm = &parts[0];
                let port = &parts[1];
                Ok(format!("__TEMP_{sm}_{port}"))
            },
            ExprNode::Word(w) => {
                let gs = self.gensym();
                let typ = expr.type_of().unwrap();
                let width_str: String = match typ.as_ref() {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = *n - 1;
                        format!("[{max_bit}:0]")
                    },
                    _ => panic!(),
                };
                writeln!(self.writer, "    wire {width_str} {gs} = {};", w.value())?;
                Ok(gs)
            },
            ExprNode::Cat(c) => {
                let gs = self.gensym();
                let mut args_ssa: Vec<SsaName> = vec![];
                for arg in &c.subexprs() {
                    let arg_ssa = self.verilog_expr(arg)?;
                    args_ssa.push(arg_ssa);
                }
                writeln!(self.writer, "    wire {gs} = {{{}}};", args_ssa.join(", "))?;
                Ok(gs)
            },
            ExprNode::Idx(i) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(&i.subject())?;
                let index = i.index();
                writeln!(self.writer, "    wire {gs} = {subject_ssa}[{index}];")?;
                Ok(gs)
            },
            ExprNode::MethodCall(m) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(&m.subject())?;
                let mut args_ssa: Vec<SsaName> = vec![];
                self.verilog_expr(&m.subject())?;
                for arg in &m.args() {
                    let arg_ssa = self.verilog_expr(arg)?;
                    args_ssa.push(arg_ssa);
                }
                let typ = expr.type_of().unwrap();
                let width_str: String = match typ.as_ref() {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = *n - 1;
                        format!("[{max_bit}:0]")
                    },
                    _ => panic!(),
                };

                match m.method().as_str() {
                    "add" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} + {};", args_ssa[0])?,
                    "sub" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} - {};", args_ssa[0])?,
                    "and" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} & {};", args_ssa[0])?,
                    "or"  => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} | {};", args_ssa[0])?,
                    "not" => writeln!(self.writer, "    wire {width_str} {gs} = ~{subject_ssa};")?,
                    "eq"  => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} == {};", args_ssa[0])?,
                    "mux" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} ? {};", args_ssa.join(" : "))?,
                    "sll" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} << {};", args_ssa.join(" : "))?,
                    "srl" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} >> {};", args_ssa.join(" : "))?,
                    "lt"  => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} < {};", args_ssa.join(" : "))?,
                    "lte" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} <= {};", args_ssa.join(" : "))?,
                    "gt"  => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} > {};", args_ssa.join(" : "))?,
                    "gte" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} >= {};", args_ssa.join(" : "))?,
                    _ => panic!("Unknown method: {}", m.method()),
                }
                Ok(gs)
            },
            ExprNode::If(ifexpr) => {
                let gs = self.gensym();
                let cond_ssa = self.verilog_expr(&ifexpr.condition())?;
                let a_ssa = self.verilog_expr(ifexpr.a())?;
                let b_ssa = self.verilog_expr(ifexpr.b())?;
                let typ = expr.type_of().unwrap();
                let width_str: String = match typ.as_ref() {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = *n - 1;
                        format!("[{max_bit}:0]")
                    },
                    _ => panic!(),
                };
                writeln!(self.writer, "    wire {width_str} {gs} = {cond_ssa} ? {a_ssa} : {b_ssa};")?;
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
        format!("__TEMP_{}", self.gensym)
    }
}

impl ModDef {
    fn ports(&self) -> Vec<&Component> {
        let mut result = vec![];

        for component in &self.components {
            match component {
                Component::Incoming(..) => result.push(component),
                Component::Outgoing(..) => result.push(component),
                _ => (),
            }
        }

        result
    }
}
