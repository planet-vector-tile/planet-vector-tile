#![allow(dead_code)]

use core::mem::size_of;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use memmap2::MmapMut;
use std::fs::File;
use std::io::Result;
use std::{
    fs::OpenOptions,
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub struct Mutant<T: Sized> {
    file: File,
    path: PathBuf,
    mmap: MmapMut,
    pub len: usize,
    pub capacity: usize,
    phantom: PhantomData<T>,
}

impl<T: Sized> Mutant<T> {
    pub fn new(dir: &Path, file_name: &str, len: usize) -> Result<Self> {
        let size = 8 + size_of::<T>() * len;
        let path = dir.join(file_name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        let header_ptr = mmap.as_ptr() as *mut u64;
        unsafe {
            *header_ptr = len as u64;
        }

        Ok(Mutant {
            file,
            path,
            mmap,
            len,
            capacity: len,
            phantom: PhantomData,
        })
    }

    pub fn with_capacity(dir: &Path, file_name: &str, capacity: usize) -> Result<Self> {
        let size = 8 + size_of::<T>() * capacity;
        let path = dir.join(file_name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };
        let header_ptr = mmap.as_ptr() as *mut u64;
        unsafe {
            *header_ptr = 0 as u64;
        }

        Ok(Mutant {
            file,
            path,
            mmap,
            len: 0,
            capacity,
            phantom: PhantomData,
        })
    }

    pub fn open(dir: &Path, file_name: &str, is_flatdata: bool) -> Result<Self> {
        // Don't use the header as info on length for flatdata,
        // as I am not sure if it uses the header that way.

        let path = dir.join(file_name);
        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        let file_size = file.metadata()?.len() as usize;
        let contents_size = file_size - 8;

        let capacity = contents_size / size_of::<T>();

        // Assume a flatdata vector is completely filled.
        let len = if is_flatdata {
            capacity
        } else {
            let header_ptr = mmap.as_ptr() as *const u64;
            unsafe { *header_ptr as usize }
        };

        Ok(Mutant {
            file,
            path,
            mmap,
            len,
            capacity,
            phantom: PhantomData,
        })
    }

    pub fn mutable_slice(&self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.mmap[8..].as_ptr() as *mut T, self.len) }
    }

    pub fn slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.mmap[8..].as_ptr() as *mut T, self.len) }
    }

    /// It's up to you to set what the len is.
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        let header_ptr = self.mmap.as_ptr() as *mut u64;
        unsafe {
            *header_ptr = len as u64;
        }
    }

    /// Trims the file size to the len, making the capacity == len.
    pub fn trim(&mut self) {
        let size = 8 + size_of::<T>() * self.len;
        self.file.set_len(size as u64).unwrap(); // NHTODO ? and return result
        self.capacity = self.len;
    }

    /// Returns a new mutant with the larger size. The old mutant
    /// is passed in, so it gets dropped.
    pub fn grow_by(mutant: Mutant<T>, capacity: usize) -> Result<Mutant<T>> {
        let total_capacity = mutant.capacity + capacity;
        let size = 8 + size_of::<T>() * total_capacity;
        let path = mutant.path.clone();
        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Mutant {
            file,
            path,
            mmap,
            len: mutant.len,
            capacity: total_capacity,
            phantom: PhantomData,
        })
    }
}
