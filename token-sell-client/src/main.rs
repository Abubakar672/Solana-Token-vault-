use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signer};
#[allow(unused_imports)]
use solana_sdk::signer::signers::Signers;
use solana_sdk::transaction::Transaction;
use solana_sdk::system_program;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use solana_sdk::commitment_config::CommitmentConfig;
use spl_token;
use spl_associated_token_account;
#[allow(unused_imports)]
use solana_sdk::signer::keypair::Keypair;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum SellInstruction{
    Generate,
    Buy{
        #[allow(dead_code)]
        amount:u64,
    },
    Withdraw{
        #[allow(dead_code)]
        amount:u64,
    },
    SetPrice{
        #[allow(dead_code)]
        price:u64,
    },
} 

fn main() {
    let matches = app_from_crate!()
        .subcommand(SubCommand::with_name("show_vault_address")
            .arg(Arg::with_name("program_id")
                .short("i")
                .long("contract_id")
                .required(true)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("generate_vault_address")
            .arg(Arg::with_name("program_id")
                .short("i")
                .long("contract_id")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("buy")
            .arg(Arg::with_name("program_id")
                .short("i")
                .long("contract_id")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .required(true)
                .takes_value(true)
            )

        )
        .subcommand(SubCommand::with_name("withdraw")
            .arg(Arg::with_name("program_id")
                .short("i")
                .long("contract_id")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .required(false)
                .takes_value(true)
            )

        )
        .subcommand(SubCommand::with_name("set_price")
            .arg(Arg::with_name("program_id")
                .short("i")
                .long("contract_id")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("price")
                .short("p")
                .long("price")
                .required(false)
                .takes_value(true)
            )
        )
        .get_matches();

    let treasury = "D4bMaJQG5EhGExPqT1tYR8Vyd39Gf2tzN4JoxAy42jgP".parse::<Pubkey>().unwrap();
    let mint = "5CZJ7e4uUWCogW2z7rvcE2yur6yE7Z7kcGYrFqLLXSL9".parse::<Pubkey>().unwrap();

    if let Some(matches) = matches.subcommand_matches("set_price") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let program_id = matches.value_of("program_id").unwrap().parse::<Pubkey>().expect("Wrong contract id format");
        
        let price = matches.value_of("price").unwrap().parse::<u64>().expect("price should be a number");
        let (price_address, _price_bump) = Pubkey::find_program_address(&["price".as_bytes()], &program_id);
        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &SellInstruction::SetPrice{price},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(price_address, false),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let hash  = client.send_transaction(&tx).expect("Transaction failed.");
        println!("Success. Check transaction: {:?}",hash);
        
    }

    if let Some(matches) = matches.subcommand_matches("withdraw") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let program_id = matches.value_of("program_id").unwrap().parse::<Pubkey>().expect("Wrong contract id format");
        
        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);
        let vault_mint_holder = spl_associated_token_account::get_associated_token_address(&vault_pda, &mint);
        let wallet_mint_holder = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &mint);

        let amount = if let Some(amount_str)=matches.value_of("amount"){
            amount_str.parse::<u64>().unwrap()
        } else {
            let token_balance_raw = client.get_token_account_balance(&vault_mint_holder).expect("Can't get token balance for creators's pda");
            token_balance_raw.amount.parse::<u64>().unwrap()
        };

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &SellInstruction::Withdraw{amount},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(vault_pda, false),

                AccountMeta::new(vault_mint_holder, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),

                AccountMeta::new(treasury, false),
                AccountMeta::new(wallet_mint_holder, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let hash  = client.send_transaction(&tx).expect("Transaction failed.");
        println!("Success. Check transaction: {:?}",hash);
     }

    if let Some(matches) = matches.subcommand_matches("buy") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let program_id = matches.value_of("program_id").unwrap().parse::<Pubkey>().expect("Wrong contract id format");
        let amount = matches.value_of("amount").unwrap().parse::<u64>().unwrap();



        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);
        let vault_mint_holder = spl_associated_token_account::get_associated_token_address(&vault_pda, &mint);
        let wallet_mint_holder = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &mint);
        let (price_address, _price_bump) = Pubkey::find_program_address(&["price".as_bytes()], &program_id);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &SellInstruction::Buy{amount},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(vault_pda, false),

                AccountMeta::new(vault_mint_holder, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),

                AccountMeta::new(treasury, false),
                AccountMeta::new(wallet_mint_holder, false),
                AccountMeta::new(price_address, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let hash  = client.send_transaction(&tx).expect("Transaction failed.");
        println!("Success. Check transaction: {:?}",hash);
     }

    if let Some(matches) = matches.subcommand_matches("show_vault_address") {
        let program_id = matches.value_of("program_id").unwrap().parse::<Pubkey>().expect("Wrong contract id format");
        let (pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);
        println!("Vault account: {:?}", pda);
    }

    if let Some(matches) = matches.subcommand_matches("generate_vault_address") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let program_id = matches.value_of("program_id").unwrap().parse::<Pubkey>().expect("Wrong contract id format");
    
        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &SellInstruction::Generate,
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(vault_pda, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        client.send_transaction(&tx).expect("Transaction failed.");
        println!("vault account generated: {:?}", vault_pda);
    }

}
