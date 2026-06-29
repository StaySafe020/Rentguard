use anchor_lang::prelude::*;

declare_id!("G68qNgwjHEkAmp3k9XorrXP6vpCqUBW5nbPTdR9ugkVJ");

pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;

use instructions::*;
use state::*;
use errors::*;

#[program]
pub mod rentguard {
    use super::*;

    /// Initialize a new rental agreement
    pub fn create_rental_agreement(
        ctx: Context<CreateRentalAgreement>,
        rent_amount: u64,
        deposit_amount: u64,
        rent_due_day: u8,
        lease_start_date: i64,
        lease_end_date: i64,
        property_address: String,
    ) -> Result<()> {
        instructions::create_rental_agreement(
            ctx,
            rent_amount,
            deposit_amount,
            rent_due_day,
            lease_start_date,
            lease_end_date,
            property_address,
        )
    }

    /// Tenant deposits the security deposit
    pub fn deposit_security(ctx: Context<DepositSecurity>) -> Result<()> {
        instructions::deposit_security(ctx)
    }

    /// Pay monthly rent
    pub fn pay_rent(ctx: Context<PayRent>) -> Result<()> {
        instructions::pay_rent(ctx)
    }

    /// Landlord withdraws rent payment
    pub fn withdraw_rent(ctx: Context<WithdrawRent>) -> Result<()> {
        instructions::withdraw_rent(ctx)
    }

    /// Initiate lease termination
    pub fn terminate_lease(ctx: Context<TerminateLease>) -> Result<()> {
        instructions::terminate_lease(ctx)
    }

    /// Approve deposit return (both parties agree)
    pub fn approve_deposit_return(
        ctx: Context<ApproveDepositReturn>,
        tenant_amount: u64,
        landlord_amount: u64,
    ) -> Result<()> {
        instructions::approve_deposit_return(ctx, tenant_amount, landlord_amount)
    }

    /// File a dispute
    pub fn file_dispute(
        ctx: Context<FileDispute>,
        reason: String,
        requested_amount: u64,
    ) -> Result<()> {
        instructions::file_dispute(ctx, reason, requested_amount)
    }

    /// Resolve dispute (requires arbitrator)
    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        tenant_amount: u64,
        landlord_amount: u64,
    ) -> Result<()> {
        instructions::resolve_dispute(ctx, tenant_amount, landlord_amount)
    }

    /// Update reputation after lease completion
    pub fn update_reputation(
        ctx: Context<UpdateReputation>,
        rating: u8,
        on_time_payments: u8,
    ) -> Result<()> {
        instructions::update_reputation(ctx, rating, on_time_payments)
    }
}
