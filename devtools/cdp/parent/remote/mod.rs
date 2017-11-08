// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use futures::future::{self, Either};
use futures::sync::{mpsc, oneshot};
use nsstring::nsCString;
use std::ffi::CStr;
use std::io;
use std::mem;
use std::net::{Ipv4Addr, SocketAddr};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::str::FromStr;
use std::thread;
use tokio_cdp::greeter::WsEndpoint;
use tokio_cdp::greeter::server::GreeterServer;
use tokio_cdp::tools::server::ToolsServerStart;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_io::{AsyncRead, AsyncWrite};
use websocket::result::WebSocketError;

mod greeter;
mod targets;

use bridge::{async_dispatch_to_main_thread, dispatch_to_main_thread, ContentParentId, TabId};
use remote::greeter::GeckoGreeterService;
use remote::targets::PageId;

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_parent_remote_start(port_str: *const c_char) -> *mut c_void {
    let port_str_decoded = CStr::from_ptr(port_str).to_string_lossy();
    let port = match u16::from_str(&port_str_decoded) {
        Ok(port) => port,
        Err(_) => {
            eprintln!("*** Invalid port specified for CDP remote server: {}", port_str_decoded);
            return ptr::null_mut();
        }
    };

    let server_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
    let shutdown_sender = start_server_thread(server_addr);
    Box::into_raw(Box::new(shutdown_sender)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_parent_remote_stop(server_ptr: *mut c_void) {
    let _ = Box::<oneshot::Sender<()>>::from_raw(server_ptr as *mut _).send(());
}

fn start_server_thread(server_addr: SocketAddr) -> oneshot::Sender<()> {
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    thread::spawn(move || if let Err(err) = run_server(server_addr, shutdown_receiver) {
        error!("Remote server stopped due to an error: {}", err);
    });
    shutdown_sender
}

fn run_server(
    server_addr: SocketAddr,
    shutdown_receiver: oneshot::Receiver<()>,
) -> Result<(), io::Error> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let listener = TcpListener::bind(&server_addr, &handle)?;
    let server_addr = listener.local_addr()?.to_string();
    eprintln!("*** CDP remote server listening on {}", server_addr);

    let accept_connections = listener.incoming().for_each(move |(tcp, remote_addr)| {
        handle.spawn(serve_connection(&handle, tcp, server_addr.clone(), remote_addr));
        Ok(())
    });

    let main_loop = accept_connections
        .select(shutdown_receiver.then(|_| {
            eprintln!("*** CDP remote server shutting down");
            Ok(())
        }))
        .map(|_| ())
        .map_err(|x| x.0);

    core.run(main_loop)
}

fn serve_connection<T>(
    handle: &Handle,
    io: T,
    server_addr: String,
    remote_addr: SocketAddr,
) -> Box<Future<Item = (), Error = ()>>
where
    T: 'static + AsyncRead + AsyncWrite,
{
    debug!("Accepted connection from {}", remote_addr);

    let service = GeckoGreeterService::new(server_addr, remote_addr);

    Box::new(
        GreeterServer::bind(&handle, io, remote_addr, service)
            .map_err(move |err| {
                error!("Error serving greeter session for {}: {}", remote_addr, err);
            })
            .and_then(move |maybe_handshake| {
                let (ws_endpoint, server_start) = match maybe_handshake {
                    Some(handshake) => handshake,
                    None => return Either::A(future::ok((()))),
                };

                debug!("Received tools handshake for {:?} from {}", ws_endpoint, remote_addr);

                Either::B(
                    match ws_endpoint {
                        WsEndpoint::Browser(maybe_browser_id) => {
                            serve_browser_tools(maybe_browser_id, server_start)
                        }
                        WsEndpoint::Page(page_id) => serve_page_tools(&page_id, server_start),
                    }.map_err(move |err| {
                        error!("Error serving tools session for {}: {}", remote_addr, err);
                    }),
                )
            })
            .map(move |_| {
                debug!("Connection to {} closed", remote_addr);
            }),
    )
}

fn serve_browser_tools<T>(
    _maybe_browser_id: Option<String>,
    server_start: ToolsServerStart<T>,
) -> Box<Future<Item = (), Error = WebSocketError>>
where
    T: 'static + AsyncRead + AsyncWrite,
{
    // TODO: Implement "browser" tools endpoint which exposes the `Target`
    //       domain.
    Box::new(server_start.reject())
}

fn serve_page_tools<T>(
    page_id: &str,
    server_start: ToolsServerStart<T>,
) -> Box<Future<Item = (), Error = WebSocketError>>
where
    T: 'static + AsyncRead + AsyncWrite,
{
    let page_id = match PageId::decode(page_id.as_bytes()) {
        Some(page_id) => page_id,
        None => return Box::new(server_start.reject()),
    };

    Box::new(
        async_dispatch_to_main_thread(start_page_tools, page_id)
            .map_err(|err| io::Error::new(io::ErrorKind::ConnectionReset, err).into())
            .and_then(move |maybe_actor| match maybe_actor {
                None => Either::A(server_start.reject()),
                Some(actor) => Either::B(server_start.accept().and_then(move |server| {
                    let (server_sink, server_stream) = server.split();
                    let (actor_sink, actor_stream) = actor.split();

                    let server_to_actor = server_stream.forward(actor_sink).map(|_| ());
                    let actor_to_server = actor_stream.from_err().forward(server_sink).map(|_| ());

                    // Use `select` so that we drop both halves of the
                    // connection if either half closes.
                    server_to_actor.select(actor_to_server).map(|x| x.0).map_err(|x| x.0)
                })),
            }),
    )
}

extern "C" fn start_page_tools(
    data: Box<(PageId, oneshot::Sender<Option<RemotePageToolsActor>>)>,
) {
    let (PageId(content_parent_id, tab_id), result_sender) = { *data };

    let (actor_sender, actor_receiver) = mpsc::unbounded::<String>();
    let actor_sender_ptr = Box::into_raw(Box::new(actor_sender)) as *mut _;

    let mut actor_ptr = ptr::null_mut();
    unsafe {
        gecko_cdp_parent_remote_page_tools_actor_start(
            content_parent_id,
            tab_id,
            actor_sender_ptr,
            &mut actor_ptr,
        )
    };
    let maybe_actor = if actor_ptr.is_null() {
        None
    } else {
        Some(unsafe { RemotePageToolsActor::new(actor_ptr, actor_receiver) })
    };

    let _ = result_sender.send(maybe_actor);
}

extern "C" {
    fn gecko_cdp_parent_remote_page_tools_actor_start(
        content_parent_id: ContentParentId,
        tab_id: TabId,
        actor_sender_ptr: *mut c_void,
        actor_ptr: *mut (*mut c_void),
    );

    fn gecko_cdp_parent_remote_page_tools_actor_send(
        actor_ptr: *mut c_void,
        msg: *const nsCString<'static>,
    ) -> bool;

    fn gecko_cdp_parent_remote_page_tools_actor_close(actor_ptr: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_parent_remote_page_tools_actor_recv<'a>(
    actor_sender_ptr: *mut c_void,
    msg: *const nsCString<'a>,
) {
    // The message is encoded from a valid UTF-8 Rust String on the content
    // side, so it will still be valid UTF-8 when it reaches this end.
    let msg_string = String::from_utf8_unchecked((*msg).as_ref().to_owned());

    let _ = (*(actor_sender_ptr as *mut mpsc::UnboundedSender<String>)).unbounded_send(msg_string);
}

#[no_mangle]
pub unsafe extern "C" fn gecko_cdp_parent_remote_page_tools_actor_closed(
    actor_sender_ptr: *mut c_void,
) {
    mem::drop(Box::<mpsc::UnboundedSender<String>>::from_raw(actor_sender_ptr as *mut _));
}

pub struct RemotePageToolsActor {
    actor_ptr: *mut c_void,
    actor_receiver: mpsc::UnboundedReceiver<String>,
}

impl RemotePageToolsActor {
    unsafe fn new(
        actor_ptr: *mut c_void,
        actor_receiver: mpsc::UnboundedReceiver<String>,
    ) -> Self {
        RemotePageToolsActor {
            actor_ptr: actor_ptr,
            actor_receiver: actor_receiver,
        }
    }
}

unsafe impl Send for RemotePageToolsActor {}

impl Drop for RemotePageToolsActor {
    fn drop(&mut self) {
        extern "C" fn callback(data: Box<SendablePtr>) {
            let SendablePtr(actor_ptr) = *data;
            unsafe {
                gecko_cdp_parent_remote_page_tools_actor_close(actor_ptr);
            }
        }

        dispatch_to_main_thread(callback, SendablePtr(self.actor_ptr));
    }
}

impl Stream for RemotePageToolsActor {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.actor_receiver.poll().map_err(|()| io::Error::from(io::ErrorKind::ConnectionReset))
    }
}

impl Sink for RemotePageToolsActor {
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        // TODO: Avoid jumping to the main thread to send messages to the child
        //       actor, perhaps using PBackground. Then check the return value
        //       of gecko_cdp_parent_remote_page_tools_actor_send and return a
        //       ConnectionReset error if false (message not sent).

        extern "C" fn callback(data: Box<(SendablePtr, String)>) {
            let (SendablePtr(actor_ptr), item) = { *data };
            unsafe {
                gecko_cdp_parent_remote_page_tools_actor_send(actor_ptr, &nsCString::from(item));
            }
        }

        dispatch_to_main_thread(callback, (SendablePtr(self.actor_ptr), item));
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

struct SendablePtr(*mut c_void);
unsafe impl Send for SendablePtr {}
