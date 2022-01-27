use std::convert::TryFrom;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
};

pub use crate::types::*;
pub mod types;

const MINT_NFT: Balance = 2_000_000_000_000_000_000_000_00; // 0.2 NEAR
const JOIN_AS_AN_AUCTION_CREATOR: Balance = 1_500_000_000_000_000_000_000_000; // 1.5 NEAR
const ENROLL: Balance = 1_000_000_000_000_000_000_000_00; // 0.1 NEAR

#[near_bindgen]
impl AuctionSystem {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: env::predecessor_account_id(),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            num_auctions: 0,
            auctions_mapping: LookupMap::new(b"auctions_mapping".to_vec()), //
            auctioned_tokens: UnorderedSet::new(b"is_token_auctioned".to_vec()),
        }
    }
    #[payable]
    pub fn mint(
        &mut self,
        id: TokenId,
        metadata: Option<TokenMetadata>,
    ) -> Token {
        assert_eq!(
            env::attached_deposit(),
            MINT_NFT,
            "Require 0.2N to mint NFT"
        );

        let owner = ValidAccountId::try_from(env::predecessor_account_id()).unwrap();
        self.tokens.mint(id, owner, metadata)
    }

    pub fn get_art(
        self, 
        id: TokenId
        ) -> Option<Token> {
        self.tokens.nft_token(id)
    }

    pub fn get_auction(
        &self, 
        auction_id: usize
        ) -> Auction {
        self.auctions_mapping.get(&auction_id).unwrap()
    }

    #[payable]
    pub fn create_auction(
        &mut self,
        art_id: TokenId,
        start_price: Balance,
        duration: u64,
    ) -> Auction {
        let owner_id = self.tokens.owner_by_id.get(&art_id).unwrap();
        assert_eq!(
            owner_id,
            env::predecessor_account_id(),
            "You not own this Art NFT"
        );
        
        assert_eq!(
            self.auctioned_tokens.contains(&art_id),
            false,
            "Already auctioned"
        );

        assert_eq!(
            env::attached_deposit(),
            JOIN_AS_AN_AUCTION_CREATOR,
            "Require 1N to create an auction"
        );

        self.tokens.internal_transfer(
            &env::predecessor_account_id(),
            &env::current_account_id(),
            &art_id,
            None,
            None,
        );

        let auction = Auction {
            owner: owner_id,
            auction_id: self.num_auctions,
            auction_token: art_id.clone(),
            start_price,
            start_time: env::block_timestamp(),
            end_time: env::block_timestamp() + duration * 1_000_000_000,
            current_price: start_price,
            winner: String::new(),
            is_near_claimed: false,
            is_nft_claimed: false,
        };
        self.auctions_mapping.insert(&self.num_auctions, &auction);
        self.auctioned_tokens.insert(&art_id);
        self.num_auctions += 1;
        auction
    }

    #[payable]
    pub fn bid(
        &mut self, 
        auction_id: usize 
        ) {
        let mut auction = self.auctions_mapping.get(&auction_id).unwrap_or_else(|| {
            panic!("This auction does not exist");
        });

        assert_eq!(
            env::block_timestamp() > auction.start_time,
            true,
            "This auction has not started"
        );

        assert_eq!(
            env::block_timestamp() < auction.end_time,
            true,
            "This auction has already done"
        );

        assert_eq!(
            env::attached_deposit() > auction.current_price,
            true,
            "Price must be greater than current winner's price"
        );

        //Refund to previous winner
        if !(auction.winner == String::new()) {
            let old_winner = Promise::new(auction.winner);
            old_winner.transfer(auction.current_price - ENROLL);
        }

        auction.winner = env::predecessor_account_id();
        auction.current_price = env::attached_deposit();
        self.auctions_mapping.insert(&auction_id, &auction);
    }

    #[payable]
    pub fn claim_nft(
        &mut self, 
        auction_id: usize
        ) {
        let mut auction = self.auctions_mapping.get(&auction_id).unwrap_or_else(|| {
            panic!("This auction does not exist");
        });
        assert_eq!(
            env::block_timestamp() > auction.end_time,
            true,
            "The auction is not over yet"
        );
        assert_eq!(
            env::predecessor_account_id(),
            auction.winner,
            "You are not the winner"
        );
        assert_eq!(
            auction.clone().is_nft_claimed,
            false,
            "You has already claimed NFT"
        );

        self.tokens.internal_transfer_unguarded(
            &auction.auction_token,
            &env::current_account_id(),
            &auction.winner,
        );

        auction.is_nft_claimed = true;
        self.auctioned_tokens.remove(&auction.auction_token);
        self.auctions_mapping.insert(&auction_id, &auction);
    }

    #[payable]
    pub fn claim_near(
        &mut self, 
        auction_id: usize 
        ) {
        let mut auction = self.auctions_mapping.get(&auction_id).unwrap_or_else(|| {
            panic!("This auction does not exist");
        });
        assert_eq!(
            env::predecessor_account_id(),
            auction.owner,
            "You are not operator of this auction"
        );
        assert_eq!(
            env::block_timestamp() > auction.end_time,
            true,
            "The auction is not over yet"
        );
        assert_eq!(auction.is_near_claimed, false, "You has already claimed N");
        Promise::new(auction.clone().owner).transfer(auction.current_price);
        auction.is_near_claimed = true;
        self.auctions_mapping.insert(&auction_id, &auction);
    }

    #[payable]
    pub fn claim_back_nft(
        &mut self, 
        auction_id: usize 
        ) {
        let mut auction = self.auctions_mapping.get(&auction_id).unwrap_or_else(|| {
            panic!("This auction does not exist");
        });
        assert_eq!(
            env::predecessor_account_id(),
            auction.owner,
            "You are not operator of this auction"
        );
        assert_eq!(
            env::block_timestamp() > auction.end_time,
            true,
            "The auction is not over yet"
        );
        assert_eq!(auction.winner, String::new(), "The NFT has sold");
        self.tokens.internal_transfer_unguarded(
            &auction.auction_token,
            &env::current_account_id(),
            &auction.owner,
        );
        auction.is_nft_claimed = true;
        self.auctioned_tokens.remove(&auction.auction_token);
        self.auctions_mapping.insert(&auction_id, &auction);
    }
}

near_contract_standards::impl_non_fungible_token_approval!(AuctionSystem, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(AuctionSystem, tokens);

