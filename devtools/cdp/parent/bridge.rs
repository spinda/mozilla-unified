// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use futures::sync::oneshot;
use nserror::{nsresult, NsresultExt};
use nsstring::nsCString;
use std::fmt;
use std::io;
use std::mem;
use std::os::raw::c_void;
use std::str::{self, Utf8Error};

pub fn nsresult_to_result(result: nsresult) -> Result<(), io::Error> {
    if result.succeeded() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, result.error_name().to_string()))
    }
}

extern "C" {
    fn gecko_cdp_parent_bridge_dispatch_to_main_thread(
        callback: extern "C" fn(*mut c_void),
        data_ptr: *mut c_void,
    );
}

pub fn dispatch_to_main_thread<T>(callback: extern "C" fn(Box<T>), data: T)
where
    T: 'static + Send,
{
    let boxed_data = Box::new(data);
    unsafe {
        gecko_cdp_parent_bridge_dispatch_to_main_thread(
            mem::transmute(callback),
            mem::transmute(boxed_data),
        );
    }
}

pub fn async_dispatch_to_main_thread<T, U>(
    callback: extern "C" fn(Box<(T, oneshot::Sender<U>)>),
    data: T,
) -> oneshot::Receiver<U>
where
    T: 'static + Send,
    oneshot::Sender<U>: 'static + Send,
{
    let (sender, receiver) = oneshot::channel();
    dispatch_to_main_thread(callback, (data, sender));
    receiver
}

extern "C" {
    fn gecko_cdp_parent_bridge_fetch_gecko_info(gecko_info: *mut GeckoInfo) -> nsresult;
}

#[repr(C)]
pub struct GeckoInfo {
    name: nsCString<'static>,
    version: nsCString<'static>,
    user_agent: nsCString<'static>,
}

impl GeckoInfo {
    fn new() -> Self {
        GeckoInfo {
            name: nsCString::new(),
            version: nsCString::new(),
            user_agent: nsCString::new(),
        }
    }

    pub fn fetch() -> Result<GeckoInfo, io::Error> {
        let mut info = GeckoInfo::new();
        nsresult_to_result(unsafe { gecko_cdp_parent_bridge_fetch_gecko_info(&mut info) })
            .map(|_| info)
    }

    pub fn name(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.name)
    }

    pub fn version(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.version)
    }

    pub fn user_agent(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.user_agent)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdType(pub u64);

impl fmt::Display for IdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

extern "C" {
    fn gecko_cdp_parent_bridge_id_iterator_next(iterator: *mut c_void, item: *mut IdType) -> bool;
    fn gecko_cdp_parent_bridge_id_iterator_drop(iterator: *mut c_void);
}

pub struct IdIterator {
    handle: *mut c_void,
}

impl IdIterator {
    unsafe fn new(handle: *mut c_void) -> Self {
        IdIterator { handle: handle }
    }
}

impl Iterator for IdIterator {
    type Item = IdType;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = IdType(0);
        let has_next = unsafe { gecko_cdp_parent_bridge_id_iterator_next(self.handle, &mut item) };
        if has_next {
            Some(item)
        } else {
            None
        }
    }
}

impl Drop for IdIterator {
    fn drop(&mut self) {
        unsafe {
            gecko_cdp_parent_bridge_id_iterator_drop(self.handle);
        }
    }
}

pub type ContentParentId = IdType;
pub type TabId = IdType;

extern "C" {
    fn gecko_cdp_parent_bridge_fetch_content_parent_ids() -> *mut c_void;
    fn gecko_cdp_parent_bridge_fetch_tab_ids_from_content_parent(
        content_parent_id: ContentParentId,
    ) -> *mut c_void;
}

pub fn fetch_content_parent_ids() -> Option<IdIterator> {
    let handle = unsafe { gecko_cdp_parent_bridge_fetch_content_parent_ids() };

    if handle.is_null() {
        None
    } else {
        Some(unsafe { IdIterator::new(handle) })
    }
}

pub fn fetch_tab_ids_from_content_parent(
    content_parent_id: ContentParentId,
) -> Option<IdIterator> {
    let handle =
        unsafe { gecko_cdp_parent_bridge_fetch_tab_ids_from_content_parent(content_parent_id) };

    if handle.is_null() {
        None
    } else {
        Some(unsafe { IdIterator::new(handle) })
    }
}
