use solana_program::program::{invoke_signed, invoke};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{Sysvar, rent::Rent},
};
use solana_program::borsh::try_from_slice_unchecked;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use spl_token;
use spl_associated_token_account;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

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

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
struct PriceData{
    price: u64,
}

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("author: Rusty 0ne <4fun.and.job.offers@gmail.com>");

    let accounts_iter = &mut accounts.iter();
    let instruction: SellInstruction = try_from_slice_unchecked(instruction_data).unwrap();

    let admin = "Ek6Vqf4cCq6zXAp9TwSqeAbQXm8Eo3Y8DV7abbJYntwv".parse::<Pubkey>().unwrap();
    let treasury = "D4bMaJQG5EhGExPqT1tYR8Vyd39Gf2tzN4JoxAy42jgP".parse::<Pubkey>().unwrap();
    let mint = "5CZJ7e4uUWCogW2z7rvcE2yur6yE7Z7kcGYrFqLLXSL9".parse::<Pubkey>().unwrap();

    match instruction{
        SellInstruction::SetPrice{price}=>{
            let payer = next_account_info(accounts_iter)?;
            let price_account_info = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            let (price_address, price_bump) = Pubkey::find_program_address(&["price".as_bytes()], &program_id);
            
            let price_data = PriceData{price};
            
            if *payer.key!=admin{
                msg!("Unauthorized access");
                return Err(ProgramError::IncorrectProgramId);
            }
            if *price_account_info.key!=price_address{
                msg!("Wrong price account");
                return Err(ProgramError::IncorrectProgramId);
            }

            if !payer.is_signer{
                msg!("Admin didn't sign transaction");
                return Err(ProgramError::IncorrectProgramId);
            }

            let size = 8;
            if price_account_info.owner != program_id{
                let required_lamports = rent
                    .minimum_balance(size)
                    .max(1)
                    .saturating_sub(price_account_info.lamports());

                invoke(
                    &system_instruction::transfer(payer.key, price_account_info.key, required_lamports),
                    &[
                        payer.clone(),
                        price_account_info.clone(),
                        sys_info.clone(),
                    ],
                )?;
                invoke_signed(
                    &system_instruction::allocate(price_account_info.key, size as u64),
                    &[
                        price_account_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[b"price", &[price_bump]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(price_account_info.key, program_id),
                    &[
                        price_account_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[b"price", &[price_bump]]],
                )?;
            }

            price_data.serialize(&mut &mut price_account_info.data.borrow_mut()[..])?;

        }
        SellInstruction::Withdraw{amount}=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let vault_info = next_account_info(accounts_iter)?;

            let vault_mint_info = next_account_info(accounts_iter)?;
            let mint_info = next_account_info(accounts_iter)?;
            let token_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let assoc_acccount_info = next_account_info(accounts_iter)?;

            let treasury_info = next_account_info(accounts_iter)?;
            let payer_mint_holder_info = next_account_info(accounts_iter)?;

            if *payer.key!=admin{
                msg!("Unauthorized access");
                return Err(ProgramError::IncorrectProgramId);
            }

            if *treasury_info.key!=treasury{
                msg!("Wrong treasury");
                return Err(ProgramError::IncorrectProgramId);
            }

            if !payer.is_signer{
                msg!("Admin didn't sign transaction");
                return Err(ProgramError::IncorrectProgramId);
            }

            if *mint_info.key!=mint{
                msg!("Wrong treasury");
                return Err(ProgramError::IncorrectProgramId);
            }


            let payer_mint_holder = spl_associated_token_account::get_associated_token_address(payer.key, &mint);
            if *payer_mint_holder_info.key!=payer_mint_holder{
                msg!("Wrong payer_mint_holder");
                return Err(ProgramError::IncorrectProgramId);
            }

            let (_vault, vault_bump) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

            if payer_mint_holder_info.owner != token_info.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        payer.key,
                        mint_info.key,
                    ),
                    &[
                        payer.clone(), 
                        payer_mint_holder_info.clone(), 
                        payer.clone(),
                        mint_info.clone(),
                        system_program.clone(),
                        token_info.clone(),
                        rent_info.clone(),
                        assoc_acccount_info.clone(),
                    ],
                    
                )?;
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_mint_info.key,
                    payer_mint_holder_info.key,
                    vault_info.key,
                    &[],
                    amount,
                )?,
                &[
                    vault_mint_info.clone(),
                    payer_mint_holder_info.clone(),
                    vault_info.clone(), 
                    token_info.clone()
                ],
                &[&[b"vault", &[vault_bump]]],
            )?;
        },

        SellInstruction::Generate=>{
            let (vault_pda, vault_bump_seed) =
                Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let pda = next_account_info(accounts_iter)?;
            
            if pda.key!=&vault_pda{
                msg!("Wrong account generated by client");
                return Err(ProgramError::IncorrectProgramId);
            }

            if pda.owner==program_id{
                msg!("Account already assigned");
                return Err(ProgramError::IncorrectProgramId);
            }

            invoke(
                &system_instruction::transfer(payer.key, &vault_pda, 2_282_880),
                &[
                    payer.clone(),
                    pda.clone(),
                    system_program.clone(),
                ],
            )?;

            invoke_signed(
                &system_instruction::assign(&vault_pda, program_id),
                &[
                    pda.clone(),
                    system_program.clone(),
                ],
                &[&[b"vault", &[vault_bump_seed]]],
            )?;
            msg!("Address generated: {:?}", vault_pda);
        },

        SellInstruction::Buy{amount}=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let vault_info = next_account_info(accounts_iter)?;

            let vault_mint_info = next_account_info(accounts_iter)?;
            let mint_info = next_account_info(accounts_iter)?;
            let token_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let assoc_acccount_info = next_account_info(accounts_iter)?;

            let treasury_info = next_account_info(accounts_iter)?;
            let payer_mint_holder_info = next_account_info(accounts_iter)?;
            let price_data_info = next_account_info(accounts_iter)?;

            let (price_address, _price_bump) = Pubkey::find_program_address(&["price".as_bytes()], &program_id);

            if *price_data_info.key!=price_address{
                msg!("Wrong price_data_info");
                return Err(ProgramError::IncorrectProgramId);
            }

            if *treasury_info.key!=treasury{
                msg!("Wrong treasury");
                return Err(ProgramError::IncorrectProgramId);
            }

            if !payer.is_signer{
                msg!("Payer didn't sign transaction");
                return Err(ProgramError::IncorrectProgramId);
            }

            if *mint_info.key!=mint{
                msg!("Wrong treasury");
                return Err(ProgramError::IncorrectProgramId);
            }

            let payer_mint_holder = spl_associated_token_account::get_associated_token_address(payer.key, &mint);
            if *payer_mint_holder_info.key!=payer_mint_holder{
                msg!("Wrong payer_mint_holder");
                return Err(ProgramError::IncorrectProgramId);
            }
            let price_data = if let Ok(data) = PriceData::try_from_slice(&price_data_info.data.borrow()){
                data
            } else {
                msg!("Price didn't set");
                return Err(ProgramError::IncorrectProgramId);
            };

            let (_vault, vault_bump) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

            invoke(
                &system_instruction::transfer(payer.key, &treasury, price_data.price*amount),
                &[
                    payer.clone(),
                    treasury_info.clone(),
                    system_program.clone(),
                ],
            )?;

            if payer_mint_holder_info.owner != token_info.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        payer.key,
                        mint_info.key,
                    ),
                    &[
                        payer.clone(), 
                        payer_mint_holder_info.clone(), 
                        payer.clone(),
                        mint_info.clone(),
                        system_program.clone(),
                        token_info.clone(),
                        rent_info.clone(),
                        assoc_acccount_info.clone(),
                    ],
                    
                )?;
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_mint_info.key,
                    payer_mint_holder_info.key,
                    vault_info.key,
                    &[],
                    amount,
                )?,
                &[
                    vault_mint_info.clone(),
                    payer_mint_holder_info.clone(),
                    vault_info.clone(), 
                    token_info.clone()
                ],
                &[&[b"vault", &[vault_bump]]],
            )?;
        }

    };
    msg!("Success");
    Ok(())
}

