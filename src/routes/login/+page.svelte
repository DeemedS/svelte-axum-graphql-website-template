<script lang="ts">
	import { LOGIN_MUTATION } from '$lib/graphql/mutations';
	import { gql } from '$lib/graphql/client';
	import FormInput from '$components/FormInput.svelte';
	import { isRequired, isEmail, passwordRules } from '$lib/validators';
	import { goto } from '$app/navigation';

	let email = '';
	let password = '';
	let emailTouched = false;
	let passwordTouched = false;
	let success = false;
	let loading = false;
	let serverError: string | null = null;

	$: emailError = emailTouched ? isRequired(email, 'Email') || isEmail(email) : undefined;
	$: passwordError = passwordTouched ? passwordRules(password) : undefined;
	$: formValid = email && password && !emailError && !passwordError;

	async function handleSubmit(e: Event) {
		e.preventDefault();
		emailTouched = true;
		passwordTouched = true;

		if (!formValid) return;

		loading = true;
		serverError = null;

		try {
			const data = await gql<{
				login: {
					success: boolean;
					message: string;
					accessToken: string;
					refreshToken: string;
				};
			}>(LOGIN_MUTATION, { email, password });

			const loginData = data.login;
			if (!loginData) throw new Error('Unexpected server response');

			if (loginData.success) {
				success = true;
				goto('/dashboard');
			} else {
				serverError = loginData.message || 'Invalid credentials.';
			}
		} catch (err: any) {
			serverError = err.message || 'An unexpected error occurred.';
		} finally {
			loading = false;
		}
	}
</script>

<main class="flex min-h-screen items-center justify-center bg-gray-100 p-6">
	<form
		class="w-full max-w-md space-y-4 rounded-2xl bg-white p-8 shadow-lg"
		on:submit={handleSubmit}
	>
		<h1 class="text-xl font-semibold">Login</h1>
		<p class="text-sm text-gray-500">Welcome back! Please enter your details.</p>

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

		<button
			type="submit"
			disabled={!formValid || loading}
			class="w-full rounded-lg bg-blue-600 py-2 font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50"
		>
			{loading ? 'Logging in…' : 'Login'}
		</button>

		<div class="mt-4 text-sm text-gray-600">
			Don’t have an account?
			<a href="/register" class="text-blue-600 hover:underline">Sign up</a>
		</div>
	</form>
</main>
