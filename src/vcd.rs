use std::collections::HashMap;
use std::io::Write;

use crate::value::Value;
use crate::types::Type;

pub struct Vcd<'a> {
    writer: &'a mut dyn Write,
//    var_names: HashMap<String, String>,
}

impl<'a> Vcd<'a> {
    pub fn new(writer: &'a mut dyn Write) -> Vcd<'a> {
        Vcd { writer }
    }

    pub fn header(&mut self) -> std::io::Result<()> {
        writeln!(self.writer, "$timescale 10ns $end")?;
        writeln!(self.writer, "$scope module top $end")?;

        writeln!(self.writer, "$var wire 1 clock clock $end")?;
        writeln!(self.writer, "$var wire 1 led_0 led_0 $end")?;
        writeln!(self.writer, "$var wire 1 led_1 led_1 $end")?;
        writeln!(self.writer, "$var wire 1 led_2 led_2 $end")?;
        writeln!(self.writer, "$var wire 1 led_3 led_3 $end")?;
        writeln!(self.writer, "$var wire 1 pmod_0 pmod_0 $end")?;
        writeln!(self.writer, "$upscope $end")?;

        Ok(())
    }

    pub fn step(&mut self, n: usize) -> std::io::Result<()> {
        writeln!(self.writer, "#{n}")
    }

    pub fn val(&mut self, var: &str, val: Value) -> std::io::Result<()> {
        match val {
            Value::Word(1, v) => write!(self.writer, "{v}")?,
            Value::Word(w, v) => write!(self.writer, "b{v:0w$b} ", w=w as usize)?,
            Value::X(typ) => {
                if let Type::Word(w) = typ.as_ref() {
                    let x_str = "x".repeat(*w as usize);
                    write!(self.writer, "{x_str}")?;
                } else {
                    panic!()
                }
            },
            _ => panic!(),
        }
        writeln!(self.writer, "{var}")?;
        Ok(())
    }
}
