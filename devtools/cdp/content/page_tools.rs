// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use cdp::{self, CdpError, CdpIncoming, DeserializeCdpCommand, SerializeCdpEvent};
use serde::Serialize;

pub trait PageToolsTransport {
    fn send_response<R>(&self, id: u64, response: &R)
    where
        R: Serialize;

    fn send_event<E>(&self, event: &E)
    where
        E: SerializeCdpEvent;

    fn send_error(&self, maybe_id: Option<u64>, error: &CdpError);
}

pub struct PageTools<T> {
    transport: T,
}

impl<T> PageTools<T> {
    pub fn new(transport: T) -> Self {
        PageTools {
            transport: transport,
        }
    }
}

impl<T> PageTools<T>
where
    T: PageToolsTransport,
{
    pub fn handle_incoming(
        &self,
        parse_result: Result<CdpIncoming, (CdpError<'static>, Option<u64>)>,
    ) {
        let incoming = match parse_result {
            Ok(incoming) => incoming,
            Err((err, maybe_id)) => {
                self.transport.send_error(maybe_id, &err);
                return;
            }
        };

        let command = {
            let result = GeckoCdpCommand::deserialize_command(
                &incoming.command_name,
                incoming.command_params,
            );
            match result {
                Ok(Ok(command)) => command,
                Ok(Err(err)) => {
                    self.transport.send_error(
                        Some(incoming.id),
                        &CdpError::invalid_params(err.to_string()),
                    );
                    return;
                }
                Err(_) => {
                    self.transport.send_error(
                        Some(incoming.id),
                        &CdpError::method_not_found(&incoming.command_name),
                    );
                    return;
                }
            }
        };

        match command {
            GeckoCdpCommand::PageNavigate(params) => self.handle_page_navigate(incoming.id, params),
        }
    }

    fn handle_page_navigate(&self, id: u64, _params: cdp::page::NavigateCommand) {
        self.transport.send_error(
            Some(id),
            &CdpError::server_error(
                "Page.navigate recognized but not yet implemented"
                    .to_owned()
                    .into(),
            ),
        );
    }
}

#[derive(DeserializeCdpCommand)]
enum GeckoCdpCommand {
    PageNavigate(cdp::page::NavigateCommand<'static>),
}
