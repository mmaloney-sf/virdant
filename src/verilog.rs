use std::io::Write;

use crate::common::*;
use crate::context::Context;

use crate::phase::*;
use crate::phase::check::CheckQ;
use crate::phase::astq::*;
use crate::phase::item_resolution::*;
use crate::phase::typecheck::*;
use crate::phase::structure::*;
use crate::phase::layout::*;

type SsaName = String;

impl Db {
    pub fn verilog<F: Write>(&self, writer: &mut F) -> VirdantResult<()> {
        let mut verilog = Verilog {
            writer,
            db: self,
            gensym: 0,
        };

        verilog.db.check()?;
        verilog.verilog_packages()?;
        Ok(())
    }
}

struct Verilog<'a> {
    writer: &'a mut dyn Write,
    db: &'a Db,
    gensym: usize,
}

impl<'a> Verilog<'a> {
    fn verilog_packages(&mut self) -> VirdantResult<()> {
        for package in self.db.packages() {
            for moddef in self.db.package_moddefs(package)? {
                self.verilog_moddef(moddef)?;
            }
        }
        Ok(())
    }

    fn verilog_moddef(&mut self, moddef_id: ModDefId) -> VirdantResult<()> {
        let moddef_name: Ident = moddef_id.name();
        let moddef = self.db.structure_moddef(moddef_id)?;

        if moddef.is_ext() {
            writeln!(self.writer, "`include \"ext/{moddef_name}.v\"")?;
            writeln!(self.writer)?;
            return Ok(());
        }

        writeln!(self.writer, "module {}(", moddef_name.clone())?;
        let ports = moddef.ports();
        for (i, port) in ports.iter().enumerate() {
            let is_last = i + 1 == ports.len();
            self.verilog_port(port.clone(), is_last)?;
        }
        writeln!(self.writer, ");")?;

        for submodule in moddef.submodules() {
            self.verilog_submodule(submodule)?;
        }


        for component in moddef.components() {
            if component.is_outgoing() || component.is_reg() || component.is_node() {
                self.verilog_component(component)?;
            }
        }

        writeln!(self.writer, "endmodule")?;
        writeln!(self.writer)?;
        Ok(())
    }

    fn verilog_port(&mut self, port: Component, is_last_port: bool) -> VirdantResult<()> {
        let direction = if port.is_incoming() {
            "input  "
        } else {
            "output "
        };
        let port_name = port.id().name();
        let typ = port.typ();

        if let Type::Word(1) = typ {
            write!(self.writer, "    {direction} wire            {port_name}")?;
        } else if let Type::Word(n) = typ {
            let max_bit = n - 1;
            let width_str = format!("[{max_bit}:0]");
            let padded_width_str = format!("{width_str: >8}");
            write!(self.writer, "    {direction} wire  {padded_width_str} {port_name}")?;
        } else if let Type::Clock = typ {
            write!(self.writer, "    {direction} wire            {port_name}")?;
        } else {
            let width_str = make_width_str(self.db, typ.clone());
            write!(self.writer, "    {direction} wire  {width_str}     {port_name}")?;
        }

        if is_last_port {
            writeln!(self.writer)?;
        } else {
            writeln!(self.writer, ",")?;
        }

        Ok(())
    }

    fn verilog_component(&mut self, component: Component) -> VirdantResult<()> {
        if component.is_outgoing() {
            let expr = component.driver().unwrap();
            let typ = component.typ();
            let component_name = component.id().name();
            writeln!(self.writer, "    // outgoing {component_name} : {typ}")?;
            let ssa = self.verilog_expr(expr, Context::empty())?;
            writeln!(self.writer, "    assign {component_name} = {ssa};")?;
            writeln!(self.writer)?;
        } else if component.is_node() {
            let expr = component.driver().unwrap();
            let typ = expr.typ();
            let width_str = make_width_str(self.db, typ.clone());
            let component_name = component.id().name();
            writeln!(self.writer, "    // node {component_name} : {typ}")?;
            writeln!(self.writer, "    wire {width_str} {component_name};")?;
            let ssa = self.verilog_expr(expr, Context::empty())?;
            writeln!(self.writer, "    assign {component_name} = {ssa};")?;
            writeln!(self.writer)?;
        } else if component.is_reg() {
            let expr = component.driver().unwrap();
            let typ = expr.typ();
            let width_str = make_width_str(self.db, typ.clone());
            let component_name = component.id().name();
            writeln!(self.writer, "    // reg {component_name} : {typ}")?;
            writeln!(self.writer, "    reg  {width_str} {component_name};")?;
            let clk = "clock"; //component.clock().unwrap();
            let connect_ssa = self.verilog_expr(expr.clone(), Context::empty())?;
            writeln!(self.writer, "    always @(posedge {clk}) begin")?;
            writeln!(self.writer, "        {component_name} <= {connect_ssa};")?;
            writeln!(self.writer, "    end")?;
            writeln!(self.writer)?;
            writeln!(self.writer, "    initial begin")?;
            writeln!(self.writer, "        {component_name} <= 1;")?;
            writeln!(self.writer, "    end")?;
            writeln!(self.writer)?;
        }

        Ok(())
    }

    fn verilog_submodule(&mut self, submodule: Submodule) -> VirdantResult<()> {
        writeln!(self.writer, "    // Submodule {} of {}", submodule.id(), submodule.moddef())?;
        let moddef_id = submodule.id().moddef();
        let submodule_moddef = self.db.structure_moddef(submodule.moddef())?;
        let ports = submodule_moddef.ports();

        // Create wires which bridge between the module and the submodule
        for port in &ports {
            let typ = port.typ();
            let width_str = make_width_str(self.db, typ);
            let submodule_name = submodule.id().name();
            let port_name = port.id().name();
            writeln!(self.writer, "    wire {width_str} __TEMP_{submodule_name}_{port_name};")?;
        }

        // Create drive the submodule's incoming ports.
        for port in &ports {
            if port.is_incoming() {
                let path = submodule.id().name().as_path().join(&port.id().name().as_path());
                eprintln!("IN MODULE {moddef_id}");
                eprintln!("INST'ING MODULE {}", submodule_moddef.id());
                eprintln!("LOOKING FOR DRIVER FOR {path}");
                let expr = submodule.driver_for(path);
                let gs = self.verilog_expr(expr, Context::empty())?;
                let submodule_name = submodule.id().name();
                let port_name = port.id().name();
                writeln!(self.writer, "    assign __TEMP_{submodule_name}_{port_name} = {gs};")?;
            }
        }


        // Instantiate the module and connect the intermediary wires.
        writeln!(self.writer, "    {} {}(", submodule_moddef.id().name(), submodule.id().name())?;
        for (i, port) in ports.iter().enumerate() {
            let last_port = i + 1 == ports.len();
            let submodule_name = submodule.id().name();
            let port_name = port.id().name();
            write!(self.writer, "        .{port_name}(__TEMP_{submodule_name}_{port_name})")?;
            if last_port {
                writeln!(self.writer)?;
            } else {
                writeln!(self.writer, ",")?;
            }
        }
        writeln!(self.writer, "    );")?;
        Ok(())
    }

    fn verilog_expr(&mut self, expr: Arc<TypedExpr>, ctx: Context<Ident, SsaName>) -> VirdantResult<SsaName> {
        match expr.as_ref() {
            TypedExpr::Reference(_typ, Referent::Local(x)) => {
                let ssa = ctx.lookup(x).unwrap();
                Ok(format!("{ssa}"))
            },
            TypedExpr::Reference(_typ, Referent::LocalComponent(component_id)) => {
                let path: Path = component_id.name().into();
                Ok(format!("{path}"))
            },
            TypedExpr::Reference(_typ, Referent::NonLocalComponent(submodule_element_id, component_id)) => {
                let path: Path = submodule_element_id.name().as_path().join(&component_id.name().into());
                let parts = path.parts();
                let sm = &parts[0];
                let port = &parts[1];
                Ok(format!("__TEMP_{sm}_{port}"))
            },
            TypedExpr::Word(_typ, w) => {
                let gs = self.gensym();
                let typ = expr.typ();
                let width_str = make_width_str(self.db, typ);
                writeln!(self.writer, "    wire {width_str} {gs} = {};", w.value)?;
                Ok(gs)
            },
            TypedExpr::Cat(typ, args) => {
                let gs = self.gensym();
                let mut arg_ssas: Vec<SsaName> = vec![];
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone(), ctx.clone())?;
                    arg_ssas.push(arg_ssa);
                }
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    wire {width_str} {gs} = {{{}}};", arg_ssas.join(", "))?;
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
                    "dec" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} - 1;")?,
                    "sub" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} - {};", args_ssa[0])?,
                    "and" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} & {};", args_ssa[0])?,
                    "or"  => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} | {};", args_ssa[0])?,
                    "not" => writeln!(self.writer, "    wire {width_str} {gs} = ~{subject_ssa};")?,
                    "xor" => writeln!(self.writer, "    wire {width_str} {gs} = {subject_ssa} ^ {};", args_ssa[0])?,
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

                let layout = self.db.union_layout(typ.clone())?;

                let mut args_ssa: Vec<SsaName> = vec![];
                for arg in args {
                    let arg_ssa = self.verilog_expr(arg.clone(), ctx.clone())?;
                    args_ssa.push(arg_ssa);
                }

                let width_str = make_width_str(self.db, typ.clone());

                let tag = self.db.union_ctor_tag(typ.clone(), ctor.clone())?;
                let fill = "1";

                write!(self.writer, "    wire {width_str} {gs} = {{ ")?;
                for arg_ssa in args_ssa.iter().rev() {
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
                let new_ctx = ctx.extend(x.clone(), e_ssa);
                let b_ssa = self.verilog_expr(b.clone(), new_ctx)?;
                let width_str = make_width_str(self.db, typ.clone());
                writeln!(self.writer, "    wire {width_str} {gs} = {b_ssa};")?;
                Ok(gs)
            },
            TypedExpr::Match(_typ, subject, _ascription, arms) => {
                let gs = self.gensym_hint("match");
                let subject_ssa = self.verilog_expr(subject.clone(), ctx.clone())?;
                let typ = expr.typ();
                let layout = self.db.union_layout(subject.typ())?;
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
                                    new_ctx = new_ctx.extend(x.clone(), x_ssa.clone());
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
            TypedExpr::Struct(typ, _name, fields) => {
                let gs = self.gensym();
                writeln!(self.writer, "    {gs} = ...{expr:?}")?;
                todo!()
            },
            _ => {
                let gs = self.gensym();
                writeln!(self.writer, "    {gs} = ...{expr:?}")?;
                todo!()
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
