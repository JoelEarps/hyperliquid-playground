use ethers::abi::ethereum_types::H128;

#[derive(Debug, Clone)]
pub struct RequiredTokenInfo {
    pub name: String,
    pub index: usize,
    pub token_id: H128,
}