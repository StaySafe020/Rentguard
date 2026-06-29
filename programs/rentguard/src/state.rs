use anchor_lang::prelude::*;

/// Rental Agreement Status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AgreementStatus {
    Created,           // Agreement created, waiting for deposit
    Active,            // Deposit paid, lease is active
    TerminationPending, // One party initiated termination
    DisputePending,    // Dispute filed, waiting for resolution
    Completed,         // Lease ended, deposit returned
    Cancelled,         // Agreement cancelled before activation
}

/// Dispute Status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DisputeStatus {
    None,
    Filed,
    UnderReview,
    Resolved,
}

/// Rental Agreement Account
#[account]
pub struct RentalAgreement {
    pub landlord: Pubkey,
    pub tenant: Pubkey,
    pub property_address: String,
    
    // Financial terms
    pub rent_amount: u64,
    pub deposit_amount: u64,
    pub rent_due_day: u8,           // Day of month (1-31)
    
    // Dates
    pub lease_start_date: i64,      // Unix timestamp
    pub lease_end_date: i64,        // Unix timestamp
    pub created_at: i64,
    
    // Status
    pub status: AgreementStatus,
    pub deposit_paid: bool,
    
    // Rent tracking
    pub last_rent_payment: i64,
    pub total_rent_paid: u64,
    pub missed_payments: u8,
    pub on_time_payments: u8,
    
    // Deposit management
    pub deposit_in_escrow: u64,
    pub landlord_deposit_approval: bool,
    pub tenant_deposit_approval: bool,
    pub approved_tenant_return: u64,
    pub approved_landlord_return: u64,
    
    // Dispute
    pub dispute_status: DisputeStatus,
    pub dispute_reason: String,
    pub dispute_requested_amount: u64,
    pub dispute_filed_by: Option<Pubkey>,
    pub arbitrator: Option<Pubkey>,
    
    // Metadata
    pub bump: u8,
}

impl RentalAgreement {
    pub const MAX_ADDRESS_LEN: usize = 200;
    pub const MAX_DISPUTE_REASON_LEN: usize = 500;
    
    pub const LEN: usize = 8 + // discriminator
        32 + // landlord
        32 + // tenant
        (4 + Self::MAX_ADDRESS_LEN) + // property_address
        8 + // rent_amount
        8 + // deposit_amount
        1 + // rent_due_day
        8 + // lease_start_date
        8 + // lease_end_date
        8 + // created_at
        1 + // status
        1 + // deposit_paid
        8 + // last_rent_payment
        8 + // total_rent_paid
        1 + // missed_payments
        1 + // on_time_payments
        8 + // deposit_in_escrow
        1 + // landlord_deposit_approval
        1 + // tenant_deposit_approval
        8 + // approved_tenant_return
        8 + // approved_landlord_return
        1 + // dispute_status
        (4 + Self::MAX_DISPUTE_REASON_LEN) + // dispute_reason
        8 + // dispute_requested_amount
        (1 + 32) + // dispute_filed_by Option
        (1 + 32) + // arbitrator Option
        1; // bump
}

/// User Reputation Account
#[account]
pub struct UserReputation {
    pub user: Pubkey,
    pub user_type: UserType,
    
    // Stats
    pub total_agreements: u32,
    pub completed_agreements: u32,
    pub disputed_agreements: u32,
    pub average_rating: u16,        // Out of 100
    pub total_ratings: u32,
    
    // Tenant specific
    pub on_time_payment_rate: u16,  // Percentage (0-100)
    pub total_payments_made: u32,
    pub late_payments: u32,
    
    // Landlord specific
    pub deposit_return_rate: u16,   // Percentage (0-100)
    pub total_deposits_returned: u32,
    
    pub created_at: i64,
    pub last_updated: i64,
    pub bump: u8,
}

impl UserReputation {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        1 + // user_type
        4 + // total_agreements
        4 + // completed_agreements
        4 + // disputed_agreements
        2 + // average_rating
        4 + // total_ratings
        2 + // on_time_payment_rate
        4 + // total_payments_made
        4 + // late_payments
        2 + // deposit_return_rate
        4 + // total_deposits_returned
        8 + // created_at
        8 + // last_updated
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum UserType {
    Tenant,
    Landlord,
    Both,
}
