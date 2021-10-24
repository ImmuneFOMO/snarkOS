// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use crate::{helpers::Tasks, ledger::Ledger, network::initialize::Initialize, Environment, NodeType, Peers};
use snarkos_ledger::storage::rocksdb::RocksDB;
use snarkvm::dpc::{Address, Network};

use anyhow::{anyhow, Result};
use rand::{thread_rng, Rng};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
};
use tokio::{runtime, sync::RwLock, task};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Status {
    Idle = 0,
    Mining,
    Syncing,
    ShuttingDown,
}

/// A node server implementation.
// #[derive(Clone)]
pub struct Node<N: Network, E: Environment> {
    /// The current status of the node.
    status: Arc<AtomicU8>,
    // /// The list of peers for the node.
    // peers: Arc<RwLock<Peers<N, E>>>,
    // /// The ledger state of the node.
    // ledger: Arc<RwLock<Ledger<N>>>,
    initialize: Initialize<N, E>,
    /// The list of tasks spawned by the node.
    tasks: Tasks<task::JoinHandle<()>>,
}

impl<N: Network, E: Environment> Node<N, E> {
    pub async fn new(port: u16, miner: Option<Address<N>>) -> Result<Self> {
        // Open the ledger from storage.
        // let ledger = Ledger::<N>::open::<RocksDB, _>(&format!(".ledger-{}", thread_rng().gen::<u8>()))?;

        // Initialize the node.
        let node = Self {
            status: Arc::new(AtomicU8::new(0)),
            // peers: Arc::new(RwLock::new(Peers::new())),
            // ledger: Arc::new(RwLock::new(ledger)),
            initialize: Initialize::initialize(port, miner).await?,
            tasks: Tasks::new(),
        };
        Ok(node)
    }

    ///
    /// Returns the current status of the node.
    ///
    #[inline]
    pub fn status(&self) -> Status {
        match self.status.load(Ordering::SeqCst) {
            0 => Status::Idle,
            1 => Status::Mining,
            2 => Status::Syncing,
            3 => Status::ShuttingDown,
            _ => unreachable!("Invalid status code"),
        }
    }

    // /// Initializes the node.
    // #[inline]
    // pub async fn start(&self, port: u16, miner_address: Address<N>) {
    //     // Connect to a peer and sync node.
    //     // Start the node in sync mode.
    //     // Once synced, start the miner.
    //
    //     // {
    //     //     // This will spawn a work-stealing runtime with 4 worker threads.
    //     //     let peers = runtime::Builder::new_multi_thread()
    //     //         .worker_threads(4)
    //     //         .enable_time()
    //     //         .build()
    //     //         .unwrap();
    //     //     // // This will spawn a work-stealing runtime with 4 worker threads.
    //     //     // let miner = runtime::Builder::new_multi_thread()
    //     //     //     .worker_threads(4)
    //     //     //     .enable_time()
    //     //     //     .build()
    //     //     //     .unwrap();
    //     //
    //     //     let node = self.clone();
    //     //     peers.block_on(async move {
    //     //         if let Ok(()) = node.start_listener(port).await {
    //     //             node.connect_to("127.0.0.1:4133".parse().unwrap()).await;
    //     //         }
    //     //     });
    //     //
    //     //     // if port == 4134 {
    //     //     //     let node = self.clone();
    //     //     //     miner.block_on(async move {
    //     //     //         if let Err(error) = node.start_miner(miner_address) {
    //     //     //             error!("Miner errored with {}", error);
    //     //     //         }
    //     //     //     });
    //     //     // }
    //     // }
    //
    //     // if let Ok(()) = self.start_listener(port).await {
    //     //     self.connect_to("127.0.0.1:4133".parse().unwrap()).await;
    //     // }
    //     // if port == 4134 {
    //     //     if let Err(error) = self.start_miner(miner_address) {
    //     //         error!("Miner errored with {}", error);
    //     //     }
    //     // }
    // }

    // /// Initializes the listener for peers.
    // #[inline]
    // pub async fn start_listener(&self, port: u16) -> Result<()> {
    //     let listener = Peers::listen(self.ledger(), self.peers(), port).await?;
    //     self.add_task(listener)
    // }

    // /// Initializes a miner.
    // #[inline]
    // pub fn start_miner(&self, miner_address: Address<N>) -> Result<()> {
    //     // If the node is a mining node, initialize a miner.
    //     match E::NODE_TYPE == NodeType::Miner {
    //         true => self.add_task(Miner::spawn(self.clone(), miner_address)),
    //         false => Err(anyhow!("Node is not a mining node")),
    //     }
    // }

    // /// Initializes the peers.
    // #[inline]
    // pub async fn connect_to(&self, peer_ip: SocketAddr) {
    //     trace!("Attempting connection to {}...", peer_ip);
    //     if let Err(error) = Peers::connect_to(self.peers(), peer_ip).await {
    //         error!("{}", error)
    //     }
    // }

    // /// Adds the given task handle to the node.
    // #[inline]
    // fn add_task(&self, handle: task::JoinHandle<()>) -> Result<()> {
    //     self.tasks.append(handle);
    //     Ok(())
    // }

    // ///
    // /// Returns the peers for the node.
    // ///
    // #[inline]
    // pub(crate) fn peers(&self) -> Arc<RwLock<Peers<N, E>>> {
    //     self.peers.clone()
    // }
    //
    // ///
    // /// Returns the ledger for the node.
    // ///
    // #[inline]
    // pub(crate) fn ledger(&self) -> Arc<RwLock<Ledger<N>>> {
    //     self.ledger.clone()
    // }

    // ///
    // /// Returns the current terminator bit for the node.
    // ///
    // #[inline]
    // pub(crate) fn terminator(&self) -> Arc<AtomicBool> {
    //     self.terminator.clone()
    // }

    /// Updates the node to the given status.
    #[inline]
    pub(crate) fn set_status(&self, state: Status) {
        self.status.store(state as u8, Ordering::SeqCst);
        match state {
            Status::ShuttingDown => {
                // debug!("Shutting down");
                // self.terminator.store(true, Ordering::SeqCst);
                self.tasks.flush();
            }
            _ => (),
        }
    }

    /// Disconnects from peers and proceeds to shut down the node.
    #[inline]
    pub async fn shut_down(&self) {
        self.set_status(Status::ShuttingDown);
        // for address in self.connected_peers() {
        //     self.disconnect_from_peer(address).await;
        // }
    }
}