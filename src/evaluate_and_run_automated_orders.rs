// use {
//     crate::{
//         handlers::{self},
//         IndexedUserStakingAccountsThreadSafe,
//     },
//     adrena_abi::{types::Cortex, Side},
//     anchor_client::{Client, Cluster},
//     solana_sdk::{pubkey::Pubkey, signature::Keypair},
//     std::sync::Arc,
// };

// pub async fn evaluate_and_run_user_staking_account_autoclaims(
//     trade_oracle_key: &Pubkey,
//     trade_oracle_data: &[u8],
//     indexed_user_staking_accounts: &IndexedUserStakingAccountsThreadSafe,
//     payer: &Arc<Keypair>,
//     endpoint: &str,
//     cortex: &Cortex,
//     median_priority_fee: u64,
// ) -> Result<(), backoff::Error<anyhow::Error>> {
//     // make a clone of the indexed user staking accounts map to iterate over (while we modify the original map)
//     let user_staking_accounts_shallow_clone = indexed_user_staking_accounts.read().await.clone();
//     let mut tasks = vec![];

//     for (user_staking_account_key, user_staking_account) in user_staking_accounts_shallow_clone
//         .iter()
//         .filter(|(_, p)| p.custody == associated_custody_key && !p.is_pending_cleanup_and_close())
//     {
//         let position_key = *position_key;
//         let position = *position;
//         let indexed_custodies = Arc::clone(indexed_custodies);
//         let cortex = *cortex;
//         let median_priority_fee = median_priority_fee;

//         let client = Client::new(
//             Cluster::Custom(endpoint.to_string(), endpoint.to_string()),
//             Arc::clone(payer),
//         );
//         let task = tokio::spawn(async move {
//             let result: Result<(), anyhow::Error> = async {
//                 match position.get_side() {
//                     Side::Long => {
//                         // Check SL
//                         if position.stop_loss_is_set()
//                             && position.stop_loss_close_position_price != 0
//                         {
//                             if let Err(e) = handlers::sl_long::sl_long(
//                                 &position_key,
//                                 &position,
//                                 &oracle_price,
//                                 &indexed_custodies,
//                                 &client,
//                                 &cortex,
//                                 median_priority_fee,
//                             )
//                             .await
//                             {
//                                 log::error!("Error in sl_long: {}", e);
//                             }
//                         }

//                         // Check TP
//                         if position.take_profit_is_set() && position.take_profit_limit_price != 0 {
//                             if let Err(e) = handlers::tp_long::tp_long(
//                                 &position_key,
//                                 &position,
//                                 &oracle_price,
//                                 &indexed_custodies,
//                                 &client,
//                                 &cortex,
//                                 median_priority_fee,
//                             )
//                             .await
//                             {
//                                 log::error!("Error in tp_long: {}", e);
//                             }
//                         }

//                         // Check LIQ
//                         if let Err(e) = handlers::liquidate_long::liquidate_long(
//                             &position_key,
//                             &position,
//                             &oracle_price,
//                             &indexed_custodies,
//                             &client,
//                             &cortex,
//                             median_priority_fee,
//                         )
//                         .await
//                         {
//                             log::error!("Error in liquidate_long: {}", e);
//                         }
//                     }
//                     Side::Short => {
//                         // Check SL
//                         if position.stop_loss_is_set()
//                             && position.stop_loss_close_position_price != 0
//                         {
//                             if let Err(e) = handlers::sl_short::sl_short(
//                                 &position_key,
//                                 &position,
//                                 &oracle_price,
//                                 &indexed_custodies,
//                                 &client,
//                                 &cortex,
//                                 median_priority_fee,
//                             )
//                             .await
//                             {
//                                 log::error!("Error in sl_short: {}", e);
//                             }
//                         }

//                         // Check TP
//                         if position.take_profit_is_set() && position.take_profit_limit_price != 0 {
//                             if let Err(e) = handlers::tp_short::tp_short(
//                                 &position_key,
//                                 &position,
//                                 &oracle_price,
//                                 &indexed_custodies,
//                                 &client,
//                                 &cortex,
//                                 median_priority_fee,
//                             )
//                             .await
//                             {
//                                 log::error!("Error in tp_short: {}", e);
//                             }
//                         }

//                         // Check LIQ
//                         if let Err(e) = handlers::liquidate_short::liquidate_short(
//                             &position_key,
//                             &position,
//                             &oracle_price,
//                             &indexed_custodies,
//                             &client,
//                             &cortex,
//                             median_priority_fee,
//                         )
//                         .await
//                         {
//                             log::error!("Error in liquidate_short: {}", e);
//                         }
//                     }
//                     _ => {}
//                 }
//                 Ok::<(), anyhow::Error>(())
//             }
//             .await;

//             if let Err(e) = result {
//                 log::error!("Error processing position {}: {:?}", position_key, e);
//             }
//         });

//         tasks.push(task);
//     }

//     // Wait for all tasks to complete
//     for task in tasks {
//         if let Err(e) = task.await {
//             log::error!("Task failed: {:?}", e);
//         }
//     }
//     Ok(())
// }
