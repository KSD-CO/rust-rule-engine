//! Memory for RETE nodes

#[derive(Debug, Clone)]
pub struct AlphaMemory {
    pub matches: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct BetaMemory {
    pub partial_matches: Vec<Vec<(String, String)>>,
}
