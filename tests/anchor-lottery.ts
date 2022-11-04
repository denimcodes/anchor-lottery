import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import { createAccount, createMint, mintTo } from "@solana/spl-token";
import { assert } from "chai";
import { AnchorLottery } from "../target/types/anchor_lottery";

describe("anchor-lottery", () => {
	// Configure the client to use the local cluster.
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const manager = provider.wallet.publicKey;
	const payer = (provider.wallet as NodeWallet).payer;
	const program = anchor.workspace.AnchorLottery as Program<AnchorLottery>;

	let lottery;
	let lotteryToken = anchor.web3.Keypair.generate();

	const playerOne = anchor.web3.Keypair.generate();
	const playerTwo = anchor.web3.Keypair.generate();
	const playerThree = anchor.web3.Keypair.generate();

	let tokenMint;
	let playerOneToken;
	let playerTwoToken;
	let playerThreeToken;

	before(async () => {
		[lottery] = await anchor.web3.PublicKey.findProgramAddress(
			[anchor.utils.bytes.utf8.encode("lottery"), payer.publicKey.toBuffer()],
			program.programId
		);
		tokenMint = await createMint(
			provider.connection,
			payer,
			manager,
			manager,
			8
		);

		playerOneToken = await createAccount(
			provider.connection,
			payer,
			tokenMint,
			playerOne.publicKey
		);
		await mintTo(
			provider.connection,
			payer,
			tokenMint,
			playerOneToken,
			manager,
			10_000_000_000
		);

		playerTwoToken = await createAccount(
			provider.connection,
			payer,
			tokenMint,
			playerTwo.publicKey
		);
		await mintTo(
			provider.connection,
			payer,
			tokenMint,
			playerTwoToken,
			manager,
			10_000_000_000
		);

		playerThreeToken = await createAccount(
			provider.connection,
			payer,
			tokenMint,
			playerThree.publicKey
		);
		await mintTo(
			provider.connection,
			payer,
			tokenMint,
			playerThreeToken,
			manager,
			10_000_000_000
		);
	});

	it("Initiliaze lottery", async () => {
		await program.methods
			.initLottery()
			.accounts({
				manager,
				tokenMint,
				lottery,
				tokenAccount: lotteryToken.publicKey,
			})
			.signers([lotteryToken])
			.rpc();
	});

	it("Enter players for lottery", async () => {
		const amount = new anchor.BN(10);

		await program.methods
			.enterPlayer(amount)
			.accounts({
				player: playerOne.publicKey,
				lottery,
				playerTokenAccount: playerOneToken,
				lotteryTokenAccount: lotteryToken.publicKey,
			})
			.signers([playerOne])
			.rpc();

		await program.methods
			.enterPlayer(amount)
			.accounts({
				player: playerTwo.publicKey,
				lottery,
				playerTokenAccount: playerTwoToken,
				lotteryTokenAccount: lotteryToken.publicKey,
			})
			.signers([playerTwo])
			.rpc();

		await program.methods
			.enterPlayer(amount)
			.accounts({
				player: playerThree.publicKey,
				lottery,
				playerTokenAccount: playerThreeToken,
				lotteryTokenAccount: lotteryToken.publicKey,
			})
			.signers([playerThree])
			.rpc();

		const lotteryAccount = await program.account.lottery.fetch(lottery);
		assert(lotteryAccount.tokenAmount.toNumber() === 30);
	});

	it("Pick winner", async () => {
		await program.methods
			.pickWinner()
			.accounts({ manager: manager, lottery })
			.rpc();
	});

	it("claim prize", async () => {
		const lotteryAccount = await program.account.lottery.fetch(lottery);

		try {
			await program.methods
				.claimPrize()
				.accounts({
          manager,
					playerTokenAccount: playerOneToken,
					lotteryTokenAccount: lotteryToken.publicKey,
					lottery,
				})
				.signers([lotteryToken])
				.rpc();
		} catch (e) {
			console.error(e);
		}

    try {
			await program.methods
				.claimPrize()
				.accounts({
          manager,
					playerTokenAccount: playerTwoToken,
					lotteryTokenAccount: lotteryToken.publicKey,
					lottery,
				})
				.signers([lotteryToken])
				.rpc();
    } catch (error) {
      console.error(error);
    }

    try {
			await program.methods
				.claimPrize()
				.accounts({
          manager,
					playerTokenAccount: playerThreeToken,
					lotteryTokenAccount: lotteryToken.publicKey,
					lottery,
				})
				.signers([lotteryToken])
				.rpc();
    } catch (error) {
      console.error(error);
    }
    
		console.log(`Amount: ${lotteryAccount.tokenAmount}`);
	});
});
