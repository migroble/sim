use super::*;
use bindgen_macro::bindgen;

// pins:
//  1-32: addr
//  33-64: data
//  65: write
#[bindgen]
pub struct Ram {
    data: Vec<u32>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Ram {
    #[must_use]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(data: &[u32]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    #[must_use]
    pub fn read(&self, addr: usize) -> u32 {
        self.data[addr]
    }
}

impl Component for Ram {
    fn pin_count(&self) -> usize {
        65
    }

    fn update(&mut self, io: &mut IO) {
        let addr = io.read_u32(&(1..=32).collect::<Vec<_>>()) as usize;
        let data_pins = (33..=64).collect::<Vec<_>>();

        if io.read(65) {
            self.data[addr] = io.read_u32(&data_pins);
        } else {
            io.write_u32(&data_pins, self.data[addr]);
        }
    }
}
