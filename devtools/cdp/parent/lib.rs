// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

extern crate cdp;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate nserror;
extern crate nsstring;
extern crate tokio_cdp;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate websocket;

mod bridge;
mod remote;

pub use remote::{gecko_cdp_parent_remote_page_tools_actor_closed,
                 gecko_cdp_parent_remote_page_tools_actor_recv, gecko_cdp_parent_remote_start,
                 gecko_cdp_parent_remote_stop};
