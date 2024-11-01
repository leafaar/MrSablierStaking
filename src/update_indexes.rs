use {
    crate::{
        process_stream_message::{StakingAccountUpdate, UserStakingAccountUpdate},
        IndexedStakingAccountsThreadSafe, IndexedUserStakingAccountsThreadSafe,
    },
    adrena_abi::{AccountDeserialize, Staking, UserStaking},
    solana_sdk::pubkey::Pubkey,
};

// Updates the indexed Staking accounts map based on the received account data.
// - Creates a new entry if the Staking account is not indexed.
// - Deletes the entry if the account data is empty (Staking account closed).
// - Updates the entry if the Staking account is indexed and data is non-empty (Staking account modified).
//
// Returns an enum with the update type and the Staking account if it was created or modified
pub async fn update_indexed_staking_accounts(
    staking_account_key: &Pubkey,
    staking_account_data: &[u8],
    indexed_staking_accounts: &IndexedStakingAccountsThreadSafe,
) -> Result<StakingAccountUpdate, backoff::Error<anyhow::Error>> {
    let mut staking_accounts = indexed_staking_accounts.write().await;

    if staking_account_data.is_empty() {
        staking_accounts.remove(staking_account_key);
        return Ok(StakingAccountUpdate::Closed);
    }

    let staking_account = Staking::try_deserialize(&mut &staking_account_data[..])
        .map_err(|e| backoff::Error::transient(e.into()))?;

    let is_new_staking_account = staking_accounts
        .insert(*staking_account_key, staking_account)
        .is_none();

    if is_new_staking_account {
        Ok(StakingAccountUpdate::Created(staking_account))
    } else {
        Ok(StakingAccountUpdate::Modified(staking_account))
    }
}

// Updates the indexed UserStaking accounts map based on the received account data.
// - Creates a new entry if the UserStaking account is not indexed.
// - Deletes the entry if the account data is empty (UserStaking account closed).
// - Updates the entry if the UserStaking account is indexed and data is non-empty (UserStaking account modified).
//
// Returns an enum with the update type and the UserStaking account if it was created or modified
pub async fn update_indexed_user_staking_accounts(
    user_staking_account_key: &Pubkey,
    user_staking_account_data: &[u8],
    indexed_user_staking_accounts: &IndexedUserStakingAccountsThreadSafe,
) -> Result<UserStakingAccountUpdate, backoff::Error<anyhow::Error>> {
    let mut user_staking_accounts = indexed_user_staking_accounts.write().await;

    if user_staking_account_data.is_empty() {
        user_staking_accounts.remove(user_staking_account_key);
        return Ok(UserStakingAccountUpdate::Closed);
    }

    let user_staking_account = UserStaking::try_deserialize(&mut &user_staking_account_data[..])
        .map_err(|e| backoff::Error::transient(e.into()))?;

    if user_staking_account.staking_type == 0 {
        return Ok(UserStakingAccountUpdate::MissingStakingType(
            user_staking_account,
        ));
    }

    let is_new_user_staking_account = user_staking_accounts
        .insert(*user_staking_account_key, user_staking_account)
        .is_none();

    if is_new_user_staking_account {
        Ok(UserStakingAccountUpdate::Created(user_staking_account))
    } else {
        Ok(UserStakingAccountUpdate::Modified(user_staking_account))
    }
}
