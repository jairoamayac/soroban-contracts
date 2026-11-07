# Verification

How anyone can check a record of a `budget-treasury` instance without an account and without
trusting Truway's servers. The data model is specified in [Proof of Spend](proof-of-spend.md),
section 7.

## 1. Check the contract

1. Read the contract meta and confirm `event_schema = proof-of-spend/1`:

   ```bash
   stellar contract info meta --contract-id <CONTRACT_ID> --network testnet
   ```

2. Compare the contract's WASM hash with the `wasm_hash` listed in
   [`deployments/`](../deployments/), and rebuild that commit to confirm the hash matches the
   source.

## 2. Find the record

Each record is an event. Look it up by transaction hash on a block explorer, or query events of
the contract by name and project:

```bash
curl -s https://soroban-testnet.stellar.org -H 'content-type: application/json' -d '{
  "jsonrpc": "2.0", "id": 1, "method": "getEvents",
  "params": {
    "startLedger": <LEDGER>,
    "filters": [{
      "type": "contract",
      "contractIds": ["<CONTRACT_ID>"],
      "topics": [["AAAADwAAABBleHBlbnNlX3JlY29yZGVk", "*"]]
    }]
  }
}'
```

The first topic is the event name as a base64 `ScVal` symbol (here `expense_recorded`; encode
others with `stellar xdr encode --type ScVal`), and `*` matches any project. RPC keeps events for
about 7 days. For older records, use an indexer or a history archive. The
current totals of a line can always be read with `line(project, line, source)`.

## 3. Check the supporting document

Compute the SHA-256 hash of the document's bytes and compare it with `receipt_hash` in the
event:

```bash
shasum -a 256 invoice.pdf
```

`receipt_use(receipt_hash)` returns the proposal that used the document, which confirms it
backs exactly one record.

## 4. Check the approvals

The event carries `proposer` and `approver`, always two different addresses. `proposal(id)`
returns the full proposal, including when it was created and decided. Binding each address to a
person is done off the ledger by the organisation.

## Expected outcomes

| Result | Meaning |
|---|---|
| Hash matches, schema declared, WASM matches the source | The record is authentic and was produced by the published rules |
| Hash differs | The document is not the one that was approved |
| No `event_schema` meta | The contract does not claim to follow Proof of Spend; its events must be ignored |
| Entry archived | Restore it (simulation does it automatically) or read the event from an indexer |
