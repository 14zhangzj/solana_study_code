import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from '@solana/web3.js';
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('MintOperation');

/**
 * Create a new token mint
 */
export async function createTokenMint(
  connection: Connection,
  payer: Keypair,
  decimals: number,
  mintAuthority?: PublicKey,
  freezeAuthority?: PublicKey
): Promise<PublicKey> {
  try {
    logger.info('Creating new token mint...');

    const mint = await createMint(
      connection,
      payer,
      mintAuthority || payer.publicKey,
      freezeAuthority || payer.publicKey,
      decimals
    );

    logger.success(`Token mint created: ${mint.toBase58()}`);
    return mint;
  } catch (error) {
    logger.error(`Failed to create mint: ${error}`);
    throw error;
  }
}

/**
 * Mint tokens to a token account
 */
export async function mintTokensTo(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  amount: number
): Promise<string> {
  try {
    logger.info(`Minting ${amount} tokens to ${destination.toBase58()}...`);

    const signature = await mintTo(
      connection,
      payer,
      mint,
      destination,
      payer,
      amount
    );

    logger.success(`Tokens minted successfully!`);
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to mint tokens: ${error}`);
    throw error;
  }
}

/**
 * Create an associated token account for a wallet
 */
export async function createTokenAccount(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  try {
    logger.info('Creating associated token account...');

    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      owner
    );

    logger.success(`Token account created: ${tokenAccount.address.toBase58()}`);
    return tokenAccount.address;
  } catch (error) {
    logger.error(`Failed to create token account: ${error}`);
    throw error;
  }
}
