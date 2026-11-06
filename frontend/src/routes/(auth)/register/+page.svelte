<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';

	const i18n = getI18n();
	let username = $state('');
	let email = $state('');
	let password = $state('');
	let error = $state('');
	let busy = $state(false);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		busy = true;
		error = '';
		try {
			// The daily envelope follows the player's own calendar day
			const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
			await post('/auth/register', { username, email, password, tz });
			location.assign('/');
		} catch (err) {
			error = i18n.error(err);
			busy = false;
		}
	}
</script>

<svelte:head><title>{i18n.t('auth.register.submit')} · Scratch-me</title></svelte:head>

<div>
	<h1>{i18n.t('auth.register.title')}</h1>
	<p class="lead">{i18n.t('auth.register.lead')}</p>
</div>
<form onsubmit={submit} novalidate>
	<label>
		{i18n.t('auth.username')}
		<input name="username" autocomplete="username" required bind:value={username} />
		<span class="hint">{i18n.t('auth.username_hint')}</span>
	</label>
	<label>
		{i18n.t('auth.email')}
		<input name="email" type="email" autocomplete="email" required bind:value={email} />
	</label>
	<label>
		{i18n.t('auth.password')}
		<input
			name="password"
			type="password"
			autocomplete="new-password"
			required
			bind:value={password}
		/>
		<span class="hint">{i18n.t('auth.password_hint')}</span>
	</label>
	{#if error}<p class="err" role="alert">{error}</p>{/if}
	<button class="btn primary" type="submit" disabled={busy}
		>{busy ? i18n.t('auth.working') : i18n.t('auth.register.submit')}</button
	>
</form>
<p class="switch">
	{i18n.t('auth.register.has_account')} <a href="/login">{i18n.t('auth.register.login_link')}</a>
</p>
