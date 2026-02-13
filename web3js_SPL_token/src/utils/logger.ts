/**
 * Log utility for consistent output formatting
 */
export class Logger {
  private context: string;

  constructor(context: string) {
    this.context = context;
  }

  info(message: string): void {
    console.log(`\x1b[36m[${this.context}]\x1b[0m ${message}`);
  }

  success(message: string): void {
    console.log(`\x1b[32m[${this.context}]\x1b[0m ${message}`);
  }

  error(message: string): void {
    console.error(`\x1b[31m[${this.context}]\x1b[0m ${message}`);
  }

  warning(message: string): void {
    console.warn(`\x1b[33m[${this.context}]\x1b[0m ${message}`);
  }

  tx(signature: string): void {
    console.log(`\x1b[35m[${this.context}] Transaction:\x1b[0m https://explorer.solana.com/tx/${signature}?cluster=devnet`);
  }
}

export function createLogger(context: string): Logger {
  return new Logger(context);
}
