#![allow(unused, dead_code)]

use core::hash;
use std::{
    alloc::{alloc, dealloc, Layout, LayoutError},
    fmt::{self, Display},
};

/// Api

pub struct GC {}

impl GC {
    pub fn new_string(content: &str) -> GcHandle {
        unsafe { allocate_string(content) }.unwrap()
    }

    pub fn new_concat_string(first: ObjString, second: ObjString) -> GcHandle {
        unsafe { concat_string(first, second) }.unwrap()
    }

    pub fn free(handle: GcHandle) {
        unsafe { deallocate_object(handle.object) }
    }
}

/// Markers
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum ObjectType {
    String,
}

pub(crate) trait IsObject {
    fn otype() -> ObjectType;
    fn from_object(object: Object) -> Self;
    fn upcast(self) -> Object;
}


/// Object Hierarchy / Layout stuff
///
/// Object
///    |
/// ObjString
///
/// Object:    --ptr-to-->   [ [<otype>], .... data ....   ]
/// ObjString: --ptr-to-->   [[[<otype>], len], ...data... ]
///                           ^-StringHeader-^
///                          ^----------StringAlloc--------^
///
/// GcHandle owns the underlying memory and must not be dropped before the corresponding Objects are.

#[derive(Debug, Clone, PartialEq)]
pub struct GcHandle {
    object: Object,
}

impl Drop for GcHandle {
    fn drop(&mut self) {
        unsafe { deallocate_object(self.object) };
    }
}

impl GcHandle {
    pub fn get_object(&self) -> Object {
        return self.object;
    }
}

#[derive(Copy, Clone)]
pub struct Object {
    ptr: *mut Header,
}

#[derive(Copy, Clone, Eq)]
pub struct ObjString {
    ptr: *mut StringHeader,
}

impl IsObject for ObjString {
    fn otype() -> ObjectType {
        ObjectType::String
    }

    fn from_object(object: Object) -> ObjString {
        ObjString { ptr: object.ptr as *mut StringHeader }
    }

    fn upcast(self) -> Object {
        Object { ptr: self.ptr as *mut Header }
    }
}

#[repr(C)]
struct Header {
    otype: ObjectType,
}

#[repr(C)]
struct StringAlloc {
    header: StringHeader,
    data: [u8],
}

#[repr(C)]
struct StringHeader {
    object_header: Header,
    len: usize,
}


const fn data_offset() -> usize {
    std::mem::size_of::<StringHeader>()
}


/// Pretty-print Object
impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_otype() {
            ObjectType::String =>
                fmt::Display::fmt(&self.downcast::<ObjString>().unwrap(), f)
        }
    }
}

impl Object {
    pub fn get_otype(&self) -> ObjectType {
        unsafe { (*self.ptr).otype }
    }

    pub(crate) fn downcast<T: IsObject>(self) -> Option<T> {
        if self.get_otype() == T::otype() {
            Some(T::from_object(self))
        } else {
            None
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_otype() {
            ObjectType::String => {
                let string = self.downcast::<ObjString>().unwrap().as_str();
                write!(f, "STR {} {:?}", string.len(), &string[..8.min(string.len())])
            }
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr == other.ptr {
            return true;
        }

        unsafe {
            if (*self.ptr).otype != (*other.ptr).otype {
                return false;
            }

            match (*self.ptr).otype {
                ObjectType::String => {
                    self.downcast::<ObjString>() == other.downcast::<ObjString>()
                }
            }
        }
    }
}

impl PartialEq for ObjString {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            if (*self.ptr).len != (*other.ptr).len {
                return false;
            }

            self.as_slice() == other.as_slice()
        }
    }
}

impl ObjString {
    fn as_slice<'a>(&self) -> &'a [u8] {
        let length = unsafe { (*self.ptr).len };
        let (layout_, offset)  = StringAlloc::layout(length).unwrap();
        unsafe {
            std::slice::from_raw_parts(
                (self.ptr as *mut u8).offset(offset as isize),
                length
            )
        }
    }

    fn as_str<'a>(&self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }
}

impl fmt::Display for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl std::hash::Hash for ObjString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash::<H>(self.as_str(), state);
    }
}

impl StringAlloc {
    fn layout(length: usize) -> Result<(Layout, usize), LayoutError> {
        let (layout, offset) = Layout::for_value(&StringHeader {
            object_header: Header {
                otype: ObjectType::String,
            },
            len: length,
        })
        .extend(Layout::array::<u8>(length)?)?;

        Ok((layout.pad_to_align(), offset))
    }
}

unsafe fn allocate_string_obj<'a>(
    length: usize,
) -> Result<(GcHandle, &'a mut [u8]), LayoutError> {
    let (layout, offset) = StringAlloc::layout(length)?;
    let allocation = alloc(layout);
    let data_ptr = allocation.offset(offset as isize);
    let header = allocation as *mut StringHeader;
    (*header).len = length;
    (*header).object_header.otype = ObjectType::String;
    let object = Object {
        ptr: header as *mut Header,
    };
    let str = std::slice::from_raw_parts_mut(data_ptr, length);
    Ok((GcHandle { object }, str))
}

unsafe fn allocate_string(content: &str) -> Result<GcHandle, LayoutError> {
    let (gc_handle, slice) = allocate_string_obj(content.len())?;
    slice.copy_from_slice(content.as_bytes());
    Ok(gc_handle)
}

unsafe fn concat_string(a: ObjString, b: ObjString) -> Result<GcHandle, LayoutError> {
    let (a_data,b_data) = (a.as_slice(), b.as_slice());

    let new_len = a_data.len() + b_data.len();
    let (gc_handle, slice) = allocate_string_obj(new_len)?;

    slice[..a_data.len()].copy_from_slice(a_data);
    slice[a_data.len()..].copy_from_slice(b_data);

    Ok(gc_handle)
}

unsafe fn deallocate_object(object: Object) {
    match object.get_otype() {
        ObjectType::String => {
            let header = object.ptr as *mut StringHeader;
            dealloc(
                object.ptr as *mut u8,
                StringAlloc::layout((*header).len).unwrap().0,
            )
        }
    }
}

