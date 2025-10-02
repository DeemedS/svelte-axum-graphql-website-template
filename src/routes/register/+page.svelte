<script lang="ts">
	import { gql } from '$lib/graphql/client';
	import { REGISTER_MUTATION } from '$lib/graphql/mutations';
	import FormInput from '$components/FormInput.svelte';
	import { isRequired, isEmail, passwordRules, confirmPassword } from '$lib/validators';

	let email = '';
	let password = '';
	let confirm = '';

	let emailTouched = false;
	let passwordTouched = false;
	let confirmTouched = false;

	let success = false;
	let loading = false;
	let serverError: string | null = null;

	// --- reactive validations ---
	$: emailError = emailTouched ? isRequired(email, 'Email') || isEmail(email) : undefined;
	$: passwordError = passwordTouched ? passwordRules(password) : undefined;
	$: confirmError = confirmTouched ? isRequired(confirm, 'Confirm Password') || confirmPassword(confirm, password) : undefined;

	$: formValid = email && password && confirm && !emailError && !passwordError && !confirmError;

	async function handleSubmit(e: Event) {
		e.preventDefault();

		emailTouched = true;
		passwordTouched = true;
		confirmTouched = true;

		if (!formValid) return;

		loading = true;
		serverError = null;

		try {
			const data = await gql<{ register: boolean }>(REGISTER_MUTATION, { email, password });
			if (data.register === true) {
				success = true;
				password = confirm = '';
			} else {
				serverError = 'Registration failed.';
			}
		} catch (err: any) {
			serverError = err.message;
		} finally {
			loading = false;
		}
	}
</script>

<main class="flex min-h-screen items-center justify-center bg-gray-100 p-6">
	{#if success}
		<div class="w-full max-w-md rounded-2xl bg-white p-8 shadow-lg text-center">
			<h1 class="text-xl font-semibold text-green-600">Registration Successful!</h1>
			<div class="mt-4">
				<a href="/login" class="text-blue-600 hover:underline font-semibold">Go to Login</a>
			</div>
		</div>
	{:else}
		<form
			class="w-full max-w-md space-y-4 rounded-2xl bg-white p-8 shadow-lg"
			on:submit|preventDefault={handleSubmit}
		>
			<h1 class="text-xl font-semibold">Register</h1>
			<p class="text-sm text-gray-500">Create an account using your email and password.</p>

			{#if serverError}
				<div class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700">
					{serverError}
				</div>
			{/if}

			<FormInput
				id="email"
				label="Email"
				type="email"
				bind:value={email}
				error={emailError}
				touched={emailTouched}
				required
				onBlur={() => (emailTouched = true)}
			/>

			<FormInput
				id="password"
				label="Password"
				type="password"
				bind:value={password}
				error={passwordError}
				touched={passwordTouched}
				required
				onBlur={() => (passwordTouched = true)}
			/>

			<FormInput
				id="confirm"
				label="Confirm Password"
				type="password"
				bind:value={confirm}
				error={confirmError}
				touched={confirmTouched}
				required
				onBlur={() => (confirmTouched = true)}
			/>

			<button
				type="submit"
				disabled={!formValid || loading}
				class="w-full rounded-lg bg-blue-600 py-2 font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50"
			>
				{loading ? 'Registering…' : 'Register'}
			</button>

			<div class="mt-4 text-sm text-gray-600">
				Already have an account?
				<a href="/login" class="text-blue-600 hover:underline">Sign in</a>
			</div>
		</form>
	{/if}
</main>
