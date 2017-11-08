// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

extern crate cdp;
#[macro_use]
extern crate cdp_derive;
extern crate nsstring;
extern crate serde;

mod page_tools;
mod remote;

pub use remote::{gecko_cdp_content_remote_page_tools_actor_opened,
                 gecko_cdp_content_remote_page_tools_actor_recv,
                 gecko_cdp_content_remote_page_tools_actor_closed};
