import { BrowserProvider, ethers } from 'ethers';

export async function connectWallet(): Promise<string | null> {
	if (!(window as any).ethereum) return null;
	const provider = new BrowserProvider((window as any).ethereum);
	const accounts = await provider.send('eth_requestAccounts', []);
	return accounts[0]; // get first account
}

export function getEthersProvider() {
	if ((window as any).ethereum) {
		return new BrowserProvider((window as any).ethereum);
	}
	return null;
}

export async function signMessage(message: string) {
	const provider = getEthersProvider();
	if (!provider) throw new Error('No wallet found');
	const signer = await provider.getSigner();
	return signer.signMessage(message);
}
