// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use futures::StreamExt;
use jsonrpsee::core::client::{Client, Subscription, SubscriptionClientT};
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use serde_json::Value;
use std::net::SocketAddr;
use std::{collections::BTreeMap, sync::Arc};
use sui::wallet_commands::{WalletCommandResult, WalletCommands, WalletContext};
use sui_core::authority::AuthorityState;
use sui_json_rpc_api::rpc_types::{
    SuiEvent, SuiMoveStruct, SuiMoveValue, SuiObjectInfo, SuiObjectRead,
};
use sui_node::SuiNode;
use sui_swarm::memory::Swarm;
use sui_types::{
    base_types::{ObjectID, SuiAddress, TransactionDigest},
    batch::UpdateItem,
    messages::{BatchInfoRequest, BatchInfoResponseItem},
};
use test_utils::network::setup_network_and_wallet;
use tokio::time::timeout;
use tokio::time::{sleep, Duration};
use tracing::info;

async fn transfer_coin(
    context: &mut WalletContext,
) -> Result<(ObjectID, SuiAddress, SuiAddress, TransactionDigest), anyhow::Error> {
    let sender = context.config.accounts.get(0).cloned().unwrap();
    let receiver = context.config.accounts.get(1).cloned().unwrap();

    let object_refs = context.gateway.get_objects_owned_by_address(sender).await?;
    let gas_object = object_refs.get(0).unwrap().object_id;
    let object_to_send = object_refs.get(1).unwrap().object_id;

    // Send an object
    info!(
        "transferring coin {:?} from {:?} -> {:?}",
        object_to_send, sender, receiver
    );
    let res = WalletCommands::Transfer {
        to: receiver,
        coin_object_id: object_to_send,
        gas: Some(gas_object),
        gas_budget: 50000,
    }
    .execute(context)
    .await?;

    let digest = if let WalletCommandResult::Transfer(_, cert, _) = res {
        cert.transaction_digest
    } else {
        panic!("transfer command did not return WalletCommandResult::Transfer");
    };

    Ok((object_to_send, sender, receiver, digest))
}

async fn get_account_and_objects(
    context: &mut WalletContext,
) -> Result<(SuiAddress, Vec<SuiObjectInfo>), anyhow::Error> {
    let sender = context.config.accounts.get(0).cloned().unwrap();
    let object_refs = context.gateway.get_objects_owned_by_address(sender).await?;
    Ok((sender, object_refs))
}

async fn emit_move_events(
    context: &mut WalletContext,
) -> Result<(SuiAddress, ObjectID, TransactionDigest), anyhow::Error> {
    let (sender, object_refs) = get_account_and_objects(context).await.unwrap();
    let gas_object = object_refs.get(0).unwrap().object_id;

    let res = WalletCommands::CreateExampleNFT {
        name: Some("example_nft_name".into()),
        description: Some("example_nft_desc".into()),
        url: Some("https://sui.io/_nuxt/img/sui-logo.8d3c44e.svg".into()),
        gas: Some(gas_object),
        gas_budget: Some(50000),
    }
    .execute(context)
    .await?;

    let (object_id, digest) = if let WalletCommandResult::CreateExampleNFT(SuiObjectRead::Exists(
        obj,
    )) = res
    {
        (obj.reference.object_id, obj.previous_transaction)
    } else {
        panic!("CreateExampleNFT command did not return WalletCommandResult::CreateExampleNFT(SuiObjectRead::Exists, got {:?}", res);
    };

    Ok((sender, object_id, digest))
}

async fn wait_for_tx(wait_digest: TransactionDigest, state: Arc<AuthorityState>) {
    let mut timeout = Box::pin(sleep(Duration::from_millis(5000)));

    let mut max_seq = Some(0);

    let mut stream = Box::pin(
        state
            .handle_batch_streaming(BatchInfoRequest {
                start: max_seq,
                length: 1000,
            })
            .await
            .unwrap(),
    );

    loop {
        tokio::select! {
            _ = &mut timeout => panic!("wait_for_tx timed out"),

            items = &mut stream.next() => {
                match items {
                    // Upon receiving a batch
                    Some(Ok(BatchInfoResponseItem(UpdateItem::Batch(batch)) )) => {
                        max_seq = Some(batch.batch.next_sequence_number);
                        info!(?max_seq, "Received Batch");
                    }
                    // Upon receiving a transaction digest we store it, if it is not processed already.
                    Some(Ok(BatchInfoResponseItem(UpdateItem::Transaction((_seq, digest))))) => {
                        info!(?digest, "Received Transaction");
                        if wait_digest == digest.transaction {
                            info!(?digest, "Digest found");
                            break;
                        }
                    },

                    Some(Err( err )) => panic!("{}", err),
                    None => {
                        info!(?max_seq, "Restarting Batch");
                        stream = Box::pin(
                                state
                                    .handle_batch_streaming(BatchInfoRequest {
                                        start: max_seq,
                                        length: 1000,
                                    })
                                    .await
                                    .unwrap(),
                            );

                    }
                }
            },
        }
    }
}

#[tokio::test]
async fn test_full_node_follows_txes() -> Result<(), anyhow::Error> {
    telemetry_subscribers::init_for_testing();

    let (swarm, mut context, _) = setup_network_and_wallet().await?;

    let config = swarm.config().generate_fullnode_config();
    let node = SuiNode::start(&config).await?;

    let (transfered_object, _, receiver, digest) = transfer_coin(&mut context).await?;
    wait_for_tx(digest, node.state().clone()).await;

    // verify that the node has seen the transfer
    let object_read = node.state().get_object_read(&transfered_object).await?;
    let object = object_read.into_object()?;

    assert_eq!(object.owner.get_owner_address().unwrap(), receiver);

    // timestamp is recorded
    let ts = node.state().get_timestamp_ms(&digest).await?;
    assert!(ts.is_some());

    Ok(())
}

#[tokio::test]
async fn test_full_node_indexes() -> Result<(), anyhow::Error> {
    let (swarm, mut context, _) = setup_network_and_wallet().await?;

    let config = swarm.config().generate_fullnode_config();
    let node = SuiNode::start(&config).await?;

    let (transfered_object, sender, receiver, digest) = transfer_coin(&mut context).await?;

    wait_for_tx(digest, node.state().clone()).await;

    let txes = node
        .state()
        .get_transactions_by_input_object(transfered_object)
        .await?;

    assert_eq!(txes.len(), 1);
    assert_eq!(txes[0].1, digest);

    let txes = node
        .state()
        .get_transactions_by_mutated_object(transfered_object)
        .await?;
    assert_eq!(txes.len(), 1);
    assert_eq!(txes[0].1, digest);

    let txes = node.state().get_transactions_from_addr(sender).await?;
    assert_eq!(txes.len(), 1);
    assert_eq!(txes[0].1, digest);

    let txes = node.state().get_transactions_to_addr(receiver).await?;
    assert_eq!(txes.len(), 1);
    assert_eq!(txes[0].1, digest);

    // Note that this is also considered a tx to the sender, because it mutated
    // one or more of the sender's objects.
    let txes = node.state().get_transactions_to_addr(sender).await?;
    assert_eq!(txes.len(), 1);
    assert_eq!(txes[0].1, digest);

    // No transactions have originated from the receiver
    let txes = node.state().get_transactions_from_addr(receiver).await?;
    assert_eq!(txes.len(), 0);

    // timestamp is recorded
    let ts = node.state().get_timestamp_ms(&digest).await?;
    assert!(ts.is_some());

    Ok(())
}

// Test for syncing a node to an authority that already has many txes.
#[tokio::test]
async fn test_full_node_cold_sync() -> Result<(), anyhow::Error> {
    telemetry_subscribers::init_for_testing();

    let (swarm, mut context, _) = setup_network_and_wallet().await?;

    let (_, _, _, _) = transfer_coin(&mut context).await?;
    let (_, _, _, _) = transfer_coin(&mut context).await?;
    let (_, _, _, _) = transfer_coin(&mut context).await?;
    let (_transfered_object, sender, _receiver, digest) = transfer_coin(&mut context).await?;

    sleep(Duration::from_millis(1000)).await;

    let config = swarm.config().generate_fullnode_config();
    let node = SuiNode::start(&config).await?;

    wait_for_tx(digest, node.state().clone()).await;

    let txes = node.state().get_transactions_from_addr(sender).await?;
    assert_eq!(txes.last().unwrap().1, digest);

    Ok(())
}

/// Call this function to set up a network and a fullnode with subscription enabled.
/// Pass in an unique port for each test case otherwise they may interfere with one another.
async fn set_up_subscription(port: u16, swarm: &Swarm) -> Result<(SuiNode, Client), anyhow::Error> {
    let ws_server_url = format!("127.0.0.1:{}", port);
    let ws_addr: SocketAddr = ws_server_url.parse().unwrap();

    let mut config = swarm.config().generate_fullnode_config();
    config.websocket_address = Some(ws_addr);

    let node = SuiNode::start(&config).await?;

    let client = WsClientBuilder::default()
        .build(&format!("ws://{}", ws_server_url))
        .await?;
    Ok((node, client))
}

#[tokio::test]
async fn test_full_node_sub_to_move_event_ok() -> Result<(), anyhow::Error> {
    let (swarm, mut context, _) = setup_network_and_wallet().await?;
    // Pass in an unique port for each test case otherwise they may interfere with one another.
    let (node, ws_client) = set_up_subscription(6666, &swarm).await?;

    let params = BTreeMap::<String, Value>::new();
    let mut sub: Subscription<SuiEvent> = ws_client
        .subscribe(
            "sui_subscribeMoveEventsByType",
            rpc_params!["0x2::devnet_nft::MintNFTEvent", params],
            // TODO: update unsub function when it's added
            "foo",
        )
        .await
        .unwrap();

    let (sender, object_id, digest) = emit_move_events(&mut context).await?;
    wait_for_tx(digest, node.state().clone()).await;

    match timeout(Duration::from_secs(5), sub.next()).await {
        Ok(Some(Ok(SuiEvent::MoveEvent {
            type_,
            fields,
            bcs: _,
        }))) => {
            assert_eq!(type_, "0x2::devnet_nft::MintNFTEvent");
            assert_eq!(
                fields,
                SuiMoveStruct::WithFields(BTreeMap::from([
                    ("creator".into(), SuiMoveValue::Address(sender)),
                    (
                        "name".into(),
                        SuiMoveValue::String("example_nft_name".into())
                    ),
                    (
                        "object_id".into(),
                        SuiMoveValue::Address(SuiAddress::from(object_id))
                    ),
                ]))
            );
            // TODO: verify bcs contents
        }
        other => panic!("Failed to get SuiEvent, but {:?}", other),
    }

    match timeout(Duration::from_secs(5), sub.next()).await {
        Err(_) => (),
        other => panic!(
            "Expect to time out because no new events are coming in. Got {:?}",
            other
        ),
    }

    Ok(())
}
