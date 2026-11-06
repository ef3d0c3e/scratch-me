<script lang="ts">
	import type { LayoutProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	const i18n = getI18n();
	let { children }: LayoutProps = $props();
	let busy = $state(false);
	let error = $state('');

	/** Straight in, no account: signing up later keeps it all */
	async function asGuest() {
		busy = true;
		error = '';
		try {
			const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
			await post('/auth/guest', { tz });
			location.assign('/');
		} catch (err) {
			error = i18n.error(err);
			busy = false;
		}
	}
</script>

<div class="auth">
	<div class="hero" aria-hidden="true">
		<div class="stub s1"></div>
		<div class="stub s2"></div>
		<div class="stub s3"></div>
	</div>
	<div class="panel">
		<p class="logo">Scratch<span>-me</span></p>
		<p class="tagline">{i18n.t('app.tagline')}</p>
		{@render children()}
		<div class="guest">
			<span class="or">{i18n.t('auth.or')}</span>
			<button type="button" class="btn ghost" disabled={busy} onclick={asGuest}
				>{i18n.t('auth.guest')}</button
			>
			<p class="hint">{i18n.t('auth.guest_hint')}</p>
			{#if error}<p class="err" role="alert">{error}</p>{/if}
		</div>
	</div>
</div>

<style>
	.guest {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 18px;
		text-align: center;
	}
	.guest .btn {
		width: 100%;
	}
	.or {
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		opacity: 0.6;
	}
	.guest .hint {
		font-size: 12px;
		opacity: 0.75;
	}
	.auth {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: calc(24px + var(--safe-top)) 16px calc(24px + var(--safe-bottom));
	}
	.hero {
		position: relative;
		width: 220px;
		height: 120px;
	}
	.stub {
		position: absolute;
		left: 50%;
		top: 50%;
		width: 150px;
		height: 82px;
		margin: -41px 0 0 -75px;
		border-radius: 10px;
		background:
			linear-gradient(115deg, transparent 35%, rgba(255, 255, 255, 0.55) 50%, transparent 65%) 0 0 /
				250% 100%,
			linear-gradient(135deg, #d9d6e6, #9d98b5);
		box-shadow: 0 12px 30px -10px rgba(0, 0, 0, 0.6);
		border: 5px solid var(--gold);
		animation: shine 3.2s ease-in-out infinite;
	}
	.s1 {
		transform: rotate(-14deg) translateX(-38px);
		border-color: var(--pink);
	}
	.s2 {
		transform: rotate(10deg) translateX(40px);
		border-color: var(--mint);
		animation-delay: 0.6s;
	}
	.s3 {
		transform: translateY(-8px);
		animation-delay: 1.2s;
	}
	@keyframes shine {
		0%,
		60% {
			background-position:
				120% 0,
				0 0;
		}
		100% {
			background-position:
				-120% 0,
				0 0;
		}
	}
	.panel {
		width: min(420px, 100%);
		display: flex;
		flex-direction: column;
		gap: 16px;
		background: rgba(31, 18, 69, 0.85);
		border: 1px solid var(--line);
		border-radius: 24px;
		padding: 24px;
		backdrop-filter: blur(8px);
	}
	.logo {
		font: 400 30px/1 var(--display);
		color: var(--gold);
		text-align: center;
		text-shadow: 0 3px 0 #9a5a00;
	}
	.logo span {
		color: var(--pink);
		text-shadow: 0 3px 0 #8a1240;
	}
	.tagline {
		text-align: center;
		color: var(--muted);
		margin-top: -6px;
	}
	.panel :global(h1) {
		font-size: 22px;
	}
	.panel :global(.lead) {
		color: var(--muted);
		margin-top: 6px;
	}
	.panel :global(form) {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.panel :global(label) {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-weight: 600;
		font-size: 14px;
	}
	.panel :global(input) {
		border: 1.5px solid var(--line);
		background: var(--night);
		border-radius: 12px;
		padding: 12px 14px;
		font-size: 16px;
	}
	.panel :global(input[aria-invalid='true']) {
		border-color: var(--danger);
	}
	.panel :global(.hint) {
		font-weight: 400;
		color: var(--muted);
		font-size: 13px;
	}
	.panel :global(.err) {
		color: var(--danger);
		font-weight: 600;
		font-size: 14px;
	}
	.panel :global(.switch) {
		color: var(--muted);
		font-size: 14px;
		text-align: center;
	}
</style>
