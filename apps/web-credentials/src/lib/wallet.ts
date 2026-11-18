import {
  getAddress,
  getNetworkDetails,
  isConnected,
  requestAccess,
  signTransaction,
} from "@stellar/freighter-api";
import { TESTNET_PASSPHRASE } from "./config";

/** Connects to the Freighter extension and returns the account address. Testnet only. */
export async function connectWallet(): Promise<string> {
  const connection = await isConnected();
  if (connection.error || !connection.isConnected) {
    throw new Error("Freighter wallet not found. Install the extension and reload.");
  }
  const access = await requestAccess();
  if (access.error) throw new Error(access.error.message ?? "Wallet access was denied");
  const network = await getNetworkDetails();
  if (network.error || network.networkPassphrase !== TESTNET_PASSPHRASE) {
    throw new Error("Switch Freighter to the Test network. Only testnet is supported.");
  }
  return access.address;
}

/** The currently selected wallet address, or `null` if the wallet is not connected. */
export async function currentAddress(): Promise<string | null> {
  const result = await getAddress();
  return result.error || !result.address ? null : result.address;
}

/** Asks the wallet to sign a transaction. */
export async function signWithWallet(xdr: string, address: string): Promise<string> {
  const signed = await signTransaction(xdr, { networkPassphrase: TESTNET_PASSPHRASE, address });
  if (signed.error) throw new Error(signed.error.message ?? "The transaction was not signed");
  return signed.signedTxXdr;
}
