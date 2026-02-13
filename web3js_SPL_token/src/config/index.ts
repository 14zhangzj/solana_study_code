import { Connection, clusterApiUrl } from '@solana/web3.js';

// Network configuration
export const NETWORK = process.env.NETWORK || 'devnet';
export const RPC_URL = process.env.RPC_URL || clusterApiUrl(NETWORK as 'devnet' | 'testnet' | 'mainnet-beta');

// Create connection instance
export const connection = new Connection(RPC_URL, 'confirmed');

// Token configuration
export const TOKEN_CONFIG = {
  decimals: 6,
  // Add your token metadata here
  name: 'My Token',
  symbol: 'MTK',
} as const;
