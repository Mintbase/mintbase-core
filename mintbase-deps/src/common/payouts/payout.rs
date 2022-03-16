use std::collections::HashMap;

use near_sdk::json_types::U128;
use near_sdk::serde::{
    Deserialize,
    Serialize,
};
use near_sdk::{
    AccountId,
    Balance,
};

use crate::common::{
    MultipliedSafeFraction,
    Royalty,
    SafeFraction,
    SplitOwners,
};
use crate::constants::MAX_LEN_PAYOUT;

/// Whom to pay. Generated from `OwnershipFractions`.
#[derive(Serialize, Deserialize)]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

/// Take the Royalty and SplitOwner information for a token, and return a Vector
/// of proportional payouts.
#[derive(Serialize, Deserialize)]
pub struct OwnershipFractions {
    pub fractions: HashMap<AccountId, MultipliedSafeFraction>,
}

impl OwnershipFractions {
    /// Generate a mapping of who receives what from a token's Royalty,
    /// SplitOwners, and normal owner data.
    pub fn new(
        owner_id: &str,
        royalty: &Option<Royalty>,
        split_owners: &Option<SplitOwners>,
    ) -> Self {
        let roy_len = royalty.as_ref().map(|r| r.split_between.len()).unwrap_or(0);
        let split_len = split_owners
            .as_ref()
            .map(|r| r.split_between.len())
            .unwrap_or(1);
        assert!((roy_len + split_len) as u32 <= MAX_LEN_PAYOUT);

        let mut payout: HashMap<AccountId, MultipliedSafeFraction> = Default::default();
        let percentage_not_taken_by_royalty = match royalty {
            Some(royalty) => {
                let (split_between, percentage) =
                    (royalty.split_between.clone(), royalty.percentage);
                split_between.iter().for_each(|(receiver, &rel_perc)| {
                    let abs_perc: MultipliedSafeFraction = percentage * rel_perc;
                    payout.insert(receiver.to_string().parse().unwrap(), abs_perc);
                });
                SafeFraction::new(10_000 - percentage.numerator)
            },
            None => SafeFraction::new(10_000u32),
        };

        match split_owners {
            Some(ref split_owners) => {
                split_owners
                    .split_between
                    .iter()
                    .for_each(|(receiver, &rel_perc)| {
                        let abs_perc: MultipliedSafeFraction =
                            percentage_not_taken_by_royalty * rel_perc;
                        // If an account is already in the payout map, update their take.
                        if let Some(&roy_perc) = payout.get(receiver) {
                            payout.insert(receiver.clone(), abs_perc + roy_perc);
                        } else {
                            payout.insert(receiver.clone(), abs_perc);
                        }
                    });
            },
            None => {
                if let Some(&roy_perc) = payout.get(&AccountId::new_unchecked(owner_id.to_string()))
                {
                    payout.insert(
                        owner_id.to_string().parse().unwrap(),
                        MultipliedSafeFraction::from(percentage_not_taken_by_royalty) + roy_perc,
                    );
                } else {
                    payout.insert(
                        owner_id.to_string().parse().unwrap(),
                        MultipliedSafeFraction::from(percentage_not_taken_by_royalty),
                    );
                }
            },
        };
        Self { fractions: payout }
    }

    pub fn into_payout(
        self,
        balance: Balance,
    ) -> Payout {
        Payout {
            payout: self
                .fractions
                .into_iter()
                .map(|(k, v)| (k, v.multiply_balance(balance).into()))
                .collect(),
        }
    }
}
