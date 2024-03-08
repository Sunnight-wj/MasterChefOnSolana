import * as anchor from "@coral-xyz/anchor";
import { Program, Wallet } from "@coral-xyz/anchor";
import { MasterChef } from "../target/types/master_chef";
import { Keypair, PublicKey, } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, createMint, mintTo, createAccount } from "@solana/spl-token";


import * as dotenv from "dotenv";

dotenv.config();


async function readyMint(connection: anchor.web3.Connection, payer: anchor.web3.Signer): Promise<([PublicKey, PublicKey])> {
  let rewardMint = await createMint(connection, payer, payer.publicKey, payer.publicKey, 9);
  console.log(`reward mint: ${rewardMint}`);
  
  let lpMint = await createMint(connection, payer, payer.publicKey, payer.publicKey, 9);
  console.log(`lp mint: ${lpMint}`);
  
  return [rewardMint, lpMint];
}

async function mint(connection: anchor.web3.Connection, payer: anchor.web3.Signer, lpMint: PublicKey, payerLpTokenAccount: PublicKey) {
  console.log("44444444")
    // create token account
  payerLpTokenAccount = await createAccount(connection, payer, lpMint, payer.publicKey);
  // mint to payer
  await mintTo(connection, payer, lpMint, payerLpTokenAccount, payer, 500000);
}

describe("master-chef", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as Wallet;
  const program = anchor.workspace.MasterChef as Program<MasterChef>;

  let masterChef: anchor.web3.Keypair;
  let rewardMint: PublicKey;
  let lpMint: PublicKey;
  let lpTokenVaultAuthority: PublicKey;
  let lpTokenVault: PublicKey;
  let rewardTokenVaultAuthority: PublicKey;
  let rewardTokenVault: PublicKey;
  let userInfoAccount: PublicKey;
  let userLpTokenAccount: PublicKey;
 
  before(async () => {
    masterChef = Keypair.generate();
    console.log(`masterChef: ${masterChef.publicKey}`);

    console.log("111111");
    [rewardMint, lpMint] = await readyMint(provider.connection, payer.payer);
    console.log("2222222");
    [lpTokenVaultAuthority] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("lp_token_vault_auth"),
        lpMint.toBuffer(),
        masterChef.publicKey.toBuffer()
      ],
      program.programId,
    );

    [lpTokenVault] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("lp_token_vault"),
        lpMint.toBuffer(),
        masterChef.publicKey.toBuffer(),
      ],
      program.programId,
    );

    [rewardTokenVaultAuthority] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_token_vault_auth"),
        lpMint.toBuffer(),
        masterChef.publicKey.toBuffer(),
      ],
      program.programId
    );

    [rewardTokenVault] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("reward_token_vault"),
        lpMint.toBuffer(),
        masterChef.publicKey.toBuffer(),
      ],
      program.programId,
    );

    [userInfoAccount] = PublicKey.findProgramAddressSync(
      [
        payer.publicKey.toBuffer(),
        lpMint.toBuffer(),
        masterChef.publicKey.toBuffer(),
      ],
      program.programId
    );

    console.log(`userInfo account: ${userInfoAccount}`);

    userLpTokenAccount = getAssociatedTokenAddressSync(lpMint, payer.publicKey);

    console.log("33333333");
    await mint(provider.connection, payer.payer, lpMint, userLpTokenAccount);

    console.log(`lp token vault: ${lpTokenVault}\nlp token vault authority: ${lpTokenVaultAuthority}\nreward token vault: ${rewardTokenVault}\nrewward token vault authority: ${rewardTokenVaultAuthority}`)
    console.log("befor end...")
  
  });

  
  it("initialize!", async () => {
    console.log("initialize start...")
    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accounts({
        admin: payer.publicKey,
        masterChef: masterChef.publicKey,
      })
      .signers([masterChef])
      .rpc();
    const masterChefAccount = await program.account.masterChef.fetch(masterChef.publicKey);
    console.log(`masterChef account: ${JSON.stringify(masterChefAccount)}`)
    console.log("initialize successfully, transaction signature is:", tx);
  });

  it("add pool!", async () => {
    let rewardPerSlot = new anchor.BN(0);
    let startSlot = new anchor.BN(0);

    const tx = await program.methods
      .addPool(rewardMint, lpMint, rewardPerSlot, startSlot)
      .accounts({
        admin: payer.publicKey,
        masterChef: masterChef.publicKey,
        lpMint,
        rewardMint,
        rewardTokenVault,
        rewardTokenVaultAuthority,
        lpTokenVault,
        lpTokenVaultAuthority,
      })
      .rpc();
    try {
      const masterChefAccount = await program.account.masterChef.fetch(masterChef.publicKey);
      console.log(`masterChef account: ${JSON.stringify(masterChefAccount)}`)
    } catch (err) {
      console.log("error: ", err)
    }
    // mint to pool
    await mintTo(provider.connection, payer.payer, rewardMint, rewardTokenVault, payer.publicKey, 20000000);

    console.log("add pool suceess, transaction signature is:", tx);
  });

  it("update reward per slot!", async () => {
    const tx = await program.methods
      .updateRewardPerSlot(lpMint, new anchor.BN("1"))
      .accounts({
        admin: payer.publicKey,
        masterChef: masterChef.publicKey,
      })
      .rpc();
 
    console.log(`update reward per slot success, transaction signature is:`, tx);
  });

  it("set admin!", async () => {
    let config = {
      admin: payer.publicKey
    }
    const tx = await program.methods
      .setAdmin(config)
      .accounts({
        admin: payer.publicKey,
        masterChef: masterChef.publicKey,
      })
      .rpc();
    console.log(`set admin success, transaction signature is:`, tx);
  });

  it("depsoit!", async () => {
    const tx = await program.methods
      .deposit(lpMint, new anchor.BN("200"))
      .accounts({
        masterChef: masterChef.publicKey,
        lpTokenVault,
        user: payer.publicKey,
        userInfo: userInfoAccount,
        userLpTokenAccount,
      })
      .rpc();
    console.log(`deposit success, transaction signature is:`, tx);
  });

  it("withdraw!", async () => {
    const tx = await program.methods
      .withdraw(lpMint, new anchor.BN("100"))
      .accounts({
        masterChef: masterChef.publicKey,
        lpTokenVault,
        user: payer.publicKey,
        userInfo: userInfoAccount,
        lpTokenVaultAuthority,
        userLpTokenAccount,
      })
      .rpc();
    console.log(`withdraw success, transaction signature is:`, tx);
  });

  it("claim reward!", async () => {
    let userRewardTokenAccount = getAssociatedTokenAddressSync(rewardMint, payer.publicKey);
    const tx = await program.methods
      .claimReward(lpMint)
      .accounts({
        masterChef: masterChef.publicKey,
        user: payer.publicKey,
        userRewardTokenAccount,
        userInfo: userInfoAccount,
        rewardTokenVault,
        rewardTokenVaultAuthority,
        rewardMint,
      })
      .rpc({skipPreflight: true});
    const userInfo = await program.account.userInfo.fetch(userInfoAccount);
    console.log(`user info: ${JSON.stringify(userInfo)}`);
    console.log(`claim reward success, transaction signature is:`, tx);
  }) 

});
