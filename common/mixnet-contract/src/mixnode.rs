// due to code generated by JsonSchema
#![allow(clippy::field_reassign_with_default)]

use crate::{IdentityKey, SphinxKey};
use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixNode {
    pub host: String,
    pub mix_port: u16,
    pub verloc_port: u16,
    pub http_api_port: u16,
    pub layer: u64,
    pub location: String,
    pub sphinx_key: SphinxKey,
    /// Base58 encoded ed25519 EdDSA public key.
    pub identity_key: IdentityKey,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixNodeBond {
    // TODO:
    // JS: When we go onto the next testnet and we have to make incompatible changes in the contract (such as upgrade to cosmwasm 0.14+)
    // I'd change `amount` from `Vec<Coin>` to just `Coin` (or maybe even `Uint128` since denomination is implicit)
    // I would also put here field like `total_delegation` which would also be a `Coin` or `Uint128` that
    // indicates the sum of all delegations towards this node
    //
    // I would also modify the `MixNode` struct:
    //  - remove `location` field
    //  - introduce `rest_api_port` field
    //  - [POTENTIALLY] introduce `verloc_port` field or keep it accessible via http api
    //
    // I would also introduce the identical changes to GatewayBond
    pub amount: Vec<Coin>,
    pub owner: Addr,
    pub mix_node: MixNode,
}

impl MixNodeBond {
    pub fn new(amount: Vec<Coin>, owner: Addr, mix_node: MixNode) -> Self {
        MixNodeBond {
            amount,
            owner,
            mix_node,
        }
    }

    pub fn identity(&self) -> &String {
        &self.mix_node.identity_key
    }

    pub fn amount(&self) -> &[Coin] {
        &self.amount
    }

    pub fn owner(&self) -> &Addr {
        &self.owner
    }

    pub fn mix_node(&self) -> &MixNode {
        &self.mix_node
    }
}

impl Display for MixNodeBond {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        if self.amount.len() != 1 {
            write!(f, "amount: {:?}, owner: {}", self.amount, self.owner)
        } else {
            write!(
                f,
                "amount: {} {}, owner: {}",
                self.amount[0].amount, self.amount[0].denom, self.owner
            )
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct PagedResponse {
    pub nodes: Vec<MixNodeBond>,
    pub per_page: usize,
    pub start_next_after: Option<IdentityKey>,
}

impl PagedResponse {
    pub fn new(
        nodes: Vec<MixNodeBond>,
        per_page: usize,
        start_next_after: Option<IdentityKey>,
    ) -> Self {
        PagedResponse {
            nodes,
            per_page,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixOwnershipResponse {
    pub address: Addr,
    pub has_node: bool,
}
