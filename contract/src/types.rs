use super::*;

#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction {
    pub owner: AccountId,
    pub auction_id: usize,
    pub auction_token: TokenId,
    pub start_price: Balance,
    pub start_time: u64,
    pub end_time: u64,
    pub current_price: Balance,
    pub winner: AccountId,
    pub is_near_claimed: bool,
    pub is_nft_claimed: bool,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    NonFungibleToken,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AuctionSystem {
    pub owner: AccountId,
    pub tokens: NonFungibleToken,
    pub num_auctions: usize,
    pub auctions_mapping: LookupMap<usize, Auction>,
    pub auctioned_tokens: UnorderedSet<TokenId>,
}

