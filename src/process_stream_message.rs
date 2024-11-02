use {
    crate::{
        generate_accounts_filter_map,
        update_caches::{
            update_claim_cache_for_account,
            update_staking_round_next_resolve_time_cache_for_account,
        },
        update_indexes::{update_indexed_staking_accounts, update_indexed_user_staking_accounts},
        IndexedStakingAccountsThreadSafe, IndexedUserStakingAccountsThreadSafe,
        StakingRoundNextResolveTimeCacheThreadSafe, UserStakingClaimCacheThreadSafe,
    },
    adrena_abi::{Staking, UserStaking},
    futures::{channel::mpsc::SendError, Sink, SinkExt},
    solana_sdk::pubkey::Pubkey,
    yellowstone_grpc_proto::geyser::{
        subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestPing, SubscribeUpdate,
    },
};

pub enum StakingAccountUpdate {
    Created(Staking),
    Modified(Staking),
    Closed,
}

pub enum UserStakingAccountUpdate {
    Created(UserStaking),
    Modified(UserStaking),
    Closed,
    MissingStakingType(UserStaking),
}

pub async fn process_stream_message<S>(
    message: Result<SubscribeUpdate, backoff::Error<anyhow::Error>>,
    indexed_staking_accounts: &IndexedStakingAccountsThreadSafe,
    indexed_user_staking_accounts: &IndexedUserStakingAccountsThreadSafe,
    claim_cache: &UserStakingClaimCacheThreadSafe,
    staking_round_next_resolve_time_cache: &StakingRoundNextResolveTimeCacheThreadSafe,
    subscribe_tx: &mut S,
) -> Result<(), backoff::Error<anyhow::Error>>
where
    S: Sink<SubscribeRequest, Error = SendError> + Unpin,
{
    let mut subscriptions_update_required = false;

    match message {
        Ok(msg) => {
            match msg.update_oneof {
                Some(UpdateOneof::Account(sua)) => {
                    let account = sua.account.expect("Account should be defined");
                    let account_key = Pubkey::try_from(account.pubkey).expect("valid pubkey");
                    let account_data = account.data.to_vec();
                    // Each loop iteration we check if we need to update the subscription request based on what previously happened

                    if msg.filters.contains(&"staking_create_update".to_owned()) {
                        // Updates the indexed Staking accounts map
                        let update = update_indexed_staking_accounts(
                            &account_key,
                            &account_data,
                            indexed_staking_accounts,
                        )
                        .await?;

                        match update {
                            StakingAccountUpdate::Created(_) => {
                                panic!("Staking account created in staking_create_update filter");
                            }
                            StakingAccountUpdate::Modified(updated_staking_account) => {
                                log::info!("(scu) Staking account modified: {:#?}", account_key);
                                // Based on the updated Staking account, update the staking round next resolve time cache (if needed)
                                update_staking_round_next_resolve_time_cache_for_account(
                                    staking_round_next_resolve_time_cache,
                                    &account_key,
                                    &updated_staking_account,
                                )
                                .await;
                            }
                            StakingAccountUpdate::Closed => {
                                panic!("Staking account closed in staking_create_update filter");
                            }
                        }
                    }

                    if msg
                        .filters
                        .contains(&"user_staking_create_update".to_owned())
                    {
                        // Updates the indexed UserStaking accounts map
                        let update = update_indexed_user_staking_accounts(
                            &account_key,
                            &account_data,
                            indexed_user_staking_accounts,
                        )
                        .await?;

                        // Update the indexed UserStaking accounts map and the subscriptions request if a new UserStaking account was created
                        match update {
                            UserStakingAccountUpdate::Created(new_user_staking_account) => {
                                log::info!(
                                    "(pcu) New UserStaking account created: {:#?}",
                                    account_key
                                );

                                // Update the claim cache with the claim time of the oldest locked stake for the new UserStaking account
                                update_claim_cache_for_account(
                                    claim_cache,
                                    account_key,
                                    &new_user_staking_account,
                                )
                                .await;

                                // We need to update the subscriptions request to include the new UserStaking account (for deletion filtering)
                                subscriptions_update_required = true;
                            }
                            UserStakingAccountUpdate::Modified(user_staking_account) => {
                                log::info!(
                                    "(pcu) UserStaking account modified: {:#?}",
                                    account_key
                                );
                                // Update the claim cache with the claim time of the oldest locked stake for the modified UserStaking account
                                update_claim_cache_for_account(
                                    claim_cache,
                                    account_key,
                                    &user_staking_account,
                                )
                                .await;
                            }
                            UserStakingAccountUpdate::MissingStakingType(_) => {
                                log::info!(
                                    "(pcu) UserStaking account missing staking type has been updated (did nothing): {:#?}",
                                    account_key
                                );
                            }
                            UserStakingAccountUpdate::Closed => {
                                log::info!("(pcu) UserStaking account closed: {:#?}", account_key);
                                // We need to remove the closed UserStaking account from the claim cache
                                claim_cache.write().await.remove(&account_key);
                                // We need to update the subscriptions request to remove the closed UserStaking account
                                subscriptions_update_required = true;
                            }
                        }
                    }
                    /* Else if is important as we only want to end up here if the message is not about a user_staking_create_update*/
                    else if msg.filters.contains(&"user_staking_close".to_owned()) {
                        // Updates the indexed UserStaking accounts map
                        let update = update_indexed_user_staking_accounts(
                            &account_key,
                            &account_data,
                            indexed_user_staking_accounts,
                        )
                        .await?;

                        match update {
                            UserStakingAccountUpdate::Created(_) => {
                                panic!("New UserStaking account created in positions_close filter");
                            }
                            UserStakingAccountUpdate::Modified(_) => {
                                panic!("UserStaking account modified in positions_close filter");
                            }
                            UserStakingAccountUpdate::Closed
                            | UserStakingAccountUpdate::MissingStakingType(_) => {
                                log::info!("(pc) UserStaking account closed: {:#?}", account_key);
                                // We need to update the subscriptions request to remove the closed UserStaking account
                                subscriptions_update_required = true;
                            }
                        }
                    }
                }
                Some(UpdateOneof::Ping(_)) => {
                    // This is necessary to keep load balancers that expect client pings alive. If your load balancer doesn't
                    // require periodic client pings then this is unnecessary
                    subscribe_tx
                        .send(SubscribeRequest {
                            ping: Some(SubscribeRequestPing { id: 1 }),
                            ..Default::default()
                        })
                        .await
                        .map_err(|e| backoff::Error::transient(e.into()))?;
                }
                _ => {}
            }
        }
        Err(error) => {
            log::error!("error: {error:?}");
            return Err(error);
        }
    }

    // Update the subscriptions request if needed
    if subscriptions_update_required {
        log::info!("  <> Update subscriptions request");
        let accounts_filter_map = generate_accounts_filter_map(indexed_user_staking_accounts).await;
        let request = SubscribeRequest {
            accounts: accounts_filter_map,
            ..Default::default()
        };
        subscribe_tx
            .send(request)
            .await
            .map_err(|e| backoff::Error::transient(e.into()))?;
    }
    Ok(())
}
