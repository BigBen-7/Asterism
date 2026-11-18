import {
  BASE_FEE,
  Contract,
  TransactionBuilder,
  rpc,
  scValToNative,
  type xdr,
} from "@stellar/stellar-sdk";
import { config, TESTNET_PASSPHRASE } from "./config";
import { signWithWallet } from "./wallet";

/**
 * Builds, simulates, signs (with the wallet) and submits a contract call, then
 * waits for it to finish and returns the decoded return value.
 */
export async function invoke(
  contractId: string,
  method: string,
  args: xdr.ScVal[],
  source: string,
): Promise<unknown> {
  if (!contractId) throw new Error(`No contract id configured for ${method}`);
  const server = new rpc.Server(config.rpcUrl);
  const account = await server.getAccount(source);
  const transaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: TESTNET_PASSPHRASE,
  })
    .addOperation(new Contract(contractId).call(method, ...args))
    .setTimeout(60)
    .build();

  // Simulation fills in the resources, fee and authorization entries.
  const prepared = await server.prepareTransaction(transaction);
  const signedXdr = await signWithWallet(prepared.toXDR(), source);
  const sent = await server.sendTransaction(
    TransactionBuilder.fromXDR(signedXdr, TESTNET_PASSPHRASE),
  );
  if (sent.status === "ERROR") throw new Error("The network rejected the transaction");

  const result = await server.pollTransaction(sent.hash, { attempts: 30 });
  if (result.status !== rpc.Api.GetTransactionStatus.SUCCESS) {
    throw new Error(`Transaction ${sent.hash} did not succeed (${result.status})`);
  }
  return result.returnValue ? scValToNative(result.returnValue) : undefined;
}
