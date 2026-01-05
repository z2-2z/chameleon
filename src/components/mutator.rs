use crate::{Chameleon, ChameleonInput};
use std::borrow::Cow;
use libafl_bolts::Named;
use libafl::prelude::{Mutator, MutationResult, Error};

pub struct ChameleonMutator {
    chameleon: Chameleon,
}

impl ChameleonMutator {
    pub fn new(chameleon: Chameleon) -> Self {
        Self {
            chameleon,
        }
    }
}

impl Named for ChameleonMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("ChameleonMutator");
        &NAME
    }
}

impl<S> Mutator<ChameleonInput, S> for ChameleonMutator {
    fn mutate(&mut self, _state: &mut S, input: &mut ChameleonInput) -> Result<MutationResult, Error> {
        loop {
            if self.chameleon.mutate(&mut input.walk, &mut input.bytes) {
                break;
            }
        }
        
        Ok(MutationResult::Mutated)
    }
    
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<libafl::prelude::CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
