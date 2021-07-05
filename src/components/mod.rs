use super::*;

pub mod cpu;
pub mod logic;
pub mod mem;

pub struct Static<const N: usize>(pub u32);
impl<const N: usize> Component for Static<N> {
    fn pin_count(&self) -> usize {
        N
    }

    fn update(&mut self, io: &mut IO) {
        io.write_u32(&(1..=N).collect::<Vec<_>>(), self.0);
    }
}

/*
pub struct Rom_AT28C256(pub [u8; 1 << 15]);
impl Component for Rom_AT28C256 {
    fn pin_count(&self) -> usize {
        28
    }

    fn update(&mut self, io: &mut IO) {
        if io.read(20) && io.read(22) {
            let addr = io.read_u32(&[10, 9, 8, 7, 6, 5, 4, 3, 25, 24, 21, 23, 2, 26, 1]) as usize;
            io.write_u32(&[11, 12, 13, 15, 16, 17, 18, 19], self.0[addr].into());
        }
    }
}

pub struct DFlipFlop_MC74HCT74A([bool; 2]);
impl Component for DFlipFlop_MC74HCT74A {
    fn pin_count(&self) -> usize {
        14
    }

    fn update(&mut self, io: &mut IO) {
        // d flip flop 1
        self.0[0] = if io.read(1) {
            false
        } else if io.read(4) {
            true
        } else if io.read(3) {
            io.read(2)
        } else {
            false
        };

        io.write(5, self.0[0]);
        io.write(6, !self.0[0]);

        // d flip flop 2
        self.0[1] = if io.read(13) {
            false
        } else if io.read(10) {
            true
        } else if io.read(11) {
            io.read(12)
        } else {
            false
        };

        io.write(5, self.0[1]);
        io.write(6, !self.0[1]);
    }
}
*/
