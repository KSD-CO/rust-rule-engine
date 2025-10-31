//! BetaNode: joins multiple AlphaMemory

use super::memory::AlphaMemory;

#[derive(Debug, Clone)]
pub struct BetaNode {
    pub left: AlphaMemory,
    pub right: AlphaMemory,
}

impl BetaNode {
    pub fn join(&self) -> Vec<((String, String), (String, String))> {
        let mut result = Vec::new();
        for l in &self.left.matches {
            for r in &self.right.matches {
                result.push((l.clone(), r.clone()));
            }
        }
        result
    }
}
