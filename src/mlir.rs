use std::io::Write;

use super::*;

impl Package {
    pub fn mlir<F: Write>(&self, mut f: F) -> std::io::Result<()> {
        for item in self.items() {
            match item {
                Item::ModDef(moddef) => moddef.mlir(&mut f)?,
            }
        }
        Ok(())
    }
}

impl ModDef {
    fn mlir<F: Write>(&self, mut f: F) -> std::io::Result<()> {
        writeln!(f, "virdant.module {} {{", self.name())?;
        for component in self.components() {
            component.mlir(&mut f)?;

        }
        for submodule in self.submodules() {
            submodule.mlir(&mut f)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}


impl Component {
    fn mlir<F: Write>(&self, mut f: F) -> std::io::Result<()> {
        match self {
            Component::Incoming(name, typ) => {
                writeln!(f, "    %{name} = virdant.incoming()")?;
            },
            Component::Outgoing(name, typ, e) => {
                if let Some(some_e) = e {
                    some_e.mlir(&mut f)?;
                }
                writeln!(f, "    %{name} = virdant.outgoing()")?;
            },
            Component::Wire(name, typ, e) => {
                if let Some(some_e) = e {
                    some_e.mlir(&mut f)?;
                }
                writeln!(f, "    %{name} = virdant.wire()")?;
            },
            Component::Reg(name, typ, clk, rst, e) => {
                if let Some(some_e) = e {
                    some_e.mlir(&mut f)?;
                }
                writeln!(f, "    %{name} = virdant.reg()")?;
            },
            _ => writeln!(f, "    {}", self.name())?,
        }
        Ok(())
    }
}

impl Submodule {
    fn mlir<F: Write>(&self, mut f: F) -> std::io::Result<()> {
        writeln!(f, "    %{} = virdant.submodule @{}", self.name(), self.moddef())?;
        Ok(())
    }
}

impl Expr {
    fn mlir<F: Write>(&self, mut f: F) -> std::io::Result<()> {
        match self {
            _ => writeln!(f, "    UNKNOWN EXPR: {self:?}")?,
        }
        Ok(())
    }
}
