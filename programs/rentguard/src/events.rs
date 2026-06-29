use anchor_lang::prelude::*;

#[event]
pub struct AgreementCreated {
    pub agreement: Pubkey,
    pub landlord: Pubkey,
    pub tenant: Pubkey,
    pub rent_amount: u64,
    pub deposit_amount: u64,
    pub lease_start_date: i64,
    pub lease_end_date: i64,
}

#[event]
pub struct DepositPaid {
    pub agreement: Pubkey,
    pub tenant: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RentPaid {
    pub agreement: Pubkey,
    pub tenant: Pubkey,
    pub amount: u64,
    pub payment_date: i64,
    pub is_on_time: bool,
}

#[event]
pub struct RentWithdrawn {
    pub agreement: Pubkey,
    pub landlord: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct LeaseTerminated {
    pub agreement: Pubkey,
    pub initiated_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DepositReturned {
    pub agreement: Pubkey,
    pub tenant_amount: u64,
    pub landlord_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DisputeFiled {
    pub agreement: Pubkey,
    pub filed_by: Pubkey,
    pub reason: String,
    pub requested_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DisputeResolved {
    pub agreement: Pubkey,
    pub resolved_by: Pubkey,
    pub tenant_amount: u64,
    pub landlord_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReputationUpdated {
    pub user: Pubkey,
    pub user_type: String,
    pub new_average_rating: u16,
    pub total_agreements: u32,
    pub timestamp: i64,
}
