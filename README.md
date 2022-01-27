### Demo flow

```sh
// Bob mint 1 token
near call $ADD mint '{"id": "0", "metadata": { "title": "Olympus Mons", "description": "Tallest mountain in charted solar system", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 1}}' --accountId bob_auction.testnet --amount 0.2

// Bob create an auction - duration 1 minute
near call $ADD create_auction '{"art_id": "0", "start_price": 100000000000000000000000000, "duration": 60}' --accountId bob_auction.testnet --amount 1.5

// Alice bid 2 Near
near call $ADD bid '{"auction_id": 0}' --accountId alice_aunction.testnet --amount 2

// Carol bid 3 Near
near call $ADD bid '{"auction_id": 0}' --accountId carol_auction.testnet --amount 4

// Alice try to claim NFT 
near call $ADD claim_nft '{"auction_id": 0}' --accountId alice_aunction.testnet --depositYocto 1 

// Carol claim his NFT 
near call $ADD claim_nft '{"auction_id": 0}' --accountId carol_auction.testnet --depositYocto 1

// Bob claim 3 Near from Carol
near call $ADD claim_near '{"auction_id": 0}' --accountId bob_auction.testnet 

