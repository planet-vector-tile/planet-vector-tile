#![allow(dead_code)]

use core::mem::size_of;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use memmap2::MmapMut;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::ptr::copy_nonoverlapping;
use std::{
    fs,
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
    is_flatdata: bool,
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
            is_flatdata: false,
            phantom: PhantomData,
        })
    }

    pub fn new_from_flatdata(
        dir: &Path,
        file_name: &str,
        flatdata_file_name: &str,
    ) -> Result<Self> {
        let flatdata_path = dir.join(flatdata_file_name);
        let flatdata_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&flatdata_path)?;
        let fd_mmap = unsafe { MmapMut::map_mut(&flatdata_file)? };
        let fd_header_ptr = fd_mmap.as_ptr() as *const u64;
        let flatdata_header = unsafe { *fd_header_ptr as u64 };

        let file_size = flatdata_file.metadata()?.len();
        let contents_size = (file_size - 8) as usize;
        let len = contents_size / size_of::<T>();

        let path = dir.join(file_name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        file.set_len(file_size)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        let header_ptr = mmap.as_ptr() as *mut u64;
        unsafe {
            *header_ptr = flatdata_header;
        }

        Ok(Mutant {
            file,
            path,
            mmap,
            len,
            capacity: len,
            is_flatdata: true,
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
            is_flatdata: false,
            phantom: PhantomData,
        })
    }

    /// Opens the file to memmap. If the file was generated by flatdata,
    /// we keep track of this so that we do not tamper with the first 8 bytes.
    /// We use those to set the length, but flatdata uses it differently.
    pub fn open(dir: &Path, file_name: &str, is_flatdata: bool) -> Result<Self> {
        // Don't use the header as info on length for flatdata,
        // as flatdata uses the header differently...

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
            is_flatdata,
            phantom: PhantomData,
        })
    }

    pub fn mv(&mut self, new_name: &str) -> Result<()> {
        let mut path = self.path.clone();
        path.pop();
        path.push(new_name);
        let _ = fs::remove_file(&path);
        fs::rename(&self.path, path)
    }

    pub fn mutable_slice(&self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.mmap[8..].as_ptr() as *mut T, self.len) }
    }

    pub fn slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.mmap[8..].as_ptr() as *mut T, self.len) }
    }

    /// Append a value to the end of the array, similar to Vec.
    /// Grows the underlying file by 2x when the capacity is reached.
    pub fn push(&mut self, item: T) -> Result<&mut Self> {
        let len = self.len;
        if len >= self.capacity {
            let cap2x = self.capacity * 2;
            let size = 8 + size_of::<T>() * cap2x;
            self.file.set_len(size as u64)?;
            self.mmap = unsafe { MmapMut::map_mut(&self.file)? };
            self.capacity = cap2x;
        }

        self.len += 1;
        let slc = self.mutable_slice();
        slc[len] = item;

        Ok(self)
    }

    pub fn append(&mut self, items: &[T]) -> Result<&mut Self> {
        let new_len = self.len + items.len();
        if new_len > self.capacity {
            let mut new_cap = self.capacity * 2;
            if new_len > new_cap {
                new_cap = new_len;
            }
            let size = 8 + size_of::<T>() * new_cap;
            self.file.set_len(size as u64)?;
            self.mmap = unsafe { MmapMut::map_mut(&self.file)? };
            self.capacity = new_cap;
        }

        let slc = self.mutable_slice();
        let src = items.as_ptr();
        let dst = slc[self.len..].as_mut_ptr();
        unsafe {
            copy_nonoverlapping(src, dst, items.len());
        };

        // NHTODO: It would be better do do a safe copy_from_slice,
        // but flatdata structs do not implement copy trait.
        // let slc = self.mutable_slice();
        // slc[self.len..new_len].copy_from_slice(items);

        self.len = new_len;

        Ok(self)
    }

    pub fn expand_to(&mut self, len: usize) -> Result<&mut Self> {
        if len < self.len {
            return Err(Error::new(
                ErrorKind::Other,
                "expand_to input len is shorter than the current length.",
            ));
        }

        if len < self.capacity {
            self.set_len(len);
        } else {
            let size = 8 + size_of::<T>() * len;
            self.file.set_len(size as u64)?;
            self.set_len(len);
            self.mmap = unsafe { MmapMut::map_mut(&self.file)? };
        }
        Ok(self)
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        if !self.is_flatdata {
            let header_ptr = self.mmap.as_ptr() as *mut u64;
            unsafe {
                *header_ptr = len as u64;
            }
        }
    }

    /// Trims the file size to the len, making the capacity == len.
    pub fn trim(&mut self) {
        let size = 8 + size_of::<T>() * self.len;
        self.file.set_len(size as u64).unwrap();
        self.capacity = self.len;
        if !self.is_flatdata {
            let header_ptr = self.mmap.as_ptr() as *mut u64;
            unsafe {
                *header_ptr = self.len as u64;
            }
        }
    }
}
