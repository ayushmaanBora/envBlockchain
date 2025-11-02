use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};

const WALLET_FILE: &str = "wallets.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Wallet {
    balance_yuki: u64,
    balance_yg: u64,
    balance_yt: u64,
}

impl Wallet {
    fn new() -> Self {
        Self {
            balance_yuki: 10,
            balance_yg: 0,
            balance_yt: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: u64,
    task: String,
    proof_metadata: String,
    verified: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let hash = Self::calculate_hash(index, timestamp, &transactions, &previous_hash);
        Self { index, timestamp, transactions, previous_hash, hash }
    }

    fn calculate_hash(index: u64, timestamp: i64, transactions: &Vec<Transaction>, previous_hash: &str) -> String {
        let input = format!("{}{}{:?}{}", index, timestamp, transactions, previous_hash);
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Listing {
    seller: String,
    price_per_token: u64,
    tokens_available: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Marketplace {
    listings: Vec<Listing>,
}

impl Marketplace {
    fn new() -> Self {
        Self { listings: Vec::new() }
    }

    fn list_tokens(&mut self, seller: String, price: u64, amount: u64) {
        self.listings.push(Listing {
            seller,
            price_per_token: price,
            tokens_available: amount,
        });
        println!("✅ Tokens listed for sale.");
    }

    fn display_listings(&self) {
        if self.listings.is_empty() {
            println!("No listings available.");
        } else {
            println!("Marketplace Listings:");
            for (index, listing) in self.listings.iter().enumerate() {
                println!(
                    "{}. Seller: {} | Price: {} Yuki/token | Tokens: {}",
                    index + 1,
                    listing.seller,
                    listing.price_per_token,
                    listing.tokens_available
                );
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Blockchain {
    chain: Vec<Block>,
    wallets: HashMap<String, Wallet>,
    mining_reward: u64,
    stake_amount: u64,
    marketplace: Marketplace,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, vec![], "0".to_string());
        let wallets = Self::load_wallets();
        Self {
            chain: vec![genesis_block],
            wallets,
            mining_reward: 10,
            stake_amount: 5,
            marketplace: Marketplace::new(),
        }
    }

    fn add_block(&mut self, wallet: &str, task: String, proof_metadata: String) {
        if let Some(wallet_data) = self.wallets.get_mut(wallet) {
            if wallet_data.balance_yuki < self.stake_amount {
                println!("❌ Stake failed. Insufficient balance.");
                return;
            }

            wallet_data.balance_yuki -= self.stake_amount;

            wallet_data.balance_yuki += self.mining_reward;
            let transaction = Transaction {
                sender: "System".to_string(),
                receiver: wallet.to_string(),
                amount: self.mining_reward,
                task,
                proof_metadata,
                verified: true,
            };

            let previous_block = self.chain.last().unwrap();
            let new_block = Block::new(previous_block.index + 1, vec![transaction], previous_block.hash.clone());
            self.chain.push(new_block);

            println!("✅ Task verified! Block added. Tokens awarded.");
            self.save_wallets();
        } else {
            println!("❌ Wallet does not exist.");
        }
    }

    fn marketplace_menu(&mut self) {
        println!("\nMarketplace Options:");
        println!("1. List Tokens for Sale");
        println!("2. Buy Tokens");
        println!("3. View Listings");
        println!("4. Exit Marketplace");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                println!("Enter your wallet address:");
                let mut wallet = String::new();
                io::stdin().read_line(&mut wallet).unwrap();
                let wallet = wallet.trim();
                if let Some(seller_wallet) = self.wallets.get_mut(wallet) {
                    println!("Enter price per token:");
                    let mut price = String::new();
                    io::stdin().read_line(&mut price).unwrap();
                    let price: u64 = price.trim().parse().unwrap_or(0);

                    println!("Enter number of tokens to list:");
                    let mut amount = String::new();
                    io::stdin().read_line(&mut amount).unwrap();
                    let amount: u64 = amount.trim().parse().unwrap_or(0);

                    if seller_wallet.balance_yt >= amount {
                        seller_wallet.balance_yt -= amount;
                        self.marketplace
                            .list_tokens(wallet.to_string(), price, amount);
                    } else {
                        println!("❌ Insufficient tokens to list.");
                    }
                } else {
                    println!("❌ Wallet not found.");
                }
            }
            "2" => {
                println!("Enter your wallet address:");
                let mut buyer_wallet_addr = String::new();
                io::stdin().read_line(&mut buyer_wallet_addr).unwrap();
                let buyer_wallet_addr = buyer_wallet_addr.trim();

                if self.wallets.contains_key(buyer_wallet_addr) {
                    self.marketplace.display_listings();

                    println!("Enter listing number to buy:");
                    let mut listing_index = String::new();
                    io::stdin().read_line(&mut listing_index).unwrap();
                    let listing_index: usize = listing_index.trim().parse().unwrap_or(0) - 1;

                    if listing_index < self.marketplace.listings.len() {
                        let listing = self.marketplace.listings[listing_index].clone();
                        let seller_id = listing.seller.clone();
                        let tokens_available = listing.tokens_available;
                        let price_per_token = listing.price_per_token;

                        println!("Enter amount of tokens to buy (available: {}):", tokens_available);
                        let mut amount = String::new();
                        io::stdin().read_line(&mut amount).unwrap();
                        let amount: u64 = amount.trim().parse().unwrap_or(0);

                        if amount <= tokens_available {
                            let total_price = price_per_token * amount;

                            let buyer_wallet = self.wallets.get_mut(buyer_wallet_addr).unwrap();
                            if buyer_wallet.balance_yuki >= total_price {
                                buyer_wallet.balance_yuki -= total_price;
                                buyer_wallet.balance_yt += amount;

                                if let Some(seller_wallet) = self.wallets.get_mut(&seller_id) {
                                    seller_wallet.balance_yuki += total_price;
                                }

                                self.marketplace.listings[listing_index].tokens_available -= amount;

                                println!("✅ Tokens purchased successfully!");

                                if self.marketplace.listings[listing_index].tokens_available == 0 {
                                    self.marketplace.listings.remove(listing_index);
                                }
                            } else {
                                println!("❌ Insufficient funds to buy tokens.");
                            }
                        } else {
                            println!("❌ Not enough tokens available in the listing.");
                        }
                    } else {
                        println!("❌ Invalid listing number.");
                    }
                } else {
                    println!("❌ Wallet not found.");
                }
            }
            "3" => self.marketplace.display_listings(),
            "4" => println!("Exiting marketplace..."),
            _ => println!("❌ Invalid choice."),
        }
    }

    fn create_wallet(&mut self, wallet: String) {
        self.wallets.insert(wallet.clone(), Wallet::new());
        self.save_wallets();
        println!("✅ Wallet created: {}", wallet);
    }

    fn save_wallets(&self) {
        let data = serde_json::to_string(&self.wallets).expect("Failed to serialize wallets.");
        fs::write(WALLET_FILE, data).expect("Failed to save wallets.");
    }

    fn load_wallets() -> HashMap<String, Wallet> {
        if let Ok(data) = fs::read_to_string(WALLET_FILE) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }
}

fn main() {
    let mut blockchain = Blockchain::new();
    println!("🌱 Riti Blockchain Initialized!");

    loop {
        println!("\n🌍 Options:");
        println!("1. Submit Task");
        println!("2. Marketplace");
        println!("3. View Blockchain");
        println!("4. Create Wallet");
        println!("5. View Wallets");
        println!("6. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                println!("Enter Wallet Address:");
                let mut wallet = String::new();
                io::stdin().read_line(&mut wallet).unwrap();

                println!("Enter Task Name:");
                let mut task = String::new();
                io::stdin().read_line(&mut task).unwrap();

                println!("Enter Proof Metadata:");
                let mut metadata = String::new();
                io::stdin().read_line(&mut metadata).unwrap();

                blockchain.add_block(wallet.trim(), task.trim().to_string(), metadata.trim().to_string());
            }
            "2" => blockchain.marketplace_menu(),
            "3" => blockchain.chain.iter().for_each(|block| println!("{:#?}", block)),
            "4" => {
                println!("Enter Wallet Name:");
                let mut wallet_name = String::new();
                io::stdin().read_line(&mut wallet_name).unwrap();
                blockchain.create_wallet(wallet_name.trim().to_string());
                println!("✅ Wallet created!");
            }
            "5" => {
                println!("All Wallets:");
                for (wallet, data) in &blockchain.wallets {
                    println!(
                        "Wallet: {} | Yuki: {} | YG: {} | YT: {}",
                        wallet, data.balance_yuki, data.balance_yg, data.balance_yt
                    );
                }
            }
            "6" => {
                println!("Exiting...");
                break;
            }
            _ => println!("❌ Invalid choice. Try again!"),
        }
    }
}
