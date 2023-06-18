// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::helpers::{BatchCertificate, SealedBatch};
use snarkvm::console::{prelude::*, types::Address};

use parking_lot::RwLock;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

pub struct Shared<N: Network> {
    /// A map of `address` to `stake`.
    committee: RwLock<HashMap<Address<N>, u64>>,
    /// The current round number.
    round: AtomicU64,
    /// The current block height.
    height: AtomicU32,
    /// A map of `round` number to a map of `addresses` to `sealed batches`.
    sealed_batches: RwLock<HashMap<u64, HashMap<Address<N>, SealedBatch<N>>>>,
    /// A map of `peer IP` to `address`.
    peer_addresses: RwLock<HashMap<SocketAddr, Address<N>>>,
    /// A map of `address` to `peer IP`.
    address_peers: RwLock<HashMap<Address<N>, SocketAddr>>,
}

impl<N: Network> Shared<N> {
    /// Initializes a new `Shared` instance.
    pub fn new(round: u64, height: u32) -> Self {
        Self {
            committee: Default::default(),
            round: AtomicU64::new(round),
            height: AtomicU32::new(height),
            sealed_batches: Default::default(),
            peer_addresses: Default::default(),
            address_peers: Default::default(),
        }
    }

    /// Adds a validator to the committee.
    pub fn add_validator(&self, address: Address<N>, stake: u64) -> Result<()> {
        // Check if the validator is already in the committee.
        if self.is_committee_member(&address) {
            bail!("Validator already in committee");
        }

        // Add the validator to the committee.
        self.committee.write().insert(address, stake);
        Ok(())
    }
}

impl<N: Network> Shared<N> {
    /// Returns the current round number.
    pub fn round(&self) -> u64 {
        self.round.load(Ordering::Relaxed)
    }

    /// Returns the current block height.
    pub fn height(&self) -> u32 {
        self.height.load(Ordering::Relaxed)
    }

    /// Returns the sealed batches for the given round.
    pub fn sealed_batches(&self, round: u64) -> Option<HashMap<Address<N>, SealedBatch<N>>> {
        self.sealed_batches.read().get(&round).cloned()
    }

    /// Returns the previous batch certificates for the given round.
    pub fn previous_certificates(&self, round: u64) -> Option<Vec<BatchCertificate<N>>> {
        // The genesis round does not require batch certificates.
        if round == 0 {
            return None;
        }
        // Retrieve the previous round's sealed batches.
        let sealed_batches = self.sealed_batches.read();
        let Some(batches) = sealed_batches.get(&(round - 1)) else {
            return None;
        };
        // Retrieve the certificates.
        let mut certificates = Vec::new();
        for batch in batches.values() {
            certificates.push(batch.certificate().clone());
        }
        // Return the certificates.
        Some(certificates)
    }

    /// Increments the round number.
    pub fn increment_round(&self) {
        self.round.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the block height.
    pub fn increment_height(&self) {
        self.height.fetch_add(1, Ordering::Relaxed);
    }
}

impl<N: Network> Shared<N> {
    /// Returns the committee.
    pub fn committee(&self) -> &RwLock<HashMap<Address<N>, u64>> {
        &self.committee
    }

    /// Returns the number of validators in the committee.
    pub fn committee_size(&self) -> usize {
        self.committee.read().len()
    }

    /// Returns `true` if the given address is in the committee.
    pub fn is_committee_member(&self, address: &Address<N>) -> bool {
        self.committee.read().contains_key(address)
    }

    /// Returns the total amount of stake in the committee.
    pub fn total_stake(&self) -> Result<u64> {
        // Compute the total power of the committee.
        let mut power = 0u64;
        for stake in self.committee.read().values() {
            // Accumulate the stake, checking for overflow.
            power = match power.checked_add(*stake) {
                Some(power) => power,
                None => bail!("Failed to calculate total stake - overflow detected"),
            };
        }
        Ok(power)
    }

    /// Returns the amount of stake required to reach a quorum threshold `(2f + 1)`.
    pub fn quorum_threshold(&self) -> Result<u64> {
        // Assuming `N = 3f + 1 + k`, where `0 <= k < 3`,
        // then `(2N + 3) / 3 = 2f + 1 + (2k + 2)/3 = 2f + 1 + k = N - f`.
        Ok(self.total_stake()?.saturating_mul(2) / 3 + 1)
    }

    /// Returns the amount of stake required to reach the availability threshold `(f + 1)`.
    pub fn availability_threshold(&self) -> Result<u64> {
        // Assuming `N = 3f + 1 + k`, where `0 <= k < 3`,
        // then `(N + 2) / 3 = f + 1 + k/3 = f + 1`.
        Ok(self.total_stake()?.saturating_add(2) / 3)
    }
}

impl<N: Network> Shared<N> {
    /// Returns the peer IP for the given address.
    pub fn get_peer_ip(&self, address: &Address<N>) -> Option<SocketAddr> {
        self.address_peers.read().get(address).copied()
    }

    /// Returns the address for the given peer IP.
    pub fn get_address(&self, peer_ip: &SocketAddr) -> Option<Address<N>> {
        self.peer_addresses.read().get(peer_ip).copied()
    }

    /// Inserts the given peer.
    pub(crate) fn insert_peer(&self, peer_ip: SocketAddr, address: Address<N>) {
        self.peer_addresses.write().insert(peer_ip, address);
        self.address_peers.write().insert(address, peer_ip);
    }

    /// Removes the given peer.
    pub(crate) fn remove_peer(&self, peer_ip: &SocketAddr) {
        if let Some(address) = self.peer_addresses.write().remove(peer_ip) {
            self.address_peers.write().remove(&address);
        }
    }
}