import * as anchor from "@coral-xyz/anchor";
import {Program,BN} from "@coral-xyz/anchor";
import {Counter} from "../target/types/counter"
import {PublicKey} from "@solana/web3.js";


async function main() {
  const provide = anchor.AnchorProvider.env();
  
  anchor.setProvider(provide)

  const counter = new web3.Keypair()
  const programId = new PublicKey("AmRzTv3uRJcHw87ym7bhWMrC6HAYuTPn5VddFLMTqiHt")
  const program = anchor.workspace.Counter

  console.log("initializing account")

  await program.methods.initialize(new BN(87))
  .accounts({
    counter:counter.publicKey,
    user:pg.wallet.publicKey,
    systemProgram:anchor.web3.SystemProgram.programId
  })
  .signers([counter])
  .rpc({'commitment':'confirmed',maxRetries:3})

  let counterAccount = await program.account.counter.fetch(counter.publicKey);
  console.log(`initial value ${counterAccount.value.toString()}`)

  await program.methods.increment()
  .accounts({counter:counter.publicKey,owner:pg.wallet.publicKey})
  .signers([pg.wallet.keypair])
  .rpc();

  counterAccount = await program.account.counter.fetch(counter.publicKey);
  console.log(`increment value ${counterAccount.value.toString()}`)
}

main()
