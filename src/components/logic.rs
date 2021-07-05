use super::*;
use bindgen_macro::{bindgen, constrgen};

#[bindgen]
#[constrgen]
pub struct Buffer;
impl Component for Buffer {
    fn pin_count(&self) -> usize {
        2
    }

    fn update(&mut self, io: &mut IO) {
        io.write(2, io.read(1));
    }
}

#[bindgen]
#[constrgen]
pub struct Not;
impl Component for Not {
    fn pin_count(&self) -> usize {
        2
    }

    fn update(&mut self, io: &mut IO) {
        io.write(2, !io.read(1));
    }
}

#[bindgen]
#[constrgen]
pub struct And;
impl Component for And {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, io.read(1) & io.read(2));
    }
}

#[bindgen]
#[constrgen]
pub struct Or;
impl Component for Or {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, io.read(1) | io.read(2));
    }
}

#[bindgen]
#[constrgen]
pub struct Nand;
impl Component for Nand {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, !(io.read(1) & io.read(2)));
    }
}

#[bindgen]
#[constrgen]
pub struct Nor;
impl Component for Nor {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, !(io.read(1) | io.read(2)));
    }
}

#[bindgen]
#[constrgen]
pub struct Xor;
impl Component for Xor {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, io.read(1) ^ io.read(2));
    }
}

#[bindgen]
#[constrgen]
pub struct Xnor;
impl Component for Xnor {
    fn pin_count(&self) -> usize {
        3
    }

    fn update(&mut self, io: &mut IO) {
        io.write(3, !(io.read(1) ^ io.read(2)));
    }
}
