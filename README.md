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
    Admin->>Program: create_round(round_id, start_time, end_time, ...)
    Note right of Program: round.status = Scheduled

    %% User actions (place / withdraw) - validated by time (cutoff)
    User->>Program: place_bet(round_id, bet_type, amount)
    alt now < end_time
      Program-->>User: accept_bet(tx confirmed)
    else now >= end_time
      Program-->>User: reject_bet("cutoff reached")
    end

    User->>Program: withdraw_bet(round_id, bet_id)
    alt now < end_time
      Program-->>User: refund_confirmed
    else
      Program-->>User: reject_withdraw("cutoff reached")
    end

    %% Keeper: activate scheduled rounds
    loop every N minutes
      Keeper->>Program: fetch_scheduled_rounds()
      Keeper->>Program: auto_activate_if(now >= start_time)
      Program-->>Keeper: ack (round.status = Active)
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
          Keeper->>Program: settle_round(round_id, price, proof?/metadata)
          Program-->>Program: determine_winners()
          Program-->>Program: calculate_and_set_claimable_amounts()
          Program-->>Keeper: ack (round.status = Ended)
        else all failed
          Keeper->>Program: set_pending_settlement(round_id, reason="oracle_fail")
          Program-->>Keeper: ack (round.status = PendingSettlement)
        end
      end
    end

    %% After settlement user claims
    User->>Program: claim_reward(round_id, bet_id)
    alt bet.claimable_amount > 0 and not bet.claimed
      Program-->>User: transfer_reward
      Program-->>User: mark_claimed
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
    J -- Yes --> K[Settle Round]
    K --> L[Users Claim Rewards]
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
    C -->|No| D[Round Closed]
    C -->|Yes| E[Wait for Settlement]
    E --> F{Bet Result}
    F -->|Win| G[Claim Reward]
    F -->|Lose| H[No Action Needed]
```

### User (Low-level)
```mermaid
stateDiagram-v2
    [*] --> Browsing: User opens app

    Browsing --> Betting: Place Bet<br/>place_bet()
    Browsing --> Waiting: Round not started yet
    Waiting --> Betting: Start time reached
    Betting --> Active: Bet Stored

    Active --> Active: Cancel Bet<br/>withdraw_bet()
    Active --> Locked: Round End<br/>end_time_reached()
    
    Locked --> Settled: Round Settled<br/>settle_round()
    Settled --> Claimed: Claim Rewards<br/>claim_rewards()

    Claimed --> [*]: User got rewards

    note right of Browsing
        User can view the list of active rounds and select the round you want to participate in.
    end note

    note right of Betting
        User selects the bet type (Gold Price / Stock Price) and the number of tokens.
    end note

    note right of Active
        The bet has been saved. Users can cancel before the cutoff.
    end note

    note right of Locked
        Cannot cancel bet. Waiting for settlement results
    end note

    note right of Settled
        The bet result is determined. Prizes are available if user win.
    end note

    note right of Claimed
        User successfully claimed prize Bet completed
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
    L --> M[Calculate Rewards for Winners]
    M --> N[Mark Bets as Won/Lost]
    N --> O[Set Round Status Ended]
    O --> P[Set Claimable Amount for Winners]
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

    Settling --> Ended: Settlement Success<br/>set_bets_won_lost() + set_claimable_amount()

    note right of Scheduled
        The round was created by an admin with a start_time in the future. Unable to accept bets yet. The guard continues to check every few minutes.
    end note

    note right of Active
        The round has started. Users can place bets. The Keeper waits until the end time reached to trigger settlement.
    end note

    note right of PendingSettlement
       Settlement failed due to failure to obtain a price. The Keeper will try again in the next loop. The user cannot bet (must lock bet).
    end note

    note right of Settling
        The Keeper is currently executing the settlement process:
        - Get price
        - Calculate winners
        - Mark bets as wins/losses
        - Calculate rewards
    end note

    note right of Ended
        Settlement complete. The round is marked as over. Users cannot bet or withdraw. They can only claim their rewards if they win
    end note
```

## Account Designs
### Config
- `admin`: The administrator of the contract.
- `token`: The Gold Rush Token (GRT) used for betting.
- `fee_gold_price`: The fee percentage charged on bets based on Gold Price.
- `fee_stock_price`: The fee percentage charged on bets based on stock price.
- `min_bet`: The minimum bet amount.
- `round_duration`: The duration of each betting round.
- `paused`: A boolean to pause all operations in the contract.
- `emergency_paused`: A boolean to pause emergency deposit and place bet.
- `current_round`: The current betting round number.
- `settlement_authority`: The authority responsible for settling rounds.
- `version`: The version of the contract.
- `oracle`: The Chainlink oracle address for fetching price data.
- `job_id`: The job ID for the Chainlink oracle.
- `oracle_whitelist`: A list of whitelisted oracle addresses.
- `treasury`: The address where the fees are sent.
- `bump`: A bump seed for PDA.

### Round
- `round`: The round number.
- `total_up_stake`: Total amount staked on Up bets.
- `total_down_stake`: Total amount staked on Down bets.
- `total_pct_stake`: Total amount staked on Percentage Change bets.
- `cutoff_time`: The time when betting closes for the round.
- `status`: The status of the round (Future, Active, PendingSettlement, Ended).
- `bump`: A bump seed for PDA.

### Bet
- `bettor`: The address of the player placing the bet.
- `round`: The round number for which the bet is placed.
- `bet_type`: The type of bet (e.g., Up, Down, Percentage Change).
- `amount`: The amount of GRT bet.
- `status`: The status of the bet (e.g., Pending, Won, Lost).
- `odds`: The odds associated with the bet type.
- `timestamp`: The timestamp when the bet was placed.
- `claimable_amount`: The amount that can be claimed as a reward.
- `claimed`: A boolean indicating if the reward has been claimed.
- `bump`: A bump seed for PDA.

### Vault
- `mint`: The mint address of the Gold Rush Token (GRT).
- `total_staked`: The total amount of GRT staked in the vault.
- `round`: The round number associated with the vault.
- `bump`: A bump seed for PDA.

### PriceFeed
- `price`: The current price fetched from the oracle.
- `timestamp`: The timestamp of the last price update.
- `slot`: The slot number of the last price update.
- `bump`: A bump seed for PDA.

## Program Instructions
### Initialize
Initializes the Gold Rush smart contract with the necessary configurations.

### UpdateConfig
Updates the configuration settings of the contract. Only the admin can perform this action.

### Pause
Pauses all operations in the contract. Only the admin can perform this action.

### Unpause
Unpauses all operations in the contract. Only the admin can perform this action.

### EmergencyPause
Pauses emergency deposit and place bet operations. Only the admin can perform this action.

### EmergencyUnpause
Unpauses emergency deposit and place bet operations. Only the admin can perform this action.

### SetOracle
Sets the Chainlink oracle address for fetching price data. Only the admin can perform this action.

### AddOracleToWhitelist
Adds an oracle address to the whitelist. Only the admin can perform this action.

### RemoveOracleFromWhitelist
Removes an oracle address from the whitelist. Only the admin can perform this action.

### StartRound
Starts a new betting round. Only the admin can perform this action.

### PlaceBet
Allows a player to place a bet on the current round. Players can choose between Up, Down, or Percentage Change bet types.

### SettleRound
Settles the current round by fetching the final price from the oracle and determining the outcome of the bets. Only the settlement authority can perform this action.

### ClaimReward
Allows players to claim their rewards after the round has been settled.

### EmergencyWithdraw
Allows players to withdraw their bets in case of emergencies.

### DepositToVault
Allows players to deposit GRT into the vault for staking.

### WithdrawFromVault
Allows players to withdraw their staked GRT from the vault.

### FetchPrice
Fetches the current price from the Chainlink oracle.

### FulfillPrice
Handles the response from the Chainlink oracle and updates the price feed.
