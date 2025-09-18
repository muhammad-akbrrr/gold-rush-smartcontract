# Gold Rush Smart Contract

Gold Rush is a token-based betting smart contract that allows users to bet on the price movement of gold or specific stocks within a specific time period (called a round).

Users place bets using Gold Rush Tokens (GRT), and winners receive rewards based on the outcome of the round.

## Overview
- Each round has a start time (start_time) and a end time (end_time), during which users can place or withdraw bets before the cutoff.
- After the round ends, the Keeper triggers a settlement process to determine the winner based on the price from the Oracle.
- Rewards are not automatically distributed but are stored as a claimable amount that winners can claim manually.

## Features

### Betting System

- Users can place bets on the price movement of gold or specific stocks (e.g., up/down).
- Bets are placed using GRT tokens.
- Bets can be withdrawn as long as they have not exceeded the cutoff.

### Round Lifecycle

- Admins create new rounds with a start_time and end_time.
- After the start_time, the round automatically becomes Active and accepts bets.
- After the end_time, bets are locked and the round enters the Settlement process and the outcome is determined.

### Keeper Automation

- The Keeper is responsible for triggering:
    - Round activation when the start_time is reached.
    - Round settlement when the end_time is reacged.
- The Keeper retrieves prices from the Oracle, calculates winners, and marks claimable prizes.
- If the price is not retrieved, the round is assigned a PendingSettlement status so it can be retried.

### Price Oracle

- Retrieves real-time prices from an external source (Chainlink or other whitelisted sources).
- Price data is only used during settlement process.

### Rewards & Claims

- Rewards are calculated from the total pool of losing bets and distributed proportionally to winners.
- Rewards are not sent automatically but are stored in the bet as claimable_amount.
- Users can claim rewards after settlement, if claimable_amount > 0.

### Admin Operations

- Create a new round (with a future schedule).
- Manage system configurations (fees, oracle, etc.).
- Pause/unpause the program in an emergency.
- Cancel a round (before settlement) to return all bets.

### Emergency & Safety

- Admins can perform an emergency pause to temporarily stop all betting operations (except claims).
- If settlement fails, the round will be marked as Pending Settlement so that no additional bets are accepted and can be reprocessed.

## Flows
### Full
```mermaid
sequenceDiagram
    participant Admin
    participant User
    participant Keeper
    participant Oracle
    participant Program

    %% Round creation
    Admin->>Program: create_round(round_id, asset, start_time, end_time, ...)
    Note right of Program: round.status = Planned

    %% User actions (place / withdraw)
    User->>Program: place_bet(round_id, bet_type, amount)
    alt now < round.end_time
      Program-->>User: accept_bet(tx confirmed)
    else
      Program-->>User: reject_bet("cutoff reached")
    end

    User->>Program: withdraw_bet(round_id, bet_id)
    alt now < round.end_time
      Program-->>User: refund_confirmed
    else
      Program-->>User: reject_withdraw("cutoff reached")
    end

    %% Keeper: activate scheduled rounds
    loop every N minutes
      Keeper->>Program: fetch_scheduled_rounds()
      Keeper->>Program: auto_activate_if(now >= start_time)
      Program-->>Keeper: ack (round.status = Active, locked_price set)
    end

    %% Keeper: settlement path (active/pending)
    loop every N minutes
      Keeper->>Program: fetch_rounds(status in {Active, PendingSettlement})
      Keeper->>Program: is_due_for_settlement(round_id)?
      alt not due
        Program-->>Keeper: skip
      else due
        %% Keeper obtains price from Oracle (off-chain)
        loop Retry up to 3 times
          Keeper->>Oracle: get_price(asset)
          Oracle-->>Keeper: price or error
        end

        alt received price
          Keeper->>Program: settle_round(round_id, final_price)
          Program-->>Program: set bet.status = Won/Lost
          Program-->>Program: sum winners_weight → round.winners_weight
          Program-->>Keeper: ack (round.status = Ended)
        else all failed
          Keeper->>Program: set_pending_settlement(round_id, reason="oracle_fail")
          Program-->>Keeper: ack (round.status = PendingSettlement)
        end
      end
    end

    %% After settlement, user claims reward
    User->>Program: claim_reward(round_id, bet_id)
    alt bet.status == Won and not bet.claimed
      Program-->>Program: reward = bet.weight / round.winners_weight * round.total_pool
      Program-->>Program: transfer_reward_from_vault(user, reward)
      Program-->>User: reward_transferred
      Program-->>Program: mark bet.claimed = true
    else
      Program-->>User: reject_claim
    end

```

### Admin (High-level)
```mermaid
flowchart TD
    A[Start] --> B{Config Initialized?}
    B -- No --> C[Initialize Config]
    B -- Yes --> D{Paused?}
    D -- Yes --> E[Unpause Program]
    D -- No --> F[Create New Round for Scheduled/Future]
    F --> G{Round Started?}
    G -- No --> H[Wait Until Start Time]
    G -- Yes --> I[Users Place Bets]
    I --> J{Cutoff Reached?}
    J -- No --> O[Wait Until Cutoff Reached]
    J -- Yes --> K[Settle Round: update bet status + sum winners_weight]
    K --> L[Users Claim Rewards: calculate reward based on weight]
    L --> M[Prepare Next Round]
    M --> F

    %% Optional cancel branch
    I --> X[Cancel Round]
    X --> Y[Refund All Bets]
    Y --> M
```

### Admin (Low-level)
```mermaid
stateDiagram-v2
    [*] --> NoConfig

    NoConfig --> Configured: Initialize Config<br/>initialize_config()
    Configured --> Configured: Update Config<br/>update_config()

    Configured --> Paused: Pause<br/>pause()
    Paused --> Configured: Unpause<br/>unpause()

    Configured --> EmergencyPaused: Emergency Pause<br/>emergency_pause()
    EmergencyPaused --> Configured: Emergency Unpause<br/>emergency_unpause()

    Configured --> Configured: Add Oracle<br/>add_oracle_to_whitelist()
    Configured --> Configured: Remove Oracle<br/>remove_oracle_from_whitelist()
    Configured --> Configured: Set Oracle<br/>set_oracle()

    Configured --> RoundScheduled: Create Scheduled Round<br/>create_round(start_time, end_time)
    RoundScheduled --> RoundActive: Start Round (when start_time)<br/>auto_activate()
    RoundActive --> Configured: Settle Round<br/>settle_round()  
        note right of RoundActive
            Settle Round marks bets as Won/Lost
            Sums winners_weight in Round
            Reward calculation deferred to user claim
        end note

    RoundScheduled --> Configured: Cancel Round<br/>cancel_round() 
    RoundActive --> Configured: Cancel Round<br/>cancel_round()

    Configured --> Configured: Upgrade Admin<br/>upgrade_admin()

    note right of NoConfig
        No configurations yet. No other operations are active
    end note

    note right of Configured
        All admin operations are active. Admin can create rounds, manage oracles, etc.
    end note

    note right of Paused
        All operations are temporarily frozen. Can be resumed with unpause()
    end note

    note right of EmergencyPaused
        Only settlement and claim are allowed. New bets are blocked
    end note

    note right of RoundScheduled
        The round has been created but has not started. Users cannot bet yet.
    end note

    note right of RoundActive
        The round is in progress. Users can bet until the cutoff.
    end note
```

### User (High-level)
```mermaid
graph TD
    A[Select Active Round] --> B[Place Bet]
    B --> C{Before End Time?}
    C -->|No| D[Round Closed - cannot bet]
    C -->|Yes| E[Wait for Round End]
    E --> F{Round Settled?}
    F -->|Yes| G{Bet Result}
    G -->|Win| H[Claim Reward - calculate reward using weight]
    G -->|Lose| I[No Action Needed]
    F -->|No| J[Wait for Settlement]

```

### User (Low-level)
```mermaid
stateDiagram-v2
    [*] --> Browsing: User opens app

    Browsing --> Betting: Place Bet<br/>place_bet()
    Browsing --> Waiting: Round not started yet
    Waiting --> Betting: Start time reached
    Betting --> Active: Bet Stored

    Active --> Active: Cancel Bet<br/>withdraw_bet() before end_time
    Active --> Locked: Round End<br/>end_time_reached()

    Locked --> Settled: Round Status = Ended<br/>settle_round() sets Won/Lost and winners_weight
    Settled --> Claiming: User claims reward<br/>claim_reward()
    Claiming --> Claimed: Transfer reward based on bet weight

    Claimed --> [*]: User got reward

    note right of Browsing
        User can view active rounds and select the round to participate.
    end note

    note right of Betting
        User selects bet type (Gold Price / Stock Price) and amount.
    end note

    note right of Active
        Bet is stored. User can cancel before end_time (cutoff).
    end note

    note right of Locked
        Cannot cancel bet. Waiting for round settlement.
    end note

    note right of Settled
        Bet status determined (Won/Lost). Reward amount not yet claimed.
    end note

    note right of Claiming
        Reward is calculated using bet weight / winners_weight * total_pool.
    end note

    note right of Claimed
        User successfully claimed reward. Bet completed.
    end note

```

### Keeper (High-level)
```mermaid
flowchart TD
    A[Start] --> B[Fetch All Scheduled Rounds]
    B --> C{Now >= start_time?}
    C -- No --> B1[Wait 5 min] --> B
    C -- Yes --> D[Set Round as Active]

    %% Settlement path
    A --> E[Fetch All Active or PendingSettlement Rounds]
    E --> F{Now >= end_time?}
    F -- No --> E1[Wait 5 min] --> E
    F -- Yes --> G[Get Current Price from Oracle]

    G --> H{Success?}
    H -- No --> I[Retry, max 3]
    I -->|Failed| J[Set Status PendingSettlement] --> E
    H -- YES --> K[Continue Settlement]

    K --> L[Determine Winners & Losers]
    L --> M[Update winners_weight in Round]
    M --> N[Mark Bets as Won/Lost]
    N --> O[Set Round Status Ended]
```

### Keeper (Low-level)
```mermaid
stateDiagram-v2
    [*] --> Scheduled

    Scheduled --> Active: Trigger Start<br/>now >= start_time
    Active --> Active: Keep Checking<br/>before end_time

    Active --> PendingSettlement: Trigger Settlement<br/>get_price() failed after retries
    Active --> Settling: Trigger Settlement<br/>get_price() success

    PendingSettlement --> Settling: Retry Settlement<br/>get_price() success

    Settling --> Ended: Settlement Success<br/>set_bets_won_lost() + update winners_weight in Round

    note right of Scheduled
        The round was created by an admin with a start_time in the future. Unable to accept bets yet. The keeper continues to check periodically.
    end note

    note right of Active
        The round has started. Users can place bets. The Keeper waits until end_time to trigger settlement.
    end note

    note right of PendingSettlement
       Settlement failed due to oracle error. Keeper retries in next loop. Users cannot bet.
    end note

    note right of Settling
        The Keeper executes settlement:
        - Obtain price
        - Determine winners/losers
        - Update winners_weight
        - Mark bets as Won/Lost
    end note

    note right of Ended
        Round is ended. Users cannot bet or withdraw. They can claim rewards based on weight.
    end note
```

## Account Designs
### Config
```rust
pub struct Config {
  // --- Authorities ---
  pub admin: Pubkey,                   // The administrator of the contract.
  pub settlement_authority: Pubkey,    // The authority responsible for settling rounds.
  pub keeper_authorities: Vec<Pubkey>, // The authority for keeper/oracle accounts allowed to keeper operations.

  // --- Token & Treasury ---
  pub token_mint: Pubkey,              // The Gold Rush Token (GRT) used for betting.
  pub treasury: Pubkey,                // The address where the fees are sent.

  // --- Fee Config ---
  pub fee_gold_price_bps: u16,         // The fee percentage charged on bets based on Gold Price.
  pub fee_stock_price_bps: u16,        // The fee percentage charged on bets based on stock price.

  // --- Betting Rules ---
  pub min_bet_amount: u64,             // The minimum bet amount.

  // --- Global State ---
  pub status: ContractStatus,          // Overall contract status (Active / Paused / EmergencyPaused)
  pub current_round_counter: u64,      // Incremental counter for new round IDs

  // --- Metadata ---
  pub version: u8,                     // The version of the contract.
  pub bump: u8,                        // A bump seed for PDA.
}

// Enum for program status flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ContractStatus {
    Active,
    Paused,
    EmergencyPaused,
}
```

### Round

```rust
pub struct Round {
  // --- Identity ---
  pub id: u64,                   // Unique identifier for the round (incremental from config.current_round_counter).
  pub asset: Pubkey,             // The asset being bet on (e.g., Gold, Stock).
  pub start_time: i64,           // The timestampt when round is scheduled to start.
  pub end_time: i64,             // The timestamp when round is scheduled to end.
  pub vault: Pubkey,             // The vault account holding the bets for this round.

  // --- State ---
  pub status: RoundStatus,       // The current status of the round (Planned, Active, PendingSettlement, Ended).
  pub locked_price: Option<u64>, // The price when round becomes Active.
  pub final_price: Option<u64>,  // The price when round is settled.
  pub total_pool: u64,           // The total amount of GRT bet in this round.
  pub total_bets: u64,           // The total number of bets placed in this round.
  pub total_fee_collected: u64,  // The total fees collected for this round.
  pub total_reward_pool: u64,    // The total reward pool after deducting fees.
  pub winners_weight: u64,       // The total weight of winning bets (for reward calculation). Default to 0 if no winners.

  // --- Metadata ---
  pub created_at: i64,           // The timestamp when the round was created.
  pub settled_at: Option<i64>,   // The timestamp when the round was settled.
  pub bump: u8,                  // A bump seed for PDA.
}

// Enum for round status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RoundStatus {
    Planned,                    // Created but not started yet
    Active,                     // Currently accepting bets
    PendingSettlement,          // Ended but settlement failed, needs retry
    Ended,                      // Successfully settled
}
```

### Bet
```rust
pub struct Bet {
  // --- Identify ---
  pub round: Pubkey,           // The round this bet is associated with.
  pub bettor: Pubkey,          // The address of the player placing the bet.

  // --- Bet Info ---
  pub amount: u64,            // The amount of GRT bet.
  pub side: BetSide,          // The type of bet (Up, Down, PercentageChange).
  pub claimed: bool,          // Whether the reward has been claimed.
  pub weight: u64,            // The weight of the bet (for reward calculation).

  // --- State ---
  pub status: BetStatus,      // The status of the bet (Pending, Won, Lost).

  // --- Metadata ---
  pub created_at: i64,        // The timestamp when the bet was placed.
  pub bump: u8,               // A bump seed for PDA.
}

// Enum for bet types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BetSide {
    Up,
    Down,
    PercentageChange(i16),   // e.g., 10 for 0.1%, -25 for -0.25%
}

// Enum for bet status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
}
```

---

## Fee Mechanism

The Gold Rush smart contract charges a fee on each bet to sustain the platform. The fee is collected in the same token used for betting (GRT) and is transferred to the treasury after the round is settled.

### Fee Structure

- **Gold Price Bets**: `fee_gold_price_bps` (in basis points, e.g., 50 bps = 0.5%)
- **Stock Price Bets**: `fee_stock_price_bps` (in basis points, e.g., 50 bps = 0.5%)
- **Treasury Account**: `treasury` (the account that receives collected fees)

> Note: Fee is calculated on the bet amount and deducted only at settlement, not at the time of placing a bet. This allows for refunds in case the round is cancelled.

### Fee Calculation

For each round, the total fee collected is calculated as:

$$
\text{fee} = \sum_{i=1}^{n} \text{amount}_i \times \frac{\text{fee\_bps}}{10000}
$$

Where:

- $n$ = total number of bets in the round  
- $\text{amount}_i$ = bet amount of the $i$-th bet  
- $\text{fee\_bps}$ = fee basis points depending on bet type (gold or stock)

The remaining balance after fee deduction becomes the **reward pool**:

$$
\text{total\_reward\_pool} = \text{total\_pool} - \text{total\_fee\_collected}
$$

- `total_pool` = sum of all bet amounts in the round  
- `total_fee_collected` = sum of all fees deducted  
- `total_reward_pool` = amount available for distribution to winning bets

### Settlement Flow for Fees

1. **Round ends**: Keeper triggers the settlement process after `end_time`.  
2. **Calculate fees**: Fee is calculated for each bet based on `fee_gold_price_bps` or `fee_stock_price_bps`.  
3. **Transfer to treasury**: `total_fee_collected` is transferred to the treasury account.  
4. **Compute reward pool**: Remaining GRT is assigned to `total_reward_pool` for winners.  

> This approach ensures fairness and transparency: users only pay fees if the round is successfully settled, and the treasury receives an accurate portion without affecting refund logic.

### Example

Suppose a round has the following bets:

| User | Bet Amount | Bet Type   | Fee (bps) |
|------|------------|------------|------------|
| A    | 10 GRT     | Gold       | 50         |
| B    | 20 GRT     | Stock      | 100        |

Calculations:

- Fee for A: \( 10 \times 0.005 = 0.05 \) GRT  
- Fee for B: \( 20 \times 0.01 = 0.2 \) GRT  

Total fee collected: \( 0.05 + 0.2 = 0.25 \) GRT  

Total reward pool: \( 10 + 20 - 0.25 = 29.75 \) GRT

---

## Reward Calculation

This section explains the reward calculation logic used in the Gold Rush betting system.  
The system uses a **weight-based model** combined with a **self-claim mechanism** to optimize on-chain operations, especially when there are thousands of bets per round.

### Concept

Each bet has a **weight** that represents its contribution to the total reward pool.  
The weight is influenced by:

1. **Bet Amount**: Larger bets contribute more.
2. **Bet Type Factor**: Bets on `PercentageChange` may carry a higher factor than simple `Up/Down` bets.
3. **Time Factor**: Bets placed earlier in the round may carry a higher weight than bets placed near the end of the round.

The general formula for a bet's weight:

$$
\text{weight} = \text{amount} \times \text{side\_factor} \times \text{time\_factor}
$$

Where:

- `amount` = number of GRT tokens bet
- `side_factor` = multiplier based on `BetSide`
  - Up/Down: `1.0`
  - PercentageChange: `>1.0` depending on the magnitude of percentage
- `time_factor` = multiplier based on how early the bet was placed (range: $0 < \text{time\_factor} \le 1.0$)

---

### Settlement

During settlement:

1. The **Keeper** determines the winning bets by comparing `locked_price` and `final_price`.
2. For all winning bets:
   - Mark `Bet.status = Won`
   - Sum their weights into `round.winners_weight`:

$$
\text{round.winners\_weight} = \sum_{\text{all winning bets}} \text{bet.weight}
$$

3. For losing bets:
   - Mark `Bet.status = Lost`
   - Weight is ignored for reward calculation.

> **Note:** No reward amount is calculated yet. Only weights are updated. This avoids looping all bets during settlement, optimizing on-chain efficiency.

---

### Self-Claim by User

When a user claims their reward:

1. Read `Bet.weight` and `round.winners_weight`.
2. Calculate the claimable reward:

$$
\text{reward} = \frac{\text{bet.weight}}{\text{round.winners\_weight}} \times \text{round.total\_pool}
$$

3. Transfer the reward from the round vault to the user's account.
4. Mark `Bet.claimed = true`.

> This approach ensures:
> - **No heavy loops** during settlement
> - Reward is calculated **on-demand**
> - Users can **preview rewards** before claiming if needed (optional off-chain)

---

### Example Calculation

| User | Amount (GRT) | Bet Type | Time Factor | Weight |
|------|--------------|----------|------------|--------|
| A    | 10           | Up       | 1.0        | 10     |
| B    | 10           | Percentage | 1.2      | 12     |

- Total weight of winners: \( 10 + 12 = 22 \)  
- Total pool: 20 GRT  

$$
\text{Reward A} = \frac{10}{22} \times 20 \approx 9.09\text{ GRT}
$$
$$
\text{Reward B} = \frac{12}{22} \times 20 \approx 10.91\text{ GRT}
$$

---

### Summary

- Settlement **only sets status** (Won/Lost) and updates `winners_weight`.
- Reward calculation is deferred to **self-claim**, ensuring efficiency.
- Weight incorporates **amount**, **bet type**, and **time factor**.
- The model is flexible and can be adjusted for new bet types or reward rules.

---


## Program Instructions
### Initialize
Initializes the Gold Rush smart contract with the necessary configurations.

### Admin: Update Config
Updates the configuration settings of the contract. Only the admin can perform this action.

### Admin: Pause
Pauses all operations in the contract. Only the admin can perform this action.

### Admin: Unpause
Unpauses all operations in the contract. Only the admin can perform this action.

### Admin: Emergency Pause
Pauses emergency deposit and place bet operations. Only the admin can perform this action.

### Admin: Emergency Unpause
Unpauses emergency deposit and place bet operations. Only the admin can perform this action.

### Admin: Create Round
Creates a new betting round with specified parameters. Only the admin can perform this action.

### Admin: Cancel Round
Cancels an active or scheduled round and refunds all bets. Only the admin can perform this action

### Keeper: Start Round
Starts a scheduled round when the start_time is reached. Only the keeper authority can perform this action.

### Keeper: Settle Round
Settles a round by determining winners and losers based on the final price from the Oracle. Only the keeper can perform this action.

### User: Place Bet
Allows a player to place a bet on the current round. Players can choose between Up, Down, or Percentage Change bet types.

### User: Withdraw Bet
Allows players to withdraw their bets before the round ends.

### User: Claim Reward
Allows players to claim their rewards after the round has been settled.

## PDA Seeds Strategy