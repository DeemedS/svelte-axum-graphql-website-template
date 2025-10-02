<script lang="ts">
	export let id: string;
	export let label: string;
	export let type: string = 'text';
	export let value: string;
	export let touched: boolean = false;
	export let required: boolean = false;
	export let error: string | undefined;
	export let onBlur: () => void = () => {};
</script>

<div>
	<label for={id} class="block text-sm font-medium text-gray-700">{label}</label>
	<input
		{id}
		{type}
		bind:value
		required={required}
		on:blur={onBlur}
		class={`mt-1 block w-full rounded-lg border p-2 focus:border-blue-500 focus:ring-2 focus:ring-blue-500
			${touched && error ? 'border-red-500 text-red-600 focus:border-red-500 focus:ring-red-500' : 'border-gray-300'}`}
	/>
	{#if touched && error}
		<div class="mt-1 text-sm text-red-600">{error}</div>
	{/if}
</div>
