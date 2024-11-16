# Solana 学习笔记
## Solana 实现 Hello world
Solana 本地开发环境的搭建较为复杂，在初学阶段，推荐大家使用 [Solana Playgroud](https://beta.solpg.io/) 进行开发。上面有搭建好的 Solana 开发环境，并且提供了 Solana 工程的开发模板，易于初学者上手.  
![alt text](image.png)  

打开界面后点击右上角的加号创建工程，并且在工程选项中选择 Native(Rust)，接下来便可以进入工程的开发。  
### 文件目录介绍
文件创建后，我们可以看到目录中有三个文件夹，分别是 src,client,tests. 其中 src 文件夹下存储了合约工程文件 lib.rs,用来编写我们的 Solana 智能合约逻辑。 client 文件夹下面存储了 client.ts 文件，为我们提供了一个与 Solana 网络交换的前端模板程序。最后的 test 文件夹下面存储了一个对 @solana/web3.js 中 API 功能的测试文件。

![alt text](image-1.png)


### 合约逻辑编辑
打开 src 文件夹下的 lib.rs 文件，我们清除掉 Solana Playgr 给我们提供的初始模板，并贴入我们下面的 hello world 程序代码。  
 
```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts:&[AccountInfo],
    _instruction_data:&[u8],
) -> ProgramResult {
    msg!("Hello world!!");
    Ok(())
}

```

完成代码后，我们点击界面左侧的 build & deploy 图标，之后对我们的工程进行 build，build 在控制台显示成功后，点击下方的 deploy 进行合约部署，这样我们就完成了一个Solana 智能合约在 devnet 网络上的部署

![alt text](image-2.png)

需要注意的是，部署合约需要 Solana 作为 gas ，如果账户内 Solana 不足的话，可以在命令行执行下面语句申请 Solana 空投

```shell
solana airdrop 2
```


### 前端文件改写
进入 client 下的 client.ts 文件夹。复制下面内容

```typescript
transaction.add({
  keys:[],
  programId:new web3.PublicKey(pg.PROGRAM_ID),
})
const txHash = await web3.sendAndConfirmTransaction(
  pg.connection,
  transaction,
  [pg.wallet.keypair],
)

console.log(txHash);
```
