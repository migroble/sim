use crate::{
    component::{Component, Wrapper},
    ComponentKey, PinId,
};
use fnv::{FnvHashMap, FnvHashSet};
use graph::Graph;
use slab::Slab;

#[cfg(target_arch = "wasm32")]
use {crate::component::JsComponent, wasm_bindgen::prelude::*};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Sim {
    graph: Graph,
    components: Slab<Wrapper>,
    values: Vec<Option<bool>>,
    pin_to_component: FnvHashMap<PinId, ComponentKey>,
    pin_to_value: FnvHashMap<PinId, usize>,
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Sim {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    #[must_use]
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        let mut s = Self {
            graph: Graph::new(),
            components: Slab::new(),
            values: Vec::new(),
            pin_to_component: FnvHashMap::with_hasher(Default::default()),
            pin_to_value: FnvHashMap::with_hasher(Default::default()),
        };

        // create pin 0 (global clk)
        s.create_pin();

        s
    }

    pub fn connect_to_clk(&mut self, c: ComponentKey, pin: PinId) {
        self.graph.add_edge(self.pin(c, pin), 0);
        self.update_connections();
    }

    pub fn connect(&mut self, c1: ComponentKey, pin1: PinId, c2: ComponentKey, pin2: PinId) {
        self._connect(c1, pin1, c2, pin2);
        self.update_connections();
    }

    /// # Panics
    ///
    /// Will panic if the lengths of the pin arrays differs
    pub fn connect_bulk(
        &mut self,
        c1: ComponentKey,
        pins1: &[PinId],
        c2: ComponentKey,
        pins2: &[PinId],
    ) {
        assert!(
            pins1.len() == pins2.len(),
            "Bulk connection pin counts differ!"
        );

        pins1
            .iter()
            .zip(pins2)
            .for_each(|(&pin1, &pin2)| self._connect(c1, pin1, c2, pin2));
        self.update_connections();
    }

    pub fn tick(&mut self) {
        let mut changes = FnvHashMap::default();
        changes.insert(0, !self._read(0).unwrap_or(false));
        self.propagate_changes(&changes, false);
    }

    #[must_use]
    pub fn read(&self, c: ComponentKey, pin: PinId) -> bool {
        self._read(self.pin(c, pin)).unwrap_or(false)
    }

    pub fn write(&mut self, c: ComponentKey, pin: PinId, value: bool) {
        let mut changes = FnvHashMap::default();
        changes.insert(self.pin(c, pin), value);
        self.propagate_changes(&changes, true);
    }

    #[must_use]
    pub fn read_clk(&self) -> bool {
        self._read(0).unwrap_or(false)
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = "add_component")]
    pub fn add_ext_component(&mut self, component: JsComponent) -> ComponentKey {
        self.add_component(component)
    }
}

impl Sim {
    fn pins(&self, c: ComponentKey) -> &[PinId] {
        &self.components[c].pins()
    }

    fn pin(&self, c: ComponentKey, pin: PinId) -> PinId {
        self.pins(c)[pin - 1]
    }

    fn create_pin(&mut self) -> PinId {
        let pin = self.graph.add_node();
        self.pin_to_value.insert(pin, self.values.len());
        self.values.push(None);
        pin
    }

    fn create_pins(&mut self, pin_count: usize) -> Vec<PinId> {
        (0..pin_count)
            .map(|_| self.create_pin())
            .collect::<Vec<_>>()
    }

    fn _connect(&mut self, c1: ComponentKey, pin1: PinId, c2: ComponentKey, pin2: PinId) {
        self.graph.add_edge(self.pin(c1, pin1), self.pin(c2, pin2));
    }

    fn _read(&self, pin: PinId) -> Option<bool> {
        self.values[self.pin_to_value[&pin]]
    }

    fn _write(&mut self, pin: PinId, value: bool) {
        self.values[self.pin_to_value[&pin]] = Some(value);
    }

    fn read_pins(&self, pins: &[PinId]) -> Vec<bool> {
        pins.iter()
            .map(|&p| self._read(p).unwrap_or(false))
            .collect()
    }

    fn update_connections(&mut self) {
        let pin_values = self
            .graph
            .nodes()
            .iter()
            .filter_map(|n| self._read(*n).map(|v| (*n, v)))
            .collect::<Vec<_>>();

        let pins = self.graph.components();
        let components = pins.values().max().unwrap_or(&0) + 1;
        self.values = vec![None; components];
        self.pin_to_value = pins;

        pin_values.iter().for_each(|(n, v)| self._write(*n, *v));
    }

    fn update_component(&mut self, key: ComponentKey) {
        let input = self.read_pins(self.pins(key));
        let c = &mut self.components[key];
        c.set_input(input);

        let changes = c.update();
        self.propagate_changes(&changes, false);
    }

    fn connected_to(&self, pin: PinId) -> impl Iterator<Item = PinId> + '_ {
        let value = self.pin_to_value[&pin];

        self.pin_to_value
            .iter()
            .filter_map(move |(p, &v)| if v == value { Some(*p) } else { None })
    }

    fn propagate_changes(&mut self, changes: &FnvHashMap<PinId, bool>, update_self: bool) {
        changes
            .iter()
            .filter_map(|(p, v)| {
                if self._read(*p) == Some(*v) {
                    None
                } else {
                    self._write(*p, *v);
                    Some(*p)
                }
            })
            .collect::<Vec<_>>() // changed pins
            .iter()
            .flat_map(|pin| {
                self.connected_to(*pin)
                    .filter(move |p| update_self || *p != *pin) // the updated pin should not trigger an update on itself
                    .map(|p| self.pin_to_component[&p])
                    .filter(|k| *pin == 0 || update_self || *k != self.pin_to_component[pin]) // prevent self updates
                    .collect::<Vec<_>>()
            })
            .collect::<FnvHashSet<_>>() // affected components
            .iter()
            .for_each(|k| self.update_component(*k))
    }

    pub fn add_component<T: 'static + Component>(&mut self, component: T) -> ComponentKey {
        let pins = self.create_pins(component.pin_count());

        let k = self
            .components
            .insert(Wrapper::new(pins.clone(), component));

        pins.iter().for_each(|p| {
            self.pin_to_component.insert(*p, k);
        });

        self.update_component(k);

        k
    }
}
