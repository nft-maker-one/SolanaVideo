use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};


// 混币合约的主要指令类型
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum MixInstruction {
    InitializeMix {
        // 转账数额
        amount: u64,
        //代理层级
        mix_layers: u8,
        // 随机种子
        seed: u64,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MixState {
    // 是否已初始化
    pub is_initialized: bool,
    // 当前混币层数
    pub current_layer: u8,
    // 总混币层数
    pub total_layers: u8,
    // 剩余金额
    pub remaining_amount: u64,
    // 最终目标地址
    pub final_recipient: Pubkey,
    // 用于生成PDA的种子
    pub seed: u64,
}

// 自定义错误类型
#[derive(Debug, Clone)]
pub enum MixError {

    InvalidInstructionData,
    NotEnoughAccounts,
    InvalidMixLayers,
    InsufficientFunds,
    PDADerivationFailed,
}

impl From<MixError> for ProgramError {
    fn from(e: MixError) -> Self {
        ProgramError::Custom(e as u32)
    }
}


entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts:&'a [AccountInfo<'a>],
    instruction_data:&[u8]
)->ProgramResult {
    let instruction = try_from_slice_unchecked::<MixInstruction>(instruction_data)
    .map_err(|_| MixError::InvalidInstructionData)?;

    match instruction {
        MixInstruction::InitializeMix { amount, mix_layers, seed } => {
            process_initialize_mix(program_id,accounts,amount,mix_layers,seed)
        }
    }
}


fn process_initialize_mix<'a>(
    program_id: &Pubkey,
    accounts:&'a [AccountInfo<'a>],
    amount:u64,
    mix_layer:u8,
    seed: u64
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proxy_payer = next_account_info(account_iter)?;
    msg!("proxy_payer = {}",proxy_payer.key);
    let payer = next_account_info(account_iter)?;
    let recipent = next_account_info(account_iter)?;
    let system_account = next_account_info(account_iter)?;
    msg!("payer.lamports = {} amount = {}",payer.lamports(),amount);
    if payer.lamports() < amount {
        return Err(MixError::InsufficientFunds.into());
    }
    if accounts.len() as u8 != mix_layer+4 {
        return  Err(MixError::NotEnoughAccounts.into());
    }

    if mix_layer==0 || mix_layer>=5 {
        return  Err(MixError::InvalidMixLayers.into());
    }

    let mut pda_accounts = Vec::new();
    for _ in 0..mix_layer {
        let account = next_account_info(account_iter)?;
        pda_accounts.push(account);
    }

    let first_pda = pda_accounts[0];

    let (expected_pda,_bump_seed) = generate_intermediate_pda(program_id, seed, 0)?;
    if *first_pda.key != expected_pda {
        msg!("第一个中间 pda 账户不符合标准，预期 {} 实际 {}",expected_pda,first_pda.key);
        return Err(MixError::PDADerivationFailed.into())
    }

    invoke(
        &system_instruction::transfer(payer.key, first_pda.key, amount),
        &[
            payer.clone(),
            first_pda.clone(),
            system_account.clone()
        ]
    )?;


    let mut current_sender = first_pda;

    for layer in 1..mix_layer {
        let current_receiver = pda_accounts[layer as usize];
        let (expect_pda,_bump) = generate_intermediate_pda(program_id, seed, layer)?;
        if *current_receiver.key != expect_pda {
            msg!("第 {} 个中间 pda 账户不符合标准，预期 {} 实际 {}",layer+1,expected_pda,first_pda.key);
            return Err(MixError::PDADerivationFailed.into());
        }

        let prev_bump = generate_intermediate_pda(program_id, seed, layer-1)?.1;
        let signer_seed = generate_pda_signer_seeds(seed, layer-1, prev_bump);
        let signer_seed_ref:Vec<&[u8]> = signer_seed.iter().map(|s| s.as_slice()).collect();
        invoke_signed(
            &system_instruction::transfer(current_sender.key, current_receiver.key, amount),
        &[
            current_sender.clone(),
            current_receiver.clone(),
            system_account.clone()
        ],
        &[&signer_seed_ref]
        )?;

        current_sender = current_receiver;
    }

    let last_bump = generate_intermediate_pda(program_id, seed, mix_layer-1)?.1;
    let signer_seeds = generate_pda_signer_seeds(seed, mix_layer-1, last_bump);
    let signer_seed_ref:Vec<&[u8]> = signer_seeds.iter().map(|s| s.as_slice()).collect();
        
    invoke_signed(
        &system_instruction::transfer(current_sender.key, recipent.key, amount),
        &[
            current_sender.clone(),
            recipent.clone(),
            system_account.clone()
        ],
        &[&signer_seed_ref]
        )?;

        Ok(())
}




/// 生成中间账户的PDA
fn generate_intermediate_pda(
    program_id: &Pubkey,
    seed: u64,
    layer: u8,
) -> Result<(Pubkey, u8), ProgramError> {
    let seed_bytes = seed.to_le_bytes();
    let layer_bytes = layer.to_le_bytes();

    let (pubkey, bump) = Pubkey::find_program_address(
        &[b"mix_intermediate", &seed_bytes, &layer_bytes],
        program_id,
    );

    Ok((pubkey, bump))
}

/// 生成PDA签名种子 - 返回种子向量
fn generate_pda_signer_seeds(seed: u64, layer: u8, bump_seed: u8) -> Vec<Vec<u8>> {
    vec![
        b"mix_intermediate".to_vec(),
        seed.to_le_bytes().to_vec(),
        layer.to_le_bytes().to_vec(),
        vec![bump_seed],
    ]
}