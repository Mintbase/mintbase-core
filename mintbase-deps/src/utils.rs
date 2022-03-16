#[cfg(feature = "market-wasm")]
use near_sdk::Balance;
use near_sdk::Gas;

/// Split a &str on the first colon
pub fn split_colon(string: &str) -> (&str, &str) {
    let pos = string.find(':').expect("no colon");
    (&string[..pos], &string[(pos + 1)..])
}

/// Gas is in TerraUnits, default gas call is 100TGas.
pub const fn ntot(near_amount: Gas) -> Gas {
    Gas(near_amount.0 * 10u64.pow(12))
}

/// Near denominated units are in 10^24
#[cfg(feature = "market-wasm")]
pub const fn ntoy(near_amount: Balance) -> Balance {
    near_amount * 10u128.pow(24)
}

// // TODO: unused, deprecated?
// pub fn to_yocto(value: &str) -> u128 {
//     let vals: Vec<_> = value.split('.').collect();
//     let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
//     if vals.len() > 1 {
//         let power = vals[1].len() as u32;
//         let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
//         part1 + part2
//     } else {
//         part1
//     }
// }

// // TODO: unused, deprecated?
// pub fn to_near(n: u128) -> u128 {
//     n * 10u128.pow(24)
// }
