/**
 * Token mint information
 */
export interface TokenInfo {
  mintAddress: string;
  decimals: number;
  authority?: string;
  freezeAuthority?: string;
}

/**
 * Token account information
 */
export interface TokenAccountInfo {
  address: string;
  mint: string;
  owner: string;
  amount: number;
}

/**
 * Operation result
 */
export interface OperationResult {
  success: boolean;
  signature?: string;
  error?: string;
}
