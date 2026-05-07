const {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  Transaction,
  Connection,
  TransactionInstruction,
  SystemProgram,
} = require("@solana/web3.js");
const fs = require("fs");
const { keccak256 } = require("js-sha3");

const programId = new PublicKey("7Nj7p7MehfVAGkwoSUzHpBkgsHaHy55o3hBUyuZH1Aet");

const payer = Keypair.fromSecretKey(
  Uint8Array.from(
    JSON.parse(fs.readFileSync("/home/divyesh/.config/solana/id.json")),
  ),
);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// ── Helpers ───────────────────────────────────────────────────────────────────

function hashPassword(password) {
  const hash = keccak256.array(Buffer.from(password, "utf8"));
  return Buffer.from(hash); // exactly 32 bytes
}

function serializeInitializeEscrow(amount, password) {
  const passwordHash = hashPassword(password);
  const buf = Buffer.alloc(1 + 8 + 32); // u8 variant + u64 + [u8;32]
  let offset = 0;
  buf.writeUInt8(0, offset);
  offset += 1; // ← u8 not u32
  buf.writeBigUInt64LE(BigInt(Math.floor(amount)), offset);
  offset += 8;
  passwordHash.copy(buf, offset);
  return buf;
}

// Manual deserialize matching Rust EscrowState layout exactly:
// intializer: Pubkey  (32 bytes)
// amount:     u64     (8 bytes)
// hash:       [u8;32] (32 bytes)
// claimed:    bool    (1 byte)
function deserializeEscrowState(data) {
  let offset = 0;
  const initializer = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;
  const amount = data.readBigUInt64LE(offset);
  offset += 8;
  const hash = data.slice(offset, offset + 32);
  offset += 32;
  const claimed = data[offset] === 1;
  return { initializer, amount, hash, claimed };
}

function getEscrowPDA(initializerPubkey) {
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), initializerPubkey.toBuffer()],
    programId,
  );
  return { pda, bump };
}

// ── Tests ─────────────────────────────────────────────────────────────────────

async function testInitializeEscrow() {
  console.log("\n=== Test 1: Initialize Escrow ===");

  const { pda: escrowPDA } = getEscrowPDA(payer.publicKey);
  console.log("  Escrow PDA:", escrowPDA.toBase58());

  const existing = await connection.getAccountInfo(escrowPDA);
  if (existing) {
    console.log("  Already exists, skipping.");
    return;
  }

  const escrowAmount = 0.1 * LAMPORTS_PER_SOL;
  const password = "supersecret";
  const data = serializeInitializeEscrow(escrowAmount, password);

  console.log("  Buffer hex:", data.toString("hex"));
  console.log("  Buffer length:", data.length); // must be 44

  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });

  const sig = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [payer],
  );
  console.log("  ✅ Tx:", sig);

  const accountInfo = await connection.getAccountInfo(escrowPDA);
  const state = deserializeEscrowState(accountInfo.data);
  console.log("  Initializer:", state.initializer.toBase58());
  console.log("  Amount:", state.amount.toString(), "lamports");
  console.log("  Claimed:", state.claimed);
  console.log("  PDA balance:", accountInfo.lamports, "lamports");
}

testInitializeEscrow();
