use std::{marker::PhantomData, path::PathBuf, fs::OpenOptions};
use memmap2::MmapMut;
use core::mem::size_of;
use core::slice::from_raw_parts_mut;


pub fn mutable_slice<T>(slc: &[T]) -> &mut [T] {
    unsafe {
        from_raw_parts_mut(
            slc.as_ptr() as *mut T, slc.len())
    }
}

pub struct Mutant<T: Sized> {
    mmap: MmapMut,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T: Sized> Mutant<T> {
    pub fn new(dir: &PathBuf, file_name: &str, len: usize) -> std::io::Result<Self> {
        // 8 bit header for flatdata
        let size = 8 + size_of::<T>() * len;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(dir.join(file_name))?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Mutant { mmap, len, phantom: PhantomData })
    }

    pub fn mutable_slice(&self) -> &mut [T] {
        unsafe {
            from_raw_parts_mut(
                self.mmap[8..].as_ptr() as *mut T, self.len)
        }
    }
}
