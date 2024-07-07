use std::io::Write;

use crate::ast::SimpleComponentKind;
use crate::common::*;
use crate::types::Type;
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
        let moddef_names = self.db.package_moddef_names()?;
        for moddef in moddef_names {
            self.verilog_moddef(moddef)?;
        }
        Ok(())
    }

    fn verilog_moddef(&mut self, moddef: Ident) -> VirdantResult<()> {
        writeln!(self.writer, "module {}(", moddef.clone())?;
        let ports = self.db.moddef_port_names(moddef.clone())?;
        for (i, port) in ports.iter().enumerate() {
            let is_last = i + 1 == ports.len();
            self.verilog_port(moddef.clone(), port.clone(), is_last)?;
        }
        writeln!(self.writer, ");")?;

        for submodule_ast in self.db.moddef_submodules(moddef.clone())? {
            self.verilog_submodule(moddef.clone(), submodule_ast)?;
        }

        for component in self.db.moddef_component_names(moddef.clone())? {
            self.verilog_component(moddef.clone(), component)?;
        }

        writeln!(self.writer, "endmodule")?;
        writeln!(self.writer)?;
        Ok(())
    }

    fn verilog_port(&mut self, moddef: Ident, port: Ident, is_last_port: bool) -> VirdantResult<()> {
        let port_ast = self.db.moddef_component_ast(moddef.clone(), port.clone())?;
        let typ = self.db.moddef_component_type(moddef.clone(), port.clone())?;
//            Component::Incoming(name, typ) => {
        //
        let direction = match port_ast.kind {
            SimpleComponentKind::Incoming => "input  ",
            SimpleComponentKind::Outgoing => "output ",
            _ => unreachable!(),
        };

        if let Type::Word(1) = typ {
            write!(self.writer, "    {direction} wire            {port}")?;
        } else if let Type::Word(n) = typ {
            let max_bit = n - 1;
            let width_str = format!("[{max_bit}:0]");
            let padded_width_str = format!("{width_str: >8}");
            write!(self.writer, "    {direction} wire  {padded_width_str} {port}")?;
        } else if let Type::Clock = typ {
            write!(self.writer, "    {direction} wire            {port}")?;
        } else {
            todo!()
        }

        if is_last_port {
            writeln!(self.writer)?;
        } else {
            writeln!(self.writer, ",")?;
        }

        Ok(())
    }

    fn verilog_component(&mut self, moddef: Ident, component: Ident) -> VirdantResult<()> {
        let component_ast = self.db.moddef_component_ast(moddef.clone(), component.clone())?;
        match component_ast.kind {
            SimpleComponentKind::Incoming => (),
            SimpleComponentKind::Outgoing => {
                let expr = self.db.moddef_typecheck_wire(moddef.clone(), component.clone().as_path())?;
                let ssa = self.verilog_expr(expr)?;
                writeln!(self.writer, "    assign {component} = {ssa};")?;
            },
            SimpleComponentKind::Node => {
                let expr = self.db.moddef_typecheck_wire(moddef.clone(), component.clone().as_path())?;
                let typ = expr.typ();
                let ssa = self.verilog_expr(expr)?;
                let width_str = make_width_str(self.db, typ);
                writeln!(self.writer, "    wire {width_str} {component};")?;
                writeln!(self.writer, "    assign {component} = {ssa};")?;
            },
            SimpleComponentKind::Reg => {
                let expr = self.db.moddef_typecheck_wire(moddef.clone(), component.clone().as_path())?;
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ);
                let clk: String = todo!();
                //let clock_ssa = self.verilog_expr(clk)?;
                let connect_ssa = self.verilog_expr(expr)?;
                let width = if let Type::Word(n) = expr.typ() { n } else { panic!() };
                let max_bit = width - 1;
                writeln!(self.writer, "    reg  {width_str} {component};")?;
                writeln!(self.writer, "    always @(posedge {clk}) begin")?;
                writeln!(self.writer, "        {component} <= {connect_ssa};")?;
                writeln!(self.writer, "    end")?;
            },
        }
        Ok(())
    }

    fn verilog_submodule(&mut self, moddef: Ident, submodule: ast::Submodule) -> VirdantResult<()> {
        let ports = self.db.moddef_port_names(submodule.moddef.clone())?;

        for port in &ports {
            let typ = self.db.moddef_component_type(submodule.moddef.clone(), port.clone())?;
            let width_str = make_width_str(self.db, typ);
            writeln!(self.writer, "    wire {width_str} __TEMP_{sm}_{port};", sm = submodule.name)?;
        }

//        for Wire(path, _wire_type, expr) in &self.db.moddef_typecheck_wire(moddef.name.clone(), submodule.name.clone())? {

        for port in &ports {
            if let Ok(expr) = self.db.moddef_typecheck_wire(moddef.clone(), submodule.name.as_path().join(&port.clone().as_path())) {
                let gs = self.verilog_expr(expr)?;
                writeln!(self.writer, "    assign __TEMP_{sm}_{port} = {gs};", sm = submodule.name)?;
            }
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

    fn verilog_expr(&mut self, expr: Arc<TypedExpr>) -> VirdantResult<SsaName> {
        match expr.as_ref() {
            TypedExpr::Reference(typ, path) if path.is_local()  => Ok(format!("{}", path)),
            TypedExpr::Reference(typ, path) => {
                let parts = path.parts();
                let sm = &parts[0];
                let port = &parts[1];
                Ok(format!("__TEMP_{sm}_{port}"))
            },
            TypedExpr::Word(typ, w) => {
                let gs = self.gensym();
                let typ = expr.typ();
                let width_str: String = match typ {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = n - 1;
                        format!("[{max_bit}:0]")
                    },
                    _ => panic!(),
                };
                writeln!(self.writer, "    wire {width_str} {gs} = {};", w.value)?;
                Ok(gs)
            },
            TypedExpr::Cat(typ, args) => {
                let gs = self.gensym();
                let mut args_ssa: Vec<SsaName> = vec![];
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone())?;
                    args_ssa.push(arg_ssa);
                }
                writeln!(self.writer, "    wire {gs} = {{{}}};", args_ssa.join(", "))?;
                Ok(gs)
            },
            TypedExpr::Idx(typ, subject, index) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(subject.clone())?;
                writeln!(self.writer, "    wire {gs} = {subject_ssa}[{index}];")?;
                Ok(gs)
            },
            TypedExpr::MethodCall(typ, subject, method, args) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(subject.clone())?;
                let mut args_ssa: Vec<SsaName> = vec![];
                self.verilog_expr(subject.clone())?;
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone())?;
                    args_ssa.push(arg_ssa);
                }
                let typ = expr.typ();
                let width_str: String = match typ {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = n - 1;
                        format!("[{max_bit}:0]")
                    },
                    _ => panic!(),
                };

                match method.as_str() {
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
                    _ => panic!("Unknown method: {}", method),
                }
                Ok(gs)
            },
            TypedExpr::If(typ, c, a, b) => {
                let gs = self.gensym();
                let cond_ssa = self.verilog_expr(c.clone())?;
                let a_ssa = self.verilog_expr(a.clone())?;
                let b_ssa = self.verilog_expr(b.clone())?;
                let typ = expr.typ();
                let width_str: String = match typ {
                    Type::Word(1) => " ".to_string(),
                    Type::Word(n) => {
                        let max_bit = n - 1;
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

struct VerilogFile {
    moddefs: Vec<VerilogModDef>,
}

struct VerilogModDef {
    name: String,
    ports: Vec<(String, Width, Dir)>,
    statements: Vec<VerilogStatement>,
}

enum VerilogExpr {
    Const(Val, Width),
}

impl std::fmt::Display for VerilogExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerilogExpr::Const(val, width) => {
                write!(f, "{width}'d{val}")
            },
        }
    }
}

enum VerilogStatement {
    Assign(String, VerilogExpr),
    Wire(String, Width, Option<VerilogExpr>),
    Reg(String, Width, Option<VerilogExpr>),
    AlwaysAtPosedge(String, Vec<VerilogStatement>),
    NonblockingAssign(String, VerilogExpr),
}

enum Dir {
    Input,
    Output,
}

impl VerilogFile {
    fn write<F: Write>(&self, f: &mut F) -> std::io::Result<()> {
        for moddef in &self.moddefs {
            moddef.write(f)?;
        }
        Ok(())
    }
}

impl VerilogModDef {
    fn write<F: Write>(&self, f: &mut F) -> std::io::Result<()> {
        writeln!(f, "module {}(", self.name);
        self.write_ports(f)?;
        writeln!(f, ");");
        self.write_statements(f)?;

        writeln!(f, "endmodule");
        Ok(())
    }

    fn write_ports<F: Write>(&self, f: &mut F) -> std::io::Result<()> {
        for (i, (name, width, dir)) in self.ports.iter().enumerate() {
            let w = bit_width(*width);
            match dir {
                Dir::Input  => write!(f, "    input  wire ")?,
                Dir::Output => write!(f, "    output wire ")?,
            }
            write!(f, "{w} {name}")?;

            let last_port = i + 1 == self.ports.len();
            if last_port {
                writeln!(f)?;
            } else {
                writeln!(f, ",")?;
            }
        }
        Ok(())
    }

    fn write_statements<F: Write>(&self, f: &mut F) -> std::io::Result<()> {
        for statement in &self.statements {
            statement.write(f)?;
        }
        Ok(())
    }
}

impl VerilogStatement {
    fn write<F: Write>(&self, f: &mut F) -> std::io::Result<()> {
        self.write_with_indent(f, 1)
    }

    fn write_with_indent<F: Write>(&self, f: &mut F, indent: usize) -> std::io::Result<()> {
        let indentation = " ".repeat(4 * indent);
        write!(f, "{indentation}")?;

        match self {
            VerilogStatement::Assign(name, expr) => writeln!(f, "assign          {name} = {expr};")?,
            VerilogStatement::Wire(name, width, expr) => {
                let w = bit_width(*width);
                if let Some(expr) = expr {
                    writeln!(f, "wire {w} {name} = {expr};")?
                } else {
                    writeln!(f, "wire {w} {name};")?
                }
            },
            VerilogStatement::Reg(name, width, expr) => {
                let w = bit_width(*width);
                if let Some(expr) = expr {
                    writeln!(f, "reg  {w} {name} = {expr};")?
                } else {
                    writeln!(f, "reg  {w} {name};")?
                }
            },
            VerilogStatement::AlwaysAtPosedge(name, stmts) => {
                writeln!(f, "always @(posedge {name}) begin")?;
                for stmt in stmts {
                    stmt.write_with_indent(f, indent + 1)?;
                }
                write!(f, "{indentation}")?;
                writeln!(f, "end")?;
            },
            VerilogStatement::NonblockingAssign(name, expr) => {
                writeln!(f, "{name} <= {expr};")?;
            },
        }
        Ok(())
    }
}

fn bit_width(width: Width) -> String {
    if width == 1 {
        "          ".to_string()
    } else {
        let max_bit = width - 1;
        let width_str = format!("[{max_bit}:0]");
        let padded_width_str = format!("{width_str: >8}");
        format!(" {padded_width_str} ")
    }
}

#[test]
fn verilog_output() {
    let verilog = VerilogFile {
        moddefs: vec![
            VerilogModDef {
                name: "Top".to_string(),
                ports: vec![
                    ("clock".to_string(), 1, Dir::Input),
                    ("reset".to_string(), 1, Dir::Input),
                    ("in".to_string(), 8, Dir::Input),
                    ("out".to_string(), 8, Dir::Output),
                ],
                statements: vec![
                    VerilogStatement::Assign("out".to_string(), VerilogExpr::Const(100, 8)),
                    VerilogStatement::Reg("w".to_string(), 8, Some(VerilogExpr::Const(100, 8))),
                    VerilogStatement::Wire("x".to_string(), 7, Some(VerilogExpr::Const(100, 8))),
                    VerilogStatement::Reg("w2".to_string(), 1, None),
                    VerilogStatement::Wire("x2".to_string(), 8, None),
                    VerilogStatement::AlwaysAtPosedge(
                        "clk".to_string(),
                        vec![
                            VerilogStatement::NonblockingAssign("x2".to_string(), VerilogExpr::Const(100, 8)),
                        ],
                    ),
                ],
            },
        ],
    };
    let mut stdout = std::io::stdout();
    verilog.write(&mut stdout).unwrap();
}

fn make_width_str(db: &Db, typ: Type) -> String {
    let n = db.bitwidth(typ.clone()).unwrap();
    if n == 1 {
        "".to_string()
    } else {
        let max_bit = n - 1;
        format!("[{max_bit}:0]")
    }
}
