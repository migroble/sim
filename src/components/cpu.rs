use super::*;
use bindgen_macro::{bindgen, constrgen};
use mips_emu::{Cpu, Mem};
use std::cell::RefCell;
use std::rc::Rc;

// pins:
//  1-32: instr_addr
//  33-64: instr
//  65-96: data_addr
//  97-128: data
//  129: data_write
//  130: clk
#[bindgen]
#[constrgen]
pub struct Mips(Cpu);

impl Component for Mips {
    fn pin_count(&self) -> usize {
        130
    }

    fn update(&mut self, io: &mut IO) {
        if io.is_falling_edge(130) || io.is_rising_edge(130) {
            println!(
                "---- CPU ({:#X}, {}, {}) ----",
                self.0.pc(),
                self.0.cycle(),
                self.0.instr_count()
            );
            let shared_io = Rc::new(RefCell::new(&mut *io));
            self.0.half_step(
                &mut M {
                    io: Rc::clone(&shared_io),
                    addr_pins: (1..=32).collect(),
                    data_pins: (33..=64).collect(),
                    write_pin: None,
                },
                &mut M {
                    io: shared_io,
                    addr_pins: (65..=96).collect(),
                    data_pins: (97..=128).collect(),
                    write_pin: Some(129),
                },
            );

            println!(
                "regs: {:?}",
                (0..32).map(|r| self.0.read_reg(r)).collect::<Vec<_>>()
            );
        }
    }
}

struct M<'a> {
    io: Rc<RefCell<&'a mut IO>>,
    addr_pins: Vec<usize>,
    data_pins: Vec<usize>,
    write_pin: Option<usize>,
}

impl Mem for M<'_> {
    fn addr(&mut self, addr: u32) {
        let mut io = self.io.borrow_mut();

        if let Some(p) = self.write_pin {
            io.write(p, false);
        }

        io.write_u32(&self.addr_pins, addr);
    }

    fn read(&mut self) -> u32 {
        self.io.borrow().read_u32(&self.data_pins)
    }

    fn write(&mut self, data: u32) {
        let mut io = self.io.borrow_mut();

        if let Some(p) = self.write_pin {
            io.write(p, true);
        }

        io.write_u32(&self.data_pins, data);
    }
}
