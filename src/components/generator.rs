use crate::{Chameleon, ChameleonInput, BabyChameleon};
use libafl::prelude::{Generator, Error};

pub struct ChameleonGenerator {
    chameleon: Chameleon,
}

impl ChameleonGenerator {
    pub fn new(chameleon: Chameleon) -> Self {
        Self {
            chameleon,
        }
    }
}

impl<S> Generator<ChameleonInput, S> for ChameleonGenerator {
    fn generate(&mut self, _state: &mut S) -> Result<ChameleonInput, Error> {
        let mut input = ChameleonInput::default();
        
        loop {
            if self.chameleon.generate(&mut input.walk, &mut input.bytes) {
                break;
            }
        }
        
        Ok(input)
    }
}

pub struct BabyChameleonGenerator {
    chameleon: BabyChameleon,
}

impl BabyChameleonGenerator {
    pub fn new(chameleon: BabyChameleon) -> Self {
        Self {
            chameleon,
        }
    }
}

impl<S> Generator<ChameleonInput, S> for BabyChameleonGenerator {
    fn generate(&mut self, _state: &mut S) -> Result<ChameleonInput, Error> {
        let mut input = ChameleonInput::default();
        
        loop {
            if self.chameleon.generate(&mut input.bytes) {
                break;
            }
        }
        
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl::prelude::HasTargetBytes;
    
    #[test]
    fn test_generator() {
        let chameleon = Chameleon::load("test-data/generator/generator.so", None).unwrap();
        let seed = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        chameleon.seed(seed as usize);
        let mut generator = ChameleonGenerator::new(chameleon);
        let input = generator.generate(&mut ()).unwrap();
        let bytes = input.target_bytes();
        let s = String::from_utf8_lossy(&bytes);
        println!("{s}");
    }
    
    #[test]
    fn bench_generator() {
        const AMOUNT: usize = 1024 * 1024 * 1024;
        let chameleon = Chameleon::load("test-data/generator/generator.so", None).unwrap();
        let mut generator = ChameleonGenerator::new(chameleon);
        
        let mut c = 0;
        let start = std::time::Instant::now();
        while c < AMOUNT {
            let input = generator.generate(&mut ()).unwrap();
            c += input.bytes.len();
        }
        let elapsed = start.elapsed();
        
        println!("{} in secs={} nsecs={}", AMOUNT, elapsed.as_secs(), elapsed.as_nanos());
    }
    
    #[test]
    fn generator_stats() {
        let mut stats = Vec::new();
        let chameleon = Chameleon::load("test-data/generator/generator.so", None).unwrap();
        let seed = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        chameleon.seed(seed as usize);
        let mut generator = ChameleonGenerator::new(chameleon);
        
        for _ in 0..1000 {
            let input = generator.generate(&mut ()).unwrap();
            let bytes = input.target_bytes();
            stats.push(bytes.len());
        }
        
        println!("{:?}", stats);
    }
}
