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

fn get_fn<T: Copy>(lib: &libloading::Library, name: String) -> Result<T> {
    let f: libloading::Symbol<T> = unsafe { lib.get(name) }?;
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
    
    pub fn seed(&mut self, seed: usize) {
        unsafe {
            (self.seed)(seed);
        }
    }
    
    pub fn mutate(&mut self, walk: &mut Vec<u32>, output: &mut Vec<u8>) {
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
        }
    }
    
    pub fn generate(&mut self, walk: &mut Vec<u32>, output: &mut Vec<u8>) {
        let mut c = ChameleonWalk {
            steps: walk.as_mut_ptr(),
            length: walk.len(),
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
        }
    }
}
