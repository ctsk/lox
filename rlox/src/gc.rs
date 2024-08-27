use std::{alloc::{alloc, dealloc, Layout, LayoutError}, fmt};

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum ObjectType {
    String,
}

#[repr(C)]
struct ObjectHeader {
    otype: ObjectType,
}

#[repr(C)]
struct ObjStringHeader {
    object_header: ObjectHeader,
    len: usize,
}

#[repr(C)]
struct ObjString {
    header: ObjStringHeader,
    data: [u8],
}

const fn data_offset() -> usize {
    std::mem::size_of::<ObjStringHeader>()
}

#[derive(Copy, Clone)]
pub struct Object {
    ptr: *mut ObjectHeader,
}

impl Object {
    pub fn get_otype(&self) -> ObjectType {
        unsafe {
            (*self.ptr).otype
        }
    }

}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_otype() {
            ObjectType::String => {
                let string = self.ptr as *mut ObjStringHeader;
                let data: &[u8] = ObjString::as_slice(string);
                write!(
                    f,
                    "STR {} {:?}",
                    data.len(),
                    &data[..8.min(data.len())],
                )
            },
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
                    let header = self.ptr as *mut ObjStringHeader;
                    let other_header = other.ptr as *mut ObjStringHeader;

                    if (*header).len != (*other_header).len {
                        return false;
                    }

                    let slice = ObjString::as_slice(header);
                    let other_slice = ObjString::as_slice(other_header);

                    slice == other_slice
                },
            }
        }
    }
}

impl ObjString {
    fn layout(length: usize) -> Result<(Layout, usize), LayoutError> {
        let (layout, offset) = Layout::for_value(&ObjStringHeader {
            object_header: ObjectHeader {
                otype: ObjectType::String,
            },
            len: length,
        })
        .extend(Layout::array::<u8>(length)?)?;

        Ok((layout.pad_to_align(), offset))
    }

    fn as_slice<'a>(ptr: *mut ObjStringHeader) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                (ptr as *mut u8).offset(data_offset() as isize),
                (*ptr).len
            )
        }
    }
}

pub unsafe fn allocate_string_obj<'a>(length: usize) -> Result<(GcHandle, &'a mut [u8]), LayoutError> {
    let (layout, offset) = ObjString::layout(length)?;
    let allocation = alloc(layout);
    let data_ptr = allocation.offset(offset as isize);
    let header = allocation as *mut ObjStringHeader;
    (*header).len = length;
    (*header).object_header.otype = ObjectType::String;
    let object = Object { ptr: header as *mut ObjectHeader };
    let str = std::slice::from_raw_parts_mut(data_ptr, length);
    Ok((GcHandle { object }, str))
}

pub unsafe fn allocate_string(content: &str) -> Result<GcHandle, LayoutError> {
    let (gc_handle, slice) = allocate_string_obj(content.len())?;
    slice.copy_from_slice(content.as_bytes());
    Ok(gc_handle)
}

pub unsafe fn concat_string(a: Object, b: Object) -> Result<GcHandle, LayoutError> {
    let a_head = a.ptr as *mut ObjStringHeader;
    let b_head = b.ptr as *mut ObjStringHeader;
    let a_data = ObjString::as_slice(a_head);
    let b_data = ObjString::as_slice(b_head);
    let new_len = a_data.len() + b_data.len();

    let (gc_handle,  slice) = allocate_string_obj(new_len)?;

    slice[..a_data.len()].copy_from_slice(a_data);
    slice[a_data.len()..].copy_from_slice(b_data);

    Ok(gc_handle)
}

unsafe fn deallocate_object(object: Object) {
    match object.get_otype() {
        ObjectType::String => {
            let header = object.ptr as *mut ObjStringHeader;
            dealloc(
                object.ptr as *mut u8,
                ObjString::layout((*header).len).unwrap().0,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GcHandle {
    object: Object
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
