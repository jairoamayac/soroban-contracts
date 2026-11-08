import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * What a member may do.
 * 
 * * `Admin`: propose any action; approve or reject any proposal.
 * * `Approver`: approve or reject budgets, expenses and payments.
 * * `Operator`: propose expenses and payments only.
 */
export type Role = {tag: "Admin", values: void} | {tag: "Approver", values: void} | {tag: "Operator", values: void};

/**
 * Error codes are part of the public interface: never renumber them.
 */
export const Errors = {
  1: {message:"Unauthorized"},
  2: {message:"SelfApproval"},
  3: {message:"NotFound"},
  4: {message:"InvalidAmount"},
  5: {message:"BudgetExceeded"},
  6: {message:"InsufficientFunds"},
  7: {message:"InvalidState"},
  8: {message:"LineNotFound"},
  9: {message:"BudgetBelowPaid"},
  10: {message:"ChartMismatch"},
  /**
   * The treasury would have no admin, or fewer than two members able to
   * approve, which would lock it. Also raised at construction with fewer
   * than two distinct admins.
   */
  11: {message:"QuorumLost"},
  12: {message:"NoAdmins"},
  /**
   * The supporting document already backs another executed expense or payment.
   */
  13: {message:"ReceiptAlreadyUsed"},
  /**
   * Every expense and payment needs a supporting document.
   */
  14: {message:"ReceiptRequired"},
  /**
   * The operation's currency does not match its budget line (or the treasury asset).
   */
  15: {message:"CurrencyMismatch"},
  /**
   * Members live in instance storage, loaded on every call, so their
   * number is capped at `MAX_MEMBERS`.
   */
  16: {message:"TooManyMembers"}
}

/**
 * Everything that changes state goes through a proposal.
 */
export type Action = {tag: "SetBudget", values: readonly [BudgetChange]} | {tag: "RecordExpense", values: readonly [Expense]} | {tag: "Pay", values: readonly [Payment]} | {tag: "AddMember", values: readonly [MemberChange]} | {tag: "RemoveMember", values: readonly [string]} | {tag: "Upgrade", values: readonly [Buffer]};


/**
 * An expense paid outside the ledger (bank transfer, cash) and recorded
 * against its budget line. No funds move.
 */
export interface Expense {
  /**
 * Account the expense books to; may be an auxiliary of the line's account.
 */
account: string;
  amount: i128;
  /**
 * Must match the chart of the budget line.
 */
chart: string;
  line: Buffer;
  /**
 * SHA-256 of the payee identifier (e.g. tax id). Never the identifier itself.
 */
payee_ref: Buffer;
  /**
 * SHA-256 of the external payment evidence (transfer receipt, cash voucher id).
 */
payment_ref: Buffer;
  project: Buffer;
  /**
 * SHA-256 of the supporting document (invoice, receipt). Required.
 */
receipt_hash: Buffer;
  source: Buffer;
}


/**
 * A payment that leaves the treasury on the ledger.
 */
export interface Payment {
  account: string;
  amount: i128;
  /**
 * Must match the chart of the budget line.
 */
chart: string;
  line: Buffer;
  project: Buffer;
  /**
 * SHA-256 of the supporting document. Required.
 */
receipt_hash: Buffer;
  /**
 * Off-chain reference (e.g. SHA-256 of the payment UUID).
 */
reference: Buffer;
  source: Buffer;
  to: string;
}


export interface Proposal {
  action: Action;
  created_at: u64;
  decided_at: u64;
  decided_by: Option<string>;
  id: u64;
  proposer: string;
  status: ProposalStatus;
}


/**
 * A budget line of a project, funded by one source. Amounts are fixed point
 * with 7 decimals in `currency` (45,000 COP is `450_000_000_000`).
 */
export interface BudgetLine {
  /**
 * Default account code in that chart, e.g. `5135`.
 */
account: string;
  budget: i128;
  /**
 * Chart of accounts the line books to, e.g. `CO_PUC` or `IFRS`.
 */
chart: string;
  /**
 * Currency of the budget, e.g. `COP` or `USDC`.
 */
currency: string;
  paid: i128;
}


export interface BudgetChange {
  account: string;
  amount: i128;
  chart: string;
  currency: string;
  line: Buffer;
  project: Buffer;
  source: Buffer;
}


export interface MemberChange {
  member: string;
  role: Role;
}

export type ProposalStatus = {tag: "Pending", values: void} | {tag: "Executed", values: void} | {tag: "Rejected", values: void};











export interface Client {
  /**
   * Construct and simulate a fund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Moves `amount` of the treasury asset from `from` into the treasury,
   * earmarked for `project` by funding `source`. Anyone may fund (e.g. the
   * funder directly). Returns what this source now holds for the project.
   */
  fund: ({from, project, source, amount}: {from: string, project: Buffer, source: Buffer, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a line transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  line: ({project, line, source}: {project: Buffer, line: Buffer, source: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<BudgetLine>>>

  /**
   * Construct and simulate a role transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  role: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<Role>>>

  /**
   * Construct and simulate a funds transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  funds: ({project, source}: {project: Buffer, source: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  token: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a reject transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Rejects a pending proposal. Admins and approvers may reject what they
   * could approve; anyone may withdraw their own proposal.
   */
  reject: ({member, id}: {member: string, id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a approve transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Approves and executes a proposal. The approver is never the proposer;
   * membership and code changes need an admin approver.
   */
  approve: ({approver, id}: {approver: string, id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a members transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  members: (options?: MethodOptions) => Promise<AssembledTransaction<Map<string, Role>>>

  /**
   * Construct and simulate a propose transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Opens a proposal. Admins may propose anything; operators may propose
   * expenses and payments; approvers may not propose. Every action is
   * validated here, so invalid proposals are never stored, and again when
   * it is approved, because state may have changed in between.
   */
  propose: ({proposer, action}: {proposer: string, action: Action}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  proposal: ({id}: {id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Option<Proposal>>>

  /**
   * Construct and simulate a receipt_use transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Proposal id of the executed expense or payment a supporting document backs.
   */
  receipt_use: ({receipt_hash}: {receipt_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<u64>>>

  /**
   * Construct and simulate a token_symbol transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  token_symbol: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admins, token, token_symbol}: {admins: Array<string>, token: string, token_symbol: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admins, token, token_symbol}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAMdXaGF0IGEgbWVtYmVyIG1heSBkby4KCiogYEFkbWluYDogcHJvcG9zZSBhbnkgYWN0aW9uOyBhcHByb3ZlIG9yIHJlamVjdCBhbnkgcHJvcG9zYWwuCiogYEFwcHJvdmVyYDogYXBwcm92ZSBvciByZWplY3QgYnVkZ2V0cywgZXhwZW5zZXMgYW5kIHBheW1lbnRzLgoqIGBPcGVyYXRvcmA6IHByb3Bvc2UgZXhwZW5zZXMgYW5kIHBheW1lbnRzIG9ubHkuAAAAAAAAAAAEUm9sZQAAAAMAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAACEFwcHJvdmVyAAAAAAAAAAAAAAAIT3BlcmF0b3I=",
        "AAAABAAAAEJFcnJvciBjb2RlcyBhcmUgcGFydCBvZiB0aGUgcHVibGljIGludGVyZmFjZTogbmV2ZXIgcmVudW1iZXIgdGhlbS4AAAAAAAAAAAAFRXJyb3IAAAAAAAAQAAAAAAAAAAxVbmF1dGhvcml6ZWQAAAABAAAAAAAAAAxTZWxmQXBwcm92YWwAAAACAAAAAAAAAAhOb3RGb3VuZAAAAAMAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAAEAAAAAAAAAA5CdWRnZXRFeGNlZWRlZAAAAAAABQAAAAAAAAARSW5zdWZmaWNpZW50RnVuZHMAAAAAAAAGAAAAAAAAAAxJbnZhbGlkU3RhdGUAAAAHAAAAAAAAAAxMaW5lTm90Rm91bmQAAAAIAAAAAAAAAA9CdWRnZXRCZWxvd1BhaWQAAAAACQAAAAAAAAANQ2hhcnRNaXNtYXRjaAAAAAAAAAoAAACiVGhlIHRyZWFzdXJ5IHdvdWxkIGhhdmUgbm8gYWRtaW4sIG9yIGZld2VyIHRoYW4gdHdvIG1lbWJlcnMgYWJsZSB0bwphcHByb3ZlLCB3aGljaCB3b3VsZCBsb2NrIGl0LiBBbHNvIHJhaXNlZCBhdCBjb25zdHJ1Y3Rpb24gd2l0aCBmZXdlcgp0aGFuIHR3byBkaXN0aW5jdCBhZG1pbnMuAAAAAAAKUXVvcnVtTG9zdAAAAAAACwAAAAAAAAAITm9BZG1pbnMAAAAMAAAASlRoZSBzdXBwb3J0aW5nIGRvY3VtZW50IGFscmVhZHkgYmFja3MgYW5vdGhlciBleGVjdXRlZCBleHBlbnNlIG9yIHBheW1lbnQuAAAAAAASUmVjZWlwdEFscmVhZHlVc2VkAAAAAAANAAAANkV2ZXJ5IGV4cGVuc2UgYW5kIHBheW1lbnQgbmVlZHMgYSBzdXBwb3J0aW5nIGRvY3VtZW50LgAAAAAAD1JlY2VpcHRSZXF1aXJlZAAAAAAOAAAAUFRoZSBvcGVyYXRpb24ncyBjdXJyZW5jeSBkb2VzIG5vdCBtYXRjaCBpdHMgYnVkZ2V0IGxpbmUgKG9yIHRoZSB0cmVhc3VyeSBhc3NldCkuAAAAEEN1cnJlbmN5TWlzbWF0Y2gAAAAPAAAAY01lbWJlcnMgbGl2ZSBpbiBpbnN0YW5jZSBzdG9yYWdlLCBsb2FkZWQgb24gZXZlcnkgY2FsbCwgc28gdGhlaXIKbnVtYmVyIGlzIGNhcHBlZCBhdCBgTUFYX01FTUJFUlNgLgAAAAAOVG9vTWFueU1lbWJlcnMAAAAAABA=",
        "AAAAAgAAADZFdmVyeXRoaW5nIHRoYXQgY2hhbmdlcyBzdGF0ZSBnb2VzIHRocm91Z2ggYSBwcm9wb3NhbC4AAAAAAAAAAAAGQWN0aW9uAAAAAAAGAAAAAQAAAAAAAAAJU2V0QnVkZ2V0AAAAAAAAAQAAB9AAAAAMQnVkZ2V0Q2hhbmdlAAAAAQAAAAAAAAANUmVjb3JkRXhwZW5zZQAAAAAAAAEAAAfQAAAAB0V4cGVuc2UAAAAAAQAAAAAAAAADUGF5AAAAAAEAAAfQAAAAB1BheW1lbnQAAAAAAQAAAAAAAAAJQWRkTWVtYmVyAAAAAAAAAQAAB9AAAAAMTWVtYmVyQ2hhbmdlAAAAAQAAAAAAAAAMUmVtb3ZlTWVtYmVyAAAAAQAAABMAAAABAAAAAAAAAAdVcGdyYWRlAAAAAAEAAAPuAAAAIA==",
        "AAAAAQAAAG1BbiBleHBlbnNlIHBhaWQgb3V0c2lkZSB0aGUgbGVkZ2VyIChiYW5rIHRyYW5zZmVyLCBjYXNoKSBhbmQgcmVjb3JkZWQKYWdhaW5zdCBpdHMgYnVkZ2V0IGxpbmUuIE5vIGZ1bmRzIG1vdmUuAAAAAAAAAAAAAAdFeHBlbnNlAAAAAAkAAABIQWNjb3VudCB0aGUgZXhwZW5zZSBib29rcyB0bzsgbWF5IGJlIGFuIGF1eGlsaWFyeSBvZiB0aGUgbGluZSdzIGFjY291bnQuAAAAB2FjY291bnQAAAAAEQAAAAAAAAAGYW1vdW50AAAAAAALAAAAKE11c3QgbWF0Y2ggdGhlIGNoYXJ0IG9mIHRoZSBidWRnZXQgbGluZS4AAAAFY2hhcnQAAAAAAAARAAAAAAAAAARsaW5lAAAD7gAAACAAAABLU0hBLTI1NiBvZiB0aGUgcGF5ZWUgaWRlbnRpZmllciAoZS5nLiB0YXggaWQpLiBOZXZlciB0aGUgaWRlbnRpZmllciBpdHNlbGYuAAAAAAlwYXllZV9yZWYAAAAAAAPuAAAAIAAAAE1TSEEtMjU2IG9mIHRoZSBleHRlcm5hbCBwYXltZW50IGV2aWRlbmNlICh0cmFuc2ZlciByZWNlaXB0LCBjYXNoIHZvdWNoZXIgaWQpLgAAAAAAAAtwYXltZW50X3JlZgAAAAPuAAAAIAAAAAAAAAAHcHJvamVjdAAAAAPuAAAAIAAAAEBTSEEtMjU2IG9mIHRoZSBzdXBwb3J0aW5nIGRvY3VtZW50IChpbnZvaWNlLCByZWNlaXB0KS4gUmVxdWlyZWQuAAAADHJlY2VpcHRfaGFzaAAAA+4AAAAgAAAAAAAAAAZzb3VyY2UAAAAAA+4AAAAg",
        "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",
        "AAAABQAAAHFUb3BpY3M6IGAoInBheW1lbnRfZXhlY3V0ZWQiLCBwcm9qZWN0KWAuIEEgcGF5bWVudCB0aGF0IGxlZnQgdGhlIHRyZWFzdXJ5Cm9uIHRoZSBsZWRnZXIsIHdpdGggaXRzIHByb29mIG9mIHNwZW5kLgAAAAAAAAAAAAAPUGF5bWVudEV4ZWN1dGVkAAAAAAEAAAAQcGF5bWVudF9leGVjdXRlZAAAABAAAAAAAAAAB3Byb2plY3QAAAAD7gAAACAAAAABAAAAAAAAAARsaW5lAAAD7gAAACAAAAAAAAAAAAAAAAZzb3VyY2UAAAAAA+4AAAAgAAAAAAAAAAAAAAALcHJvcG9zYWxfaWQAAAAABgAAAAAAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAAAAAAIY3VycmVuY3kAAAARAAAAAAAAAAAAAAAFY2hhcnQAAAAAAAARAAAAAAAAAAAAAAAHYWNjb3VudAAAAAARAAAAAAAAAAAAAAAMcmVjZWlwdF9oYXNoAAAD7gAAACAAAAAAAAAAAAAAAAlyZWZlcmVuY2UAAAAAAAPuAAAAIAAAAAAAAAAAAAAACHByb3Bvc2VyAAAAEwAAAAAAAAAAAAAACGFwcHJvdmVyAAAAEwAAAAAAAAAAAAAACWxpbmVfcGFpZAAAAAAAAAsAAAAAAAAAAAAAAAtsaW5lX2J1ZGdldAAAAAALAAAAAAAAAAAAAAAMc291cmNlX2Z1bmRzAAAACwAAAAAAAAAC",
        "AAAABQAAACNUb3BpY3M6IGAoInByb3Bvc2FsX2NyZWF0ZWQiLCBpZClgLgAAAAAAAAAAD1Byb3Bvc2FsQ3JlYXRlZAAAAAABAAAAEHByb3Bvc2FsX2NyZWF0ZWQAAAADAAAAAAAAAAJpZAAAAAAABgAAAAEAAAAAAAAACHByb3Bvc2VyAAAAEwAAAAAAAAAAAAAABGtpbmQAAAARAAAAAAAAAAI=",
        "AAAABQAAAJBUb3BpY3M6IGAoImNvbnRyYWN0X3VwZ3JhZGVkIiwgcHJvcG9zYWxfaWQpYC4gRW1pdHRlZCBiZWZvcmUgdGhlIGNvZGUgaXMKcmVwbGFjZWQsIHNvIGEgcmVhZGVyIGtub3dzIHRoZSBydWxlcyBtYXkgaGF2ZSBjaGFuZ2VkIGZyb20gdGhpcyBwb2ludC4AAAAAAAAAEENvbnRyYWN0VXBncmFkZWQAAAABAAAAEWNvbnRyYWN0X3VwZ3JhZGVkAAAAAAAABAAAAAAAAAALcHJvcG9zYWxfaWQAAAAABgAAAAEAAAAAAAAACXdhc21faGFzaAAAAAAAA+4AAAAgAAAAAAAAAAAAAAAIcHJvcG9zZXIAAAATAAAAAAAAAAAAAAAIYXBwcm92ZXIAAAATAAAAAAAAAAI=",
        "AAAABQAAAHlUb3BpY3M6IGAoInByb3Bvc2FsX2V4ZWN1dGVkIiwgaWQpYC4gRW1pdHRlZCBmb3IgZXZlcnkgZXhlY3V0ZWQgcHJvcG9zYWwsCnNvIGdvdmVybmFuY2UgY2FuIGJlIGF1ZGl0ZWQgZnJvbSBldmVudHMgYWxvbmUuAAAAAAAAAAAAABBQcm9wb3NhbEV4ZWN1dGVkAAAAAQAAABFwcm9wb3NhbF9leGVjdXRlZAAAAAAAAAQAAAAAAAAAAmlkAAAAAAAGAAAAAQAAAAAAAAAEa2luZAAAABEAAAAAAAAAAAAAAAhwcm9wb3NlcgAAABMAAAAAAAAAAAAAAAhhcHByb3ZlcgAAABMAAAAAAAAAAg==",
        "AAAABQAAACRUb3BpY3M6IGAoInByb3Bvc2FsX3JlamVjdGVkIiwgaWQpYC4AAAAAAAAAEFByb3Bvc2FsUmVqZWN0ZWQAAAABAAAAEXByb3Bvc2FsX3JlamVjdGVkAAAAAAAAAgAAAAAAAAACaWQAAAAAAAYAAAABAAAAAAAAAAtyZWplY3RlZF9ieQAAAAATAAAAAAAAAAI=",
        "AAAAAAAAANBNb3ZlcyBgYW1vdW50YCBvZiB0aGUgdHJlYXN1cnkgYXNzZXQgZnJvbSBgZnJvbWAgaW50byB0aGUgdHJlYXN1cnksCmVhcm1hcmtlZCBmb3IgYHByb2plY3RgIGJ5IGZ1bmRpbmcgYHNvdXJjZWAuIEFueW9uZSBtYXkgZnVuZCAoZS5nLiB0aGUKZnVuZGVyIGRpcmVjdGx5KS4gUmV0dXJucyB3aGF0IHRoaXMgc291cmNlIG5vdyBob2xkcyBmb3IgdGhlIHByb2plY3QuAAAABGZ1bmQAAAAEAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAHcHJvamVjdAAAAAPuAAAAIAAAAAAAAAAGc291cmNlAAAAAAPuAAAAIAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAALAAAAAw==",
        "AAAAAAAAAAAAAAAEbGluZQAAAAMAAAAAAAAAB3Byb2plY3QAAAAD7gAAACAAAAAAAAAABGxpbmUAAAPuAAAAIAAAAAAAAAAGc291cmNlAAAAAAPuAAAAIAAAAAEAAAPoAAAH0AAAAApCdWRnZXRMaW5lAAA=",
        "AAAAAAAAAAAAAAAEcm9sZQAAAAEAAAAAAAAABm1lbWJlcgAAAAAAEwAAAAEAAAPoAAAH0AAAAARSb2xl",
        "AAAAAAAAAAAAAAAFZnVuZHMAAAAAAAACAAAAAAAAAAdwcm9qZWN0AAAAA+4AAAAgAAAAAAAAAAZzb3VyY2UAAAAAA+4AAAAgAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAAFdG9rZW4AAAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAHxSZWplY3RzIGEgcGVuZGluZyBwcm9wb3NhbC4gQWRtaW5zIGFuZCBhcHByb3ZlcnMgbWF5IHJlamVjdCB3aGF0IHRoZXkKY291bGQgYXBwcm92ZTsgYW55b25lIG1heSB3aXRoZHJhdyB0aGVpciBvd24gcHJvcG9zYWwuAAAABnJlamVjdAAAAAAAAgAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAAAAAAJpZAAAAAAABgAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAHlBcHByb3ZlcyBhbmQgZXhlY3V0ZXMgYSBwcm9wb3NhbC4gVGhlIGFwcHJvdmVyIGlzIG5ldmVyIHRoZSBwcm9wb3NlcjsKbWVtYmVyc2hpcCBhbmQgY29kZSBjaGFuZ2VzIG5lZWQgYW4gYWRtaW4gYXBwcm92ZXIuAAAAAAAAB2FwcHJvdmUAAAAAAgAAAAAAAAAIYXBwcm92ZXIAAAATAAAAAAAAAAJpZAAAAAAABgAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAHbWVtYmVycwAAAAAAAAAAAQAAA+wAAAATAAAH0AAAAARSb2xl",
        "AAAAAAAAAQdPcGVucyBhIHByb3Bvc2FsLiBBZG1pbnMgbWF5IHByb3Bvc2UgYW55dGhpbmc7IG9wZXJhdG9ycyBtYXkgcHJvcG9zZQpleHBlbnNlcyBhbmQgcGF5bWVudHM7IGFwcHJvdmVycyBtYXkgbm90IHByb3Bvc2UuIEV2ZXJ5IGFjdGlvbiBpcwp2YWxpZGF0ZWQgaGVyZSwgc28gaW52YWxpZCBwcm9wb3NhbHMgYXJlIG5ldmVyIHN0b3JlZCwgYW5kIGFnYWluIHdoZW4KaXQgaXMgYXBwcm92ZWQsIGJlY2F1c2Ugc3RhdGUgbWF5IGhhdmUgY2hhbmdlZCBpbiBiZXR3ZWVuLgAAAAAHcHJvcG9zZQAAAAACAAAAAAAAAAhwcm9wb3NlcgAAABMAAAAAAAAABmFjdGlvbgAAAAAH0AAAAAZBY3Rpb24AAAAAAAEAAAPpAAAABgAAAAM=",
        "AAAAAAAAAAAAAAAIcHJvcG9zYWwAAAABAAAAAAAAAAJpZAAAAAAABgAAAAEAAAPoAAAH0AAAAAhQcm9wb3NhbA==",
        "AAAAAAAAAEtQcm9wb3NhbCBpZCBvZiB0aGUgZXhlY3V0ZWQgZXhwZW5zZSBvciBwYXltZW50IGEgc3VwcG9ydGluZyBkb2N1bWVudCBiYWNrcy4AAAAAC3JlY2VpcHRfdXNlAAAAAAEAAAAAAAAADHJlY2VpcHRfaGFzaAAAA+4AAAAgAAAAAQAAA+gAAAAG",
        "AAAAAAAAAAAAAAAMdG9rZW5fc3ltYm9sAAAAAAAAAAEAAAAR",
        "AAAAAAAAAW5DcmVhdGVzIHRoZSB0cmVhc3VyeSBmb3Igb25lIG9yZ2FuaXNhdGlvbi4gYGFkbWluc2AgYXJlIHRoZQpvcmdhbmlzYXRpb24ncyBvd24gcGVvcGxlOyBUcnV3YXksIGFzIGRlcGxveWVyLCBpcyBub3QgYSBtZW1iZXIuCmB0b2tlbmAgaXMgdGhlIGFzc2V0IGhlbGQgb24gdGhlIGxlZGdlciBhbmQgYHRva2VuX3N5bWJvbGAgaXRzIGNvZGUKKGUuZy4gYFVTRENgKSwgd2hpY2ggcGF5bWVudCBidWRnZXQgbGluZXMgbXVzdCB1c2UuCkF0IGxlYXN0IHR3byBkaXN0aW5jdCBhZG1pbnMgYXJlIHJlcXVpcmVkLCBzbyB0aGF0IHRoZSBmaXJzdCBwcm9wb3NhbApjYW4gYmUgYXBwcm92ZWQgYnkgc29tZW9uZSBvdGhlciB0aGFuIGl0cyBwcm9wb3Nlci4AAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAwAAAAAAAAAGYWRtaW5zAAAAAAPqAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAARAAAAAA==" ]),
      options
    )
  }
  public readonly fromJSON = {
    fund: this.txFromJSON<Result<i128>>,
        line: this.txFromJSON<Option<BudgetLine>>,
        role: this.txFromJSON<Option<Role>>,
        funds: this.txFromJSON<i128>,
        token: this.txFromJSON<string>,
        reject: this.txFromJSON<Result<void>>,
        approve: this.txFromJSON<Result<void>>,
        members: this.txFromJSON<Map<string, Role>>,
        propose: this.txFromJSON<Result<u64>>,
        proposal: this.txFromJSON<Option<Proposal>>,
        receipt_use: this.txFromJSON<Option<u64>>,
        token_symbol: this.txFromJSON<string>
  }
}