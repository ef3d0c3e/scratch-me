<script lang="ts">
	import type { PageProps } from './$types';
	import { refreshAll } from '$app/navigation';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { api, post } from '#lib/api.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { SKIN_SWATCH } from '#lib/series.ts';
	import { achievementName } from '#lib/prize.ts';
	import LevelBadge from '#lib/components/LevelBadge.svelte';

	let { data }: PageProps = $props();
	const i18n = getI18n();
	const session = getSession();
	const live = getLive();
	const p = $derived(data.profile);
	const self = $derived(p.username === session.me.username);
	const earnedMine = $derived(data.mine.filter((a) => a.earned_at));
	let error = $state('');
	let busy = $state(false);

	async function setTitle(e: Event) {
		const value = (e.currentTarget as HTMLSelectElement).value || null;
		busy = true;
		error = '';
		try {
			session.me.title = await post<string | null>('/me/title', { title: value });
			await refreshAll();
		} catch (err) {
			error = i18n.error(err);
		} finally {
			busy = false;
		}
	}

	async function toggleMute() {
		busy = true;
		error = '';
		try {
			await api(`/mutes/${encodeURIComponent(p.username)}`, {
				method: p.muted ? 'DELETE' : 'POST'
			});
			await live.loadMutes();
			await refreshAll();
		} catch (err) {
			error = i18n.error(err);
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head><title>{p.username} · Scratch-me</title></svelte:head>

<div
	class="profile"
	style:--swatch={SKIN_SWATCH[p.theme][1]}
	style:--swatch-2={SKIN_SWATCH[p.theme][2]}
>
	<header class="card">
		<LevelBadge level={p.level} />
		<div class="who">
			<h1>{p.username}</h1>
			{#if p.title}<p class="title">{achievementName(i18n, p.title)}</p>{/if}
			<p class="since">
				<span class="dot" class:on={p.online}></span>
				{p.online
					? i18n.t('profile.online')
					: i18n.t('profile.since', { date: i18n.date(p.joined) })}
			</p>
		</div>
	</header>

	<dl class="stats">
		<div>
			<dt>{i18n.t('profile.scratched')}</dt>
			<dd class="num">{i18n.num(p.scratched)}</dd>
		</div>
		<div>
			<dt>{i18n.t('profile.wins')}</dt>
			<dd class="num">{i18n.num(p.wins)}</dd>
		</div>
		<div>
			<dt>{i18n.t('profile.best_win')}</dt>
			<dd class="num">🪙 {i18n.num(p.best_win)}</dd>
		</div>
		<div>
			<dt>{i18n.t('profile.best_streak')}</dt>
			<dd class="num">🔥 {i18n.num(p.best_streak)}</dd>
		</div>
	</dl>

	{#if self}
		<label class="pick">
			{i18n.t('profile.choose_title')}
			<select onchange={setTitle} disabled={busy} value={p.title ?? ''}>
				<option value="">{i18n.t('profile.no_title')}</option>
				{#each earnedMine as a (a.id)}
					<option value={a.id}>{achievementName(i18n, a.id)}</option>
				{/each}
			</select>
		</label>
	{:else}
		<button type="button" class="btn ghost small" disabled={busy} onclick={toggleMute}
			>{p.muted ? i18n.t('profile.unmute') : i18n.t('profile.mute')}</button
		>
	{/if}
	{#if error}<p class="err" role="alert">{error}</p>{/if}

	<section>
		<h2>{i18n.t('profile.trophies', { n: p.achievements.length })}</h2>
		<ul class="cups">
			{#each p.achievements as id (id)}
				<li>🏆 {achievementName(i18n, id)}</li>
			{:else}
				<li class="none">{i18n.t('profile.no_trophies')}</li>
			{/each}
		</ul>
	</section>
</div>

<style>
	.profile {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px;
		border-radius: 18px;
		background: linear-gradient(135deg, var(--swatch), var(--night-2));
		border: 1px solid var(--line);
		box-shadow: inset 0 -3px 0 var(--swatch-2);
	}
	.who h1 {
		font-size: 24px;
		overflow-wrap: anywhere;
	}
	.title {
		color: var(--gold);
		font-weight: 700;
	}
	.since {
		color: var(--muted);
		font-size: 13px;
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--muted);
	}
	.dot.on {
		background: var(--mint);
	}
	.stats {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
		margin: 0;
	}
	.stats div {
		padding: 12px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	dt {
		font-size: 12px;
		color: var(--muted);
		font-weight: 600;
	}
	dd {
		margin: 2px 0 0;
		font-size: 20px;
		font-weight: 800;
	}
	.pick {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-weight: 700;
		font-size: 14px;
	}
	select {
		border: 1.5px solid var(--line);
		background: var(--night);
		color: inherit;
		border-radius: 12px;
		padding: 10px 12px;
		font: inherit;
	}
	h2 {
		font-size: 15px;
		margin-bottom: 8px;
	}
	.cups {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.cups li {
		padding: 6px 10px;
		border-radius: 999px;
		background: var(--night-2);
		border: 1px solid var(--gold);
		font-size: 13px;
		font-weight: 600;
	}
	.cups .none {
		border-color: var(--line);
		color: var(--muted);
	}
	.err {
		color: var(--danger);
		font-weight: 600;
	}
</style>
