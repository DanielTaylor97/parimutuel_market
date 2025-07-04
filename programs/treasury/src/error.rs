use anchor_lang::error_code;

#[error_code]
pub enum InitError {

    #[msg("The supplied token mint is not the expected one")]
    WrongTokenMint = 0,

}

#[error_code]
pub enum TransactionError {

    #[msg("The signer provided is not the authority of the treasury")]
    SignerNotAuthority = 100,

    #[msg("The ATA supplied is not that of the Treasury")]
    WrongATA = 101,

}
