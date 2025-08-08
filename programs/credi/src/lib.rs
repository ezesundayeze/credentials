// Import necessary modules from the Anchor framework and SPL token program.
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

// Specifically import the mint_to function for issuing tokens.
use anchor_spl::token_interface::mint_to;

// Declare the unique identifier for this program.
declare_id!("84DES1yt9xCXdQf5j9iRphCT1cGYm6Y9vdFsZYkqmfSi");

// Define the main program module.
#[program]
pub mod credential_system {
    use super::*;

    /// Initializes a new mint for the credentials with NonTransferable extension.
    /// This function sets up the mint account but does not issue any tokens.
    pub fn initialize_credential_mint(ctx: Context<InitializeCredentialMint>) -> Result<()> {
        // Initialize the NonTransferable extension. This makes the tokens non-transferable.
        non_transferable_mint_initialize(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022_extensions::NonTransferableMintInitialize {
                mint: ctx.accounts.mint.to_account_info(),
                token_program_id: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // Initialize the mint itself, setting decimals to 0 and defining authorities.
        anchor_spl::token_interface::initialize_mint2(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::InitializeMint2 {
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            0, // Decimals are set to 0 because credentials are whole units and cannot be fractional.
            &ctx.accounts.mint.key(),       // The mint authority is the program-derived address (PDA) itself.
            Some(&ctx.accounts.mint.key()), // The freeze authority is also the PDA.
        )?;

        Ok(())
    }

    /// Issues a single credential (1 token) to a specified recipient.
    /// This function can only be called by the mint authority.
    pub fn issue_credential(ctx: Context<IssueCredential>) -> Result<()> {
        // Mint one token to the recipient's associated token account.
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.recipient_ata.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1, // Mint exactly one token.
        )?;

        Ok(())
    }
}

/// Defines the accounts required for the `initialize_credential_mint` instruction.
#[derive(Accounts)]
pub struct InitializeCredentialMint<'info> {
    // The mint account to be initialized as a Program Derived Address (PDA).
    #[account(
        init,
        payer = payer,
        // The space allocation for the account's data:
        // 8 bytes: for the account discriminator, a unique identifier for the account type in Anchor.
        // 82 bytes: the standard size of a SPL Token Mint account.
        // 8 bytes: additional space reserved for Token-2022 extensions (e.g., NonTransferable).
        space = 8 + 82 + 8,
        owner = token_program.key(),
        // Defines the seeds for the Program Derived Address (PDA).
        // A PDA is an address owned and controlled by the program itself, not a private key.
        // Using "mint" as a seed creates a single, canonical address for our credential mint,
        // making it easy to find and use from client applications.
        seeds = [b"mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    // The account paying for the transaction and rent.
    #[account(mut)]
    pub payer: Signer<'info>,

    // System program, required for creating accounts.
    pub system_program: Program<'info, System>,
    // The SPL token program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Defines the accounts required for the `issue_credential` instruction.
#[derive(Accounts)]
pub struct IssueCredential<'info> {
    // The mint account, must be mutable.
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        constraint = mint.mint_authority.unwrap() == authority.key() // Ensure the authority is the mint authority.
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    // The authority signing the transaction (must be the mint authority).
    #[account(mut)]
    pub authority: Signer<'info>,

    // The recipient's associated token account, created if it doesn't exist.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    // The recipient of the credential.
    pub recipient: Signer<'info>,

    // The SPL token program.
    pub token_program: Interface<'info, TokenInterface>,
    // The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
    // The system program.
    pub system_program: Program<'info, System>,
}