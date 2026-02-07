use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("Bridge is paused")]
    Paused,

    #[msg("Invalid validator signatures")]
    InvalidSignatures,

    #[msg("Replay detected")]
    Replay,

    #[msg("Message expired")]
    Expired,

    #[msg("Threshold not met")]
    ThresholdNotMet,

    #[msg("Supply invariant violated")]
    SupplyInvariant,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Overflow occurred")]
    Overflow,
}
