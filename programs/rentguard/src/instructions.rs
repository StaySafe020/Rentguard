use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::state::*;
use crate::errors::*;
use crate::events::*;

const SECONDS_PER_DAY: i64 = 86400;

// ============================================
// CREATE RENTAL AGREEMENT
// ============================================

#[derive(Accounts)]
#[instruction(rent_amount: u64, deposit_amount: u64)]
pub struct CreateRentalAgreement<'info> {
    #[account(
        init,
        payer = landlord,
        space = RentalAgreement::LEN,
        seeds = [
            b"rental_agreement",
            landlord.key().as_ref(),
            tenant.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(mut)]
    pub landlord: Signer<'info>,
    
    /// CHECK: Tenant doesn't need to sign at creation
    pub tenant: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_rental_agreement(
    ctx: Context<CreateRentalAgreement>,
    rent_amount: u64,
    deposit_amount: u64,
    rent_due_day: u8,
    lease_start_date: i64,
    lease_end_date: i64,
    property_address: String,
) -> Result<()> {
    // Validations
    require!(rent_due_day >= 1 && rent_due_day <= 28, RentGuardError::InvalidRentDueDay);
    require!(lease_end_date > lease_start_date, RentGuardError::InvalidLeaseDates);
    require!(
        property_address.len() <= RentalAgreement::MAX_ADDRESS_LEN,
        RentGuardError::PropertyAddressTooLong
    );

    let agreement = &mut ctx.accounts.rental_agreement;
    let clock = Clock::get()?;

    agreement.landlord = ctx.accounts.landlord.key();
    agreement.tenant = ctx.accounts.tenant.key();
    agreement.property_address = property_address;
    agreement.rent_amount = rent_amount;
    agreement.deposit_amount = deposit_amount;
    agreement.rent_due_day = rent_due_day;
    agreement.lease_start_date = lease_start_date;
    agreement.lease_end_date = lease_end_date;
    agreement.created_at = clock.unix_timestamp;
    agreement.status = AgreementStatus::Created;
    agreement.deposit_paid = false;
    agreement.last_rent_payment = 0;
    agreement.total_rent_paid = 0;
    agreement.missed_payments = 0;
    agreement.on_time_payments = 0;
    agreement.deposit_in_escrow = 0;
    agreement.landlord_deposit_approval = false;
    agreement.tenant_deposit_approval = false;
    agreement.approved_tenant_return = 0;
    agreement.approved_landlord_return = 0;
    agreement.dispute_status = DisputeStatus::None;
    agreement.dispute_reason = String::new();
    agreement.dispute_requested_amount = 0;
    agreement.dispute_filed_by = None;
    agreement.arbitrator = None;
    agreement.bump = ctx.bumps.rental_agreement;

    emit!(AgreementCreated {
        agreement: ctx.accounts.rental_agreement.key(),
        landlord: agreement.landlord,
        tenant: agreement.tenant,
        rent_amount,
        deposit_amount,
        lease_start_date,
        lease_end_date,
    });

    msg!("Rental agreement created successfully!");
    Ok(())
}

// ============================================
// DEPOSIT SECURITY
// ============================================

#[derive(Accounts)]
pub struct DepositSecurity<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.status == AgreementStatus::Created @ RentGuardError::InvalidAgreementStatus,
        constraint = !rental_agreement.deposit_paid @ RentGuardError::DepositAlreadyPaid,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(
        mut,
        constraint = tenant.key() == rental_agreement.tenant @ RentGuardError::UnauthorizedTenant
    )]
    pub tenant: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn deposit_security(ctx: Context<DepositSecurity>) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let clock = Clock::get()?;

    // Transfer deposit from tenant to agreement PDA
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.tenant.to_account_info(),
            to: ctx.accounts.rental_agreement.to_account_info(),
        },
    );
    transfer(cpi_context, agreement.deposit_amount)?;

    // Update agreement state
    agreement.deposit_paid = true;
    agreement.deposit_in_escrow = agreement.deposit_amount;
    agreement.status = AgreementStatus::Active;

    emit!(DepositPaid {
        agreement: ctx.accounts.rental_agreement.key(),
        tenant: ctx.accounts.tenant.key(),
        amount: agreement.deposit_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Security deposit paid: {} lamports", agreement.deposit_amount);
    Ok(())
}

// ============================================
// PAY RENT
// ============================================

#[derive(Accounts)]
pub struct PayRent<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.status == AgreementStatus::Active @ RentGuardError::InvalidAgreementStatus,
        constraint = rental_agreement.deposit_paid @ RentGuardError::DepositNotPaid,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(
        mut,
        constraint = tenant.key() == rental_agreement.tenant @ RentGuardError::UnauthorizedTenant
    )]
    pub tenant: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn pay_rent(ctx: Context<PayRent>) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Check if lease has started
    require!(current_time >= agreement.lease_start_date, RentGuardError::LeaseNotStarted);
    
    // Check if lease has ended
    require!(current_time <= agreement.lease_end_date, RentGuardError::LeaseEnded);

    // Calculate if rent is due
    let days_since_last_payment = if agreement.last_rent_payment == 0 {
        (current_time - agreement.lease_start_date) / SECONDS_PER_DAY
    } else {
        (current_time - agreement.last_rent_payment) / SECONDS_PER_DAY
    };

    require!(days_since_last_payment >= 25, RentGuardError::RentNotDue);

    // Check if payment is on time (within 5 days grace period)
    let is_on_time = days_since_last_payment <= 35;

    // Transfer rent from tenant to agreement PDA
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.tenant.to_account_info(),
            to: ctx.accounts.rental_agreement.to_account_info(),
        },
    );
    transfer(cpi_context, agreement.rent_amount)?;

    // Update agreement state
    agreement.last_rent_payment = current_time;
    agreement.total_rent_paid = agreement.total_rent_paid
        .checked_add(agreement.rent_amount)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    if is_on_time {
        agreement.on_time_payments = agreement.on_time_payments
            .checked_add(1)
            .ok_or(RentGuardError::ArithmeticOverflow)?;
    } else {
        agreement.missed_payments = agreement.missed_payments
            .checked_add(1)
            .ok_or(RentGuardError::ArithmeticOverflow)?;
    }

    emit!(RentPaid {
        agreement: ctx.accounts.rental_agreement.key(),
        tenant: ctx.accounts.tenant.key(),
        amount: agreement.rent_amount,
        payment_date: current_time,
        is_on_time,
    });

    msg!("Rent paid: {} lamports. On time: {}", agreement.rent_amount, is_on_time);
    Ok(())
}

// ============================================
// WITHDRAW RENT
// ============================================

#[derive(Accounts)]
pub struct WithdrawRent<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(
        mut,
        constraint = landlord.key() == rental_agreement.landlord @ RentGuardError::UnauthorizedLandlord
    )]
    pub landlord: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn withdraw_rent(ctx: Context<WithdrawRent>) -> Result<()> {
    let agreement = &ctx.accounts.rental_agreement;
    let clock = Clock::get()?;

    // Calculate available rent (total balance minus deposit)
    let total_balance = ctx.accounts.rental_agreement.to_account_info().lamports();
    let rent_balance = total_balance
        .checked_sub(agreement.deposit_in_escrow)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    // Account for rent required to keep the account alive
    let rent_exempt = Rent::get()?.minimum_balance(RentalAgreement::LEN);
    let available_rent = rent_balance
        .checked_sub(rent_exempt)
        .ok_or(RentGuardError::NoRentToWithdraw)?;

    require!(available_rent > 0, RentGuardError::NoRentToWithdraw);

    // Transfer rent to landlord
    **ctx.accounts.rental_agreement.to_account_info().try_borrow_mut_lamports()? -= available_rent;
    **ctx.accounts.landlord.to_account_info().try_borrow_mut_lamports()? += available_rent;

    emit!(RentWithdrawn {
        agreement: ctx.accounts.rental_agreement.key(),
        landlord: ctx.accounts.landlord.key(),
        amount: available_rent,
        timestamp: clock.unix_timestamp,
    });

    msg!("Rent withdrawn: {} lamports", available_rent);
    Ok(())
}

// ============================================
// TERMINATE LEASE
// ============================================

#[derive(Accounts)]
pub struct TerminateLease<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.status == AgreementStatus::Active @ RentGuardError::InvalidAgreementStatus,
        constraint = rental_agreement.dispute_status == DisputeStatus::None @ RentGuardError::CannotTerminateDuringDispute,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(mut)]
    pub initiator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn terminate_lease(ctx: Context<TerminateLease>) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let initiator_key = ctx.accounts.initiator.key();
    let clock = Clock::get()?;

    // Verify initiator is either landlord or tenant
    require!(
        initiator_key == agreement.landlord || initiator_key == agreement.tenant,
        RentGuardError::UnauthorizedLandlord
    );

    agreement.status = AgreementStatus::TerminationPending;

    emit!(LeaseTerminated {
        agreement: ctx.accounts.rental_agreement.key(),
        initiated_by: initiator_key,
        timestamp: clock.unix_timestamp,
    });

    msg!("Lease termination initiated by: {}", initiator_key);
    Ok(())
}

// ============================================
// APPROVE DEPOSIT RETURN
// ============================================

#[derive(Accounts)]
pub struct ApproveDepositReturn<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.status == AgreementStatus::TerminationPending @ RentGuardError::InvalidAgreementStatus,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(mut)]
    pub approver: Signer<'info>,
    
    /// CHECK: Landlord account for receiving funds
    #[account(mut)]
    pub landlord: UncheckedAccount<'info>,
    
    /// CHECK: Tenant account for receiving funds
    #[account(mut)]
    pub tenant: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn approve_deposit_return(
    ctx: Context<ApproveDepositReturn>,
    tenant_amount: u64,
    landlord_amount: u64,
) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let approver_key = ctx.accounts.approver.key();
    let clock = Clock::get()?;

    // Verify approver is either landlord or tenant
    require!(
        approver_key == agreement.landlord || approver_key == agreement.tenant,
        RentGuardError::UnauthorizedLandlord
    );

    // Verify amounts equal total deposit
    let total_distribution = tenant_amount
        .checked_add(landlord_amount)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    require!(
        total_distribution == agreement.deposit_in_escrow,
        RentGuardError::InvalidDepositDistribution
    );

    // Record approval
    if approver_key == agreement.landlord {
        agreement.landlord_deposit_approval = true;
    } else {
        agreement.tenant_deposit_approval = true;
    }

    agreement.approved_tenant_return = tenant_amount;
    agreement.approved_landlord_return = landlord_amount;

    // If both parties approved, distribute deposit
    if agreement.landlord_deposit_approval && agreement.tenant_deposit_approval {
        // Transfer to tenant
        if tenant_amount > 0 {
            **ctx.accounts.rental_agreement.to_account_info().try_borrow_mut_lamports()? -= tenant_amount;
            **ctx.accounts.tenant.to_account_info().try_borrow_mut_lamports()? += tenant_amount;
        }

        // Transfer to landlord
        if landlord_amount > 0 {
            **ctx.accounts.rental_agreement.to_account_info().try_borrow_mut_lamports()? -= landlord_amount;
            **ctx.accounts.landlord.to_account_info().try_borrow_mut_lamports()? += landlord_amount;
        }

        agreement.deposit_in_escrow = 0;
        agreement.status = AgreementStatus::Completed;

        emit!(DepositReturned {
            agreement: ctx.accounts.rental_agreement.key(),
            tenant_amount,
            landlord_amount,
            timestamp: clock.unix_timestamp,
        });

        msg!("Deposit distributed - Tenant: {} lamports, Landlord: {} lamports", 
            tenant_amount, landlord_amount);
    } else {
        msg!("Approval recorded. Waiting for other party's approval.");
    }

    Ok(())
}

// ============================================
// FILE DISPUTE
// ============================================

#[derive(Accounts)]
pub struct FileDispute<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.dispute_status == DisputeStatus::None @ RentGuardError::DisputeAlreadyFiled,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(mut)]
    pub filer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn file_dispute(
    ctx: Context<FileDispute>,
    reason: String,
    requested_amount: u64,
) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let filer_key = ctx.accounts.filer.key();
    let clock = Clock::get()?;

    // Verify filer is either landlord or tenant
    require!(
        filer_key == agreement.landlord || filer_key == agreement.tenant,
        RentGuardError::UnauthorizedLandlord
    );

    require!(
        reason.len() <= RentalAgreement::MAX_DISPUTE_REASON_LEN,
        RentGuardError::DisputeReasonTooLong
    );

    agreement.dispute_status = DisputeStatus::Filed;
    agreement.dispute_reason = reason.clone();
    agreement.dispute_requested_amount = requested_amount;
    agreement.dispute_filed_by = Some(filer_key);
    agreement.status = AgreementStatus::DisputePending;

    emit!(DisputeFiled {
        agreement: ctx.accounts.rental_agreement.key(),
        filed_by: filer_key,
        reason,
        requested_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Dispute filed by: {}", filer_key);
    Ok(())
}

// ============================================
// RESOLVE DISPUTE
// ============================================

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(
        mut,
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.dispute_status == DisputeStatus::Filed @ RentGuardError::NoActiveDispute,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(mut)]
    pub arbitrator: Signer<'info>,
    
    /// CHECK: Landlord account for receiving funds
    #[account(mut)]
    pub landlord: UncheckedAccount<'info>,
    
    /// CHECK: Tenant account for receiving funds
    #[account(mut)]
    pub tenant: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    tenant_amount: u64,
    landlord_amount: u64,
) -> Result<()> {
    let agreement = &mut ctx.accounts.rental_agreement;
    let clock = Clock::get()?;

    // Verify amounts equal total deposit
    let total_distribution = tenant_amount
        .checked_add(landlord_amount)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    require!(
        total_distribution == agreement.deposit_in_escrow,
        RentGuardError::InvalidDepositDistribution
    );

    // Transfer to tenant
    if tenant_amount > 0 {
        **ctx.accounts.rental_agreement.to_account_info().try_borrow_mut_lamports()? -= tenant_amount;
        **ctx.accounts.tenant.to_account_info().try_borrow_mut_lamports()? += tenant_amount;
    }

    // Transfer to landlord
    if landlord_amount > 0 {
        **ctx.accounts.rental_agreement.to_account_info().try_borrow_mut_lamports()? -= landlord_amount;
        **ctx.accounts.landlord.to_account_info().try_borrow_mut_lamports()? += landlord_amount;
    }

    agreement.deposit_in_escrow = 0;
    agreement.dispute_status = DisputeStatus::Resolved;
    agreement.status = AgreementStatus::Completed;
    agreement.arbitrator = Some(ctx.accounts.arbitrator.key());

    emit!(DisputeResolved {
        agreement: ctx.accounts.rental_agreement.key(),
        resolved_by: ctx.accounts.arbitrator.key(),
        tenant_amount,
        landlord_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Dispute resolved - Tenant: {} lamports, Landlord: {} lamports", 
        tenant_amount, landlord_amount);
    Ok(())
}

// ============================================
// UPDATE REPUTATION
// ============================================

#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(
        seeds = [
            b"rental_agreement",
            rental_agreement.landlord.as_ref(),
            rental_agreement.tenant.as_ref(),
            &rental_agreement.created_at.to_le_bytes()
        ],
        bump = rental_agreement.bump,
        constraint = rental_agreement.status == AgreementStatus::Completed @ RentGuardError::LeaseNotCompleted,
    )]
    pub rental_agreement: Account<'info, RentalAgreement>,
    
    #[account(
        init_if_needed,
        payer = rater,
        space = UserReputation::LEN,
        seeds = [b"reputation", user_to_rate.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,
    
    /// CHECK: The user being rated
    pub user_to_rate: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub rater: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn update_reputation(
    ctx: Context<UpdateReputation>,
    rating: u8,
    on_time_payments: u8,
) -> Result<()> {
    require!(rating >= 1 && rating <= 5, RentGuardError::InvalidRating);

    let agreement = &ctx.accounts.rental_agreement;
    let reputation = &mut ctx.accounts.user_reputation;
    let clock = Clock::get()?;
    let user_key = ctx.accounts.user_to_rate.key();

    // Initialize if needed
    if reputation.total_agreements == 0 {
        reputation.user = user_key;
        reputation.user_type = if user_key == agreement.tenant {
            UserType::Tenant
        } else {
            UserType::Landlord
        };
        reputation.created_at = clock.unix_timestamp;
        reputation.bump = ctx.bumps.user_reputation;
    }

    // Update stats
    reputation.total_agreements = reputation.total_agreements
        .checked_add(1)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    reputation.completed_agreements = reputation.completed_agreements
        .checked_add(1)
        .ok_or(RentGuardError::ArithmeticOverflow)?;

    // Update average rating
    let total_rating_points = (reputation.average_rating as u32)
        .checked_mul(reputation.total_ratings)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    let new_total_rating = total_rating_points
        .checked_add((rating as u32) * 20)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    reputation.total_ratings = reputation.total_ratings
        .checked_add(1)
        .ok_or(RentGuardError::ArithmeticOverflow)?;
    
    reputation.average_rating = (new_total_rating / reputation.total_ratings) as u16;

    // Update payment stats for tenant
    if user_key == agreement.tenant {
        reputation.total_payments_made = reputation.total_payments_made
            .checked_add(on_time_payments as u32)
            .ok_or(RentGuardError::ArithmeticOverflow)?;
        
        let late = agreement.missed_payments;
        reputation.late_payments = reputation.late_payments
            .checked_add(late as u32)
            .ok_or(RentGuardError::ArithmeticOverflow)?;
        
        let total_payments = reputation.total_payments_made + reputation.late_payments;
        if total_payments > 0 {
            reputation.on_time_payment_rate = 
                ((reputation.total_payments_made * 100) / total_payments) as u16;
        }
    }

    // Update deposit return stats for landlord
    if user_key == agreement.landlord {
        if agreement.approved_tenant_return > 0 {
            reputation.total_deposits_returned = reputation.total_deposits_returned
                .checked_add(1)
                .ok_or(RentGuardError::ArithmeticOverflow)?;
            
            reputation.deposit_return_rate = 
                ((reputation.total_deposits_returned * 100) / reputation.completed_agreements) as u16;
        }
    }

    reputation.last_updated = clock.unix_timestamp;

    emit!(ReputationUpdated {
        user: user_key,
        user_type: format!("{:?}", reputation.user_type),
        new_average_rating: reputation.average_rating,
        total_agreements: reputation.total_agreements,
        timestamp: clock.unix_timestamp,
    });

    msg!("Reputation updated for user: {}", user_key);
    Ok(())
}
