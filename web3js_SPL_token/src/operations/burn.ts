import {
  Connection,
  Keypair,
  PublicKey,
} from '@solana/web3.js';
import {
  getAccount,
  burn,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { createLogger } from '../utils/logger';

const logger = createLogger('BurnOperation');

/**
 * Burn tokens from a token account
 */
export async function burnTokens(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  account: Keypair,
  owner: PublicKey,
  amount: number
): Promise<string> {
  try {
    logger.info(`Burning ${amount} tokens...`);

    // Get token account info
    const tokenAccount = await getAccount(connection, account.publicKey);

    // Burn tokens
    const signature = await burn(
      connection,
      payer,
      tokenAccount.mint,
      tokenAccount.address,
      owner,
      amount,
      [],
      TOKEN_PROGRAM_ID
    );

    logger.success('Tokens burned successfully!');
    logger.tx(signature);
    return signature;
  } catch (error) {
    logger.error(`Failed to burn tokens: ${error}`);
    throw error;
  }
}
