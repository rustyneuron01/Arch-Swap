# Arch-Swap

A smart contract platform on the Arch Network for Rune & Inscription swaps and liquidity pools.

## Project Structure

```
arch-swap/
├── program/              # Smart Contract Implementation
│   ├── src/
│   │   ├── lib.rs       # Main contract entry point
│   │   ├── account.rs   # Account management
│   │   ├── instruction.rs # Instruction processing
│   │   └── utxo.rs      # UTXO handling
│   └── Cargo.toml       # Rust dependencies
├── app/
│   └── frontend/        # Web interface
└── common/              # Shared utilities
```

## Smart Contract Implementation

### 1. Main Contract Entry Point (lib.rs)

The main entry point handles instruction routing for different swap operations:

```rust
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    match instruction_data[0] {
        0 => {
            // Swap Inscription And Rune

            msg!("Swap Inscription And Rune ");

            swap_inscription_rune(accounts, program_id, instruction_data)
        }
        1 => {
            // Swap Rune And Inscription

            msg!("Swap Rune And Inscription ");

            swap_rune_inscription(accounts, program_id, instruction_data)
        }
        _ => {
            msg!("Invalid argument provided !");
            return Err(ProgramError::InvalidArgument);
        }
    }
}
```

This code:

- Takes program ID, account info array, and instruction data as parameters
- Uses the first byte of instruction data to determine the operation type
- Routes to specific handlers (e.g., rune_swap)
- Returns ProgramError for invalid instructions

### 2. UTXO Transaction Processing (rune_swap.rs)

The UTXO handling for Rune swaps:

```rust
for (i, txid) in params.rune_txids.iter().enumerate() {
    let vout = params.rune_vouts.get(i).unwrap(); // Safely get vout corresponding to txid
    user_swap_tx.input.push(TxIn {
        previous_output: OutPoint {
            txid: Txid::from_str(txid).unwrap(),
            vout: *vout as u32,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    });
}

let tx_to_sign = TransactionToSign {
    tx_bytes: &bitcoin::consensus::serialize(&user_swap_tx),
    inputs_to_sign: &inputs_to_sign,
};

set_transaction_to_sign(accounts, tx_to_sign)
```

This code:

- Processes UTXO inputs for the swap transaction
- Creates transaction inputs with proper OutPoints
- Initializes empty script signatures and witnesses
- Prepares the transaction for signing

## Setup and Deployment

### 1. Project Creation

Create a new project using the Arch CLI:

```bash
cli create project --name example --network testnet --rpc-url http://localhost:9002
```

### 2. Program Account Setup

Create a program-controlled account for asset management:

```bash
cli create account --name ArchAccount --program-id xxx --rpc-url http://localhost:9002 --network testnet
```

This account:

- Holds all assets (Rune, Inscription, BTC)
- Is controlled by the smart contract
- Requires program ID from deployed contract

### 3. Smart Contract Deployment

Deploy the compiled contract:

```bash
cli deploy ./target/deploy/Arch-Swap.so --network-mode testnet --rpc-url http://localhost:9002
```

## Backend Integration

### Transaction Signing and Submission

Example of backend integration with proper error handling:

```typescript
const messageHash = MessageUtil.hash(messageObj);
const { signature } = await createSignature(
  messageHash,
  process.env.ARCH_PRIVATE_KEY!
);
const signatureBuffer = new Uint8Array(Buffer.from(signature));

const tx = {
  version: 16,
  signatures: [signatureBuffer],
  message: messageObj,
};

for (let i = 0; i < MAX_RETRIES; i++) {
  try {
    const result = await client.sendTransaction(tx);
    return result;
  } catch (error) {
    if (i === MAX_RETRIES - 1) throw error;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}
```

This code:

- Creates a message hash for signing
- Signs the transaction with the Arch private key
- Implements retry logic for transaction submission
- Handles failed transactions with exponential backoff

Important: Transactions must be signed using the Arch Program account, or they will be rejected.

## Dependencies

### Smart Contract

```toml
[dependencies]
borsh = "1.4.0"      # Serialization
bitcoin = "0.31.0"   # Bitcoin integration
serde = "1.0.198"    # Data serialization
```

### Frontend

- TypeScript/Node.js for type safety
- Web3 libraries for blockchain interaction

## Development Notes

1. Account Management:

   - Use program-controlled accounts for all asset operations
   - Implement proper signature verification
   - Handle transaction retries with backoff

2. Transaction Processing:

   - Validate all UTXO inputs
   - Ensure proper witness and script handling
   - Implement comprehensive error handling

3. Security:
   - Never expose private keys
   - Always verify signatures
   - Validate all transaction parameters
