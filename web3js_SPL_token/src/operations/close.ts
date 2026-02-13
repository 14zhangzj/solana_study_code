import {
  Connection,
  Keypair,
  PublicKey,
} from '@solana/web3.js';
import {
  closeAccount,
  getAccount,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('CloseOperation');

/**
 * Close a token account and reclaim the rent
 */
export async function closeTokenAccount(
  connection: Connection,
  feePayer: Keypair,
  tokenAccountAddress: PublicKey,
  owner: Keypair
): Promise<string> {
  try {
    logger.info(`Closing token account ${tokenAccountAddress.toBase58()}...`);

    const signature = await closeAccount(
      connection,
      feePayer,
      tokenAccountAddress,
      owner.publicKey,
      owner,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Token account closed successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to close token account: ${error}`);
    throw error;
  }
}
