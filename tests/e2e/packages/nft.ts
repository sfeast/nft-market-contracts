import { config } from "dotenv";
import { TestConfig } from "../packages/configure"
import { CEP47Client } from "casper-cep47-js-client";
import { sleep, getDeploy } from "../utils";

import {
    RuntimeArgs,
    DeployUtil,
    CLValueBuilder,
    CLByteArray,
    CLKey,
    Keys
} from "casper-js-sdk";

export class NFTTester {
    nodeAddress: string;
    paymentAmounts: any;
    cep47: CEP47Client;
    nftContractHash: string;
    nftContractPackageHash: string;
    marketContractHash: string;
    marketContractPackageHash: string;

    constructor(public testConfig: TestConfig) {
        config({ path: this.testConfig.configPath });
        const {
            NODE_ADDRESS,
            MINT_ONE_PAYMENT_AMOUNT,
            TRANSFER_PAYMENT_AMOUNT,
            DEPLOY_PAYMENT_AMOUNT
        } = process.env;

        this.nodeAddress = NODE_ADDRESS!;

        this.paymentAmounts = {
            mint: MINT_ONE_PAYMENT_AMOUNT!,
            transfer: TRANSFER_PAYMENT_AMOUNT!,
            deploy: DEPLOY_PAYMENT_AMOUNT!
        }

        this.cep47 = this.testConfig.cep47;
        this.nftContractHash = this.testConfig.nftContractHash;
        this.nftContractPackageHash = this.testConfig.nftContractPackageHash;
        this.marketContractHash = this.testConfig.marketContractHash;
        this.marketContractPackageHash = this.testConfig.marketContractPackageHash;
    }

    public async contractInfo() {
        const name = await this.cep47.name();
        console.log(`... Contract name: ${name}`);

        const symbol = await this.cep47.symbol();
        console.log(`... Contract symbol: ${symbol}`);

        const meta = await this.cep47.meta();
        console.log(`... Contract meta: ${JSON.stringify(meta)}`);

        let totalSupply = await this.cep47.totalSupply();
        console.log(`... Total supply: ${totalSupply}`);
    };

    // //****************//
    // //*     Mint     *//
    // //****************//
    public async mint(keys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log(`... Mint token ${token_id} \n`);

        const mintDeploy = await this.cep47.mint(
            keys.publicKey,
            [token_id],
            [new Map([
                ['number', 'one']
            ])],
            this.paymentAmounts.mint,
            keys.publicKey,
            [keys]
        );

        const mintDeployHash = await mintDeploy.send(this.nodeAddress);

        console.log("...... Mint deploy hash: ", mintDeployHash);

        await getDeploy(this.nodeAddress, mintDeployHash);
        console.log("...... Token minted successfully");

        // //* Checks after mint *//
        const balanceOf = await this.cep47.balanceOf(keys.publicKey);

        console.log('...... Balance of master account: ', balanceOf);

        let ownerOfToken = await this.cep47.getOwnerOf(token_id);

        console.log(`...... Owner of token ${token_id}: `, ownerOfToken);

        const tokenMeta = await this.cep47.getTokenMeta(token_id);

        console.log(`...... Token ${token_id} metadata: `, tokenMeta);

        const indexByToken = await this.cep47.getIndexByToken(keys.publicKey, token_id);
        console.log(`...... index of token ${token_id}: `, indexByToken);

        const tokenByIndex = await this.cep47.getTokenByIndex(keys.publicKey, indexByToken);
        console.log(`...... token ${token_id} id: `, tokenByIndex);
    }

    //************//
    //* Transfer *//
    //************//
    public async transfer(senderKeys: Keys.AsymmetricKey, recipientKeys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log(`... Transfer #${token_id}\n`);

        let ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`...... Owner of token ${token_id} is ${ownerOfToken}`);

        // const transferOneRecipient = CLPublicKey.fromHex("016e5ee177b4008a538d5c9df7f8beb392a890a06418e5b9729231b077df9d7215");
        const transferRecipient = recipientKeys.publicKey;
        const transferDeploy = await this.cep47.transfer(
            transferRecipient,
            [token_id],
            this.paymentAmounts.transfer,
            senderKeys.publicKey,
            [senderKeys]
        );

        console.log(`...... Transfer from ${senderKeys.publicKey.toAccountHashStr()} to ${transferRecipient.toAccountHashStr()}`);

        const transferHash = await transferDeploy.send(this.nodeAddress);

        console.log("...... Transfer deploy hash: ", transferHash);

        await getDeploy(this.nodeAddress, transferHash);
        console.log("...... Token transfered successfully");

        ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`...... Owner of token ${token_id} is ${ownerOfToken}`);

        console.log('\n*************************\n');
    }

    // ********************//
    // * Approve Contract *//
    // ********************//
    public async approveContractForTransfer(approverKeys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log('... Contract Approve\n');

        // const contract_hash_str="a26fac2936d939f924c187dca44d88e70815b58f5056ada663abd875ab51a346";
        // don't want the hash- prefix (or contract-) since it gets contructed here as a byte array:
        const hex = Uint8Array.from(Buffer.from(this.marketContractPackageHash.replace('hash-', ''), "hex"));
        const contract_hash = new CLKey(new CLByteArray(hex)); //could just use new CLByteArray(hex) & skip .value()

        const approveDeploy = await this.cep47.approve(
            contract_hash.value(),
            [token_id],
            this.paymentAmounts.deploy,
            approverKeys!.publicKey,
            [approverKeys]
        );

        const approveDeployHash = await approveDeploy.send(this.nodeAddress);

        console.log("...... Approval deploy hash: ", approveDeployHash);

        await getDeploy(this.nodeAddress, approveDeployHash);
        console.log("...... Token approved successfully");

        //   // ** Checks after approval **//
        const allowanceOfToken = await this.cep47.getAllowance(approverKeys.publicKey, token_id);
        console.log(`...... Allowance of token ${token_id}: ${allowanceOfToken}`);

        console.log('\n*************************\n');
    };

}