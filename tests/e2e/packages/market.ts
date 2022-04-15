import { config } from "dotenv";
import { TestConfig } from "../packages/configure"
import { CEP47Client } from "casper-cep47-js-client";
import { PaymentClient } from "../payment/payment_client";
import { sleep, getDeploy } from "../utils";
import { BigNumber } from '@ethersproject/bignumber';

import {
    CasperClient,
    RuntimeArgs,
    Contracts,
    DeployUtil,
    CLValueBuilder,
    Keys
} from "casper-js-sdk";

export class MarketTester {
    nodeAddress: string;
    chainName: string;
    paymentWasmPath: string;
    paymentAmounts: any;
    cep47: CEP47Client;
    client: CasperClient;
    contract: Contracts.Contract;
    paymentClient: PaymentClient;
    nftContractHash: string;
    nftContractPackageHash: string;
    marketContractHash: string;
    marketContractPackageHash: string;
    userBalances: any;

    constructor(public testConfig: TestConfig) {
        config({ path: this.testConfig.configPath });

        const {
            NODE_ADDRESS,
            CHAIN_NAME,
            PAYMENT_WASM_PATH,
            LISTING_INSTALL_PAYMENT_AMOUNT,
            OFFER_INSTALL_PAYMENT_AMOUNT,
            DEPLOY_PAYMENT_AMOUNT
        } = process.env;

        this.nodeAddress = NODE_ADDRESS!;
        this.chainName = CHAIN_NAME!;
        this.paymentWasmPath = PAYMENT_WASM_PATH!;

        this.cep47 = this.testConfig.cep47;
        this.client = this.testConfig.casperClient;
        this.contract = this.testConfig.contractClient;
        this.paymentClient = this.testConfig.paymentClient;
        this.nftContractHash = this.testConfig.nftContractHash;
        this.nftContractPackageHash = this.testConfig.nftContractPackageHash;
        this.marketContractHash = this.testConfig.marketContractHash;
        this.marketContractPackageHash = this.testConfig.marketContractPackageHash;

        this.paymentAmounts = {
            listing_install: LISTING_INSTALL_PAYMENT_AMOUNT!,
            offer_install: OFFER_INSTALL_PAYMENT_AMOUNT!,
            deploy: DEPLOY_PAYMENT_AMOUNT!
        }

        this.userBalances = [];
    }

    fromMotes(amt: any) { return amt / 1000000000 }
    toMotes(amt: any) { return amt * 1000000000 }

    /**************************/
    /******Create Listing******/
    /**************************/
    public async listForSale(sellerKeys: Keys.AsymmetricKey, token_id: string, price: string) {
        console.log('\n*************************\n');

        console.log('... List NFT for Sale\n');
        const runtimeArgs = RuntimeArgs.fromMap({
            token_id: CLValueBuilder.string(token_id),
            token_contract_hash: CLValueBuilder.string(this.nftContractHash.replace('hash', 'contract')),
            price: CLValueBuilder.u512(this.toMotes(price))
        });

        const createListingDeploy = await this.contract.callEntrypoint(
            'create_listing',
            runtimeArgs,
            sellerKeys.publicKey,
            this.chainName,
            this.paymentAmounts.deploy,
            [sellerKeys]
        );

        const createListingDeployHash = await createListingDeploy.send(this.nodeAddress);
        console.log("...... create_listing deploy hash: ", createListingDeployHash);

        await getDeploy(this.nodeAddress, createListingDeployHash);
        console.log("...... create_listing called successfully");

        const ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`...... ${ownerOfToken} has listed token ${token_id} for ${price} CSPR`);

        console.log('\n*************************\n');
    }

    /*************************/
    /*******Buy Listing*******/
    /*************************/
    public async buyListing(
        buyerKeys: Keys.AsymmetricKey,
        token_id: string,
        price: string,
        sellerKeys: Keys.AsymmetricKey //just for checking account balance
    ) {
        console.log('\n*************************\n');

        console.log('... Buy NFT Listing \n');

        const installDeployHash = await this.paymentClient.install(
            this.paymentWasmPath, {
                market_contract_hash: this.marketContractHash.replace('hash', 'contract'),
                entry_point_name: 'buy_listing',
                token_contract_hash: this.nftContractHash.replace('hash', 'contract'),
                token_id: token_id,
                amount: parseInt(price)
            },
            this.paymentAmounts.listing_install,
            buyerKeys.publicKey,
            [buyerKeys],
        );

        let buyerBalance1 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;
        let sellerBalance1 = (await this.client.balanceOfByPublicKey(sellerKeys.publicKey)).toBigInt() / 1000000000n;
        let ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`...... Current owner of token ${token_id} is ${ownerOfToken}\n`);

        const hash = await installDeployHash.send(this.nodeAddress);
        console.log(`... buy_listing deploy hash: ${hash}`);

        await getDeploy(this.nodeAddress, hash);
        console.log(`... buy_listing called successfully`);

        ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`\n...... New owner of token ${token_id} is ${ownerOfToken}\n`);

        let buyerBalance2 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;
        let sellerBalance2 = (await this.client.balanceOfByPublicKey(sellerKeys.publicKey)).toBigInt() / 1000000000n;

        // console.log('\n...... Balance Before');
        // console.log(`... seller balance: ${(sellerBalance1).toString()}`);
        // console.log(`... buyer balance : ${(buyerBalance1).toString()}`);

        // console.log('\n...... Balance After');
        // console.log(`... seller balance: ${(sellerBalance2).toString()}`);
        // console.log(`... buyer balance : ${(buyerBalance2).toString()}`);

        // console.log('\n...... Balance Changes');
        console.log(`...... Seller gain: ${(sellerBalance2 - sellerBalance1).toString()} CSPR`);
        console.log(`...... Buyer spent: ${(buyerBalance1 - buyerBalance2).toString()} CSPR`);
        console.log('\n*************************\n');
    }

    /**************************/
    /******Cancel Listing******/
    /**************************/
    public async cancelListing(sellerKeys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log('... Cancel Listing\n');
        const runtimeArgs = RuntimeArgs.fromMap({
            token_id: CLValueBuilder.string(token_id),
            token_contract_hash: CLValueBuilder.string(this.nftContractHash.replace('hash', 'contract'))
        });

        const deploy = await this.contract.callEntrypoint(
            'cancel_listing',
            runtimeArgs,
            sellerKeys.publicKey,
            this.chainName,
            this.paymentAmounts.deploy,
            [sellerKeys]
        );

        const deployHash = await deploy.send(this.nodeAddress);
        console.log("...... deploy hash: ", deployHash);

        await getDeploy(this.nodeAddress, deployHash);
        console.log("...... deploy called successfully");

        console.log(`...... canceled listing`);

        console.log('\n*************************\n');
    }

    /*************************/
    /*******Make Offer*******/
    /*************************/
    public async makeOffer(
        buyerKeys: Keys.AsymmetricKey,
        token_id: string,
        offer: string
    ) {
        console.log('\n*************************\n');

        console.log('... Make Offer \n');

        const deploy = await this.paymentClient.install(
            this.paymentWasmPath, {
                market_contract_hash: this.marketContractHash.replace('hash', 'contract'),
                entry_point_name: 'make_offer',
                token_contract_hash: this.nftContractHash.replace('hash', 'contract'),
                token_id: token_id,
                amount: parseInt(offer)
            },
            this.paymentAmounts.offer_install,
            buyerKeys.publicKey,
            [buyerKeys],
        );

        let buyerBalance1 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;

        const deployHash = await deploy.send(this.nodeAddress);
        console.log("...... deploy hash: ", deployHash);

        await getDeploy(this.nodeAddress, deployHash);
        console.log("...... deploy called successfully");

        let buyerBalance2 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;

        console.log(`...... Buyer spent: ${(buyerBalance1 - buyerBalance2).toString()} CSPR`);
        console.log('\n*************************\n');

        await this.getOfferPurseBalance();
    }


    /**************************/
    /******Withdraw Offer******/
    /**************************/
    public async withdrawOffer(buyerKeys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log('... Withdraw Offer\n');
        const runtimeArgs = RuntimeArgs.fromMap({
            token_id: CLValueBuilder.string(token_id),
            token_contract_hash: CLValueBuilder.string(this.nftContractHash.replace('hash', 'contract'))
        });

        const deploy = await this.contract.callEntrypoint(
            'withdraw_offer',
            runtimeArgs,
            buyerKeys.publicKey,
            this.chainName,
            this.paymentAmounts.deploy,
            [buyerKeys]
        );

        let buyerBalance1 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;

        const deployHash = await deploy.send(this.nodeAddress);
        console.log("...... deploy hash: ", deployHash);

        await getDeploy(this.nodeAddress, deployHash);
        console.log("...... deploy called successfully");

        console.log(`...... withdrew offer`);

        let buyerBalance2 = (await this.client.balanceOfByPublicKey(buyerKeys.publicKey)).toBigInt() / 1000000000n;

        console.log(`...... Buyer received: ${(buyerBalance2 - buyerBalance1).toString()} CSPR`);

        await this.getOfferPurseBalance();

        console.log('\n*************************\n');
    }


    /**************************/
    /******Accept Offer********/
    /**************************/
    public async acceptOffer(sellerKeys: Keys.AsymmetricKey, buyerKeys: Keys.AsymmetricKey, token_id: string) {
        console.log('\n*************************\n');

        console.log('... Accept Offer\n');
        const runtimeArgs = RuntimeArgs.fromMap({
            token_id: CLValueBuilder.string(token_id),
            token_contract_hash: CLValueBuilder.string(this.nftContractHash.replace('hash', 'contract')),
            accepted_offer: CLValueBuilder.string(buyerKeys.publicKey.toAccountHashStr())
        });

        const deploy = await this.contract.callEntrypoint(
            'accept_offer',
            runtimeArgs,
            sellerKeys.publicKey,
            this.chainName,
            this.paymentAmounts.deploy,
            [sellerKeys]
        );

        let sellerBalance1 = (await this.client.balanceOfByPublicKey(sellerKeys.publicKey)).toBigInt() / 1000000000n;
        let ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`...... Current owner of token ${token_id} is ${ownerOfToken}\n`);

        const deployHash = await deploy.send(this.nodeAddress);
        console.log("...... deploy hash: ", deployHash);

        await getDeploy(this.nodeAddress, deployHash);
        console.log("...... deploy called successfully");

        ownerOfToken = await this.cep47.getOwnerOf(token_id);
        console.log(`\n...... New owner of token ${token_id} is ${ownerOfToken}\n`);

        let sellerBalance2 = (await this.client.balanceOfByPublicKey(sellerKeys.publicKey)).toBigInt() / 1000000000n;
        console.log(`...... Seller gain: ${(sellerBalance2 - sellerBalance1).toString()} CSPR`);

        await this.getOfferPurseBalance();

        console.log('\n*************************\n');
    }

    /**************************/
    /***********Utils**********/
    /**************************/
    public async saveBalances(accounts: Keys.AsymmetricKey[]) {
        const balances = []
        for (let i=0; i<accounts.length; i++) {
            balances.push((await this.client.balanceOfByPublicKey(accounts[i].publicKey)).toBigInt() / 1000000000n);
        }
        this.userBalances.push(balances);
    }

    public async reportBalances(accounts: Keys.AsymmetricKey[]) {
        // assumes same accounts are used in both calls & only compares first & 2nd saved balances
        for (let i=0; i<this.userBalances[0].length; i++) {
            const balance1 = this.userBalances[0][i];
            const balance2 = this.userBalances[1][i];
            console.log(`...... Buyer ${i} difference: ${(balance2 - balance1).toString()} CSPR`);
        }
    }

    public async getOfferPurseBalance() {
        // how to get the uref dynamically?
        // doesn't work:
        // let purse_uref = await this.contract.queryContractData(['stored_purse']);
        // console.log(purse_uref);
        return;

        let offer_purse_balance = (await this.balanceOfByURef("uref-36f8572f0b8f54bf9a8e5dc4e246d8c84e53d1ffe33a0c18c20da30a07860d57-007")).toBigInt() / 1000000000n;
        console.log(`...... Offer purse: ${(offer_purse_balance).toString()} CSPR`);
    }

    // I think this can be simplified - for example in /packages/client-helper/src/helps/utils there's a getStateRootHash function
    public async balanceOfByURef(
        uRefStr: string
    ): Promise < BigNumber > {
        try {
            const stateRootHash = await this.client.nodeClient
                .getLatestBlockInfo()
                .then(it => it.block?.header.state_root_hash);
            // Find the balance Uref and cache it if we don't have it.
            if (!stateRootHash) {
                return BigNumber.from(0);
            }

            return await this.client.nodeClient.getAccountBalance(
                stateRootHash,
                uRefStr
            );
        } catch (e) {
            return BigNumber.from(0);
        }
    }

    // public async checkBalances(
    //     offerKeys: Keys.AsymmetricKey[]
    // ): {
    //     offerKeys.forEach(key => {
    //         const balance = (await this.client.balanceOfByPublicKey(key.publicKey)).toBigInt() / 1000000000n;
    //         console.log(`...... Seller gain: ${(balance).toString()} CSPR`);
    //     }
    // })

    // query example
    // let owner = await this.contract.queryContractData(['owner']);
    // let seller = await this.contract.queryContractData(['seller']);
    // Buffer.from(owner.value()).toString("hex")
}