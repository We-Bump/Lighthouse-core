use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Registeration is close")]
    RegisterationClose {},

    #[error("Collection Exists")]
    CollectionExists {},

    #[error("LightHouse contract is not set to be MINTER_ROLE")]
    NotMinter {},

    #[error("Supply lower than minted")]
    SupplyLowerThanMinted {},

    #[error("Invalid Chain")]
    InvalidChain {},

    #[error("Sold out")]
    SoldOut {},

    #[error("Invalid Mint Group")]
    InvalidMintGroup {},

    #[error("Group Not Open to Mint")]
    GroupNotOpenToMint {},

    #[error("Invalid Merkle Proof")]
    InvalidMerkleProof  {},

    #[error("Cannot Mint More Than Max Tokens")]
    MaxTokensMinted {},

    #[error("Reserved Supply Ran Out")]
    ReservedSupplyRanOut {},

    #[error("Invalid Funds")]
    InvalidFunds {},

    #[error("Invalid Token")]
    PartnerNotFound {},

    #[error("InvalidFeePercent")]
    InvalidFeePercent,

    #[error("Insufficient Balance for Token Gate")]
    InsufficientBalanceForTokenGate {},

    #[error("Invalid Gate")]
    InvalidGate {},

    #[error("Invalid Gate Args")]
    InvalidGateArgs {},

    #[error("Invalid Owner of NFT")]
    InvalidOwnerOfNft {},

    #[error("AlreadyMintedForGatedTokenId")]
    AlreadyMintedForGatedTokenId {},

    #[error("Not implemented")]
    NotImplemented {},

    #[error("Not available for EVM")]
    NotAvailableForEVM {},

    #[error("Address not associated")]
    NotAssociatedAddress {},

    #[error("Invalid Token Id")]
    InvalidTokenId {},
}
