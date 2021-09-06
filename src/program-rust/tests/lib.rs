use borsh::BorshDeserialize;
use solana_bpf_flavotes::{process_instruction, FlavourAccount};
use solana_program_test::{*, tokio};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::mem;


#[tokio::test]
async fn test_flavotes() {

    println!("Instruction data: {:?}", vec![1].extend("mango".as_bytes().to_vec()));

    let program_id = Pubkey::new_unique();
    let flavours_pubkey = Pubkey::new_unique();
    // let flavours_pubkey = Pubkey::new_unique();


    let mut program_test = ProgramTest::new(
        "flavotes", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );
    program_test.add_account(
        flavours_pubkey,
        Account {
            lamports: 10000,
            data: vec![0_u8; mem::size_of::<u32>()],
            owner: program_id,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Verify account has zero greetings
    let flavour_account = banks_client
        .get_account(flavours_pubkey)
        .await
        .expect("get_account")
        .expect("flavours_pubkey not found");
    assert_eq!(
        FlavourAccount::try_from_slice(&flavour_account.data)
            .unwrap().flavours.len(),
        0
    );


    // Greet once
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[1], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(flavours_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let flavour_account = banks_client
        .get_account(flavours_pubkey)
        .await
        .expect("get_account")
        .expect("flavours_pubkey not found");
    assert_eq!(
        FlavourAccount::try_from_slice(&flavour_account.data)
            .unwrap().flavours.len(),
        1
    );
    

    // // Verify account has one greeting
    // let greeted_account = banks_client
    //     .get_account(greeted_pubkey)
    //     .await
    //     .expect("get_account")
    //     .expect("greeted_account not found");
    // assert_eq!(
    //     GreetingAccount::try_from_slice(&greeted_account.data)
    //         .unwrap()
    //         .counter,
    //     1
    // );

    // // Greet again
    // let mut transaction = Transaction::new_with_payer(
    //     &[Instruction::new_with_bincode(
    //         program_id,
    //         &[1], // ignored but makes the instruction unique in the slot
    //         vec![AccountMeta::new(greeted_pubkey, false)],
    //     )],
    //     Some(&payer.pubkey()),
    // );
    // transaction.sign(&[&payer], recent_blockhash);
    // banks_client.process_transaction(transaction).await.unwrap();

    // // Verify account has two greetings
    // let greeted_account = banks_client
    //     .get_account(greeted_pubkey)
    //     .await
    //     .expect("get_account")
    //     .expect("greeted_account not found");
    // assert_eq!(
    //     GreetingAccount::try_from_slice(&greeted_account.data)
    //         .unwrap()
    //         .counter,
    //     2
    // );
}
