use anchor_lang::error_code;

#[error_code]
pub enum InitError {

    #[msg("Wrong token name given at initialisation")]
    WrongName = 0,

    #[msg("Wrong token symbol given at initialisation")]
    WrongSymbol = 1,

    #[msg("Wrong token uri given at initialisation")]
    WrongUri = 2,

    #[msg("Wrong number of token decimals given at initialisation")]
    WrongDecimals = 3,

}
