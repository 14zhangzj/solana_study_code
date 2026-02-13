import {
  Connection,
  Keypair,
  PublicKey,
} from '@solana/web3.js';
import {
  approve,
  revoke,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('ApproveOperation');

/**
 * Approve a delegate to transfer tokens from an account
 */
export async function approveDelegate(
  connection: Connection,
  owner: Keypair,
  mint: PublicKey,
  accountAddress: PublicKey,
  delegate: PublicKey,
  amount: number
): Promise<string> {
  try {
    logger.info(`Approving delegate ${delegate.toBase58()} for ${amount} tokens...`);

    const signature = await approve(
      connection,
      owner,
      accountAddress,
      delegate,
      owner,
      amount,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Delegate approved successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to approve delegate: ${error}`);
    throw error;
  }
}

/**
 * Revoke delegate authority
 */
export async function revokeDelegate(
  connection: Connection,
  owner: Keypair,
  mint: PublicKey,
  accountAddress: PublicKey
): Promise<string> {
  try {
    logger.info('Revoking delegate authority...');

    const signature = await revoke(
      connection,
      owner,
      accountAddress,
      owner,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Delegate authority revoked successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to revoke delegate: ${error}`);
    throw error;
  }
}
