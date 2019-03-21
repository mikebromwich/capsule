/*
* Copyright 2019 Comcast Cable Communications Management, LLC
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
* http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*
* SPDX-License-Identifier: Apache-2.0
*/

use std::fmt;
use packets::icmp::v6::{Icmpv6, Icmpv6Packet, Icmpv6Payload, NdpPayload};

/*  From (https://tools.ietf.org/html/rfc4861#section-4.1)
    Router Solicitation Message Format

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |     Code      |          Checksum             |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                            Reserved                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |   Options ...
    +-+-+-+-+-+-+-+-+-+-+-+-

    Reserved       This field is unused.  It MUST be initialized to
                   zero by the sender and MUST be ignored by the
                   receiver.
    
   Valid Options:

    Source link-layer address
                   The link-layer address of the sender, if
                   known.  MUST NOT be included if the Source Address
                   is the unspecified address.  Otherwise, it SHOULD
                   be included on link layers that have addresses.
*/

/// Router solicitation message
#[derive(Default, Debug)]
#[repr(C, packed)]
pub struct RouterSolicitation {
    reserved: u32
}

impl NdpPayload for RouterSolicitation {}

impl Icmpv6Payload for RouterSolicitation {
    fn size() -> usize {
        4
    }
}

impl Icmpv6<RouterSolicitation> {
    #[inline]
    pub fn reserved(&self) -> u32 {
        u32::from_be(self.payload().reserved)
    }
}

impl fmt::Display for Icmpv6<RouterSolicitation> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {} code: {} checksum: 0x{:04x} reserved: {}",
            self.msg_type(),
            self.code(),
            self.checksum(),
            self.reserved()
        )
    }
}

impl Icmpv6Packet<RouterSolicitation> for Icmpv6<RouterSolicitation> {
    fn payload(&self) -> &mut RouterSolicitation {
        unsafe { &mut (*self.payload) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::{Packet, RawPacket, Ethernet};
    use packets::ip::v6::Ipv6;
    use packets::icmp::v6::{Icmpv6, Icmpv6Types};
    use dpdk_test;

    #[rustfmt::skip]
    const ROUTER_SOLICIT_PACKET: [u8; 70] = [
        // ** ethernet header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        0x86, 0xDD,
        // ** IPv6 header
        0x60, 0x00, 0x00, 0x00,
        // payload length
        0x00, 0x10,
        0x3a,
        0xff,
        0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0xf0, 0x45, 0xff, 0xfe, 0x0c, 0x66, 0x4b,
        0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // ** ICMPv6 header
        // type
        0x85,
        // code
        0x00,
        // checksum
        0xf5, 0x0c,
        // ** router solicitation message
        // reserved
        0x00, 0x00, 0x00, 0x00,
        // ** source link-layer address option
        0x01, 0x01, 0x70, 0x3a, 0xcb, 0x1b, 0xf9, 0x7a
    ];

    #[test]
    fn parse_router_solicitation_packet() {
        dpdk_test! {
            let packet = RawPacket::from_bytes(&ROUTER_SOLICIT_PACKET).unwrap();
            let ethernet = packet.parse::<Ethernet>().unwrap();
            let ipv6 = ethernet.parse::<Ipv6>().unwrap();
            let icmpv6 = ipv6.parse::<Icmpv6<()>>().unwrap();
            let solicit = icmpv6.downcast::<RouterSolicitation>();

            assert_eq!(Icmpv6Types::RouterSolicitation, solicit.msg_type());
            assert_eq!(0, solicit.reserved());
        }
    }
}
