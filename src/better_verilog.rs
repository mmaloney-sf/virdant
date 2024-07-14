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

