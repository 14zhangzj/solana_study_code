import {
  Connection,
  Keypair,
  PublicKey,
} from '@solana/web3.js';
import {
  getOrCreateAssociatedTokenAccount,
  transfer,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('TransferOperation');

/**
 * Transfer tokens from one account to another
 */
export async function transferTokens(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  fromOwner: Keypair,
  toDestination: PublicKey,
  amount: number
): Promise<string> {
  try {
    logger.info(`Transferring ${amount} tokens to ${toDestination.toBase58()}...`);

    // Get or create source token account
    const sourceAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      fromOwner.publicKey
    );

    // Get or create destination token account
    const destinationAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      toDestination
    );

    // Transfer tokens
    const signature = await transfer(
      connection,
      payer,
      sourceAccount.address,
      destinationAccount.address,
      fromOwner,
      amount,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Tokens transferred successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to transfer tokens: ${error}`);
    throw error;
  }
}
