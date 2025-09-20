use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub fee: u16, // 100 1.00 2000 20.00
    pub bump: u8,
    pub lp_bump: u8,
}