import { Connection, PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import dotenv from "dotenv";

dotenv.config();

class SolanaService {
  private connection: Connection;

  constructor() {
    this.connection = new Connection(process.env.SOLANA_RPC_URL || "https://api.mainnet-beta.solana.com", "confirmed");
  }

  public async getAccountInfo(publicKey: string): Promise<any> {
    try {
      const accountInfo = await this.connection.getAccountInfo(new PublicKey(publicKey));
      return accountInfo;
    } catch (error) {
      throw new Error("Failed to fetch account info: " + error);
    }
  }

  public async sendTransaction(transaction: Transaction, signer: any): Promise<string> {
    try {
      const signature = await sendAndConfirmTransaction(this.connection, transaction, [signer]);
      return signature;
    } catch (error) {
      throw new Error("Transaction failed: " + error);
    }
  }
}

export default new SolanaService();
