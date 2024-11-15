use {
    crate::handlers::create_finalize_locked_stake_ix,
    adrena_abi::{
        get_governing_token_holding_pda, get_staking_pda, get_token_owner_record_pda,
        get_transfer_authority_pda, ADRENA_GOVERNANCE_REALM_CONFIG_ID, ADRENA_GOVERNANCE_REALM_ID,
        ADRENA_GOVERNANCE_SHADOW_TOKEN_MINT, ADX_MINT, SPL_TOKEN_PROGRAM_ID, USDC_MINT,
    },
    anchor_client::Program,
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{compute_budget::ComputeBudgetInstruction, pubkey::Pubkey, signature::Keypair},
    spl_associated_token_account::instruction::create_associated_token_account_idempotent,
    std::sync::Arc,
};

pub async fn finalize_locked_stake(
    user_staking_account_key: &Pubkey,
    owner_pubkey: &Pubkey,
    program: &Program<Arc<Keypair>>,
    median_priority_fee: u64,
    staked_token_mint: &Pubkey,
    locked_stake_id: u64,
) -> Result<(), backoff::Error<anyhow::Error>> {
    log::info!(
        "  <*> Finalizing locked stake for UserStaking account {:#?} (owner: {:#?} staked token: {:#?})",
        user_staking_account_key,
        owner_pubkey,
        staked_token_mint
    );
    let transfer_authority_pda = get_transfer_authority_pda().0;
    let staking_pda = get_staking_pda(staked_token_mint).0;

    let governance_governing_token_holding_pda =
        get_governing_token_holding_pda(&staking_pda, &ADRENA_GOVERNANCE_REALM_CONFIG_ID);

    let governance_governing_token_owner_record_pda = get_token_owner_record_pda(
        &governance_governing_token_holding_pda,
        &ADRENA_GOVERNANCE_SHADOW_TOKEN_MINT,
        &ADRENA_GOVERNANCE_REALM_ID,
    );

    let rpc_client = program.rpc();

    let (finalize_locked_stake_params, finalize_locked_stake_accounts) =
        create_finalize_locked_stake_ix(
            &program.payer(),
            owner_pubkey,
            locked_stake_id,
            &transfer_authority_pda,
            &staking_pda,
            user_staking_account_key,
            &governance_governing_token_holding_pda,
            &governance_governing_token_owner_record_pda,
        );

    let tx_simulation = program
        .request()
        .instruction(ComputeBudgetInstruction::set_compute_unit_price(
            median_priority_fee,
        ))
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
        .instruction(create_associated_token_account_idempotent(
            &program.payer(),
            owner_pubkey,
            &ADX_MINT,
            &SPL_TOKEN_PROGRAM_ID,
        ))
        .instruction(create_associated_token_account_idempotent(
            &program.payer(),
            owner_pubkey,
            &USDC_MINT,
            &SPL_TOKEN_PROGRAM_ID,
        ))
        .args(finalize_locked_stake_params)
        .accounts(finalize_locked_stake_accounts)
        .signed_transaction()
        .await
        .map_err(|e| {
            log::error!("   <> Transaction generation failed with error: {:?}", e);
            backoff::Error::transient(e.into())
        })?;

    let simulation = rpc_client
        .simulate_transaction(&tx_simulation)
        .await
        .map_err(|e| {
            log::error!("   <> Simulation failed with error: {:?}", e);
            backoff::Error::transient(e.into())
        })?;
    // log::info!("Simulation result: {:?}", simulation);

    let simulated_cu = simulation.value.units_consumed.unwrap_or(0);

    if simulated_cu == 0 {
        log::warn!(
            "   <> CU consumed: {} - Seems that the simulation cannot be performed due to low sol balance OR that the state is not updated yet (postpone)",
            simulated_cu
        );
        return Err(backoff::Error::transient(anyhow::anyhow!(
            "0 cu simulation"
        )));
    }
    // log::info!("CU consumed: {}", simulated_cu);

    let (finalize_locked_stake_params, finalize_locked_stake_accounts) =
        create_finalize_locked_stake_ix(
            &program.payer(),
            owner_pubkey,
            locked_stake_id,
            &transfer_authority_pda,
            &staking_pda,
            user_staking_account_key,
            &governance_governing_token_holding_pda,
            &governance_governing_token_owner_record_pda,
        );

    let tx = program
        .request()
        .instruction(ComputeBudgetInstruction::set_compute_unit_price(
            median_priority_fee,
        ))
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(
            (simulated_cu as f64 * 1.02) as u32, // +2% for any jitter due to find_pda calls
        ))
        .instruction(create_associated_token_account_idempotent(
            &program.payer(),
            owner_pubkey,
            &ADX_MINT,
            &SPL_TOKEN_PROGRAM_ID,
        ))
        .instruction(create_associated_token_account_idempotent(
            &program.payer(),
            owner_pubkey,
            &USDC_MINT,
            &SPL_TOKEN_PROGRAM_ID,
        ))
        .args(finalize_locked_stake_params)
        .accounts(finalize_locked_stake_accounts)
        .signed_transaction()
        .await
        .map_err(|e| {
            log::error!("   <> Transaction generation failed with error: {:?}", e);
            backoff::Error::transient(e.into())
        })?;

    let tx_hash = rpc_client
        .send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(0),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| {
            log::error!("   <> Transaction sending failed with error: {:?}", e);
            backoff::Error::transient(e.into())
        })?;

    log::info!(
        "   <> Finalize locked stake for staking account {:#?} - TX sent: {:#?}",
        user_staking_account_key,
        tx_hash.to_string(),
    );

    // TODO wait for confirmation and retry if needed

    Ok(())
}
