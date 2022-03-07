use std::collections::HashMap;
use std::convert::{
    TryFrom,
    TryInto,
};
use std::fmt;

#[cfg(feature = "wasm")]
pub use near_sdk::{
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
    collections::*,
    json_types::*,
    *,
};

use crate::*;

impl NearTime {
    pub fn is_before_timeout(&self) -> bool {
        now().0 < self.0
    }

    pub fn new(span: TimeUnit) -> Self {
        match span {
            TimeUnit::Hours(n) => Self::now_plus_n_hours(n),
        }
    }

    fn now_plus_n_hours(n: u64) -> Self {
        assert!(n > 0);
        assert!(
            n < 70_000,
            "maximum argument for hours is 70,000 (~8 years)"
        );
        let now = env::block_timestamp();
        let hour_ns = 10u64.pow(9) * 3600;
        Self(now + n * hour_ns)
    }
}

impl StorageCostsMarket {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            list: storage_price_per_byte * LIST_STORAGE as u128,
        }
    }
}

impl TokenOffer {
    /// Timeout is in days.
    pub fn new(
        price: u128,
        timeout: TimeUnit,
        id: u64,
    ) -> Self {
        let timeout = NearTime::new(timeout);
        Self {
            id,
            price,
            from: env::predecessor_account_id(),
            timestamp: now(),
            timeout,
        }
    }

    /// An offer is active if it has yet to timeout.
    pub fn is_active(&self) -> bool {
        self.timeout.is_before_timeout()
    }
}

impl TokenListing {
    /// Check that the given `account_id` is valid before instantiating a
    /// `Token`. Note that all input validation for `Token` functions should
    /// be performed at the `Marketplace` level.
    pub fn new(
        owner_id: AccountId,
        store_id: AccountId,
        id: u64,
        approval_id: u64,
        autotransfer: bool,
        asking_price: U128,
    ) -> Self {
        Self {
            id,
            owner_id,
            store_id,
            approval_id,
            autotransfer,
            asking_price,
            current_offer: None,
            num_offers: 0,
            locked: false,
        }
    }

    /// Unique identifier of the Token.
    pub fn get_token_key(&self) -> TokenKey {
        TokenKey::new(self.id, self.store_id.to_string().try_into().unwrap())
    }

    /// Unique identifier of the Token, which is also unique across
    /// relistings of the Token.
    pub fn get_list_id(&self) -> String {
        format!("{}:{}:{}", self.id, self.approval_id, self.store_id)
    }

    pub fn assert_not_locked(&self) {
        assert!(!self.locked);
    }
}

impl StorageCosts {
    pub fn new(storage_price_per_byte: u128) -> Self {
        Self {
            storage_price_per_byte,
            common: storage_price_per_byte * 80_u64 as u128,
            token: storage_price_per_byte * 360_u64 as u128,
        }
    }
}

impl fmt::Display for NftEventError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

////////////////
// Core Logic //
////////////////

/// Stable
impl Royalty {
    /// Validates all arguments. Addresses must be valid and percentages must be
    /// within accepted values. Hashmap percentages must add to 10000.
    pub fn new(royalty_args: RoyaltyArgs) -> Self {
        assert!(!royalty_args.split_between.is_empty());
        let percentage = royalty_args.percentage;
        let split_between = royalty_args.split_between;

        assert!(
            percentage <= ROYALTY_UPPER_LIMIT,
            "percentage: {} must be <= 5000",
            percentage
        );
        assert!(percentage > 0, "percentage cannot be zero");
        assert!(!split_between.is_empty(), "royalty mapping is empty");

        let mut sum: u32 = 0;
        let split_between: SplitBetween = split_between
            .into_iter()
            .map(|(addr, numerator)| {
                assert!(AccountId::try_from(addr.to_string()).is_ok());
                // assert!(env::is_valid_account_id(addr.as_bytes()));
                assert!(numerator > 0, "percentage cannot be zero");
                let sf = SafeFraction::new(numerator);
                sum += sf.numerator;
                (addr, sf)
            })
            .collect();
        assert_eq!(sum, 10_000, "fractions don't add to 10,000");

        Self {
            percentage: SafeFraction::new(percentage),
            split_between,
        }
    }
}

impl Default for NFTContractMetadata {
    fn default() -> Self {
        Self {
            spec: "".to_string(),
            name: "".to_string(),
            symbol: "".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        }
    }
}

impl TokenMetadata {
    /// Get the metadata and its size in bytes.
    pub fn from_with_size(
        args: TokenMetadata,
        copies: u64,
    ) -> (Self, u64) {
        if args.media_hash.is_some() {
            assert!(args.media.is_some());
        }

        if args.reference_hash.is_some() {
            assert!(args.reference.is_some());
        }

        let metadata = Self {
            title: args.title,
            description: args.description,
            media: args.media,
            media_hash: args.media_hash,
            copies: (copies as u16).into(),
            expires_at: args.expires_at,
            starts_at: args.starts_at,
            extra: args.extra,
            reference: args.reference,
            reference_hash: args.reference_hash,
        };

        let size = serde_json::to_vec(&metadata).unwrap().len();

        // let size = metadata.try_to_vec().unwrap().len();

        (metadata, size as u64)
    }
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

impl Token {
    /// - `metadata` validation performed in `TokenMetadataArgs::new`
    /// - `royalty` validation performed in `Royalty::new`
    pub fn new(
        owner_id: AccountId,
        token_id: u64,
        metadata_id: u64,
        royalty_id: Option<u64>,
        split_owners: Option<SplitOwners>,
        minter: AccountId,
    ) -> Self {
        Self {
            owner_id: Owner::Account(owner_id),
            id: token_id,
            metadata_id,
            royalty_id,
            split_owners,
            approvals: HashMap::new(),
            minter,
            loan: None,
            composeable_stats: ComposeableStats::new(),
            origin_key: None,
        }
    }

    /// If the token is loaned, return the loaner as the owner.
    pub fn get_owner_or_loaner(&self) -> Owner {
        self.loan
            .as_ref()
            .map(|l| Owner::Account(l.holder.clone()))
            .unwrap_or_else(|| self.owner_id.clone())
    }

    pub fn is_pred_owner(&self) -> bool {
        self.owner_id.to_string() == near_sdk::env::predecessor_account_id().to_string()
    }

    pub fn is_loaned(&self) -> bool {
        self.loan.is_some()
    }
}

impl fmt::Display for Owner {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Owner::Account(s) => write!(f, "{}", s),
            Owner::TokenId(n) => write!(f, "{}", n),
            Owner::CrossKey(key) => write!(f, "{}", key),
            Owner::Lock(_) => panic!("locked"),
        }
    }
}

impl Loan {
    pub fn new(
        holder: AccountId,
        loan_contract: AccountId,
    ) -> Self {
        Self {
            holder,
            loan_contract,
        }
    }
}

impl ComposeableStats {
    fn new() -> Self {
        Self {
            local_depth: 0,
            cross_contract_children: 0,
        }
    }
}

impl TokenKey {
    pub fn new(
        n: u64,
        account_id: AccountId,
    ) -> Self {
        Self {
            token_id: n,
            account_id: account_id.into(),
        }
    }

    pub fn split(self) -> (u64, String) {
        (self.token_id, self.account_id)
    }
}
impl fmt::Display for TokenKey {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}:{}", self.token_id, self.account_id)
    }
}
impl From<String> for TokenKey {
    fn from(s: String) -> Self {
        let (id, account_id) = split_colon(&s);
        Self {
            token_id: id.parse::<u64>().unwrap(),
            account_id: account_id.to_string(),
        }
    }
}

impl SafeFraction {
    /// Take a u32 numerator to a 10^4 denominator.
    ///
    /// Upper limit is 10^4 so as to prevent multiplication with overflow.
    pub fn new(numerator: u32) -> Self {
        assert!(
            (0..=10000).contains(&numerator),
            "{} not between 0 and 10,000",
            numerator
        );
        SafeFraction { numerator }
    }

    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 10_000u128 * self.numerator as u128
    }
}

impl std::ops::Sub for SafeFraction {
    type Output = SafeFraction;

    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        assert!(self.numerator >= rhs.numerator);
        Self {
            numerator: self.numerator - rhs.numerator,
        }
    }
}

impl std::ops::SubAssign for SafeFraction {
    fn sub_assign(
        &mut self,
        rhs: Self,
    ) {
        assert!(self.numerator >= rhs.numerator);
        self.numerator -= rhs.numerator;
    }
}

impl std::ops::Mul for SafeFraction {
    type Output = MultipliedSafeFraction;

    fn mul(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator * rhs.numerator,
        }
    }
}

impl From<SafeFraction> for MultipliedSafeFraction {
    fn from(f: SafeFraction) -> Self {
        MultipliedSafeFraction {
            numerator: f.numerator * 10_000,
        }
    }
}

impl std::ops::Add for MultipliedSafeFraction {
    type Output = Self;

    fn add(
        self,
        rhs: Self,
    ) -> Self::Output {
        MultipliedSafeFraction {
            numerator: self.numerator + rhs.numerator,
        }
    }
}

impl MultipliedSafeFraction {
    /// Fractionalize a balance.
    pub fn multiply_balance(
        &self,
        value: Balance,
    ) -> Balance {
        value / 100_000_000u128 * self.numerator as u128
    }
}

#[cfg(feature = "all")]
impl<'a> std::io::Write for StdioLock<'a> {
    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<usize> {
        match self {
            StdioLock::Stdout(lock) => lock.write(buf),
            StdioLock::Stderr(lock) => lock.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.flush(),
            StdioLock::Stderr(lock) => lock.flush(),
        }
    }

    fn write_all(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<()> {
        match self {
            StdioLock::Stdout(lock) => lock.write_all(buf),
            StdioLock::Stderr(lock) => lock.write_all(buf),
        }
    }
}

#[cfg(feature = "all")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MyMakeWriter {
    type Writer = StdioLock<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        StdioLock::Stdout(self.stdout.lock())
    }

    fn make_writer_for(
        &'a self,
        meta: &tracing::Metadata<'_>,
    ) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if meta.level() <= &tracing::Level::WARN {
            return StdioLock::Stderr(self.stderr.lock());
        }

        // Otherwise, we'll return stdout.
        StdioLock::Stdout(self.stdout.lock())
    }
}
