import { startAnchor } from "solana-bankrun";
import { BankrunProvider } from "anchor-bankrun";
import { PublicKey,Keypair } from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import { BN, Program } from "@coral-xyz/anchor";


const IDL = require("../target/idl/voting.json");
import { Voting } from '../target/types/voting';

const PUPPET_PROGRAM_ID = new PublicKey("5s3PtT8kLYCv1WEp6dSh3T7EuF35Z6jSu5Cvx4hWG79H");

describe('Create a system account', () => {

  it("bankrun", async () => {
    const context = await startAnchor("", [{name: "voting", programId: PUPPET_PROGRAM_ID}], []);
    const provider = new BankrunProvider(context);


    const provider_ar = anchor.AnchorProvider.env();
    // console.log("provider_ar", provider_ar);

    const payer_ar = provider_ar.wallet as anchor.Wallet;
    console.log("payer_ar", payer_ar.publicKey.toString());

    const counterKeypair = Keypair.generate();
    console.log("counterKeypair", counterKeypair.publicKey.toString());

    const puppetProgram = new Program<Voting>(
      IDL,
      provider,
    );

    const [pollAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      puppetProgram.programId
    );

    await puppetProgram.methods.initializePoll(
      new anchor.BN(1),
        new anchor.BN(0),
        new anchor.BN(1759508293),
        "test-poll",
        "description",
    ).rpc();

    const pollAccount = await puppetProgram.account.pollAccount.fetch(pollAddress);
    console.log("pollAccount结果",pollAccount);
  });

});