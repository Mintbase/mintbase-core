import { NearAccount } from "near-workspaces-ava";

export function createPayout(spec: [NearAccount, string][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = amount;
  });
  return payout;
}

export function createPayoutPercentage(spec: [NearAccount, number][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = amount;
  });
  return payout;
}

export function createPayoutNumerators(spec: [NearAccount, number][]) {
  const payout = {};
  spec.forEach(([account, amount]) => {
    payout[account.accountId] = { numerator: amount };
  });
  return payout;
}
