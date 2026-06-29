use anchor_lang::prelude::*;

#[error_code]
pub enum RentGuardError {
    #[msg("Invalid rent due day. Must be between 1 and 28.")]
    InvalidRentDueDay,
    
    #[msg("Invalid lease dates. End date must be after start date.")]
    InvalidLeaseDates,
    
    #[msg("Lease has not started yet.")]
    LeaseNotStarted,
    
    #[msg("Lease has already ended.")]
    LeaseEnded,
    
    #[msg("Security deposit already paid.")]
    DepositAlreadyPaid,
    
    #[msg("Security deposit not paid yet.")]
    DepositNotPaid,
    
    #[msg("Rent not due yet. Please wait until the due date.")]
    RentNotDue,
    
    #[msg("Rent already paid for this period.")]
    RentAlreadyPaid,
    
    #[msg("Insufficient funds for rent payment.")]
    InsufficientFunds,
    
    #[msg("No rent available to withdraw.")]
    NoRentToWithdraw,
    
    #[msg("Unauthorized. Only landlord can perform this action.")]
    UnauthorizedLandlord,
    
    #[msg("Unauthorized. Only tenant can perform this action.")]
    UnauthorizedTenant,
    
    #[msg("Agreement is not in the correct status for this action.")]
    InvalidAgreementStatus,
    
    #[msg("Both parties must approve the deposit distribution.")]
    DepositApprovalRequired,
    
    #[msg("Deposit distribution amounts must equal total deposit.")]
    InvalidDepositDistribution,
    
    #[msg("Dispute already filed.")]
    DisputeAlreadyFiled,
    
    #[msg("No active dispute to resolve.")]
    NoActiveDispute,
    
    #[msg("Only arbitrator can resolve disputes.")]
    UnauthorizedArbitrator,
    
    #[msg("Property address too long.")]
    PropertyAddressTooLong,
    
    #[msg("Dispute reason too long.")]
    DisputeReasonTooLong,
    
    #[msg("Invalid rating. Must be between 1 and 5.")]
    InvalidRating,
    
    #[msg("Lease must be completed before updating reputation.")]
    LeaseNotCompleted,
    
    #[msg("Arithmetic overflow occurred.")]
    ArithmeticOverflow,
    
    #[msg("Cannot terminate lease during active dispute.")]
    CannotTerminateDuringDispute,
}
