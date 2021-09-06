use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    decode_error::DecodeError,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VoteError {
    #[error("Unexpected Candidate")]
    UnexpectedCandidate,
    #[error("Incorrect Owner")]
    IncorrectOwner,
    #[error("Account Not Rent Exempt")]
    AccountNotRentExempt,
    #[error("Account Not Check Account")]
    AccountNotCheckAccount,
    #[error("Already Voted")]
    AlreadyVoted,
}
impl From<VoteError> for ProgramError {
    fn from(e: VoteError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for VoteError {
    fn type_of() -> &'static str {
        "Vote Error"
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Flavour {
    pub vote_count: usize,
    pub id: usize,
}


// state of Flavour Account
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct FlavourAccount {
    pub flavours: Vec<Flavour>,
}

// state for check account
// Check if voter has already voted, and who for
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VoterCheck {
    pub voted_for: u32,
}


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,      // Public key of program account
    accounts: &[AccountInfo], // data accounts
    instruction_data: &[u8],  // [0] -> 0/1; addFlavour {name: String}, vote {id: u8}
) -> ProgramResult {
    msg!("Rust program entrypoint");

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account that holds the vote count
    let flavour_account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if flavour_account.owner != program_id {
        msg!(
            "Flavour account ({}) not owned by program, actual: {}, expected: {}",
            flavour_account.key,
            flavour_account.owner,
            program_id
        );
        return Err(VoteError::IncorrectOwner.into());
    }

    

    let mut flavour_data = FlavourAccount::try_from_slice(&flavour_account.data.borrow_mut())?;

    msg!("Loaded flavours account!");


    // Add new flavour
    if instruction_data[0] == 1 {
        msg!("Adding Flavour!");
        msg!("Yum, new flavour ID {}", &flavour_data.flavours.len()+1);
        let flavour = Flavour {
            id: flavour_data.flavours.len()+1,
            vote_count: 0,
        };

        // push the flavour on to flavour account data
        flavour_data.flavours.push(flavour);

        // serialize back the data
        // flavour_data.serialize(&mut &mut flavour_account.data.borrow_mut()[..])?;
        msg!("New flavour added!")
    }
    else{
        msg!("Voting for {}", instruction_data[0]);
        // Get the account that checks for dups
        let check_account = next_account_info(accounts_iter)?;

        // The check account must be owned by the program in order to modify its data
        if check_account.owner != program_id {
            msg!("Check account not owned by program");
            return Err(VoteError::IncorrectOwner.into());
        }

        // the voter
        let voter_account = next_account_info(accounts_iter)?;

        if !voter_account.is_signer {
            msg!("Voter account is not signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let expected_check_account_pubkey =
            Pubkey::create_with_seed(voter_account.key, "checkvote", program_id)?;

        if expected_check_account_pubkey != *check_account.key {
            msg!("Voter fraud! not the correct check_account");
            return Err(VoteError::AccountNotCheckAccount.into());
        }

        let vote_check = VoterCheck::try_from_slice(&check_account.data.borrow())?;

        if vote_check.voted_for != 0 {
            msg!("Voter fraud! You already voted");
            return Err(VoteError::AlreadyVoted.into());
        }
        // Start voting
        let vote = instruction_data[1] as usize;
        msg!("Voting for flavour id {:?}!", &vote);

        // push the flavour on to flavour account data
        flavour_data.flavours[vote].vote_count += 1;

        // serialize back the data
        flavour_data.serialize(&mut &mut flavour_account.data.borrow_mut()[..])?;
        msg!(
            "Voted successfully! Thank you for participating, Vote count: {:?}",
            flavour_data.flavours[vote].vote_count
        )
    }
    
    Ok(())
}
