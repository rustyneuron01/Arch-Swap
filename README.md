# Arch-Swap

Arch-Swap is a smart contract developed on the Arch Network. It facilitates Rune & Rune, Rune & Inscription swaps, and liquidity pools using smart contracts on the Arch Network.

## Features
- Rune Swap
- Rune & Inscription Swap
- Liquidity Pool Management

## Getting Started

### Create a New Project

To create a new project, use the `arch-cli` command line tool:

```
arch-cli create project --name example --network testnet --rpc-url http://localhost:9002
```

### Create a Program Account Controlled by the Smart Contract. 

You can get program Id after deploy Smart Contract. All assets (Rune, Inscription, BTC) should be saved on this account. You can create it with the following command:

```
arch-cli create account --name ArchAccount --program-id xxx --rpc-url http://localhost:9002 --network testnet
```

### Make Smart Contract in Rust
In your lib.rs file:

```
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    match instruction_data[0] {
        0 => {
            // Rune Swap
            msg!("Rune Swap");
            rune_swap(accounts, program_id, instruction_data)
        }
        _ => {
            msg!("Invalid argument provided!");
            return Err(ProgramError::InvalidArgument);
        }
    }
}
```

In the rune_swap.rs file:
```
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

### Deploy Smart Contract
```
arch-cli deploy --network testnet --rpc-url http://localhost:9002
```

### Connecting the Smart Contract on the Backend
To serialize all data, sign it using the Arch Account, and send it to the smart contract, follow this example:

```
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

## Languages Used
- Rust
- TypeScript
- Node.js

## Contributing
I welcome contributions! Please submit issues and pull requests to help improve Arch-Swap.
