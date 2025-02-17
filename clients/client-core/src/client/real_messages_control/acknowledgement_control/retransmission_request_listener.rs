// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::{
    action_controller::{Action, ActionSender},
    PendingAcknowledgement, RetransmissionRequestReceiver,
};
use crate::client::{
    real_messages_control::real_traffic_stream::{BatchRealMessageSender, RealMessage},
    topology_control::TopologyAccessor,
};

use client_connections::TransmissionLane;
use futures::StreamExt;
use log::*;
use nymsphinx::{
    acknowledgements::AckKey, addressing::clients::Recipient, preparer::MessagePreparer,
};
use rand::{CryptoRng, Rng};
use std::sync::{Arc, Weak};

// responsible for packet retransmission upon fired timer
pub(super) struct RetransmissionRequestListener<R>
where
    R: CryptoRng + Rng,
{
    ack_key: Arc<AckKey>,
    ack_recipient: Recipient,
    message_preparer: MessagePreparer<R>,
    action_sender: ActionSender,
    real_message_sender: BatchRealMessageSender,
    request_receiver: RetransmissionRequestReceiver,
    topology_access: TopologyAccessor,
}

impl<R> RetransmissionRequestListener<R>
where
    R: CryptoRng + Rng,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        ack_key: Arc<AckKey>,
        ack_recipient: Recipient,
        message_preparer: MessagePreparer<R>,
        action_sender: ActionSender,
        real_message_sender: BatchRealMessageSender,
        request_receiver: RetransmissionRequestReceiver,
        topology_access: TopologyAccessor,
    ) -> Self {
        RetransmissionRequestListener {
            ack_key,
            ack_recipient,
            message_preparer,
            action_sender,
            real_message_sender,
            request_receiver,
            topology_access,
        }
    }

    async fn on_retransmission_request(&mut self, timed_out_ack: Weak<PendingAcknowledgement>) {
        let timed_out_ack = match timed_out_ack.upgrade() {
            Some(timed_out_ack) => timed_out_ack,
            None => {
                debug!("We received an ack JUST as we were about to retransmit [1]");
                return;
            }
        };
        let packet_recipient = &timed_out_ack.recipient;
        let chunk_clone = timed_out_ack.message_chunk.clone();
        let frag_id = chunk_clone.fragment_identifier();

        let topology_permit = self.topology_access.get_read_permit().await;
        let topology_ref = match topology_permit
            .try_get_valid_topology_ref(&self.ack_recipient, Some(packet_recipient))
        {
            Some(topology_ref) => topology_ref,
            None => {
                warn!("Could not retransmit the packet - the network topology is invalid");
                // we NEED to start timer here otherwise we will have this guy permanently stuck in memory
                self.action_sender
                    .unbounded_send(Action::new_start_timer(frag_id))
                    .unwrap();
                return;
            }
        };

        let prepared_fragment = self
            .message_preparer
            .prepare_chunk_for_sending(chunk_clone, topology_ref, &self.ack_key, packet_recipient)
            .unwrap();

        // if we have the ONLY strong reference to the ack data, it means it was removed from the
        // pending acks
        if Arc::strong_count(&timed_out_ack) == 1 {
            // while we were messing with topology, wrapping data in sphinx, etc. we actually received
            // this ack after all! no need to retransmit then
            debug!("We received an ack JUST as we were about to retransmit [2]");
            return;
        }
        // we no longer need the reference - let's drop it so that if somehow `UpdateTimer` action
        // reached the controller before this function terminated, the controller would not panic.
        drop(timed_out_ack);

        let new_delay = prepared_fragment.total_delay;

        // We know this update will be reflected by the `StartTimer` Action performed when this
        // message is sent through the mix network.
        // Reason being: UpdateTimer is now pushed onto the Action queue and `StartTimer` will
        // only be pushed when the below `RealMessage` (which we are about to create)
        // is sent to the `OutQueueControl` and has gone through its internal queue
        // with the additional poisson delay.
        // And since Actions are executed in order `UpdateTimer` will HAVE TO be executed before `StartTimer`
        self.action_sender
            .unbounded_send(Action::new_update_delay(frag_id, new_delay))
            .unwrap();

        // send to `OutQueueControl` to eventually send to the mix network
        self.real_message_sender
            .send((
                vec![RealMessage::new(prepared_fragment.mix_packet, frag_id)],
                TransmissionLane::Retransmission,
            ))
            .await
            .expect("BatchRealMessageReceiver has stopped receiving!");
    }

    pub(super) async fn run_with_shutdown(&mut self, mut shutdown: task::ShutdownListener) {
        debug!("Started RetransmissionRequestListener with graceful shutdown support");

        while !shutdown.is_shutdown() {
            tokio::select! {
                timed_out_ack = self.request_receiver.next() => match timed_out_ack {
                    Some(timed_out_ack) => self.on_retransmission_request(timed_out_ack).await,
                    None => {
                        log::trace!("RetransmissionRequestListener: Stopping since channel closed");
                        break;
                    }
                },
                _ = shutdown.recv() => {
                    log::trace!("RetransmissionRequestListener: Received shutdown");
                }
            }
        }
        shutdown.recv_timeout().await;
        log::debug!("RetransmissionRequestListener: Exiting");
    }

    // todo: think whether this is still required
    #[allow(dead_code)]
    pub(super) async fn run(&mut self) {
        debug!("Started RetransmissionRequestListener without graceful shutdown support");

        while let Some(timed_out_ack) = self.request_receiver.next().await {
            self.on_retransmission_request(timed_out_ack).await;
        }
    }
}
