import * as web3 from '@solana/web3.js';
import base58 from 'bs58';

var secretKey = "f511997af2086e5120fa855fca63475601d087072138b67f8148052ade50014c532de0f1388127e1bdb6851b21e3b3901be2a1c044773a9962f07743694e5393";

async function main() {
  const connection = new web3.Connection(web3.clusterApiUrl('mainnet-beta'), 'confirmed');
  const fromWallet = web3.Keypair.fromSecretKey(base58.decode(secretKey));
  const to = new web3.PublicKey("CyiAXTMr17ug1QgqHvJnoKNB1mkagxgZnYPPkrZXFBne");
  // Add transfer instruction to transaction
  var transaction = new web3.Transaction().add(
    web3.SystemProgram.transfer({
      fromPubkey: fromWallet.publicKey,
      toPubkey: to,
      lamports: 1, // number of SOL to send
    }),
  );

  // Sign transaction, broadcast, and confirm
  var signature = await web3.sendAndConfirmTransaction(connection, transaction, [
    fromWallet,
  ]);
  console.log('SIGNATURE', signature);
}

main()
  .then()
  .catch(err => console.log(err))

