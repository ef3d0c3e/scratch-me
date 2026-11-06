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
	import type { Bought, DuelDto, Series, ShopDto } from '#lib/types.ts';

	let { data }: PageProps = $props();
	const i18n = getI18n();
	const session = getSession();
	const live = getLive();
	const p = $derived(data.profile);
	const self = $derived(p.username === session.me.username);
	const earnedMine = $derived(data.mine.filter((a) => a.earned_at));
	let error = $state('');
	let busy = $state(false);
	let done = $state('');
	const STAKE = 100;

	/** What can be sent: the shop's tickets the sender's level opens */
	let offers = $state<ShopDto['tickets']>([]);
	let gift = $state<Series | ''>('');
	let giving = $state(false);
	async function openGift() {
		giving = true;
		const shop = await api<ShopDto>('/shop').catch(() => null);
		offers = shop?.tickets.filter((t) => t.level <= session.me.level.level) ?? [];
		gift = offers[0]?.series ?? '';
	}
	const giftPrice = $derived(offers.find((o) => o.series === gift)?.price ?? 0);

	async function act(f: () => Promise<string>) {
		busy = true;
		error = '';
		done = '';
		try {
			done = await f();
		} catch (err) {
			error = i18n.error(err);
		} finally {
			busy = false;
		}
	}
	const sendGift = () =>
		act(async () => {
			const r = await post<Bought>('/shop/buy', { kind: 'ticket', series: gift, to: p.username });
			session.apply(r.progress);
			session.me.charms = r.charms;
			giving = false;
			return i18n.t('profile.gift_sent', {
				series: i18n.t(`series.${gift as Series}`),
				name: r.gift_to ?? p.username
			});
		});
	const challenge = () =>
		act(async () => {
			await post<DuelDto>('/duels', { opponent: p.username });
			await session.refresh();
			return i18n.t('profile.challenged', { name: p.username });
		});

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
	{:else if !session.me.guest && !p.guest}
		<!-- Gifts, duels and mutes are between members: guests have none -->
		<div class="acts">
			{#if giving}
				<label class="pick">
					{i18n.t('profile.gift_pick')}
					<select bind:value={gift} disabled={busy}>
						{#each offers as o (o.series)}
							<option value={o.series}>{i18n.t(`series.${o.series}`)} · 🪙 {o.price}</option>
						{/each}
					</select>
				</label>
				<button
					type="button"
					class="btn primary small"
					disabled={busy || !gift || session.me.coins < giftPrice}
					onclick={sendGift}>{i18n.t('profile.gift_send', { n: giftPrice })}</button
				>
			{:else}
				<button type="button" class="btn small" disabled={busy} onclick={openGift}
					>🎁 {i18n.t('profile.gift')}</button
				>
			{/if}
			<button
				type="button"
				class="btn small"
				disabled={busy || session.me.coins < STAKE}
				onclick={challenge}>⚔️ {i18n.t('profile.challenge', { n: STAKE })}</button
			>
			{#if !session.me.guest}
				<button type="button" class="btn ghost small" disabled={busy} onclick={toggleMute}
					>{p.muted ? i18n.t('profile.unmute') : i18n.t('profile.mute')}</button
				>
			{/if}
		</div>
	{/if}
	{#if done}<p class="done" role="status">{done}</p>{/if}
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
	.acts {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.done {
		color: #34e0a1;
		font-weight: 700;
	}
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
