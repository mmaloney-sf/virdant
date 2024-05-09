use std::io::Write;

use crate::types::Type;
use crate::hir::*;

type SsaName = String;

impl Package {
    pub fn verilog<F: Write>(&self, writer: &mut F) -> std::io::Result<()> {
        let mut verilog = Verilog {
            writer,
            gensym: 0,
        };

        verilog.verilog_package::<dyn Write>(self)?;
        Ok(())
    }
}

struct Verilog<'a> {
    writer: &'a mut dyn Write,
    gensym: usize,
}

impl<'a> Verilog<'a> {
    fn verilog_package<F: Write + ?Sized>(&mut self, package: &Package) -> std::io::Result<()> {
        for moddef in package.moddefs.values() {
            self.verilog_moddef(moddef)?;
        }
        Ok(())
    }

    fn verilog_moddef(&mut self, moddef: &ModDef) -> std::io::Result<()> {
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
            self.verilog_submodule(submodule)?;
        }

        for component in &moddef.components {
            self.verilog_component(component)?;
        }

        writeln!(self.writer, "endmodule")?;
        writeln!(self.writer)?;
        Ok(())
    }

    fn verilog_port(&mut self, component: &Component, is_last_port: bool) -> std::io::Result<()> {
        match component {
            Component::Incoming(name, typ) => {
                if let Type::Word(1) = typ.as_ref() {
                    write!(self.writer, "    input  wire            {name}")?;
                } else if let Type::Word(n) = typ.as_ref() {
                    let max_bit = n + 1;
                    let width_str = format!("[{max_bit}:0]");
                    let padded_width_str = format!("{width_str: >8}");
                    write!(self.writer, "    input  wire {padded_width_str} {name}")?;
                } else if let Type::Clock = typ.as_ref() {
                    write!(self.writer, "    input  wire          {name}")?;
                } else {
                    todo!()
                }
            },
            Component::Outgoing(name, typ, _expr) => {
                if let Type::Word(1) = typ.as_ref() {
                    write!(self.writer, "    output wire             {name}")?;
                } else if let Type::Word(n) = typ.as_ref() {
                    let max_bit = n + 1;
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

    fn verilog_component(&mut self, component: &Component) -> std::io::Result<()> {
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
                let clock_ssa = self.verilog_expr(clk)?;
                let connect_ssa = self.verilog_expr(&expr)?;
                writeln!(self.writer, "    reg  [31:0] {name};")?;
                writeln!(self.writer, "    always @(posedge clk) begin")?;
                writeln!(self.writer, "        {name} <= {connect_ssa};")?;
                writeln!(self.writer, "    end")?;
            },
        }
        Ok(())
    }

    fn verilog_submodule(&mut self, submodule: &Submodule) -> std::io::Result<()> {
        writeln!(self.writer, "    {} {}();", submodule.moddef, submodule.name)?;
        Ok(())
    }

    /*
    fn verilog_type(&mut self, typ: Arc<Type>) -> std::io::Result<()> {
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

    fn verilog_expr(&mut self, expr: &Expr) -> std::io::Result<SsaName> {
        match expr.as_node() {
            ExprNode::Reference(r) => Ok(format!("{}", r.path())),
            ExprNode::Word(w) => {
                let gs = self.gensym();
                writeln!(self.writer, "    wire [31:0] {gs} = {};", w.value())?;
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
                match m.method().as_str() {
                    "add" => {
                        writeln!(self.writer, "    wire [31:0] {gs} = {subject_ssa} + {};", args_ssa.join(", "))?;
                    },
                    _ => panic!(),
                }
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
