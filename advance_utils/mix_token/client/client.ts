import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";



const PROGRAM_ID = pg.PROGRAM_ID;

// 代理混币指令数据结构
class MixInstruction {
  constructor(properties) {
    Object.assign(this, properties);
  }
}

// Borsh 序列化模式
const MixInstructionSchema = new Map([
  [
    MixInstruction,
    {
      kind: "struct",
      fields: [
        ["instruction_type", "u8"], // 0 = InitializeMixWithProxy
        ["amount", "u64"],
        ["mix_layers", "u8"],
        ["seed", "u64"],
      ],
    },
  ],
]);

async function generateIntermediatePDA(programId, seed, layer) {
  const seedBytes = Buffer.alloc(8);
  seedBytes.writeBigUInt64LE(BigInt(seed), 0);

  const layerBytes = Buffer.alloc(1);
  layerBytes.writeUInt8(layer, 0);

  const [pda, bump] = await PublicKey.findProgramAddress(
    [Buffer.from("mix_intermediate"), seedBytes, layerBytes],
    programId
  );

  return [pda, bump];
}

async function createProxyMixTransaction(
  proxyPayer,
  recipient,
  amount,
  mixLayers,
  seed
) {
  console.log(`创建${mixLayers}层代理混币交易`);
  console.log(`支付者: ${proxyPayer.publicKey.toString()}`);
  console.log(`支付者: ${proxyPayer.publicKey.toString()}`);
  console.log(`最终接收者: ${recipient.toString()}`);
  console.log(`金额: ${amount / LAMPORTS_PER_SOL} SOL`);
  console.log(`种子: ${seed}`);

  // 生成所有中间 PDA 地址
  const intermediatePDAs = [];
  for (let i = 0; i < mixLayers; i++) {
    const [pda, bump] = await generateIntermediatePDA(PROGRAM_ID, seed, i);
    intermediatePDAs.push(pda);
    console.log(`第${i + 1}层 PDA: ${pda.toString()}`);
  }

  // 构造指令数据
  const instructionData = new MixInstruction({
    instruction_type: 0, // InitializeMixWithProxy
    amount: BigInt(amount),
    mix_layers: mixLayers,
    seed: BigInt(seed),
  });

  // 序列化指令数据
  const serializedData = borsh.serialize(MixInstructionSchema, instructionData);

  // 构造账户列表
  const accounts = [
    { pubkey: feePayer.publicKey, isSigner: true, isWritable: true }, // 交易费用支付者
    { pubkey: proxyPayer.publicKey, isSigner: true, isWritable: true }, // 代理支付者
    { pubkey: recipient, isSigner: false, isWritable: true }, // 接收人
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // 系统程序
  ];

  // 添加所有中间 PDA 账户
  for (const pda of intermediatePDAs) {
    accounts.push({ pubkey: pda, isSigner: false, isWritable: true });
  }

  // 创建交易指令
  const instruction = new TransactionInstruction({
    keys: accounts,
    programId: PROGRAM_ID,
    data: Buffer.from(serializedData),
  });

  // 创建交易
  const transaction = new Transaction().add(instruction);

  return transaction;
}

async function executeProxyMixTransaction(
  feePayer,
  proxyPayer,
  recipient,
  amount,
  mixLayers
) {
  try {
    const connection = pg.connection;
    const seed = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);

    // 检查代理支付者余额
    const payerBalance = await connection.getBalance(proxyPayer.publicKey);
    console.log(`代理支付者余额: ${payerBalance / LAMPORTS_PER_SOL} SOL`);

    if (payerBalance < amount) {
      throw new Error("代理支付者余额不足");
    }

    // 检查接收者初始余额
    const initialRecipientBalance = await connection.getBalance(recipient);
    console.log(
      `接收者初始余额: ${initialRecipientBalance / LAMPORTS_PER_SOL} SOL`
    );

    // 创建交易
    const transaction = await createProxyMixTransaction(
      proxyPayer,
      recipient,
      amount,
      mixLayers,
      seed
    );

    // 发送并确认交易
    console.log("发送代理混币交易...");
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [feePayer,proxyPayer],
      {
        commitment: "confirmed",
        preflightCommitment: "confirmed",
      }
    );

    console.log(`代理混币交易成功! 签名: ${signature}`);
    console.log(
      `浏览器查看: https://solscan.io/tx/${signature}?cluster=devnet`
    );

    // 检查接收人余额变化
    const finalRecipientBalance = await connection.getBalance(recipient);
    console.log(
      `接收者最终余额: ${finalRecipientBalance / LAMPORTS_PER_SOL} SOL`
    );
    console.log(
      `接收金额: ${
        (finalRecipientBalance - initialRecipientBalance) / LAMPORTS_PER_SOL
      } SOL`
    );
  } catch (error) {
    console.error("代理混币交易失败:", error);
    throw error;
  }
}

const proxyPayer = pg.wallet.keypair;
const recipient = new PublicKey("");  // address to receive solana
const feePayer = Keypair.fromSecretKey(
  new Uint8Array([]) //your fee payer private key
);
console.log(`feePayer = ${feePayer.publicKey.toString()}`);
executeProxyMixTransaction(
  feePayer,
  proxyPayer,
  recipient,
  0.1 * LAMPORTS_PER_SOL,
  4
);
