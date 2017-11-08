// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use cdp;
use futures::{Async, Future, Poll};
use futures::sync::oneshot;
use std::net::SocketAddr;
use tokio_cdp::greeter::{BrowserInfo, Target, TargetKind, WsEndpoint};
use tokio_cdp::greeter::server::{GreeterError, GreeterRequest, GreeterResponse};
use tokio_service::Service;

use bridge::{async_dispatch_to_main_thread, fetch_content_parent_ids,
             fetch_tab_ids_from_content_parent, GeckoInfo};
use remote::targets::PageId;

#[derive(Clone, Debug)]
pub struct GeckoGreeterService {
    server_addr: String,
    remote_addr: SocketAddr,
}

impl GeckoGreeterService {
    pub fn new(server_addr: String, remote_addr: SocketAddr) -> Self {
        GeckoGreeterService {
            server_addr: server_addr,
            remote_addr: remote_addr,
        }
    }
}

impl Service for GeckoGreeterService {
    type Request = GreeterRequest;
    type Response = GreeterResponse;
    type Error = GreeterError;
    type Future = GeckoGreeterFuture;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!("Incoming greeter request from {}: {:?}", self.remote_addr, req);
        GeckoGreeterFuture::new(
            async_dispatch_to_main_thread(handle_request, (self.server_addr.clone(), req)),
        )
    }
}

pub struct GeckoGreeterFuture {
    receiver: oneshot::Receiver<Result<GreeterResponse, GreeterError>>,
}

impl GeckoGreeterFuture {
    fn new(receiver: oneshot::Receiver<Result<GreeterResponse, GreeterError>>) -> Self {
        GeckoGreeterFuture { receiver: receiver }
    }
}

impl Future for GeckoGreeterFuture {
    type Item = GreeterResponse;
    type Error = GreeterError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.receiver.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(result)) => result.map(Async::Ready),
            Err(_) => Err("Request dropped".into()),
        }
    }
}

extern "C" fn handle_request(
    data: Box<((String, GreeterRequest), oneshot::Sender<Result<GreeterResponse, GreeterError>>)>,
) {
    let ((server_addr, req), sender) = { *data };

    let result = match req {
        GreeterRequest::Browser => handle_browser_request(),
        GreeterRequest::List => handle_list_request(server_addr),
        _ => Err(format!("Unsupported command: {}", req.command_name()).into()),
    };

    let _ = sender.send(result);
}

fn handle_browser_request() -> Result<GreeterResponse, GreeterError> {
    let gecko_info = GeckoInfo::fetch()
        .map_err(|err| GreeterError::from(format!("Error retrieving data: {}", err)))?;

    let name = gecko_info
        .name()
        .map_err(|err| GreeterError::from(format!("Error decoding browser name: {}", err)))?;
    let version = gecko_info
        .version()
        .map_err(|err| GreeterError::from(format!("Error decoding browser version: {}", err)))?;
    let user_agent = gecko_info
        .user_agent()
        .map_err(|err| GreeterError::from(format!("Error decoding browser user agent: {}", err)))?;

    let id = format!("{}/{}", name, version);

    let browser_info = BrowserInfo {
        id: id.into(),
        user_agent: user_agent.into(),
        ws_url: None,
        protocol_version: cdp::STABLE_PROTOCOL_VERSION.into(),
        webkit_version: None,
        v8_version: None,
        android_package: None,
    };
    Ok(GreeterResponse::browser(&browser_info))
}

fn handle_list_request(server_addr: String) -> Result<GreeterResponse, GreeterError> {
    // TODO: Populate the list with a broader range of targets (extensions,
    //       service workers, etc).

    let mut targets = Vec::new();

    let content_parent_ids = fetch_content_parent_ids()
        .ok_or_else(|| GreeterError::from("Failed to fetch content parent ids"))?;

    for content_parent_id in content_parent_ids {
        let tab_ids = fetch_tab_ids_from_content_parent(content_parent_id).ok_or_else(|| {
            GreeterError::from(
                format!("Failed to fetch tab ids from content parent {}", content_parent_id),
            )
        })?;

        for tab_id in tab_ids {
            // TODO: Retrieve more information from each target (title, url,
            //       etc).

            let id = PageId(content_parent_id, tab_id).encode();
            let ws_url = WsEndpoint::page_endpoint_url(&server_addr, &id);
            targets.push(Target {
                id: id.into(),
                kind: TargetKind::Page,
                url: "".into(),
                title: "".into(),
                description: None,
                favicon_url: None,
                ws_url: ws_url.into(),
                frontend_url: None,
            })
        }
    }

    Ok(GreeterResponse::list(targets.iter()))
}
