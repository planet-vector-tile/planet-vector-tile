use std::fs::File;
use std::{marker::PhantomData, path::{Path, PathBuf}, fs::OpenOptions};
use std::io::Result;
use memmap2::MmapMut;
use core::mem::size_of;
use core::slice::{from_raw_parts_mut, from_raw_parts};

pub struct Mutant<T: Sized> {
    file: File,
    path: PathBuf,
    mmap: MmapMut,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T: Sized> Mutant<T> {

    pub fn new(dir: &PathBuf, file_name: &str, len: usize) -> Result<Self> {
        // 8 bit header for flatdata
        let size = 8 + size_of::<T>() * len;
        let path = dir.join(file_name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Mutant { file, path, mmap, len, phantom: PhantomData })
    }

    pub fn open(dir: &PathBuf, file_name: &str) -> Result<Self> {
        let path = dir.join(file_name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        
        let len = file.metadata()?.len() as usize;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(Mutant { file, path, mmap, len, phantom: PhantomData })
    }

    pub fn mutable_slice(&self) -> &mut [T] {
        unsafe {
            from_raw_parts_mut(
                self.mmap[8..].as_ptr() as *mut T, self.len)
        }
    }

    pub fn slice(&self) -> &[T] {
        unsafe {
            from_raw_parts(
                self.mmap[8..].as_ptr() as *mut T, self.len)
        }
    }

    /// Returns a new mutant with the larger size. The old mutant
    /// is passed in, so it gets dropped.
    pub fn grow_by(mutant: Mutant<T>, len: usize) -> Result<Mutant<T>> {
        let total_len = mutant.len + len;
        let size = 8 + size_of::<T>() * total_len;
        let path = mutant.path.clone();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Mutant { file, path, mmap, len: total_len, phantom: PhantomData })
    }
}
