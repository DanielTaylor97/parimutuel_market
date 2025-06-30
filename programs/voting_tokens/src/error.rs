use anchor_lang::error_code;

#[error_code]
pub enum InitError {

    #[msg("Wrong token name given at initialisation")]
    WrongName,

    #[msg("Wrong token symbol given at initialisation")]
    WrongSymbol,

    #[msg("Wrong token uri given at initialisation")]
    WrongUri,

    #[msg("Wrong number of token decimals given at initialisation")]
    WrongDecimals,

}
