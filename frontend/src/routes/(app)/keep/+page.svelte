<script lang="ts">
	import { goto, refreshAll } from '$app/navigation';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';

	/** A guest signs up: the same account, with a name and a password of their own */
	const i18n = getI18n();
	const session = getSession();
	const live = getLive();
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
			const r = await post<{ username: string }>('/auth/claim', { username, email, password });
			await session.refresh();
			// Pages that read the layout's player (the chat's guard) see a member now
			await refreshAll();
			// The chat opens: its socket reconnects as a member, with the shouts
			live.stop();
			live.start();
			session.tell('welcome', i18n.t('keep.done', { name: r.username }));
			await goto('/');
		} catch (err) {
			error = i18n.error(err);
			busy = false;
		}
	}
</script>

<svelte:head><title>{i18n.t('keep.title')} · Scratch-me</title></svelte:head>

<div class="keep">
	<div>
		<h1>{i18n.t('keep.title')}</h1>
		<p class="lead">{i18n.t('keep.lead')}</p>
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
			>{busy ? i18n.t('auth.working') : i18n.t('keep.submit')}</button
		>
	</form>
</div>

<style>
	.keep {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	h1 {
		font-size: 22px;
	}
	.lead {
		color: var(--muted);
		margin-top: 6px;
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-weight: 600;
		font-size: 14px;
	}
	input {
		border: 1.5px solid var(--line);
		background: var(--night);
		border-radius: 12px;
		padding: 12px 14px;
		font-size: 16px;
	}
	.hint {
		font-weight: 400;
		color: var(--muted);
		font-size: 13px;
	}
	.err {
		color: var(--danger);
		font-weight: 600;
		font-size: 14px;
	}
</style>
