<script lang="ts">
import { connectWallet, signMessage } from '$lib/eth';
let address: string | null = null;


async function onConnect() {
address = await connectWallet();
}


async function checkBalance() {
if (!address) return alert('connect first');
const res = await fetch('/api/read_balance_proxy', {
method: 'POST',
headers: { 'content-type': 'application/json' },
body: JSON.stringify({ contract: '0xTOKEN_ADDRESS', owner: address })
});
const j = await res.json();
alert('balance: ' + j.balance);
}
</script>


<button on:click={onConnect}>{address ? address : 'Connect Wallet'}</button>
<button on:click={checkBalance}>Check Token Balance</button>