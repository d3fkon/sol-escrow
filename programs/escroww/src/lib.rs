use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};

declare_id!("EQRkmHnvVF8BP4z4MQVtEZV4d2yLSvvTmMix1y3gasLK");

const VAULT_SEED: &[u8] = b"1011";

#[program]
pub mod escroww {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        buyer_pk: Pubkey,
        seller_pk: Pubkey,
        seed_bump: u8,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.buyer = buyer_pk;
        vault.seller = seller_pk;
        vault.bump = seed_bump;
        Ok(())
    }

    pub fn initiate_transaction(ctx: Context<InitiateTransaction>, lamports: u64, txn_bump: u8) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let tx = &mut ctx.accounts.transaction;

        tx.vault = vault.key();
        tx.lamports = lamports;
        tx.verifications = vec![false; 2];
        tx.bump = txn_bump;
        tx.is_executed = false;

        vault.num_transactions = vault.num_transactions + 1;

        msg!("Transaction initiated");

        let from_pk = vault.buyer.key();
        let from_account = ctx.accounts.buyer.to_account_info();
        let to_pk = vault.key();
        let to_account = ctx.accounts.vault.to_account_info();

        let ix = transfer(&from_pk, &to_pk, lamports);
        let _tx = invoke(&ix, &[from_account, to_account]);


        Ok(())
    }

    pub fn confirm_transaction(ctx: Context<ConfirmTransaction>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        let is_seller = vault.seller.key() == ctx.accounts.confirmation_by.key();
        // Basic check if the caller is either the buyer or the seller
        let txn = &mut ctx.accounts.transaction;

        if is_seller {
            txn.verifications[1] = true;
        } else {
            txn.verifications[0] = true;
        }

        Ok(())
    }

    pub fn execute_transaction(ctx: Context<ExecuteTrasaction>) -> Result<()>{
        let txn = &mut ctx.accounts.transaction;
        let vault = &mut ctx.accounts.vault;
       
        require!(txn.is_executed, ErrorCode::ConstraintTokenOwner);
        require!(txn.verifications[0] == false || txn.verifications[1] == false, ErrorCode::ConstraintAssociated);

        let from = vault.to_account_info();
        let to = ctx.accounts.seller.to_account_info();


        // let ix = transfer(&from_pk, &to_pk, txn.lamports);
        // let _tx = invoke(&ix, &[from, to]);

        **from.try_borrow_mut_lamports()? -= txn.lamports;
        **to.try_borrow_mut_lamports()? += txn.lamports;

        txn.is_executed = true;
        
       Ok(()) 
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = buyer, seeds= [VAULT_SEED.as_ref()], bump, space = 1000)]
    vault: Account<'info, Vault>,
    #[account(mut)]
    buyer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitiateTransaction<'info> {
    #[account(mut, seeds = [VAULT_SEED.as_ref()], bump)]
    vault: Account<'info, Vault>,
    #[account(init, 
        payer = buyer,
        seeds = [vault.num_transactions.to_le_bytes().as_ref(), VAULT_SEED.as_ref()],
        bump,
        space = 1000 
    )]
    transaction: Account<'info, Transaction>,
    #[account(mut)]
    buyer: Signer<'info>,
    system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ConfirmTransaction<'info> {
    #[account(mut, seeds = [VAULT_SEED.as_ref()], bump)]
    vault: Account<'info, Vault>,
    #[account(mut)]
    transaction: Account<'info, Transaction>,
    #[account(constraint = confirmation_by.key() == vault.buyer.key() || confirmation_by.key() == vault.seller.key())]
    confirmation_by: Signer<'info>
}

#[derive(Accounts)]
pub struct ExecuteTrasaction <'info> {
    #[account(mut, seeds = [VAULT_SEED.as_ref()], bump)]
    vault: Account<'info, Vault>,
    #[account(mut)]
    transaction: Account<'info, Transaction>,
    #[account(constraint = execution_by.key() == vault.buyer.key() || execution_by.key() == vault.seller.key())]
    // #[account(mut)]
    execution_by: Signer<'info>,
    /// CHECK: Chill as the seller only recieves the amount
    #[account(mut, constraint = seller.key() == vault.seller.key())]
    seller: AccountInfo<'info>
}

#[account]
pub struct Vault {
    buyer: Pubkey,
    seller: Pubkey,
    bump: u8,
    num_transactions: u64,
}

#[account]
pub struct Transaction {
    // The vault this transaction belongs to
    vault: Pubkey,
    // Bump seed for the account
    bump: u8,
    // Checks if the txn is complete
    is_executed: bool,
    // Who all have verified the transaction [Buyer, Seller]
    verifications: Vec<bool>,
    // Lamports to transfer to the seller
    lamports: u64,
}

