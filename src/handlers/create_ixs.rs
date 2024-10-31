use {
    adrena_abi::{ADX_MINT, CORTEX_ID, SPL_TOKEN_PROGRAM_ID, USDC_MINT},
    solana_sdk::{pubkey::Pubkey, system_program},
};

pub fn create_resolve_staking_round_ix(
    payer: &Pubkey,
    transfer_authority_pda: Pubkey,
    staking_pda: &Pubkey,
    staking_staked_token_vault_pda: &Pubkey,
    staking_reward_token_vault_pda: &Pubkey,
    staking_lm_reward_token_vault_pda: &Pubkey,
) -> (
    adrena_abi::instruction::ResolveStakingRound,
    adrena_abi::accounts::ResolveStakingRound,
) {
    let args = adrena_abi::instruction::ResolveStakingRound {};
    let resolve_staking_round = adrena_abi::accounts::ResolveStakingRound {
        caller: *payer,
        payer: *payer,
        staking_staked_token_vault: *staking_staked_token_vault_pda,
        staking_reward_token_vault: *staking_reward_token_vault_pda,
        staking_lm_reward_token_vault: *staking_lm_reward_token_vault_pda,
        transfer_authority: transfer_authority_pda,
        staking: *staking_pda,
        cortex: CORTEX_ID,
        lm_token_mint: ADX_MINT,
        fee_redistribution_mint: USDC_MINT,
        adrena_program: adrena_abi::ID,
        system_program: system_program::ID,
        token_program: SPL_TOKEN_PROGRAM_ID,
    };
    let accounts = resolve_staking_round;
    (args, accounts)
}

// pub fn create_claim_stakes_ix(
//     payer: &Pubkey,
//     user_staking_account_key: &Pubkey,
//     user_staking_account: &UserStaking,
//     receiving_account: Pubkey,
//     transfer_authority_pda: Pubkey,
//     lm_staking: Pubkey,
//     lp_staking: Pubkey,
//     cortex: &Cortex,
//     staking: &Staking,
//     user_profile: Option<Pubkey>,
//     position_take_profit_pda: Pubkey,
//     position_stop_loss_pda: Pubkey,
//     staking_reward_token_custody: &adrena_abi::types::Custody,
//     custody: &adrena_abi::types::Custody,
//     limit_price: u64,
// ) -> (
//     adrena_abi::instruction::ClosePositionLong,
//     adrena_abi::accounts::ClosePositionLong,
// ) {
//     let args = adrena_abi::instruction::ClosePositionLong {
//         params: adrena_abi::types::ClosePositionLongParams {
//             price: Some(limit_price),
//         },
//     };
//     let accounts = adrena_abi::accounts::ClaimStakes {
//         caller: *payer,
//         payer: todo!(),
//         owner: user_staking_account.,
//         reward_token_account: todo!(),
//         lm_token_account: todo!(),
//         staking_reward_token_vault: todo!(),
//         staking_lm_reward_token_vault: todo!(),
//         transfer_authority: transfer_authority_pda,
//         user_staking: todo!(),
//         staking: todo!(),
//         cortex: CORTEX_ID,
//         pool: position.pool,
//         genesis_lock: todo!(),
//         lm_token_mint: todo!(),
//         fee_redistribution_mint: todo!(),
//         adrena_program: adrena_abi::ID,
//         system_program: todo!(),
//         token_program: SPL_TOKEN_PROGRAM_ID,
//         lp_staking,
//         position: *position_key,
//         staking_reward_token_custody: USDC_CUSTODY_ID,
//         staking_reward_token_custody_oracle: staking_reward_token_custody.oracle,
//         staking_reward_token_custody_token_account: staking_reward_token_custody.token_account,
//         custody: position.custody,
//         custody_oracle: custody.oracle,
//         custody_trade_oracle: custody.trade_oracle,
//         custody_token_account: custody.token_account,
//         lm_staking_reward_token_vault: adrena_abi::pda::get_staking_reward_token_vault_pda(
//             &lm_staking,
//         )
//         .0,
//         lp_staking_reward_token_vault: adrena_abi::pda::get_staking_reward_token_vault_pda(
//             &lp_staking,
//         )
//         .0,
//         lp_token_mint: ALP_MINT,
//         protocol_fee_recipient: cortex.protocol_fee_recipient,
//         user_profile,
//         take_profit_thread: position_take_profit_pda,
//         stop_loss_thread: position_stop_loss_pda,
//         sablier_program: SABLIER_THREAD_PROGRAM_ID,
//         receiving_account,
//         lm_staking,
//     };
//     (args, accounts)
// }
