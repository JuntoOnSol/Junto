use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use solana_program::clock::Clock;

pub mod instructions;
use instructions::*;

#[program]
pub mod junto_dao {
    use super::*;

    pub fn initialize(ctx: Context<InitializeDao>, authority: Pubkey, min_tokens: u64, max_voting_duration: i64) -> Result<()> {
        let dao_state = &mut ctx.accounts.dao_state;
        dao_state.authority = authority;
        dao_state.governance_mint = ctx.accounts.governance_mint.key();
        dao_state.min_tokens_to_propose = min_tokens;
        dao_state.proposal_count = 0;
        dao_state.max_voting_duration = max_voting_duration;
        Ok(())
    }

    pub fn create_proposal(ctx: Context<CreateProposal>, title: String, description: String) -> Result<()> {
        create_proposal(ctx, title, description)
    }

    pub fn cast_vote(ctx: Context<CastVote>, proposal_id: u64, vote_in_favor: bool) -> Result<()> {
        cast_vote(ctx, proposal_id, vote_in_favor)
    }

    pub fn finalize_proposal(ctx: Context<FinalizeProposal>, proposal_id: u64) -> Result<()> {
        finalize_proposal(ctx, proposal_id)
    }
}
