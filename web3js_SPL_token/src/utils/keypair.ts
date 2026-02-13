import * as fs from 'fs';
import * as path from 'path';
import { Keypair } from '@solana/web3.js';

const KEYPair_DIR = path.join(process.cwd(), '.keypairs');
const WALLET_FILE = path.join(KEYPair_DIR, 'wallet.json');

/**
 * Ensure the keypairs directory exists
 */
export function ensureKeypairDir(): void {
  if (!fs.existsSync(KEYPair_DIR)) {
    fs.mkdirSync(KEYPair_DIR, { recursive: true });
  }
}

/**
 * Load a keypair from a file
 */
export function loadKeypairFromFile(filePath: string = WALLET_FILE): Keypair {
  if (!fs.existsSync(filePath)) {
    throw new Error(`Keypair file not found: ${filePath}`);
  }

  const secretKey = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
  return Keypair.fromSecretKey(new Uint8Array(secretKey));
}

/**
 * Save a keypair to a file
 */
export function saveKeypairToFile(keypair: Keypair, filePath: string = WALLET_FILE): void {
  ensureKeypairDir();
  fs.writeFileSync(filePath, JSON.stringify(Array.from(keypair.secretKey)));
  console.log(`Keypair saved to: ${filePath}`);
}

/**
 * Load or create a wallet keypair
 */
export function getOrCreateWallet(): Keypair {
  try {
    return loadKeypairFromFile();
  } catch (error) {
    console.log('Creating new wallet...');
    const newWallet = Keypair.generate();
    saveKeypairToFile(newWallet);
    console.log(`New wallet address: ${newWallet.publicKey.toBase58()}`);
    return newWallet;
  }
}

/**
 * Generate a new keypair
 */
export function generateKeypair(): Keypair {
  return Keypair.generate();
}
