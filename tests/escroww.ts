import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { assert, expect } from "chai";
import { Escroww } from "../target/types/escroww";

describe("escroww", async () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Escroww as Program<Escroww>;
  const buyer = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([
      150, 236, 19, 248, 58, 254, 112, 201, 182, 226, 5, 146, 219, 168, 39, 211,
      127, 4, 89, 185, 213, 218, 234, 54, 153, 209, 215, 120, 239, 51, 132, 225,
      85, 162, 141, 205, 180, 254, 168, 20, 50, 143, 204, 136, 202, 242, 12,
      227, 45, 12, 102, 95, 175, 144, 176, 136, 196, 25, 149, 180, 62, 189, 2,
      57,
    ])
  );

  const seller = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([
      183, 241, 7, 165, 83, 145, 73, 226, 167, 241, 135, 222, 59, 142, 159, 159,
      79, 2, 70, 170, 6, 224, 96, 149, 56, 169, 59, 26, 78, 4, 83, 36, 137, 247,
      62, 253, 124, 54, 182, 49, 254, 82, 52, 122, 171, 181, 23, 99, 126, 43,
      104, 65, 196, 133, 60, 220, 149, 70, 130, 18, 121, 175, 181, 62,
    ])
  );

  const VAULT_SEED = Buffer.from("1011");

  const [vaultAddress, escrowBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [VAULT_SEED],
      program.programId
    );

  const calculateTxnBump = async (id) =>
    await anchor.web3.PublicKey.findProgramAddress(
      [new BN(id).toBuffer("le", 8), VAULT_SEED],
      program.programId
    );

  const getLatestTxnId = async () =>
    (await program.account.vault.fetch(vaultAddress)).numTransactions;

  it("Is initialized!", async () => {
    console.log("Buyer PK", buyer.publicKey.toString());
    console.log("Seller PK", seller.publicKey.toString());
    // console.log("Already initialized");
    // return;
    const tx = await program.methods
      .initialize(buyer.publicKey, seller.publicKey, escrowBump)
      .accounts({
        vault: vaultAddress,
        buyer: buyer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("should create a new transaction", async () => {
    const txns = await (
      await program.account.vault.fetch(vaultAddress)
    ).numTransactions;

    console.log(txns);

    const [txnKey, txnBump] = await calculateTxnBump(await getLatestTxnId());
    const amount = anchor.web3.LAMPORTS_PER_SOL * 2;
    const tx = await program.methods
      .initiateTransaction(new BN(amount), txnBump)
      .accounts({
        vault: vaultAddress,
        transaction: txnKey,
        buyer: buyer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Buyer should confirm the latest transaction", async () => {
    const [txnKey, txnBump] = await calculateTxnBump(
      await (await getLatestTxnId()).sub(new BN(1))
    );
    const txn = await program.methods
      .confirmTransaction()
      .accounts({
        confirmationBy: buyer.publicKey,
        transaction: txnKey,
        vault: vaultAddress,
      })
      .signers([buyer])
      .rpc();
    const vTxn = await program.account.transaction.fetch(txnKey);
    assert(vTxn.verifications[0] == true, "Confirmation not recorded by Buyer");
    console.log(vTxn.verifications);
  });

  it("Seller should confirm the latest transaction", async () => {
    const [txnKey, txnBump] = await calculateTxnBump(
      await (await getLatestTxnId()).sub(new BN(1))
    );
    const txn = await program.methods
      .confirmTransaction()
      .accounts({
        confirmationBy: seller.publicKey,
        transaction: txnKey,
        vault: vaultAddress,
      })
      .signers([seller])
      .rpc();
    const vTxn = await program.account.transaction.fetch(txnKey);
    assert(
      vTxn.verifications[1] == true,
      "Confirmation note recorded by seller"
    );
    console.log(vTxn.verifications);
  });

  it("should execute the transaction", async () => {
    const [txnKey, txnBump] = await calculateTxnBump(
      await (await getLatestTxnId()).sub(new BN(1))
    );
    await program.methods
      .executeTransaction()
      .accounts({
        executionBy: buyer.publicKey,
        vault: vaultAddress,
        transaction: txnKey,
        seller: seller.publicKey,
      })
      .signers([buyer])
      .rpc();

    const txn = await program.account.transaction.fetch(txnKey);
    console.log(txn);
  });
});
