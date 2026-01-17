use serde::{Serialize, Deserialize};
use libafl::prelude::{Input, CorpusId, HasTargetBytes};
use libafl_bolts::prelude::{generic_hash_std, HasLen, OwnedSlice};

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct ChameleonInput<const W: usize = 1024, const B: usize = 4096> {
    pub(crate) walk: Vec<u8>,
    pub(crate) bytes: Vec<u8>,
}

impl<const W: usize, const B: usize> Default for ChameleonInput<W, B> {
    fn default() -> Self {
        Self {
            walk: Vec::with_capacity(W),
            bytes: Vec::with_capacity(B),
        }
    }
}

// We must preserve the same capacity for clones of ChameleonInput
// so we have to manually do it ourselves
impl Clone for ChameleonInput {
    fn clone(&self) -> Self {
        let mut other = Self::default();
        other.walk.extend_from_slice(&self.walk);
        other.bytes.extend_from_slice(&self.bytes);
        other
    }
}

impl Input for ChameleonInput {
    fn generate_name(&self, _id: Option<CorpusId>) -> String {
        format!("chameleon-{:016x}.bin", generic_hash_std(self))
    }
}

impl HasLen for ChameleonInput {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl HasTargetBytes for ChameleonInput {
    fn target_bytes(&self) -> OwnedSlice<'_, u8> {
        (&self.bytes).into()
    }
}
