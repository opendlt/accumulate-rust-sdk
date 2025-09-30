#!/usr/bin/env node

/**
 * Random Transaction Envelope Generator for Rust Fuzzing
 *
 * Generates N random transaction envelopes with fixed seed PRNG
 * for deterministic fuzz testing between TypeScript and Rust SDKs.
 */

import crypto from 'crypto';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Configuration
const DEFAULT_COUNT = 1000;
const CI_COUNT = 200;
const MAX_COUNT = 2000;
const SEED = 'accumulate-rust-ts-fuzz-seed-2024';

// Get count from environment or use default
const fuzzCount = process.env.TS_FUZZ_N ?
    Math.min(parseInt(process.env.TS_FUZZ_N), MAX_COUNT) :
    DEFAULT_COUNT;

console.log(`Generating ${fuzzCount} random transaction envelopes...`);

/**
 * Seeded PRNG for deterministic random generation
 */
class SeededRandom {
    constructor(seed) {
        this.seed = crypto.createHash('sha256').update(seed).digest();
        this.state = new Uint32Array(4);
        for (let i = 0; i < 4; i++) {
            this.state[i] = this.seed.readUInt32BE(i * 4);
        }
    }

    // XorShift128 algorithm for deterministic random
    next() {
        let t = this.state[3];
        let s = this.state[0];
        this.state[3] = this.state[2];
        this.state[2] = this.state[1];
        this.state[1] = s;

        t ^= t << 11;
        t ^= t >>> 8;
        t ^= s ^ (s >>> 19);
        this.state[0] = t;

        return (t >>> 0) / 0x100000000;
    }

    // Random integer in range [min, max)
    int(min, max) {
        return Math.floor(this.next() * (max - min)) + min;
    }

    // Random hex string of specified length
    hex(length) {
        const chars = '0123456789abcdef';
        let result = '';
        for (let i = 0; i < length; i++) {
            result += chars[this.int(0, 16)];
        }
        return result;
    }

    // Random choice from array
    choice(array) {
        return array[this.int(0, array.length)];
    }

    // Random string with specified length
    string(length) {
        const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
        let result = '';
        for (let i = 0; i < length; i++) {
            result += chars[this.int(0, chars.length)];
        }
        return result;
    }
}

/**
 * Canonical JSON implementation - matches Accumulate protocol requirements
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
    return crypto.createHash('sha256').update(data).digest('hex');
}

/**
 * Binary encoding for transaction envelope
 * This is a simplified implementation for testing purposes
 */
function encodeTransactionEnvelope(envelope) {
    // In a real implementation, this would use proper binary encoding
    // For testing, we'll use a deterministic binary representation
    const json = JSON.stringify(envelope);
    const buffer = Buffer.from(json, 'utf8');

    // Add some binary headers to simulate real encoding
    const header = Buffer.alloc(8);
    header.writeUInt32BE(0x41434355, 0); // "ACCU" magic bytes
    header.writeUInt32BE(buffer.length, 4); // payload length

    return Buffer.concat([header, buffer]);
}

/**
 * Generate random transaction types
 */
const TRANSACTION_TYPES = [
    'send-tokens',
    'create-identity',
    'create-token-account',
    'create-data-account',
    'write-data',
    'create-key-page',
    'create-key-book',
    'add-credits',
    'update-key',
    'system-genesis'
];

/**
 * Generate random URL
 */
function generateRandomUrl(rng) {
    const authorities = ['alice.acme', 'bob.acme', 'test.acme', 'demo.acme', 'fuzz.acme'];
    const paths = ['tokens', 'data', 'keys', 'staking', 'identity', 'book', 'page'];

    const authority = rng.choice(authorities);
    const hasPath = rng.next() > 0.3;

    if (hasPath) {
        const path = rng.choice(paths);
        const hasIndex = rng.next() > 0.5;
        if (hasIndex) {
            return `acc://${authority}/${path}/${rng.int(0, 100)}`;
        }
        return `acc://${authority}/${path}`;
    }

    return `acc://${authority}`;
}

/**
 * Generate random transaction header
 */
function generateRandomHeader(rng) {
    return {
        principal: generateRandomUrl(rng),
        timestamp: 1234567890000 + rng.int(0, 1000000000), // deterministic but varied
        ...(rng.next() > 0.7 && { initiator: rng.hex(64) }) // sometimes add initiator
    };
}

/**
 * Generate random transaction body based on type
 */
function generateRandomBody(rng, txType) {
    const body = { type: txType };

    switch (txType) {
        case 'send-tokens':
            body.to = [];
            const numRecipients = rng.int(1, 4);
            for (let i = 0; i < numRecipients; i++) {
                body.to.push({
                    url: generateRandomUrl(rng),
                    amount: String(rng.int(1, 1000000))
                });
            }
            break;

        case 'create-identity':
            body.url = generateRandomUrl(rng);
            body.keyBook = {
                publicKeyHash: rng.hex(64)
            };
            break;

        case 'create-token-account':
            body.url = generateRandomUrl(rng);
            body.tokenUrl = generateRandomUrl(rng);
            if (rng.next() > 0.5) {
                body.keyBook = {
                    publicKeyHash: rng.hex(64)
                };
            }
            break;

        case 'write-data':
            body.entry = {
                data: [rng.hex(rng.int(10, 200))]
            };
            if (rng.next() > 0.6) {
                body.scratch = true;
            }
            break;

        case 'create-key-page':
            body.keys = [];
            const numKeys = rng.int(1, 5);
            for (let i = 0; i < numKeys; i++) {
                body.keys.push({
                    publicKeyHash: rng.hex(64),
                    priority: rng.int(0, 256)
                });
            }
            break;

        case 'add-credits':
            body.recipient = generateRandomUrl(rng);
            body.amount = String(rng.int(1000, 1000000));
            break;

        default:
            // Generic body for other types
            body.data = rng.hex(rng.int(20, 100));
            break;
    }

    return body;
}

/**
 * Generate random signature
 */
function generateRandomSignature(rng, txHash) {
    return {
        type: 'ed25519',
        publicKey: rng.hex(64),
        signature: rng.hex(128),
        signer: generateRandomUrl(rng) + '/book/' + rng.int(0, 10),
        signerVersion: rng.int(1, 100),
        timestamp: 1700000000000 + rng.int(0, 100000000),
        transactionHash: txHash
    };
}

/**
 * Generate a random transaction envelope
 */
function generateRandomEnvelope(rng, index) {
    const txType = rng.choice(TRANSACTION_TYPES);

    const transaction = {
        header: generateRandomHeader(rng),
        body: generateRandomBody(rng, txType)
    };

    // Calculate canonical JSON and hash
    const canonicalJson = canonicalJSON(transaction);
    const txHashHex = sha256(canonicalJson);

    // Generate random signatures (1-3 signatures)
    const numSignatures = rng.int(1, 4);
    const signatures = [];
    for (let i = 0; i < numSignatures; i++) {
        signatures.push(generateRandomSignature(rng, txHashHex));
    }

    const envelope = {
        signatures,
        transaction: [transaction]
    };

    // Encode to binary
    const binaryData = encodeTransactionEnvelope(envelope);
    const hexBin = binaryData.toString('hex');

    return {
        hexBin,
        canonicalJson,
        txHashHex,
        meta: {
            index,
            txType,
            numSignatures,
            binarySize: binaryData.length,
            canonicalSize: canonicalJson.length
        }
    };
}

/**
 * Main function to generate and export random vectors
 */
function exportRandomVectors() {
    const rng = new SeededRandom(SEED);

    console.log(`Using seed: ${SEED}`);
    console.log(`Generating ${fuzzCount} vectors...`);

    for (let i = 0; i < fuzzCount; i++) {
        const vector = generateRandomEnvelope(rng, i);

        // Output as JSON Lines format
        console.log(JSON.stringify(vector));

        // Progress indicator
        if ((i + 1) % 100 === 0) {
            console.error(`Generated ${i + 1}/${fuzzCount} vectors...`);
        }
    }

    console.error(`✓ Generated ${fuzzCount} random transaction envelopes`);
    console.error(`✓ Use: cargo test --test ts_fuzz_roundtrip to verify`);
}

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
    exportRandomVectors();
}

export {
    generateRandomEnvelope,
    canonicalJSON,
    sha256,
    SeededRandom
};