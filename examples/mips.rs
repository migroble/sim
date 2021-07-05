use sim_rs::{
    components::{
        cpu::Mips,
        logic::{And, Not},
        mem::Ram,
    },
    Sim,
};

fn main() {
    let mut s = Sim::new();

    let mut instr = [0; 1 << 8];
    instr[0..30].copy_from_slice(&[
        0x20040002, 0x20050002, 0x201d0080, 0x0c100008, 0xac020000, 0x2008ffff, 0xad000000,
        0x08100007, 0x23bdfff8, 0xafb00004, 0xafbf0000, 0x14800002, 0x20a20001, 0x0810001a,
        0x14a00004, 0x2084ffff, 0x20050001, 0x0c100008, 0x0810001a, 0x00808020, 0x20a5ffff,
        0x0c100008, 0x2204ffff, 0x00402820, 0x0c100008, 0x0810001a, 0x8fb00004, 0x8fbf0000,
        0x23bd0008, 0x03e00008,
    ]);
    let data = [0; 32];

    let cpu = s.add_component(Mips::new());
    let instr_ram = s.add_component(Ram::new(&instr));
    let data_ram = s.add_component(Ram::new(&data));
    let not = s.add_component(Not);
    let and = s.add_component(And);

    s.connect(not, 1, cpu, 96);
    s.connect_to_clk(and, 1);
    s.connect(and, 2, not, 2);
    s.connect(and, 3, cpu, 130);

    fn range(start: usize, end: usize) -> Vec<usize> {
        (start..=end).collect()
    }

    s.connect_bulk(cpu, &range(3, 10), instr_ram, &range(1, 8));
    s.connect_bulk(cpu, &range(33, 64), instr_ram, &range(33, 64));

    s.connect_bulk(cpu, &range(67, 71), data_ram, &range(1, 5));
    s.connect_bulk(cpu, &range(97, 128), data_ram, &range(33, 64));
    s.connect(cpu, 129, data_ram, 65);

    loop {
        s.tick();
    }
}
