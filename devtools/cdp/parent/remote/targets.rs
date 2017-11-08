// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at http://mozilla.org/MPL/2.0/.

use nom::{self, IResult, Needed};

use bridge::{ContentParentId, IdType, TabId};

// TODO: The security model of the Chrome DevTools Protocol depends on the ids
//       used to access the WebSocket tools interface being unpredictable.
//       The same-origin policy is not applied to WebSockets, so arbitrary
//       browser content can attempt to connect to our WebSocket listener. The
//       ids are discoverable over the HTTP portion of the protocol, to which
//       the same-origin policy does apply.
//
//       Suggested implementation: securely generate a random UUID unique to the
//       server instance, to serve as the id for its "browser" endpoint. For
//       "page" endpoint ids, concatenate the information needed to look up the
//       corresponding target in Gecko (content parent id, tab id) onto this
//       unique browser id. On handshake, check against the stored browser id in
//       a constant-time fashion.
//
//       The current, prototype implementation uses predictable ids based simply
//       on hex-encoded (content parent id, tab id).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageId(pub ContentParentId, pub TabId);

impl PageId {
    pub fn encode(&self) -> String {
        let PageId(IdType(content_parent_id), IdType(tab_id)) = *self;

        // Construct an id string which is the concatenation of the fixed-width
        // hex representations of the two component ids.
        format!("{:016x}{:016x}", content_parent_id, tab_id)
    }

    pub fn decode(input: &[u8]) -> Option<PageId> {
        match parse_page_id(input) {
            IResult::Done(rest, page_id) if rest.len() == 0 => Some(page_id),
            _ => None,
        }
    }
}

named!(parse_page_id(&[u8]) -> PageId,
    do_parse!(
        content_parent_id: hex_u64 >>
        tab_id: hex_u64 >>
        (PageId(IdType(content_parent_id), IdType(tab_id)))
    )
);

fn hex_u64(input: &[u8]) -> IResult<&[u8], u64> {
    if input.len() < 16 {
        IResult::Incomplete(Needed::Size(16))
    } else {
        let mut acc: u64 = 0;
        for c in &input[0..16] {
            acc <<= 4;
            match *c {
                c @ b'0'...b'9' => acc += (c - b'0') as u64,
                c @ b'a'...b'f' => acc += (c - b'a' + 10) as u64,
                _ => return IResult::Error(nom::Err::Code(nom::ErrorKind::HexDigit)),
            }
        }
        IResult::Done(&input[16..], acc)
    }
}
