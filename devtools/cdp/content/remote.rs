// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use cdp::{CdpError, CdpIncoming, CdpOutgoing, SerializeCdpEvent};
use nsstring::nsCString;
use serde::Serialize;
use std::mem;
use std::os::raw::c_void;

use page_tools::{PageTools, PageToolsTransport};

extern "C" {
    fn gecko_cdp_content_remote_page_tools_actor_send(
        actor_ptr: *mut c_void,
        msg: *const nsCString<'static>,
    ) -> bool;

    fn gecko_cdp_content_remote_page_tools_actor_drop(actor_ptr: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_content_remote_page_tools_actor_opened(
    actor_ptr: *mut c_void,
) -> *mut c_void {
    let transport = RemotePageToolsTransport::new(actor_ptr);
    let tools = PageTools::new(transport);
    Box::into_raw(Box::new(tools)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_content_remote_page_tools_actor_recv<'a>(
    tools_ptr: *mut c_void,
    msg: *const nsCString<'a>,
) {
    let parse_result = CdpIncoming::parse_from_slice((*msg).as_ref());
    (*(tools_ptr as *mut RemotePageTools)).handle_incoming(parse_result);
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_content_remote_page_tools_actor_closed(tools_ptr: *mut c_void) {
    mem::drop(Box::<RemotePageTools>::from_raw(tools_ptr as *mut _));
}

type RemotePageTools = PageTools<RemotePageToolsTransport>;

struct RemotePageToolsTransport {
    actor_ptr: *mut c_void,
}

impl RemotePageToolsTransport {
    unsafe fn new(actor_ptr: *mut c_void) -> Self {
        RemotePageToolsTransport {
            actor_ptr: actor_ptr,
        }
    }

    fn send(&self, msg: String) {
        unsafe {
            gecko_cdp_content_remote_page_tools_actor_send(self.actor_ptr, &nsCString::from(msg));
        }
    }
}

impl Drop for RemotePageToolsTransport {
    fn drop(&mut self) {
        unsafe {
            gecko_cdp_content_remote_page_tools_actor_drop(self.actor_ptr);
        }
    }
}

impl PageToolsTransport for RemotePageToolsTransport {
    fn send_response<R>(&self, id: u64, response: &R)
    where
        R: Serialize,
    {
        let mut msg = String::new();

        match CdpOutgoing::serialize_response_to_string(&mut msg, id, response) {
            Ok(()) => self.send(msg),
            Err(err) => {
                msg.clear();

                CdpOutgoing::serialize_error_to_string(
                    &mut msg,
                    Some(id),
                    &CdpError::internal_error(format!("Error serializing response: {}", err)),
                ).expect(
                    "gecko-cdp-content: serialization of response serialization error failed",
                );

                self.send(msg);
            }
        }
    }

    fn send_event<E>(&self, event: &E)
    where
        E: SerializeCdpEvent,
    {
        let mut msg = String::new();

        match CdpOutgoing::serialize_event_to_string(&mut msg, event) {
            Ok(()) => self.send(msg),
            Err(err) => {
                msg.clear();

                CdpOutgoing::serialize_error_to_string(
                    &mut msg,
                    None,
                    &CdpError::internal_error(format!(
                        "Error serializing {} event: {}",
                        event.event_name(),
                        err
                    )),
                ).expect(
                    "gecko-cdp-content: serialization of event serialization error failed",
                );

                self.send(msg);
            }
        }
    }

    fn send_error(&self, maybe_id: Option<u64>, error: &CdpError) {
        let mut msg = String::new();

        match CdpOutgoing::serialize_error_to_string(&mut msg, maybe_id, error) {
            Ok(()) => self.send(msg),
            Err(err) => {
                msg.clear();

                CdpOutgoing::serialize_error_to_string(
                    &mut msg,
                    maybe_id,
                    &CdpError::internal_error(format!("Error serializing error: {}", err)),
                ).expect(
                    "gecko-cdp-content: serialization of error serialization error failed",
                );

                self.send(msg);
            }
        }
    }
}
