## All functions documented

- [ ] `approvals.rs`
- [ ] `burning.rs`
- [x] `core.rs`
- [ ] `enumeration.rs`
- [x] `lib.rs`
- [x] `metadata.rs`
- [ ] `minting.rs`
- [x] `ownership.rs`
- [x] `payout.rs`

## Accessible storage variables

- [x] `minters`
- [x] `metadata`
- [x] `token_metadata`
- [ ] `token_royalty` -> impossible due to `LookupMap`
- [x] `tokens`
- [ ] `tokens_per_owner` -> impossible due to `LookupMap`
- [ ] `composeables` -> TODO: deprecate
- [ ] `tokens_minted`
- [ ] `tokens_burned`
- [ ] `num_approved`
- [x] `owner_id`
- [x] `storage_costs`
- [x] `allow_moves` -> TODO: deprecate

## Notes

- Because multiply is not implemented contract-side, all the `copies` are
  basically wrong, unnecessary amounts of storage are occupied, and methods are
  wrong, e.g.:
  - `get_token_remaining_copies`
