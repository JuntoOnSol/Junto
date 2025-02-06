import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Junto } from "../target/types/junto";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("Junto DAO - End-to-End Test Suite", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Junto as Program<Junto>;

  // Initialize Keypairs
  const daoAuthority = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const governanceMint = Keypair.generate();
  let daoState: PublicKey;
  let proposalAccount: PublicKey;

  before(async () => {
    console.log("Airdropping SOL for testing...");
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(daoAuthority.publicKey, 1e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user1.publicKey, 1e9),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user2.publicKey, 1e9),
      "confirmed"
    );

    console.log("Deploying DAO...");
    daoState = await PublicKey.createWithSeed(
      daoAuthority.publicKey,
      "dao_state",
      program.programId
    );

    // Initialize the DAO
    await program.methods
      .initialize(new anchor.BN(10), new anchor.BN(86400))
      .accounts({
        daoState,
        authority: daoAuthority.publicKey,
        governanceMint: governanceMint.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([daoAuthority])
      .rpc();

    console.log("DAO Initialized!");
  });

  it("Creates a proposal", async () => {
    proposalAccount = await PublicKey.createWithSeed(
      user1.publicKey,
      "proposal_1",
      program.programId
    );

    await program.methods
      .createProposal("Upgrade Treasury", "Allocate funds for development.")
      .accounts({
        daoState,
        signer: user1.publicKey,
        proposal: proposalAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    console.log("Proposal Created!");

    const proposal = await program.account.proposal.fetch(proposalAccount);
    assert.equal(proposal.title, "Upgrade Treasury");
    assert.equal(proposal.description, "Allocate funds for development.");
  });

  it("Casts votes on the proposal", async () => {
    await program.methods
      .vote(new anchor.BN(1), true)
      .accounts({
        proposal: proposalAccount,
        signer: user1.publicKey,
        voterTokenAccount: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    await program.methods
      .vote(new anchor.BN(1), false)
      .accounts({
        proposal: proposalAccount,
        signer: user2.publicKey,
        voterTokenAccount: user2.publicKey,
      })
      .signers([user2])
      .rpc();

    console.log("Votes Casted!");

    const updatedProposal = await program.account.proposal.fetch(proposalAccount);
    assert.isAbove(updatedProposal.votes_for.toNumber(), 0);
    assert.isAbove(updatedProposal.votes_against.toNumber(), 0);
  });

  it("Finalizes the proposal", async () => {
    await program.methods
      .finalizeProposal(new anchor.BN(1))
      .accounts({
        daoState,
        signer: daoAuthority.publicKey,
        proposal: proposalAccount,
      })
      .signers([daoAuthority])
      .rpc();

    console.log("Proposal Finalized!");

    const finalizedProposal = await program.account.proposal.fetch(proposalAccount);
    assert.oneOf(finalizedProposal.final_outcome, [1, 2]);
  });
});

