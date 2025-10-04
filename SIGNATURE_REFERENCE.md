# Accumulate Protocol Signature Reference

## Signature Type Hierarchy

Based on `protocol/types_gen.go` - complete signature structure reference.

## Base Interfaces

### Signature Interface
All signatures implement:
- `Type()` - Returns signature type
- `RoutingLocation()` - Network routing information
- `GetVote()` - Vote type (accept/reject/abstain)
- `GetSigner()` - Signer URL
- `GetTransactionHash()` - Hash of signed transaction
- `Hash()` - Signature hash
- `Metadata()` - Signature metadata

### UserSignature Interface
User-generated signatures (extends Signature):
- `Initiator()` - Transaction initiator
- `Verify()` - Cryptographic verification

### KeySignature Interface
Cryptographic signatures (extends UserSignature):
- `GetSignature()` - Raw signature bytes
- `GetPublicKeyHash()` - Hash of public key
- `GetPublicKey()` - Public key bytes
- `GetSignerVersion()` - Version of signer
- `GetTimestamp()` - Signature timestamp

## Cryptographic Signatures

### ED25519Signature
**File**: `protocol/types_gen.go:347`
**Category**: Primary Ed25519 cryptographic signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | Ed25519 public key |
| Signature | []byte | ✅ | Ed25519 signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### ETHSignature
**File**: `protocol/types_gen.go:361`
**Category**: Ethereum ECDSA secp256k1 signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | Ethereum public key |
| Signature | []byte | ✅ | Ethereum signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### BTCSignature
**File**: `protocol/types_gen.go:170`
**Category**: Bitcoin signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | Bitcoin public key |
| Signature | []byte | ✅ | Bitcoin signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### BTCLegacySignature
**File**: `protocol/types_gen.go:156`
**Category**: Legacy Bitcoin signature format

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | Bitcoin legacy public key |
| Signature | []byte | ✅ | Bitcoin legacy signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### EcdsaSha256Signature
**File**: `protocol/types_gen.go:375`
**Category**: ECDSA SHA256 signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | ECDSA public key |
| Signature | []byte | ✅ | ECDSA signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### RsaSha256Signature
**File**: `protocol/types_gen.go:786`
**Category**: RSA SHA256 signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | RSA public key |
| Signature | []byte | ✅ | RSA signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### TypedDataSignature
**File**: `protocol/types_gen.go:1041`
**Category**: EIP-712 typed data signature

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | Public key for typed data |
| Signature | []byte | ✅ | Typed data signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |
| ChainID | *big.Int | ✅ | Chain ID for typed data |

**Special Note**: TypedDataSignature is the exception to the "omit null fields" rule. Some EIP-712 implementations may require explicit null handling for chain ID or other fields.

### RCD1Signature
**File**: `protocol/types_gen.go:704`
**Category**: Factom RCD1 signature format

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| PublicKey | []byte | ✅ | RCD1 public key |
| Signature | []byte | ✅ | RCD1 signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Timestamp | uint64 | ❌ | Timestamp of signing |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Memo | string | ❌ | Optional memo |
| Data | []byte | ❌ | Additional signature data |

### LegacyED25519Signature
**File**: `protocol/types_gen.go:511`
**Category**: Legacy Ed25519 signature format

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Timestamp | uint64 | ✅ | Legacy timestamp field (required) |
| PublicKey | []byte | ✅ | Ed25519 public key |
| Signature | []byte | ✅ | Ed25519 signature bytes |
| Signer | *url.URL | ✅ | URL of the signing entity |
| SignerVersion | uint64 | ✅ | Version of the signer |
| Vote | VoteType | ❌ | Vote type |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |

**Note**: In LegacyED25519Signature, timestamp is required (unlike other signatures).

## System Signatures

### AuthoritySignature
**File**: `protocol/types_gen.go:139`
**Category**: System signature for authority consensus

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Origin | *url.URL | ✅ | The signer that produced this signature |
| Authority | *url.URL | ✅ | The authority that produced this signature |
| Vote | VoteType | ❌ | The authority's vote |
| TxID | *url.TxID | ✅ | The ID of the transaction this was produced for |
| Cause | *url.TxID | ✅ | The ID of the signature that produced this |
| Delegator | []*url.URL | ✅ | Delegation chain |
| Memo | string | ❌ | Optional memo field |

### InternalSignature
**File**: `protocol/types_gen.go:453`
**Category**: Internal signature for system-generated transactions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Cause | [32]byte | ✅ | Hash of the transaction that produced the signed transaction |
| TransactionHash | [32]byte | ✅ | Hash of the signed transaction |

### PartitionSignature
**File**: `protocol/types_gen.go:670`
**Category**: Signature for routing between network partitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| SourceNetwork | *url.URL | ✅ | Source network of transaction |
| DestinationNetwork | *url.URL | ✅ | Destination network of transaction |
| SequenceNumber | uint64 | ✅ | Sequence number of transaction |
| TransactionHash | [32]byte | ❌ | Hash of transaction |

### ReceiptSignature
**File**: `protocol/types_gen.go:725`
**Category**: Cryptographic receipt proving transaction inclusion

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| SourceNetwork | *url.URL | ✅ | Network that produced the transaction |
| Proof | merkle.Receipt | ✅ | Merkle proof receipt |
| TransactionHash | [32]byte | ❌ | Hash of transaction |

## Wrapper Signatures

### DelegatedSignature
**File**: `protocol/types_gen.go:312`
**Category**: Wraps another signature to delegate authority

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Signature | Signature | ✅ | The wrapped signature |
| Delegator | *url.URL | ✅ | The authority that delegated its authority to the signer |

**Usage**: Delegation chains up to 5 levels deep.

### RemoteSignature
**File**: `protocol/types_gen.go:735`
**Category**: Used when forwarding a signature from one partition to another

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Destination | *url.URL | ✅ | Destination partition for forwarding |
| Signature | Signature | ✅ | The wrapped signature |
| Cause | [][32]byte | ✅ | Hash chain of causes |

## Aggregate Signatures

### SignatureSet
**File**: `protocol/types_gen.go:827`
**Category**: Container for multiple signatures with shared authority

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Vote | VoteType | ❌ | Vote type for the set |
| Signer | *url.URL | ✅ | URL of the signing entity |
| TransactionHash | [32]byte | ❌ | Hash of signed transaction |
| Signatures | []Signature | ✅ | Set of signatures |
| Authority | *url.URL | ✅ | Authority for the signature set |

## Usage Patterns

### JSON Format vs Internal Structure

**Important Protocol Rules**:
1. JSON envelope format uses lowercase type names, while internal Go structures use CamelCase names
2. **Null/empty fields are omitted entirely** from JSON (not encoded as null)
3. Exception: TypedDataSignature may require explicit null handling for some fields

#### JSON Format (for APIs and tools)
```json
{
  "type": "ed25519",  // lowercase in JSON
  "publicKey": "...",
  "signature": "...",
  "signer": "acc://user.acme/book/1",
  "signerVersion": 1,
  "timestamp": 1234567890,
  "transactionHash": "..."
}
```

#### Type Name Mapping

| Go Structure | JSON Type | Category |
|--------------|-----------|----------|
| `ED25519Signature` | `"ed25519"` | Cryptographic |
| `ETHSignature` | `"eth"` | Cryptographic |
| `BTCSignature` | `"btc"` | Cryptographic |
| `BTCLegacySignature` | `"btcLegacy"` | Cryptographic |
| `EcdsaSha256Signature` | `"ecdsaSha256"` | Cryptographic |
| `RsaSha256Signature` | `"rsaSha256"` | Cryptographic |
| `TypedDataSignature` | `"typedData"` | Cryptographic |
| `RCD1Signature` | `"rcd1"` | Cryptographic |
| `LegacyED25519Signature` | `"legacyEd25519"` | Cryptographic |
| `DelegatedSignature` | `"delegated"` | Wrapper |
| `RemoteSignature` | `"remote"` | Wrapper |
| `AuthoritySignature` | `"authority"` | System |
| `InternalSignature` | `"internal"` | System |
| `PartitionSignature` | `"partition"` | System |
| `ReceiptSignature` | `"receipt"` | System |
| `SignatureSet` | `"set"` | Aggregate |

### Standard User Transaction

#### Minimal Required Fields
```json
{
  "type": "ed25519",
  "publicKey": "...",
  "signature": "...",
  "signer": "acc://user.acme/book/1",
  "signerVersion": 1
  // Optional fields omitted if null/empty
}
```

#### With Optional Fields
```json
{
  "type": "ed25519",
  "publicKey": "...",
  "signature": "...",
  "signer": "acc://user.acme/book/1",
  "signerVersion": 1,
  "timestamp": 1234567890,                         // Only if present
  "transactionHash": "...",                        // Only if needed
  "memo": "signature note"                         // Only if non-empty
}
```

### Delegated Authority
```json
{
  "type": "DelegatedSignature",
  "signature": {
    "type": "ED25519Signature",
    // ... inner signature fields
  },
  "delegator": "acc://authority.acme/book/1"
}
```

### Multi-signature Set
```json
{
  "type": "SignatureSet",
  "signer": "acc://multisig.acme",
  "authority": "acc://multisig.acme/book/1",
  "signatures": [
    {
      "type": "ED25519Signature",
      // ... signature 1
    },
    {
      "type": "ETHSignature",
      // ... signature 2
    }
  ]
}
```

### Cross-partition Transaction
```json
{
  "type": "RemoteSignature",
  "destination": "acc://other-partition.acme",
  "signature": {
    "type": "ED25519Signature",
    // ... inner signature
  },
  "cause": ["hash1", "hash2", "hash3"]
}
```

## Hash Coverage Rules

- **KeySignatures**: Cover serialized transaction bytes (Header + Body)
- **AuthoritySignatures**: Cover specific transaction and cause IDs
- **SystemSignatures**: Cover relevant system operation data
- **WrapperSignatures**: Inherit coverage from wrapped signature

## Vote Types

Standard vote values:
- `"accept"` - Approve the transaction
- `"reject"` - Reject the transaction
- `"abstain"` - Abstain from voting
- `null` - No vote specified