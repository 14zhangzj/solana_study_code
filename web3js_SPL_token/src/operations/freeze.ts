import {
  Connection,
  Keypair,
  PublicKey,
} from '@solana/web3.js';
import {
  getAccount,
  freezeAccount,
  thawAccount,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('FreezeOperation');

/**
 * Freeze a token account (requires freeze authority)
 */
export async function freezeTokenAccount(
  connection: Connection,
  feePayer: Keypair,
  mint: PublicKey,
  targetAccount: PublicKey,
  freezeAuthority: Keypair
): Promise<string> {
  try {
    logger.info(`Freezing token account ${targetAccount.toBase58()}...`);

    const signature = await freezeAccount(
      connection,
      feePayer,
      targetAccount,
      mint,
      freezeAuthority,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Token account frozen successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to freeze account: ${error}`);
    throw error;
  }
}

/**
 * Thaw (unfreeze) a token account (requires freeze authority)
 */
export async function thawTokenAccount(
  connection: Connection,
  feePayer: Keypair,
  mint: PublicKey,
  targetAccount: PublicKey,
  freezeAuthority: Keypair
): Promise<string> {
  try {
    logger.info(`Thawing token account ${targetAccount.toBase58()}...`);

    const signature = await thawAccount(
      connection,
      feePayer,
      targetAccount,
      mint,
      freezeAuthority,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Token account thawed successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to thaw account: ${error}`);
    throw error;
  }
}
