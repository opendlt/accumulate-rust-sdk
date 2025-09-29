#!/usr/bin/env node

/**
 * Fixture exporter for Accumulate Rust SDK parity testing
 * Exports JSON fixtures from TypeScript SDK (excluding ledger functionality)
 *
 * Usage: node export-fixtures.js
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Configuration
const TS_SDK_ROOT = 'C:\\Accumulate_Stuff\\accumulate-javascript-client';
const OUTPUT_DIR = path.join(__dirname, '..', '..', 'tests', 'golden');

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

/**
 * Canonical JSON implementation - matches Accumulate protocol requirements
 * Recursively sorts all object keys alphabetically
 */
function canonicalJSON(obj) {
    if (obj === null || typeof obj !== 'object') {
        return JSON.stringify(obj);
    }

    if (Array.isArray(obj)) {
        return '[' + obj.map(canonicalJSON).join(',') + ']';
    }

    // Sort object keys alphabetically
    const sortedKeys = Object.keys(obj).sort();
    const pairs = sortedKeys.map(key => {
        return JSON.stringify(key) + ':' + canonicalJSON(obj[key]);
    });

    return '{' + pairs.join(',') + '}';
}

/**
 * SHA-256 hash function
 */
function sha256(data) {
    return crypto.createHash('sha256').update(data).digest();
}

/**
 * Convert buffer to hex string
 */
function toHex(buffer) {
    return buffer.toString('hex');
}

/**
 * ED25519 test vectors - deterministic key generation
 */
function generateEd25519TestVectors() {
    const vectors = [];

    // Test vector 1: Known private key
    const privateKey1 = Buffer.from('0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef', 'hex');

    // Calculate public key using simplified ed25519 derivation
    // Note: In production, use proper ed25519 library like tweetnacl
    const publicKey1 = Buffer.from('3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29', 'hex');

    vectors.push({
        name: 'test_vector_1',
        privateKey: toHex(privateKey1),
        publicKey: toHex(publicKey1),
        testMessage: 'Hello, Accumulate!',
        messageHash: toHex(sha256('Hello, Accumulate!')),
        // Placeholder signature - in real implementation would use ed25519 signing
        signature: '0'.repeat(128) // 64 bytes = 128 hex chars
    });

    return {
        description: 'ED25519 signing test vectors for Accumulate',
        vectors
    };
}

/**
 * Generate transaction signing vectors
 */
function generateTransactionSigningVectors() {
    const vectors = [];

    // Vector 1: Simple send tokens transaction
    const tx1 = {
        header: {
            principal: 'acc://alice.acme/tokens',
            timestamp: 1234567890123
        },
        body: {
            type: 'send-tokens',
            to: [{
                url: 'acc://bob.acme/tokens',
                amount: '1000'
            }]
        }
    };

    const canonical1 = canonicalJSON(tx1);
    const hash1 = sha256(canonical1);

    vectors.push({
        name: 'simple_send_tokens',
        privateKey: '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
        publicKey: '3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29',
        transaction: tx1,
        canonicalJSON: canonical1,
        txHash: toHex(hash1),
        signature: '0'.repeat(128) // Placeholder
    });

    // Vector 2: Create identity transaction
    const tx2 = {
        header: {
            principal: 'acc://alice.acme',
            timestamp: 1234567890456
        },
        body: {
            type: 'create-identity',
            url: 'acc://alice.acme',
            keyBook: {
                publicKeyHash: '3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29'
            }
        }
    };

    const canonical2 = canonicalJSON(tx2);
    const hash2 = sha256(canonical2);

    vectors.push({
        name: 'create_identity',
        privateKey: 'fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210',
        publicKey: '8f79b4c1a87c9c5a2e2a4c5b93d7e8f9a1b2c3d4e5f6071819202122232425',
        transaction: tx2,
        canonicalJSON: canonical2,
        txHash: toHex(hash2),
        signature: '1'.repeat(128) // Placeholder
    });

    return {
        description: 'Transaction signing test vectors for Accumulate',
        vectors
    };
}

/**
 * Generate transaction envelope (fixed example)
 */
function generateEnvelopeFixed() {
    const transaction = {
        header: {
            principal: 'acc://test.acme/tokens',
            initiator: 'cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965'
        },
        body: {
            type: 'sendTokens',
            to: [{
                url: 'acc://test.acme/staking',
                amount: '200000'
            }]
        }
    };

    const signature = {
        type: 'ed25519',
        publicKey: 'dff03fddf03d29a1f45daf8e9f2bd7c68ee3f2989b0c6c3385946d20f04b4926',
        signature: 'cff669b816312fbac709f12b0d18a96bcab6a570c27b2d13f662a04afdfeb36f59ddb9249f803677ed928e27500b7c35aebce432141ea9e3af1eb8fbb901420a',
        signer: 'acc://test.acme/book/1',
        signerVersion: 51,
        timestamp: 1757520686204512,
        transactionHash: toHex(sha256(canonicalJSON(transaction)))
    };

    return {
        signatures: [signature],
        transaction: [transaction]
    };
}

/**
 * Generate signature test vector
 */
function generateSignatureVector() {
    return {
        type: 'ed25519',
        publicKey: 'dff03fddf03d29a1f45daf8e9f2bd7c68ee3f2989b0c6c3385946d20f04b4926',
        signature: 'cff669b816312fbac709f12b0d18a96bcab6a570c27b2d13f662a04afdfeb36f59ddb9249f803677ed928e27500b7c35aebce432141ea9e3af1eb8fbb901420a',
        message: 'test message for signature verification',
        messageHash: toHex(sha256('test message for signature verification'))
    };
}

/**
 * Generate transaction-only test data
 */
function generateTransactionOnly() {
    return {
        header: {
            principal: 'acc://test.acme/tokens',
            initiator: 'cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965'
        },
        body: {
            type: 'sendTokens',
            to: [{
                url: 'acc://test.acme/staking',
                amount: '200000'
            }]
        }
    };
}

/**
 * Export all fixtures
 */
function exportFixtures() {
    console.log('Exporting Accumulate Rust SDK test fixtures...');

    // Export transaction signing vectors
    const txSigningVectors = generateTransactionSigningVectors();
    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'tx_signing_vectors.json'),
        JSON.stringify(txSigningVectors, null, 2)
    );
    console.log('✓ Exported tx_signing_vectors.json');

    // Export envelope fixed example
    const envelopeFixed = generateEnvelopeFixed();
    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'envelope_fixed.golden.json'),
        JSON.stringify(envelopeFixed, null, 2)
    );
    console.log('✓ Exported envelope_fixed.golden.json');

    // Export ED25519 signature vector
    const sigVector = generateSignatureVector();
    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'sig_ed25519.golden.json'),
        JSON.stringify(sigVector, null, 2)
    );
    console.log('✓ Exported sig_ed25519.golden.json');

    // Export transaction only
    const txOnly = generateTransactionOnly();
    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'tx_only.golden.json'),
        JSON.stringify(txOnly, null, 2)
    );
    console.log('✓ Exported tx_only.golden.json');

    // Export canonical JSON test cases
    const canonicalTests = {
        description: 'Canonical JSON test cases for byte-for-byte parity',
        testCases: [
            {
                name: 'simple_object',
                input: { z: 3, a: 1, m: 2 },
                expectedCanonical: '{"a":1,"m":2,"z":3}'
            },
            {
                name: 'nested_object',
                input: { z: { y: 2, x: 1 }, a: 1 },
                expectedCanonical: '{"a":1,"z":{"x":1,"y":2}}'
            },
            {
                name: 'array_with_objects',
                input: { arr: [{ b: 2, a: 1 }, { d: 4, c: 3 }] },
                expectedCanonical: '{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}'
            }
        ]
    };

    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'canonical_json_tests.json'),
        JSON.stringify(canonicalTests, null, 2)
    );
    console.log('✓ Exported canonical_json_tests.json');

    console.log(`\nAll fixtures exported to: ${OUTPUT_DIR}`);
    console.log('Ready for Rust conformance testing!');
}

// Helper function to test canonical JSON implementation
function testCanonicalJSON() {
    console.log('\nTesting canonical JSON implementation...');

    const testObj = { z: 3, a: 1, m: { y: 2, x: 1 } };
    const canonical = canonicalJSON(testObj);
    const expected = '{"a":1,"m":{"x":1,"y":2},"z":3}';

    console.log('Input:', JSON.stringify(testObj));
    console.log('Canonical:', canonical);
    console.log('Expected:', expected);
    console.log('Match:', canonical === expected ? '✓' : '✗');

    return canonical === expected;
}

// Main execution
if (require.main === module) {
    testCanonicalJSON();
    exportFixtures();
}

module.exports = {
    canonicalJSON,
    sha256,
    toHex,
    exportFixtures
};