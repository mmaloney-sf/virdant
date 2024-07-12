use std::io::Write;

use crate::ast::SimpleComponentKind;
use crate::common::*;
use crate::types::Type;
use crate::db::*;
use crate::ast;
use crate::context::Context;

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
                let typ = expr.typ();
                writeln!(self.writer, "    // outgoing {component} : {typ}")?;
                let ssa = self.verilog_expr(expr, Context::empty())?;
                writeln!(self.writer, "    assign {component} = {ssa};")?;
                writeln!(self.writer)?;
            },
            SimpleComponentKind::Node => {
                let expr = self.db.moddef_typecheck_wire(moddef.clone(), component.clone().as_path())?;
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    // node {component} : {typ}")?;
                writeln!(self.writer, "    wire {width_str} {component};")?;
                let ssa = self.verilog_expr(expr, Context::empty())?;
                writeln!(self.writer, "    assign {component} = {ssa};")?;
                writeln!(self.writer)?;
            },
            SimpleComponentKind::Reg => {
                let expr = self.db.moddef_typecheck_wire(moddef.clone(), component.clone().as_path())?;
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    // reg {component} : {typ}")?;
                writeln!(self.writer, "    reg  {width_str} {component};")?;
                let clk = component_ast.clock.unwrap();
                let connect_ssa = self.verilog_expr(expr.clone(), Context::empty())?;
                writeln!(self.writer, "    always @(posedge {clk}) begin")?;
                writeln!(self.writer, "        {component} <= {connect_ssa};")?;
                writeln!(self.writer, "    end")?;
                writeln!(self.writer)?;
                writeln!(self.writer, "    initial begin")?;
                writeln!(self.writer, "        {component} <= 0;")?;
                writeln!(self.writer, "    end")?;
                writeln!(self.writer)?;
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
                let gs = self.verilog_expr(expr, Context::empty())?;
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

    fn verilog_expr(&mut self, expr: Arc<TypedExpr>, ctx: Context<Path, SsaName>) -> VirdantResult<SsaName> {
        match expr.as_ref() {
            TypedExpr::Reference(_typ, path) => {
                if path.is_local() {
                    if let Some(ssa) = ctx.lookup(path) {
                        Ok(format!("{ssa}"))
                    } else {
                        Ok(format!("{path}"))
                    }
                } else {
                    let parts = path.parts();
                    let sm = &parts[0];
                    let port = &parts[1];
                    Ok(format!("__TEMP_{sm}_{port}"))
                }
            },
            TypedExpr::Word(_typ, w) => {
                let gs = self.gensym();
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ);
                writeln!(self.writer, "    wire {width_str} {gs} = {};", w.value)?;
                Ok(gs)
            },
            TypedExpr::Cat(_typ, args) => {
                let gs = self.gensym();
                let mut args_ssa: Vec<SsaName> = vec![];
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone(), ctx.clone())?;
                    args_ssa.push(arg_ssa);
                }
                writeln!(self.writer, "    wire {gs} = {{{}}};", args_ssa.join(", "))?;
                Ok(gs)
            },
            TypedExpr::Idx(_typ, subject, i) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(subject.clone(), ctx)?;
                writeln!(self.writer, "    wire {gs} = {subject_ssa}[{i}];")?;
                Ok(gs)
            },
            TypedExpr::IdxRange(typ, subject, j, i) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(subject.clone(), ctx)?;
                let end = *j - 1;
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa}[{end}:{i}];")?;
                Ok(gs)
            },
            TypedExpr::MethodCall(_typ, subject, method, args) => {
                let gs = self.gensym();
                let subject_ssa = self.verilog_expr(subject.clone(), ctx.clone())?;
                let mut args_ssa: Vec<SsaName> = vec![];
                self.verilog_expr(subject.clone(), ctx.clone())?;
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone(), ctx.clone())?;
                    args_ssa.push(arg_ssa);
                }
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ.clone());

                match method.as_str() {
                    "add" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} + {};", args_ssa[0])?,
                    "inc" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} + 1;")?,
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
            TypedExpr::Ctor(typ, ctor, args) => {
                let gs = self.gensym();

                let layout = self.db.alttype_layout(typ.clone())?;

                let mut args_ssa: Vec<SsaName> = vec![];
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone(), ctx.clone())?;
                    args_ssa.push(arg_ssa);
                }

                let width_str = make_width_str(self.db, typ.clone());

                let tag = self.db.alttypedef_ctor_tag(typ.clone(), ctor.clone())?;
                let fill = "1";

                write!(self.writer, "    wire {width_str} {gs} = {{ ")?;
                for arg_ssa in &args_ssa {
                    write!(self.writer, "{arg_ssa}, ")?;
                }
                let tag_width = layout.tag_width();
                writeln!(self.writer, "{tag_width}'d{tag} }};")?;

                if layout.ctor_width(ctor.clone()) < layout.width() {
                    let bot_bit = layout.ctor_width(ctor.clone());
                    let top_bit = layout.width() - 1;
                    let bits = top_bit - bot_bit + 1;

                    writeln!(self.writer, "    // fill remaining space with {fill}")?;
                    writeln!(self.writer, "    assign {gs}[{top_bit}:{bot_bit}] = {bits}'b{};", fill.repeat(bits as usize))?;
                }

                Ok(gs)
            },
            TypedExpr::As(_typ, subject, _typ_ast) => {
                self.verilog_expr(subject.clone(), ctx.clone())
            },
            TypedExpr::If(_typ, c, a, b) => {
                let gs = self.gensym();
                let cond_ssa = self.verilog_expr(c.clone(), ctx.clone())?;
                let a_ssa = self.verilog_expr(a.clone(), ctx.clone())?;
                let b_ssa = self.verilog_expr(b.clone(), ctx.clone())?;
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    wire {width_str} {gs} = {cond_ssa} ? {a_ssa} : {b_ssa};")?;
                Ok(gs)
            },
            TypedExpr::Let(typ, x, _ascription, e, b) => {
                let gs = self.gensym();
                let e_ssa = self.verilog_expr(e.clone(), ctx.clone())?;
                let new_ctx = ctx.extend(x.as_path(), e_ssa);
                let b_ssa = self.verilog_expr(b.clone(), new_ctx)?;
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    wire {width_str} {gs} = {b_ssa};")?;
                Ok(gs)
            },
            TypedExpr::Match(_typ, subject, _ascription, arms) => {
                let gs = self.gensym_hint("match");
                let subject_ssa = self.verilog_expr(subject.clone(), ctx.clone())?;
                let typ = expr.typ();
                let layout = self.db.alttype_layout(subject.typ())?;
                let width_str = make_width_str(self.db, typ.clone());

                let tag_ssa = self.gensym();
                let tag_width = layout.tag_width();
                let tag_top = tag_width - 1;

                let mut arm_ssas: Vec<(Tag, Ident, SsaName)> = vec![];
                writeln!(self.writer, "    // match arm")?;
                for TypedMatchArm(pat, e) in arms {
                    match pat {
                        TypedPat::At(_typ, ctor, pats) => {
                            writeln!(self.writer, "    // case {ctor}")?;
                            let tag = layout.tag_for(ctor.clone());
                            let mut new_ctx = ctx.clone();
                            writeln!(self.writer, "    // (pats are {pats:?})")?;
                            for (i, pat) in pats.iter().enumerate() {
                                let (offset, width) = layout.ctor_slot(ctor.clone(), i);
                                let width_minus_1 = width - 1;
                                if let TypedPat::Bind(_typ, x) = pat {
                                    let x_ssa = self.gensym_hint(&x.to_string());
                                    new_ctx = new_ctx.extend(x.as_path(), x_ssa.clone());
                                    let bot_bit = offset;
                                    let top_bit = offset + width - 1;
                                    writeln!(self.writer, "    // binding variable {x} to slot")?;
                                    writeln!(self.writer, "    wire [{width_minus_1}:0] {x_ssa} = {subject_ssa}[{top_bit}:{bot_bit}];")?;
                                } else {
                                    panic!()
                                }
                            }
                            let arm_ssa = self.verilog_expr(e.clone(), new_ctx)?;
                            arm_ssas.push((tag, ctor.clone(), arm_ssa));
                        },
                        _ => todo!(),
                    }
                }

                writeln!(self.writer, "    // project tag ({tag_width} bits)")?;
                let tag_width_str = if tag_width == 1 {
                    format!("")
                } else {
                    format!("[{tag_top}:0]")
                };

                let subject_tag_idx = if tag_width == 1 {
                    format!("[0]")
                } else {
                    format!("[{tag_top}:0]")
                };

                writeln!(self.writer, "    reg {width_str} {gs};")?;

                writeln!(self.writer, "    wire {tag_width_str} {tag_ssa} = {subject_ssa}{subject_tag_idx};")?;

                writeln!(self.writer, "    always @(*) begin")?;
                writeln!(self.writer, "        case ({tag_ssa})")?;

                for (tag, ctor, arm_ssa) in &arm_ssas {
                    writeln!(self.writer, "            // @{ctor}:")?;
                    writeln!(self.writer, "            {tag}: {gs} <= {arm_ssa};")?;
                }

                writeln!(self.writer, "            default: {gs} <= 32'bx;")?;
                writeln!(self.writer, "        endcase")?;
                writeln!(self.writer, "    end")?;

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

    fn gensym_hint(&mut self, hint: &str) -> SsaName {
        self.gensym += 1;
        format!("__TEMP_{}_{hint}", self.gensym)
    }
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

/*
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
        writeln!(f, "module {}(", self.name)?;
        self.write_ports(f)?;
        writeln!(f, ");")?;
        self.write_statements(f)?;

        writeln!(f, "endmodule")?;
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
*/
