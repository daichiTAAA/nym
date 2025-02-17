// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use nymsphinx_addressing::nodes::{NymNodeRoutingAddress, NymNodeRoutingAddressError};
use nymsphinx_params::{PacketMode, PacketSize};
use nymsphinx_types::SphinxPacket;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum MixPacketFormattingError {
    TooFewBytesProvided,
    InvalidPacketMode,
    InvalidPacketSize(usize),
    InvalidAddress,
    MalformedSphinxPacket,
}

impl Display for MixPacketFormattingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use MixPacketFormattingError::*;
        match self {
            TooFewBytesProvided => write!(f, "Too few bytes provided to recover from bytes"),
            InvalidAddress => write!(f, "address field was incorrectly encoded"),
            InvalidPacketSize(actual) =>
                write!(
                    f,
                    "received request had invalid size. (actual: {}, but expected one of: {} (ACK), {} (REGULAR), {}, {}, {} (EXTENDED))",
                    actual, PacketSize::AckPacket.size(), PacketSize::RegularPacket.size(),
                    PacketSize::ExtendedPacket8.size(), PacketSize::ExtendedPacket16.size(),
                    PacketSize::ExtendedPacket32.size()
                ),
            MalformedSphinxPacket => write!(f, "received sphinx packet was malformed"),
            InvalidPacketMode => write!(f, "provided packet mode is invalid")
        }
    }
}

impl std::error::Error for MixPacketFormattingError {}

impl From<NymNodeRoutingAddressError> for MixPacketFormattingError {
    fn from(_: NymNodeRoutingAddressError) -> Self {
        MixPacketFormattingError::InvalidAddress
    }
}

pub struct MixPacket {
    next_hop: NymNodeRoutingAddress,
    sphinx_packet: SphinxPacket,
    packet_mode: PacketMode,
}

impl Debug for MixPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MixPacket to {:?} with packet_mode {:?}. Sphinx header: {:?}, payload length: {}",
            self.next_hop,
            self.packet_mode,
            self.sphinx_packet.header,
            self.sphinx_packet.payload.len()
        )
    }
}

impl MixPacket {
    pub fn new(
        next_hop: NymNodeRoutingAddress,
        sphinx_packet: SphinxPacket,
        packet_mode: PacketMode,
    ) -> Self {
        MixPacket {
            next_hop,
            sphinx_packet,
            packet_mode,
        }
    }

    pub fn next_hop(&self) -> NymNodeRoutingAddress {
        self.next_hop
    }

    pub fn sphinx_packet(&self) -> &SphinxPacket {
        &self.sphinx_packet
    }

    pub fn into_sphinx_packet(self) -> SphinxPacket {
        self.sphinx_packet
    }

    pub fn packet_mode(&self) -> PacketMode {
        self.packet_mode
    }

    // the message is formatted as follows:
    // PACKET_MODE || FIRST_HOP || SPHINX_PACKET
    pub fn try_from_bytes(b: &[u8]) -> Result<Self, MixPacketFormattingError> {
        let packet_mode = match PacketMode::try_from(b[0]) {
            Ok(mode) => mode,
            Err(_) => return Err(MixPacketFormattingError::InvalidPacketMode),
        };

        let next_hop = NymNodeRoutingAddress::try_from_bytes(&b[1..])?;
        let addr_offset = next_hop.bytes_min_len();

        let sphinx_packet_data = &b[addr_offset + 1..];
        let packet_size = sphinx_packet_data.len();
        if PacketSize::get_type(packet_size).is_err() {
            Err(MixPacketFormattingError::InvalidPacketSize(packet_size))
        } else {
            let sphinx_packet = match SphinxPacket::from_bytes(sphinx_packet_data) {
                Ok(packet) => packet,
                Err(_) => return Err(MixPacketFormattingError::MalformedSphinxPacket),
            };

            Ok(MixPacket {
                next_hop,
                sphinx_packet,
                packet_mode,
            })
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        std::iter::once(self.packet_mode as u8)
            .chain(self.next_hop.as_bytes().into_iter())
            .chain(self.sphinx_packet.to_bytes().into_iter())
            .collect()
    }
}

// TODO: test for serialization and errors!
