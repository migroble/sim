use crate::{ComponentKey, PinId};
use fnv::FnvHashMap;
use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
use {
    js_sys::Array,
    std::ops::{Deref, DerefMut},
    wasm_bindgen::prelude::*,
};

pub trait PinIO {
    fn read(&self, pin: usize) -> bool;
    fn write(&mut self, pin: usize, value: bool);

    // bit n => 1 << n
    fn read_u32(&self, pins: &[usize]) -> u32 {
        assert!(pins.len() <= 32, "Cannot read more than 32 pins");

        pins.iter()
            .rev()
            .map(|p| self.read(*p))
            .fold(0, |acc, v| (acc << 1) + v as u32)
    }

    // bit n => 1 << n
    fn write_u32(&mut self, pins: &[usize], value: u32) {
        assert!(pins.len() <= 32, "Cannot write to more than 32 pins");

        (0..pins.len()).for_each(|i| self.write(pins[i], value & (1 << i) != 0))
    }
}

#[repr(C)]
pub struct IO {
    pins: Vec<usize>,
    values: Vec<Signal>,
    changes: FnvHashMap<usize, bool>,
}

impl IO {
    fn new(pins: Vec<usize>, values: Vec<Signal>) -> Self {
        Self {
            pins,
            values,
            changes: FnvHashMap::with_hasher(Default::default()),
        }
    }

    fn changes(self) -> FnvHashMap<usize, bool> {
        self.changes
    }

    #[must_use]
    pub fn is_rising_edge(&self, pin: usize) -> bool {
        matches!(self.values[pin - 1], Signal::RisingEdge)
    }

    #[must_use]
    pub fn is_falling_edge(&self, pin: usize) -> bool {
        matches!(self.values[pin - 1], Signal::FallingEdge)
    }
}

impl PinIO for IO {
    #[must_use]
    fn read(&self, pin: usize) -> bool {
        self.values[pin - 1].into_bool()
    }

    fn write(&mut self, pin: usize, value: bool) {
        self.values[pin - 1].next_value(value);
        self.changes.insert(self.pins[pin - 1], value);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "IO")]
pub struct JsIO(*mut IO);

#[cfg(target_arch = "wasm32")]
impl Deref for JsIO {
    type Target = IO;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

#[cfg(target_arch = "wasm32")]
impl DerefMut for JsIO {
    fn deref_mut(&mut self) -> &mut IO {
        unsafe { &mut *self.0 }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_class = "IO")]
impl JsIO {
    pub fn read(&self, pin: usize) -> bool {
        unsafe { PinIO::read(&*self.0, pin) }
    }

    pub fn write(&mut self, pin: usize, value: bool) {
        unsafe {
            PinIO::write(&mut *self.0, pin, value);
        }
    }

    pub fn read_u32(&self, pins: &[usize]) -> u32 {
        unsafe { PinIO::read_u32(&*self.0, pins) }
    }

    pub fn write_u32(&mut self, pins: &[usize], values: u32) {
        unsafe {
            PinIO::write_u32(&mut *self.0, pins, values);
        }
    }

    pub fn is_rising_edge(&self, pin: usize) -> bool {
        unsafe { IO::is_rising_edge(&*self.0, pin) }
    }

    pub fn is_falling_edge(&self, pin: usize) -> bool {
        unsafe { IO::is_falling_edge(&*self.0, pin) }
    }
}

pub trait Component {
    fn pin_count(&self) -> usize;
    fn update(&mut self, _io: &mut IO) {}
}

impl<T: Component> Component for Rc<RefCell<T>> {
    fn pin_count(&self) -> usize {
        self.borrow().pin_count()
    }

    fn update(&mut self, io: &mut IO) {
        self.borrow_mut().update(io);
    }
}

#[derive(Clone, Copy)]
enum Signal {
    RisingEdge,
    FallingEdge,
    Static(bool),
}

impl Signal {
    fn next_value(self, value: bool) -> Signal {
        match self {
            Signal::RisingEdge => {
                if value {
                    Signal::Static(true)
                } else {
                    Signal::FallingEdge
                }
            }
            Signal::FallingEdge => {
                if value {
                    Signal::RisingEdge
                } else {
                    Signal::Static(false)
                }
            }
            Signal::Static(v) => {
                if value == v {
                    Signal::Static(v)
                } else if value {
                    Signal::RisingEdge
                } else {
                    Signal::FallingEdge
                }
            }
        }
    }

    fn into_bool(self) -> bool {
        match self {
            Signal::RisingEdge => true,
            Signal::FallingEdge => false,
            Signal::Static(v) => v,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    pub type JsComponent;

    #[wasm_bindgen(structural, method)]
    pub fn pin_count(this: &JsComponent) -> usize;

    #[wasm_bindgen(structural, method)]
    pub fn update(this: &JsComponent, io: JsIO);
}

#[cfg(target_arch = "wasm32")]
impl Component for JsComponent {
    fn pin_count(&self) -> usize {
        JsComponent::pin_count(self)
    }

    fn update(&mut self, io: &mut IO) {
        JsComponent::update(self, JsIO(io));
    }
}

pub struct Wrapper {
    pins: Vec<usize>,
    input: Vec<Signal>,
    component: Box<dyn Component>,
}

impl Wrapper {
    pub fn new<T: 'static + Component>(pins: Vec<usize>, component: T) -> Self {
        Self {
            input: vec![Signal::Static(false); pins.len()],
            pins,
            component: Box::new(component),
        }
    }

    pub fn pins(&self) -> &[usize] {
        &self.pins
    }

    pub fn set_input(&mut self, input: Vec<bool>) {
        self.input = self
            .input
            .iter()
            .zip(input)
            .map(|(old, new)| old.next_value(new))
            .collect::<Vec<_>>();
    }

    pub fn update(&mut self) -> FnvHashMap<usize, bool> {
        let mut c = IO::new(self.pins.clone(), self.input.clone());
        self.component.update(&mut c);
        c.changes()
    }
}

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct MetaComponent {
    pins: Vec<(ComponentKey, PinId)>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl MetaComponent {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    #[must_use]
    pub fn new(pins: &[usize]) -> Self {
        Self {
            pins: pins
                .to_vec()
                .chunks(2)
                .map(|v| (v[0], v[1]))
                .collect::<Vec<_>>(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn pin(&self, pin: usize) -> (ComponentKey, PinId) {
        self.pins[pin - 1]
    }

    #[cfg(target_arch = "wasm32")]
    pub fn pin(&self, pin: usize) -> Array {
        let (c, p) = self.pins[pin - 1];
        Array::of2(&(c as u32).into(), &(p as u32).into())
    }
}
