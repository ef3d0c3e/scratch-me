<script lang="ts">
	import type { PageProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { achievementName } from '#lib/prize.ts';

	/** The top members by coins or by level, and where the viewer stands */
	let { data }: PageProps = $props();
	const i18n = getI18n();
	const session = getSession();
	const board = $derived(data.board);
	const me = $derived(session.me.username);
	const inTop = $derived(board.rows.some((r) => r.username === me));
	const MEDALS = ['🥇', '🥈', '🥉'];
	const value = (v: number) =>
		board.by === 'coins' ? i18n.num(v) : i18n.t('top.xp', { n: i18n.num(v) });
</script>

<svelte:head><title>{i18n.t('top.title')} · Scratch-me</title></svelte:head>

<div class="top">
	<div class="head">
		<h1>{i18n.t('top.title')}</h1>
		<nav class="switch" aria-label={i18n.t('top.by')}>
			<a href="?by=coins" aria-current={board.by === 'coins' ? 'page' : undefined}
				>🪙 {i18n.t('top.coins')}</a
			>
			<a href="?by=level" aria-current={board.by === 'level' ? 'page' : undefined}
				>⭐ {i18n.t('top.level')}</a
			>
		</nav>
	</div>

	{#if session.me.guest}
		<p class="note">{i18n.t('top.guest')}</p>
	{/if}

	{#snippet line(rank: number, username: string, level: number, title: string | null, v: number)}
		<span class="rank num" aria-label={i18n.t('top.rank', { n: rank })}
			>{rank <= 3 ? MEDALS[rank - 1] : `#${rank}`}</span
		>
		<span class="who">
			<a href="/players/{encodeURIComponent(username)}">{username}</a>
			<small
				>{[i18n.t('top.level_n', { n: level }), title && achievementName(i18n, title)]
					.filter(Boolean)
					.join(' · ')}</small
			>
		</span>
		<span class="value num">{value(v)}</span>
	{/snippet}

	<ol aria-label={i18n.t(board.by === 'coins' ? 'top.coins' : 'top.level')}>
		{#each board.rows as r (r.username)}
			<li class:mine={r.username === me} class:podium={r.rank <= 3} data-rank={r.rank}>
				{@render line(r.rank, r.username, r.level, r.title, r.value)}
			</li>
		{:else}
			<li class="empty">{i18n.t('top.empty')}</li>
		{/each}
	</ol>

	{#if board.me && !inTop}
		<!-- Out of the top: the viewer's own place, pinned below it -->
		<ol class="you" data-testid="my-rank" aria-label={i18n.t('top.you')}>
			<li class="mine" data-rank={board.me.rank}>
				{@render line(board.me.rank, me, session.me.level.level, session.me.title, board.me.value)}
			</li>
		</ol>
	{/if}
</div>

<style>
	.top {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.switch {
		display: flex;
		padding: 4px;
		gap: 4px;
		border-radius: 999px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	.switch a {
		padding: 8px 14px;
		border-radius: 999px;
		font-weight: 700;
		font-size: 14px;
		color: var(--muted);
		text-decoration: none;
	}
	.switch a[aria-current='page'] {
		background: var(--gold);
		color: #3a2300;
	}
	.note {
		padding: 10px 14px;
		border-radius: 14px;
		background: rgba(87, 184, 255, 0.15);
		border: 1px solid rgba(87, 184, 255, 0.45);
		font-size: 14px;
	}
	ol {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	li {
		display: grid;
		grid-template-columns: 44px 1fr auto;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	li.podium {
		border-color: rgba(255, 194, 61, 0.45);
	}
	li.mine {
		border-color: var(--gold);
		box-shadow: 0 0 18px -6px rgba(255, 194, 61, 0.6);
	}
	li.empty {
		display: block;
		text-align: center;
		color: var(--muted);
	}
	.rank {
		font-size: 15px;
		font-weight: 800;
		text-align: center;
	}
	li.podium .rank {
		font-size: 24px;
	}
	.who {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.who a {
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: inherit;
	}
	.who small {
		font-size: 12px;
		color: var(--muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.value {
		font: 400 18px/1 var(--display);
		color: var(--gold);
	}
	.you {
		position: sticky;
		bottom: calc(84px + var(--safe-bottom));
		list-style: none;
	}
</style>
