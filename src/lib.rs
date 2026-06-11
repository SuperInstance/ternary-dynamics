//! Dynamic ternary logic simulation engine

/// Ternary value used in dynamic simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriVal {
    False,
    Unknown,
    True,
}

/// A simulation node with optional propagation delay
#[derive(Debug, Clone)]
pub struct SimNode {
    pub id: usize,
    pub value: TriVal,
    pub delay_ns: u64,
}

/// A gate connecting inputs to an output node
#[derive(Debug, Clone)]
pub struct SimGate {
    pub gate_type: GateType,
    pub inputs: Vec<usize>,
    pub output: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum GateType {
    And,
    Or,
    Not,
    Majority,
}

/// Simulation engine
pub struct Simulation {
    pub nodes: Vec<SimNode>,
    pub gates: Vec<SimGate>,
    pub time: u64,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation { nodes: vec![], gates: vec![], time: 0 }
    }

    pub fn add_node(&mut self, value: TriVal, delay_ns: u64) -> usize {
        let id = self.nodes.len();
        self.nodes.push(SimNode { id, value, delay_ns });
        id
    }

    pub fn step(&mut self) -> u64 {
        self.time += 1;
        self.time
    }

    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }
}

impl Default for Simulation {
    fn default() -> Self { Self::new() }
}
