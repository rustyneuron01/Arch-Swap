use anyhow::{anyhow, Context, Result};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use colored::*;
use config::Config;
use std::thread;
use std::time::Duration;
pub struct WalletManager {
    pub client: Client,
    wallet_name: String,
}

impl WalletManager {
    pub fn new(config: &Config) -> Result<Self> {
        let rpc_user = config
            .get_string("bitcoin_rpc_user")
            .context("Failed to get Bitcoin RPC username")?;
        let rpc_password = config
            .get_string("bitcoin_rpc_password")
            .context("Failed to get Bitcoin RPC password")?;
        let wallet_name = config
            .get_string("bitcoin_rpc_wallet")
            .unwrap_or_else(|_| "devwallet".to_string());

        let wallet_rpc_uri = format!(
            "{}:{}/wallet/{}",
            config.get_string("bitcoin_rpc_endpoint")?,
            config.get_string("bitcoin_rpc_port")?,
            wallet_name
        );
        println!("Wallet RPC URI: {}", wallet_rpc_uri);
        let client = Client::new(&wallet_rpc_uri, Auth::UserPass(rpc_user, rpc_password))
            .context("Failed to create RPC client")?;

        // Print something to prove teh client is connected
        println!(
            "Client connected: {}",
            client.get_best_block_hash().unwrap()
        );
        let wallet_manager = Self {
            client,
            wallet_name,
        };
        wallet_manager.load_or_create_wallet(0)?;

        Ok(wallet_manager)
    }
    fn load_or_create_wallet(&self, retry_count: u8) -> Result<()> {
        if retry_count >= 5 {
            return Err(
                anyhow!(
                    "Max retry attempts reached. Please check if another Bitcoin Core instance is running."
                )
            );
        }

        match self.client.load_wallet(&self.wallet_name) {
            Ok(_) => {
                println!(
                    "  {} Wallet '{}' loaded successfully.",
                    "✓".bold().green(),
                    self.wallet_name.yellow()
                );
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("is already loaded") {
                    println!(
                        "  {} Wallet '{}' is already loaded.",
                        "✓".bold().green(),
                        self.wallet_name.yellow()
                    );
                    Ok(())
                } else if e.to_string().contains("Wallet file verification failed")
                    || e.to_string().contains("Requested wallet does not exist")
                    || e.to_string().contains("Unable to obtain an exclusive lock")
                {
                    println!(
                        "  {} Failed to load wallet '{}'. Error: {}",
                        "ℹ".bold().blue(),
                        self.wallet_name.yellow(),
                        e.to_string().red()
                    );
                    println!("Attempting to resolve the issue...");

                    // Try to unload the wallet first
                    let _ = self.client.unload_wallet(Some(&self.wallet_name));
                    thread::sleep(Duration::from_secs(2));

                    // Now try to create the wallet
                    match self
                        .client
                        .create_wallet(&self.wallet_name, None, None, None, None)
                    {
                        Ok(_) => {
                            println!(
                                "  {} Wallet '{}' created successfully.",
                                "✓".bold().green(),
                                self.wallet_name.yellow()
                            );
                            Ok(())
                        }
                        Err(create_err) => {
                            println!(
                                "  {} Failed to create wallet. Error: {}",
                                "⚠".bold().yellow(),
                                create_err.to_string().red()
                            );
                            println!("Waiting for 10 seconds before retrying...");
                            thread::sleep(Duration::from_secs(10));
                            self.load_or_create_wallet(retry_count + 1)
                        }
                    }
                } else {
                    Err(anyhow!("Unexpected error while loading wallet: {}", e))
                }
            }
        }
    }

    pub fn close_wallet(&self) -> Result<()> {
        match self.client.unload_wallet(Some(&self.wallet_name)) {
            Ok(_) => {
                println!(
                    "  {} Wallet '{}' unloaded successfully.",
                    "✓".bold().green(),
                    self.wallet_name.yellow()
                );
                Ok(())
            }
            Err(e) => {
                println!(
                    "  {} Failed to unload wallet '{}'. Error: {}",
                    "⚠".bold().yellow(),
                    self.wallet_name.yellow(),
                    e.to_string().red()
                );
                Err(anyhow!("Failed to unload wallet: {}", e))
            }
        }
    }

    // Add other methods as needed, e.g., get_balance, send_to_address, etc.
}

pub fn setup_bitcoin_rpc_client(config: &Config) -> Result<WalletManager> {
    WalletManager::new(config)
}
