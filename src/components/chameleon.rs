use std::path::Path;
use std::ops::Deref;
use anyhow::Result;

pub const DEFAULT_PREFIX: &str = "chameleon";

#[repr(C)]
struct ChameleonWalk {
    steps: *mut u32,
    length: usize,
    capacity: usize,
}

type FunctionSeed = unsafe extern "C" fn(usize);
type FunctionMutate = unsafe extern "C" fn(*mut ChameleonWalk, *mut u8, usize) -> usize;
type FunctionGenerate = unsafe extern "C" fn(*mut ChameleonWalk, *mut u8, usize) -> usize;
type FunctionBabyGenerate = unsafe extern "C" fn(*mut u8, usize) -> usize;

fn get_fn<T: Copy>(lib: &libloading::Library, name: String) -> Result<T> {
    let f: libloading::Symbol<T> = unsafe {
        match lib.get(name) {
            Err(libloading::Error::DlSym { .. }) => {
                #[allow(clippy::missing_transmute_annotations)]
                Ok(std::mem::transmute(std::ptr::null::<()>()))
            },
            v => v,
        }
    }?;
    Ok(*f.deref())
}

#[derive(Clone)]
pub struct Chameleon {
    seed: FunctionSeed,
    mutate: FunctionMutate,
    generate: FunctionGenerate,
}

impl Chameleon {
    pub fn load<P: AsRef<Path>>(shared_object: P, prefix: Option<&str>) -> Result<Self> {
        let shared_object = shared_object.as_ref();
        let prefix = prefix.unwrap_or(DEFAULT_PREFIX);
        let lib = unsafe {
            libloading::Library::new(shared_object)
        }?;
        
        let seed = get_fn::<FunctionSeed>(&lib, format!("{prefix}_seed"))?;
        let mutate = get_fn::<FunctionMutate>(&lib, format!("{prefix}_mutate"))?;
        let generate = get_fn::<FunctionGenerate>(&lib, format!("{prefix}_generate"))?;
        
        std::mem::forget(lib);
        
        Ok(Self {
            seed,
            mutate,
            generate,
        })
    }
    
    pub fn seed(&self, seed: usize) {
        assert_ne!(self.seed as usize, 0);
        
        unsafe {
            (self.seed)(seed);
        }
    }
    
    pub fn mutate(&self, walk: &mut Vec<u32>, output: &mut Vec<u8>) -> bool {
        assert_ne!(self.mutate as usize, 0);
        
        let mut c = ChameleonWalk {
            steps: walk.as_mut_ptr(),
            length: walk.len(),
            capacity: walk.capacity(),
        };
        
        unsafe {
            let new_len = (self.mutate)(
                &mut c as *mut ChameleonWalk,
                output.as_mut_ptr(),
                output.capacity()
            );
            
            walk.set_len(c.length);
            output.set_len(new_len);
            
            new_len < output.capacity()
        }
    }
    
    pub fn generate(&self, walk: &mut Vec<u32>, output: &mut Vec<u8>) -> bool {
        assert_ne!(self.generate as usize, 0);
        
        let mut c = ChameleonWalk {
            steps: walk.as_mut_ptr(),
            length: 0,
            capacity: walk.capacity(),
        };
        
        unsafe {
            let new_len = (self.generate)(
                &mut c as *mut ChameleonWalk,
                output.as_mut_ptr(),
                output.capacity()
            );
            
            walk.set_len(c.length);
            output.set_len(new_len);
            
            new_len < output.capacity()
        }
    }
}

#[derive(Clone)]
pub struct BabyChameleon {
    seed: FunctionSeed,
    generate: FunctionBabyGenerate,
}

impl BabyChameleon {
    pub fn load<P: AsRef<Path>>(shared_object: P, prefix: Option<&str>) -> Result<Self> {
        let shared_object = shared_object.as_ref();
        let prefix = prefix.unwrap_or(DEFAULT_PREFIX);
        let lib = unsafe {
            libloading::Library::new(shared_object)
        }?;
        
        let seed = get_fn::<FunctionSeed>(&lib, format!("{prefix}_seed"))?;
        let generate = get_fn::<FunctionBabyGenerate>(&lib, format!("{prefix}_generate"))?;
        
        std::mem::forget(lib);
        
        Ok(Self {
            seed,
            generate,
        })
    }
    
    pub fn seed(&self, seed: usize) {
        assert_ne!(self.seed as usize, 0);
        
        unsafe {
            (self.seed)(seed);
        }
    }
    
    pub fn generate(&self, output: &mut Vec<u8>) -> bool {
        assert_ne!(self.generate as usize, 0);
        
        unsafe {
            let new_len = (self.generate)(
                output.as_mut_ptr(),
                output.capacity()
            );
            
            output.set_len(new_len);
            
            new_len < output.capacity()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_baby() {
        let seed = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let mut buffer = vec![0u8; 4096];
        let generator = BabyChameleon::load("test-data/baby/generator.so", None).unwrap();
        generator.seed(seed as usize);
        generator.generate(&mut buffer);
        let s = String::from_utf8_lossy(&buffer);
        println!("{}", s);
    }
}
