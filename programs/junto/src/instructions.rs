use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, system_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// -------------------------------------------------------------
// DATA STRUCTURES
// -------------------------------------------------------------

/// Stores global DAO parameters such as required stake for proposals,
/// managed treasury accounts, and a counter for unique proposal IDs.
#[account]
pub struct DaoState {
    /// The authority responsible for validating proposals or performing admin tasks.
    pub authority: Pubkey,

    /// The governance token used for voting and proposal creation.
    pub governance_mint: Pubkey,

    /// Minimum number of staked tokens required to create a proposal.
    pub min_tokens_to_propose: u64,

    /// Counter to ensure unique proposal IDs.
    pub proposal_count: u64,

    /// Maximum voting duration in seconds.
    pub max_voting_duration: i64,

    /// Reserved space for future expansion.
    pub _reserved: [u8; 64],
}

/// Structure representing an on-chain proposal.
#[account]
pub struct Proposal {
    pub proposal_id: u64,
    pub dao_state: Pubkey,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub voting_deadline: i64,
    pub votes_for: u64,
    pub votes_against: u64,
    /// 0 = pending, 1 = approved, 2 = rejected.
    pub final_outcome: u8,
}

// -------------------------------------------------------------
// EVENTS & ERRORS
// -------------------------------------------------------------

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub created_at: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_in_favor: bool,
    pub voting_power: u64,
}

#[event]
pub struct ProposalFinalized {
    pub proposal_id: u64,
    pub final_outcome: u8,
}

#[error_code]
pub enum DaoError {
    #[msg("Insufficient tokens to create a proposal.")]
    NotEnoughTokensToPropose,

    #[msg("Proposal is not active or voting deadline has passed.")]
    ProposalNotActive,

    #[msg("Voting is closed.")]
    VotingClosed,

    #[msg("No voting power (insufficient staked tokens).")]
    NoVotingPower,

    #[msg("Proposal title is too short or missing.")]
    InvalidProposalTitle,

    #[msg("Proposal has already been finalized.")]
    AlreadyFinalized,
}

// -------------------------------------------------------------
// ANCHOR CONTEXT STRUCTS
// -------------------------------------------------------------

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub dao_state: Account<'info, DaoState>,
    #[account(
        init,
        payer = signer,
        space = 8 + std::mem::size_of::<Proposal>(),
        seeds = [b"proposal", dao_state.proposal_count.checked_add(1).unwrap().to_le_bytes().as_ref()],
        bump
    )]
    pub proposal_account: Account<'info, Proposal>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub dao_state: Account<'info, DaoState>,
    #[account(mut)]
    pub proposal_account: Account<'info, Proposal>,
    #[account(
        constraint = voter_token_account.mint == dao_state.governance_mint,
        constraint = voter_token_account.owner == signer.key()
    )]
    pub voter_token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub dao_state: Account<'info, DaoState>,
    #[account(mut)]
    pub proposal_account: Account<'info, Proposal>,
}

// -------------------------------------------------------------
// PROGRAM LOGIC (INSTRUCTIONS)
// -------------------------------------------------------------

pub fn create_proposal(ctx: Context<CreateProposal>, title: String, description: String) -> Result<()> {
    let dao_state = &mut ctx.accounts.dao_state;
    let signer = &ctx.accounts.signer;
    let proposal_account = &mut ctx.accounts.proposal_account;
    
    require!(title.len() > 2, DaoError::InvalidProposalTitle);
    
    dao_state.proposal_count = dao_state.proposal_count.checked_add(1).unwrap();
    proposal_account.proposal_id = dao_state.proposal_count;
    proposal_account.dao_state = dao_state.key();
    proposal_account.proposer = signer.key();
    proposal_account.title = title.clone();
    proposal_account.description = description.clone();
    proposal_account.created_at = clock::Clock::get().unwrap().unix_timestamp;
    proposal_account.voting_deadline = proposal_account.created_at.checked_add(dao_state.max_voting_duration).unwrap();
    proposal_account.votes_for = 0;
    proposal_account.votes_against = 0;
    proposal_account.final_outcome = 0;
    
    emit!(ProposalCreated {
        proposal_id: proposal_account.proposal_id,
        proposer: signer.key(),
        title,
        created_at: proposal_account.created_at,
    });

    Ok(())
}

pub fn cast_vote(ctx: Context<CastVote>, proposal_id: u64, vote_in_favor: bool) -> Result<()> {
    let proposal_account = &mut ctx.accounts.proposal_account;
    let voter_token_account = &ctx.accounts.voter_token_account;
    let now = clock::Clock::get().unwrap().unix_timestamp;
    
    require!(proposal_id == proposal_account.proposal_id && proposal_account.final_outcome == 0, DaoError::ProposalNotActive);
    require!(now < proposal_account.voting_deadline, DaoError::VotingClosed);
    require!(voter_token_account.amount > 0, DaoError::NoVotingPower);
    
    if vote_in_favor {
        proposal_account.votes_for = proposal_account.votes_for.checked_add(voter_token_account.amount).unwrap();
    } else {
        proposal_account.votes_against = proposal_account.votes_against.checked_add(voter_token_account.amount).unwrap();
    }
    
    emit!(VoteCast { proposal_id, voter: ctx.accounts.signer.key(), vote_in_favor, voting_power: voter_token_account.amount });

    Ok(())
}
