<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { setLive } from '#lib/live.svelte.ts';
	import type { LayoutProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { setSession } from '#lib/session.svelte.ts';
	import { setCookie } from '#lib/theme.ts';
	import Counter from '#lib/components/Counter.svelte';
	import LevelBadge from '#lib/components/LevelBadge.svelte';
	import NewsToasts from '#lib/components/NewsToasts.svelte';
	import ScratchSheet from '#lib/components/ScratchSheet.svelte';
	import Symbol from '#lib/components/Symbol.svelte';

	let { data, children }: LayoutProps = $props();
	const i18n = getI18n();
	// svelte-ignore state_referenced_locally
	const session = setSession(data.me);
	const live = setLive();
	onMount(() => {
		live.start();
		return () => live.stop();
	});
	// A ticket on screen: our own big win waits for the reveal before it hits the feed
	$effect(() => {
		if (session.open) live.hold(session.me.username);
		else live.release();
	});
	$effect(() => {
		live.chatOpen = page.url.pathname === '/chat';
		if (live.chatOpen) live.unread = 0;
	});

	// The theme worn: on the page now, and in a cookie so the server renders it next time
	$effect(() => {
		const skin = session.me.theme;
		document.documentElement.dataset.skin = skin;
		setCookie('skin', skin);
	});

	const tabs = [
		{ href: '/', key: 'nav.lobby', icon: '🎟️' },
		{ href: '/shop', key: 'nav.shop', icon: '🛍️' },
		{ href: '/chat', key: 'nav.chat', icon: '💬' },
		{ href: '/trophies', key: 'nav.trophies', icon: '🏆' }
	] as const;
	const left = $derived(
		session.open ? session.me.tickets.filter((t) => t.id !== session.open?.id).length : 0
	);
</script>

<div class="app">
	<header class="top">
		<a class="logo" href="/">Scratch<span>-me</span></a>
		<div class="right">
			<LevelBadge level={session.me.level} />
			<div class="wallet" aria-label={i18n.t('lobby.coins')}>
				<span class="coin" aria-hidden="true"><Symbol symbol="coin" label="" /></span>
				<Counter value={session.me.coins} duration={900} testid="coins" />
			</div>
		</div>
	</header>

	<main>{@render children()}</main>

	<nav class="tabs" aria-label={i18n.t('nav.label')}>
		{#each tabs as tab (tab.href)}
			<a
				href={tab.href}
				aria-current={page.url.pathname === tab.href ? 'page' : undefined}
				class:current={page.url.pathname === tab.href}
			>
				<span aria-hidden="true">{tab.icon}</span>
				{i18n.t(tab.key)}
				{#if tab.href === '/chat' && live.unread > 0}
					<span class="unread" data-testid="unread">{live.unread > 9 ? '9+' : live.unread}</span>
				{/if}
			</a>
		{/each}
	</nav>
</div>

<NewsToasts />

{#if session.open}
	{#key session.open.id}
		<ScratchSheet
			ticket={session.open}
			{left}
			tool={session.me.tool}
			theme={session.me.theme}
			foil={session.me.foil}
			onresult={(r) => session.spent(r)}
			ongone={(id) => session.forget(id)}
			onnext={() => session.scratch()}
			onclose={() => session.scratch(null)}
		/>
	{/key}
{/if}

<style>
	.app {
		max-width: 480px;
		margin: 0 auto;
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
		padding: calc(12px + var(--safe-top)) 16px calc(84px + var(--safe-bottom));
	}
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
	}
	.top {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 20px;
	}
	.logo {
		font: 400 20px/1 var(--display);
		color: var(--gold);
		text-shadow: 0 2px 0 var(--gold-shadow);
		text-decoration: none;
	}
	.logo span {
		color: var(--pink);
		text-shadow: 0 2px 0 var(--pink-shadow);
	}
	.right {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.wallet {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 14px 6px 6px;
		border-radius: 999px;
		background: var(--night-2);
		border: 1px solid var(--line);
		font-weight: 800;
		font-size: 18px;
	}
	.coin {
		width: 28px;
		height: 28px;
	}
	.tabs {
		position: fixed;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 20;
		display: flex;
		justify-content: center;
		gap: 8px;
		padding: 8px 16px calc(8px + var(--safe-bottom));
		background: color-mix(in srgb, var(--night) 88%, transparent);
		backdrop-filter: blur(10px);
		border-top: 1px solid var(--line);
	}
	.tabs a {
		flex: 1 1 0;
		max-width: 130px;
		flex-direction: column;
		gap: 0;
		font-size: 12px;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 52px;
		border-radius: 14px;
		color: var(--muted);
		font-weight: 700;
		text-decoration: none;
	}
	.unread {
		min-width: 20px;
		height: 20px;
		padding: 0 6px;
		border-radius: 99px;
		display: grid;
		place-items: center;
		font-size: 11px;
		font-weight: 800;
		color: #fff;
		background: var(--pink);
	}
	.tabs a.current {
		color: var(--ink);
		background: var(--night-3);
	}
</style>
