import { Connection, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';

/**
 * Get SOL balance of an account
 */
export async function getBalance(connection: Connection, publicKey: PublicKey): Promise<number> {
  const balance = await connection.getBalance(publicKey);
  return balance / LAMPORTS_PER_SOL;
}

/**
 * Airdrop SOL to an account (devnet/testnet only)
 */
export async function requestAirdrop(
  connection: Connection,
  publicKey: PublicKey,
  amount: number = 1
): Promise<string> {
  const signature = await connection.requestAirdrop(publicKey, amount * LAMPORTS_PER_SOL);
  await connection.confirmTransaction(signature, 'confirmed');
  return signature;
}

/**
 * Wait for transaction confirmation
 */
export async function waitForConfirmation(
  connection: Connection,
  signature: string
): Promise<void> {
  await connection.confirmTransaction(signature, 'confirmed');
}
