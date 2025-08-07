use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022_extensions::non_transferable_mint_initialize,
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
    },
};

use anchor_spl::token_interface::mint_to;

declare_id!("84DES1yt9xCXdQf5j9iRphCT1cGYm6Y9vdFsZYkqmfSi");

#[program]
pub mod credential_system {
    use super::*;

    /// Initializes a new mint for the credentials with NonTransferable extension.
    pub fn initialize_credential_mint(ctx: Context<InitializeCredentialMint>) -> Result<()> {
        // Initialize the NonTransferable extension using Anchor's helper
        non_transferable_mint_initialize(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022_extensions::NonTransferableMintInitialize {
                mint: ctx.accounts.mint.to_account_info(),
                token_program_id: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // Then initialize the mint
        anchor_spl::token_interface::initialize_mint2(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::InitializeMint2 {
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            0,                              // decimals
            &ctx.accounts.mint.key(),       // mint authority is the PDA itself
            Some(&ctx.accounts.mint.key()), // freeze authority is the PDA
        )?;

        Ok(())
    }

    /// Issues a single credential (1 token) to a recipient.
    pub fn issue_credential(ctx: Context<IssueCredential>) -> Result<()> {
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.recipient_ata.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1, // mint 1 token
        )?;

        Ok(())
    }
}

/// Accounts for the `initialize_credential_mint` instruction.
#[derive(Accounts)]
pub struct InitializeCredentialMint<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 82 + 8, // discriminator + mint + extension space
        owner = token_program.key(),
        seeds = [b"mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// Accounts for the `issue_credential` instruction.
#[derive(Accounts)]
pub struct IssueCredential<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        constraint = mint.mint_authority.unwrap() == authority.key()
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    pub recipient: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}