# Gold Rush Smart Contract

A betting smart contract for a game called "Gold Rush". Users can bets based on Gold Price movements or the company's stock price and win rewards based on the outcome. Users can place bets using Gold Rush Token (GRT). Multiple bet types are supported, including betting going Up/Down and percentage changes.

## Features
- *Betting*: Players can place bets on the price movement of Gold or a specific stock with Gold Rush Token (GRT).
- *Price Feeds*: Integrates with Chainlink oracles to fetch real-time price data.
- *Rewards*: Winners receive rewards based on their bets.
- *Admin Fee*: Fee charged on each bet to sustain the platform.
- *Rounds*: The game operates in weekly rounds.
- *Claim Rewards*: Players can claim their rewards after the round ends. The rewards are calculated based on the total pool, individual bets, and bet types. The percentage change bets have more larger rewards.
- *Emergency Withdraw*: Players can withdraw their bets in case of emergencies.

## Flows
### Normal
```mermaid
flowchart TD
    A[Admin: Create Round] --> B[Users: Place Bets]
    B -->|Betting Period Ends| C[Backend: Trigger Settlement]
    C --> D[Contract: Calculate Outcomes & Rewards]
    D --> E[Users: Claim Rewards]
    E --> F[Bet Status: Mark as Claimed]
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

### Admin Lifecycle (Low-level)
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

    Configured --> RoundScheduled: Create Scheduled Round<br/>create_round(start_time)
    RoundScheduled --> RoundActive: Start Round (when start_time)<br/>auto_activate()
    RoundActive --> Configured: Settle Round<br/>settle_round()
    RoundScheduled --> Configured: Cancel Round<br/>cancel_round() 
    RoundActive --> Configured: Cancel Round<br/>cancel_round()

    Configured --> Configured: Upgrade Admin<br/>upgrade_admin()

    note right of NoConfig
        Belum ada konfigurasi
        Tidak ada operasi lain yang aktif
    end note

    note right of Configured
        Semua operasi admin aktif
        Bisa buat round, kelola oracle, dll.
    end note

    note right of Paused
        Semua operasi dibekukan sementara
        Bisa dilanjutkan dengan unpause()
    end note

    note right of EmergencyPaused
        Hanya operasi darurat yang aktif
        Digunakan saat ada kondisi kritis
    end note

    note right of RoundScheduled
        Round sudah dibuat tapi belum mulai
        User belum bisa bet
    end note

    note right of RoundActive
        Round sedang berjalan
        User bisa bet sampai cutoff
    end note
```

### User (High-level)
```mermaid
graph TD
    A[Select Active Round] --> B[Place Bet]
    B --> C{Before Cutoff?}
    C -->|No| D[Round Cloed]
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
    Betting --> Active: Bet Stored

    Active --> Active: Cancel Bet<br/>withdraw_bet()
    Active --> Locked: Round Cutoff<br/>cutoff_reached()
    
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
- `status`: The status of the round (e.g., Active, Ended).
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
