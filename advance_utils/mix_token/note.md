## 程序架构
```
传统转账：A → B (直接可见)
混币转账：A → 中间地址1 → 中间地址2 → ... → B (增强隐私)
```

## pda 签名
pda 账户生成算法  
```rust
let (pda, bump) = Pubkey::find_program_address(
    &[b"mix_intermediate", &seed_bytes, &layer_bytes],
    program_id,
);
```
pda invoke 交易
```rust
// 普通账户转账 (有私钥)
invoke(&instruction, &accounts)?;

// PDA 转账 (程序代理签名)
invoke_signed(&instruction, &accounts, &[&signer_seeds])?;
```



