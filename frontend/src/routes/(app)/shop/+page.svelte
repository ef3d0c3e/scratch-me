<script lang="ts">
	import type { PageProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { SERIES_LOOK, cosmeticKey } from '#lib/series.ts';
	import type { Bought, Cosmetic, TicketDto } from '#lib/types.ts';
	import CosmeticBadge from '#lib/components/CosmeticBadge.svelte';

	let { data }: PageProps = $props();
	const i18n = getI18n();
	const session = getSession();

	let busy = $state<string | null>(null);
	let error = $state('');
	/** The ticket just bought, offered to scratch at once */
	let bought = $state<TicketDto | null>(null);
	let notice = $state('');

	const THEMES: Cosmetic[] = (['midnight', 'sunset', 'mint', 'neon', 'royal'] as const).map(
		(id) => ({
			kind: 'theme',
			id
		})
	);
	const TOOLS: Cosmetic[] = (['coin', 'key', 'eraser', 'laser'] as const).map((id) => ({
		kind: 'tool',
		id
	}));
	const price = (item: Cosmetic) =>
		data.shop.cosmetics.find((c) => c.item.kind === item.kind && c.item.id === item.id)?.price;
	const wearing = (item: Cosmetic) =>
		item.kind === 'theme' ? session.me.theme === item.id : session.me.tool === item.id;
	const keyOf = (item: Cosmetic) => `${item.kind}:${item.id}`;

	async function run(key: string, f: () => Promise<void>) {
		busy = key;
		error = '';
		notice = '';
		try {
			await f();
		} catch (e) {
			error = i18n.error(e);
		} finally {
			busy = null;
		}
	}

	const buyTicket = (series: TicketDto['series']) =>
		run(series, async () => {
			const r = await post<Bought>('/shop/buy', { kind: 'ticket', series });
			session.apply(r.progress);
			if (r.ticket) {
				session.add([r.ticket]);
				bought = r.ticket;
			}
		});

	const buyCosmetic = (item: Cosmetic) =>
		run(keyOf(item), async () => {
			const r = await post<Bought>('/shop/buy', { kind: 'cosmetic', item });
			session.me.owned = [...session.me.owned, item];
			session.apply(r.progress);
			bought = null;
			notice = i18n.t('shop.got', { name: i18n.t(cosmeticKey(item)) });
		});

	const wear = (item: Cosmetic) =>
		run(keyOf(item), async () => {
			const r = await post<{ theme: typeof session.me.theme; tool: typeof session.me.tool }>(
				'/cosmetics/equip',
				{ item }
			);
			session.me.theme = r.theme;
			session.me.tool = r.tool;
		});
</script>

<svelte:head><title>{i18n.t('shop.title')} · Scratch-me</title></svelte:head>

<div class="shop">
	<div>
		<h1>{i18n.t('shop.title')}</h1>
		<p class="lead">{i18n.t('shop.lead')}</p>
	</div>

	<div class="status" aria-live="polite">
		{#if error}
			<p class="err" role="alert">{error}</p>
		{:else if bought}
			<div class="bought">
				<p>{i18n.t('shop.bought', { name: i18n.t(`series.${bought.series}`) })}</p>
				<button type="button" class="btn primary small" onclick={() => session.scratch(bought)}
					>{i18n.t('shop.scratch_now')}</button
				>
			</div>
		{:else if notice}
			<p class="ok">{notice}</p>
		{/if}
	</div>

	<section aria-labelledby="tickets-title">
		<h2 id="tickets-title">{i18n.t('shop.tickets')}</h2>
		<ul class="offers">
			{#each data.shop.tickets as offer (offer.series)}
				{@const look = SERIES_LOOK[offer.series]}
				<li
					class="offer ticket"
					style:--edge={look.edge}
					style:--edge-ink={look.ink}
					data-series={offer.series}
				>
					<div class="what">
						<h3>{i18n.t(`series.${offer.series}`)}</h3>
						<p>{i18n.t(`series.${offer.series}.rules`)}</p>
					</div>
					{#if session.me.level.level < offer.level}
						<span class="tag locked">🔒 {i18n.t('shop.level', { n: offer.level })}</span>
					{:else}
						<button
							type="button"
							class="btn small primary"
							disabled={busy !== null || session.me.coins < offer.price}
							onclick={() => buyTicket(offer.series)}
							aria-label="{i18n.t('shop.buy')} {i18n.t(`series.${offer.series}`)}"
							>🪙 {i18n.num(offer.price)}</button
						>
					{/if}
				</li>
			{/each}
		</ul>
	</section>

	{#each [{ title: 'shop.themes', items: THEMES }, { title: 'shop.tools', items: TOOLS }] as const as group (group.title)}
		<section>
			<h2>{i18n.t(group.title)}</h2>
			<ul class="offers grid">
				{#each group.items as item (keyOf(item))}
					{@const p = price(item)}
					{@const owned = session.owns(item)}
					<li class="offer cosmetic" class:wearing={wearing(item)} data-item={keyOf(item)}>
						<CosmeticBadge {item} />
						<h3>{i18n.t(cosmeticKey(item))}</h3>
						{#if wearing(item)}
							<span class="tag">{i18n.t('shop.wearing')}</span>
						{:else if owned}
							<button
								type="button"
								class="btn small"
								disabled={busy !== null}
								onclick={() => wear(item)}
								aria-label="{i18n.t('shop.wear')} {i18n.t(cosmeticKey(item))}"
								>{i18n.t('shop.wear')}</button
							>
						{:else if p !== undefined}
							<button
								type="button"
								class="btn small primary"
								disabled={busy !== null || session.me.coins < p}
								onclick={() => buyCosmetic(item)}
								aria-label="{i18n.t('shop.buy')} {i18n.t(cosmeticKey(item))}"
								>🪙 {i18n.num(p)}</button
							>
						{:else}
							<span class="tag locked">{i18n.t('shop.golden_only')}</span>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
	{/each}
</div>

<style>
	.shop {
		display: flex;
		flex-direction: column;
		gap: 18px;
	}
	h1 {
		font-size: 26px;
	}
	h2 {
		font-size: 16px;
		margin-bottom: 10px;
	}
	.lead {
		color: var(--muted);
		margin-top: 4px;
	}
	.status:empty {
		display: none;
	}
	.err {
		color: var(--danger);
		font-weight: 600;
	}
	.ok {
		color: var(--mint);
		font-weight: 700;
	}
	.bought {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 12px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--gold);
		font-weight: 600;
	}
	.offers {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.offers.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
	}
	.offer {
		border-radius: 16px;
		background: var(--night-2);
		border: 1px solid var(--line);
		padding: 12px;
	}
	.offer.ticket {
		display: flex;
		align-items: center;
		gap: 12px;
		color: var(--paper-ink);
		background:
			radial-gradient(circle at 0 50%, transparent 8px, var(--paper-2) 8.5px) left / 51% 100%
				no-repeat,
			radial-gradient(circle at 100% 50%, transparent 8px, var(--paper-2) 8.5px) right / 51% 100%
				no-repeat;
		border: 0;
		box-shadow: 0 0 0 3px var(--edge);
		padding: 14px 18px;
	}
	.what {
		flex: 1;
	}
	.what h3 {
		font-size: 17px;
		color: var(--edge-ink);
	}
	.what p {
		font-size: 13px;
		opacity: 0.75;
		margin-top: 2px;
	}
	.offer.cosmetic {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		text-align: center;
	}
	.offer.cosmetic h3 {
		font: 700 15px/1.2 var(--body);
	}
	.offer.wearing {
		border-color: var(--gold);
		box-shadow: 0 0 18px -6px var(--gold);
	}
	.tag {
		font-size: 12px;
		font-weight: 700;
		color: var(--gold);
		min-height: 38px;
		display: grid;
		place-items: center;
	}
	.tag.locked {
		color: var(--muted);
	}
</style>
