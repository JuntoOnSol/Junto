import * as anchor from "@project-serum/anchor";
import { Keypair, Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";
import fs from "fs";
import dotenv from "dotenv";

dotenv.config();

const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

const PROGRAM_ID = new PublicKey("Junto1111111111111111111111111111111111111");
const IDL_PATH = "./target/idl/junto.json";
const keypairPath = process.env.WALLET_PATH || "~/.config/solana/id.json";

async function deployProgram() {
    try {
        const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8"));
        const program = new anchor.Program(idl, PROGRAM_ID, provider);
        
        console.log("Deploying Junto DAO program...");
        console.log("Program ID:", PROGRAM_ID.toString());
        console.log("Wallet:", keypairPath);

        // Example: Initialize DAO after deployment
        const daoState = Keypair.generate();
        await program.rpc.initialize(new PublicKey(process.env.AUTHORITY), new anchor.BN(100), new anchor.BN(86400), {
            accounts: {
                daoState: daoState.publicKey,
                governanceMint: new PublicKey(process.env.GOVERNANCE_MINT),
                payer: provider.wallet.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
            signers: [daoState],
        });
        console.log("Junto DAO successfully initialized.");
    } catch (error) {
        console.error("Deployment failed:", error);
    }
}

deployProgram();

