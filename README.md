# 🏠 RentGuard - Decentralized Rental Agreement & Deposit Escrow

**Smart rental contracts with automatic deposit release based on conditions**

Built on Solana using Rust + Anchor framework.

---

## 🎯 **What It Does**

RentGuard eliminates trust issues between landlords and tenants by putting rental agreements on the blockchain:

- **Escrow Protection**: Security deposits held safely in smart contract, not by landlord
- **Automated Payments**: Monthly rent transfers automatically on due dates
- **Fair Disputes**: Neutral arbitrators resolve conflicts fairly
- **Reputation System**: Build credit history for both tenants and landlords
- **Transparent**: All terms visible on-chain, no hidden clauses

Think of it as **Venmo + Airbnb's trust system, but for long-term rentals on blockchain**.

---

## 🏗️ **Tech Stack**

- **Language**: Rust
- **Framework**: Anchor (Solana smart contracts)
- **Blockchain**: Solana
- **Key Features**: Time-based logic, Multi-sig elements, PDA escrow accounts

---

## 🚀 **Features**

### 1️⃣ **Rental Agreement Creation**
Landlord creates a new agreement with:
- Rent amount & deposit amount
- Monthly due date (1-28)
- Lease start/end dates
- Property address

### 2️⃣ **Deposit Escrow**
- Tenant deposits security into Program Derived Address (PDA)
- Funds locked until lease ends
- Protected from either party acting unilaterally

### 3️⃣ **Automated Rent Payments**
- Tenant pays monthly on the due date
- Contract tracks on-time vs late payments
- 5-day grace period before marking late
- Prevents double-payment for same period

### 4️⃣ **Landlord Rent Withdrawal**
- Landlord can withdraw accumulated rent anytime
- Deposit remains locked separately
- Automatic calculation of available funds

### 5️⃣ **Lease Termination & Deposit Return**
- Either party can initiate termination
- Both parties must approve deposit distribution
- Automatic transfer when both agree
- Fair split (e.g., 90% tenant, 10% landlord for damages)

### 6️⃣ **Dispute Resolution**
- Either party files dispute with detailed reason
- Neutral arbitrator reviews evidence
- Arbitrator decides fair deposit split
- Automatic distribution based on decision

### 7️⃣ **Reputation System**
- Tracks completed agreements
- Calculates on-time payment rate (tenant)
- Tracks deposit return rate (landlord)
- Builds credit score for future rentals
- Average ratings from past agreements

---

## 📁 **Project Structure**

```
programs/rentguard/src/
├── lib.rs              # Main program entry point (9 instructions)
├── state.rs            # Data structures (RentalAgreement, UserReputation)
├── instructions.rs     # Business logic for all operations
├── errors.rs           # Custom error types
└── events.rs           # Event emissions for transparency

tests/
└── rentguard.ts        # TypeScript tests

Anchor.toml             # Anchor configuration
Cargo.toml              # Rust dependencies
```

---

## 🔧 **Smart Contract Instructions**

| Instruction | Description |
|------------|-------------|
| `create_rental_agreement` | Landlord creates new lease agreement |
| `deposit_security` | Tenant deposits security into escrow |
| `pay_rent` | Tenant pays monthly rent |
| `withdraw_rent` | Landlord withdraws accumulated rent |
| `terminate_lease` | Either party initiates lease end |
| `approve_deposit_return` | Both parties approve deposit split |
| `file_dispute` | Either party files dispute |
| `resolve_dispute` | Arbitrator resolves dispute fairly |
| `update_reputation` | Update user reputation after lease ends |

---

## 📊 **Data Structures**

### RentalAgreement
Stores all lease information on-chain:
- Landlord & tenant public keys
- Financial terms (rent, deposit amounts)
- Lease dates & status
- Payment tracking (on-time, late, total paid)
- Dispute information
- Deposit approval status

### UserReputation
Tracks user history across all rentals:
- Total agreements (completed, disputed)
- Average rating (out of 100)
- On-time payment rate (tenant)
- Deposit return rate (landlord)

---

## 🎬 **How It Works (Example Flow)**

1. **Create Agreement**: Landlord creates contract for 2 SOL/month rent, 4 SOL deposit
2. **Deposit**: Tenant sends 4 SOL → locked in PDA escrow
3. **Monthly Payments**: Tenant pays 2 SOL on the 1st of each month
4. **Landlord Withdraws**: Landlord withdraws accumulated rent anytime
5. **Lease Ends**: Both parties agree: Tenant gets 3.5 SOL, Landlord gets 0.5 SOL (cleaning)
6. **Automatic Distribution**: Contract splits deposit exactly as agreed
7. **Reputation Update**: Both parties get rated, building credit history

---

## 🔐 **Security Features**

- **PDA Escrow**: Deposits held by program, not individuals
- **Multi-sig Approval**: Both parties must agree on deposit return
- **Time Locks**: Rent can only be paid once per period
- **Overflow Protection**: Safe math operations prevent exploits
- **Access Control**: Only authorized parties can perform actions

---

## 🛠️ **Setup & Development**

### Prerequisites
- Rust 1.96.0 or later
- Solana CLI tools
- Anchor Framework
- Node.js & Yarn

### Installation

```bash
# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test

# Deploy (configure Anchor.toml first)
anchor deploy
```

---

## 🧪 **Testing**

```bash
# Run all tests
anchor test

# Run specific test
anchor test -- --test test_name
```

---

## 📝 **Events Emitted**

All major actions emit events for transparency:
- `AgreementCreated`
- `DepositPaid`
- `RentPaid`
- `RentWithdrawn`
- `LeaseTerminated`
- `DepositReturned`
- `DisputeFiled`
- `DisputeResolved`
- `ReputationUpdated`



