<script lang="ts">
	import type { PageProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { SERIES_LOOK, cosmeticKey } from '#lib/series.ts';
	import type { Bought, Charm, CharmKind, Cosmetic, MeDto, TicketDto } from '#lib/types.ts';
	import {
		CHARM_ICON,
		CHARM_KINDS,
		duration,
		going,
		onTickets,
		ticketPrice,
		timeLeft,
		usesText
	} from '#lib/charms.ts';
	import { countdown } from '#lib/time.ts';
	import { onMount } from 'svelte';
	import { achievementName } from '#lib/prize.ts';
	import CosmeticBadge from '#lib/components/CosmeticBadge.svelte';
	import SealBadge from '#lib/components/SealBadge.svelte';
	import { SEAL_LOOK, TIER_PAPER, sealEffects } from '#lib/seals.ts';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { BundleOffer } from '#lib/types.ts';

	let { data }: PageProps = $props();
	const i18n = getI18n();
	const session = getSession();

	let busy = $state<string | null>(null);
	let error = $state('');
	/** The ticket just bought, offered to scratch at once */
	let bought = $state<TicketDto | null>(null);
	/** Offered only while it is still unscratched in the stack */
	const fresh = $derived(
		bought && session.me.tickets.some((t) => t.id === bought?.id) ? bought : null
	);
	let notice = $state('');
	/** The bundle just bought: its name over the "scratch it now" offer */
	let boughtBundle = $state<string | null>(null);

	/** Bundles first, then the tiers, then charms and cosmetics. In the URL, so a reload keeps it. */
	const TABS = ['bundles', 'tier1', 'tier2', 'tier3', 'style'] as const;
	type Tab = (typeof TABS)[number];
	let tab = $state<Tab>(TABS.find((x) => x === page.url.searchParams.get('tab')) ?? 'bundles');
	const tabLabel = (t: Tab) =>
		t === 'bundles'
			? i18n.t('shop.tab.bundles')
			: t === 'style'
				? i18n.t('shop.tab.style')
				: i18n.t('shop.tier', { n: Number(t.slice(4)) });
	function pick(t: Tab) {
		notice = '';
		error = '';
		tab = t;
		void goto(`?tab=${t}`, { replace: true, shallow: true });
	}
	const tierOf = (t: Tab) => (t.startsWith('tier') ? Number(t.slice(4)) : 0);
	const tierOffers = $derived(data.shop.tickets.filter((o) => o.tier === tierOf(tab)));
	/** The level a tier opens at: its cheapest ticket's */
	const tierLevel = $derived(Math.min(...tierOffers.map((o) => o.level)));

	/** Every cosmetic by kind, in the server's order, with where it comes from */
	const groups = (['theme', 'tool', 'foil'] as const).map((kind) => ({
		title: `shop.${kind}s` as const,
		offers: data.shop.cosmetics.filter((c) => c.item.kind === kind)
	}));
	const wearing = (item: Cosmetic) =>
		item.kind === 'theme'
			? session.me.theme === item.id
			: item.kind === 'tool'
				? session.me.tool === item.id
				: session.me.foil === item.id;
	const keyOf = (item: Cosmetic) => `${item.kind}:${item.id}`;

	/** Ticks so prices and charms follow a charm running out */
	let now = $state(Date.now());
	onMount(() => {
		const id = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(id);
	});
	const charmGroups = CHARM_KINDS.map((kind) => ({
		kind,
		offers: data.shop.charms.filter((c) => c.kind === kind)
	}));
	const wornOf = (kind: CharmKind) => going(session.me.charms, now).find((c) => c.kind === kind);

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
			session.me.charms = r.charms;
			boughtBundle = null;
			if (r.ticket) {
				session.add([r.ticket]);
				bought = r.ticket;
			}
		});

	const buyBundle = (b: BundleOffer) =>
		run(`bundle:${b.key}`, async () => {
			const r = await post<Bought>('/shop/buy', { kind: 'bundle', bundle: b.key });
			session.apply(r.progress);
			session.add(r.tickets);
			bought = r.tickets[0] ?? null;
			boughtBundle = i18n.t(`bundle.${b.key}` as never);
		});

	const buyCharm = (charm: Charm) =>
		run(`${charm.kind}:${charm.tier}`, async () => {
			const r = await post<Bought>('/shop/buy', { kind: 'charm', charm });
			session.apply(r.progress);
			session.me.charms = r.charms;
			bought = null;
			notice = i18n.t('shop.got', {
				name: `${i18n.t(`charm.tier.${charm.tier}`)} ${i18n.t(`charm.${charm.kind}`)}`
			});
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
			const r = await post<Pick<MeDto, 'theme' | 'tool' | 'foil'>>('/cosmetics/equip', { item });
			session.me.theme = r.theme;
			session.me.tool = r.tool;
			session.me.foil = r.foil;
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
		{:else if fresh}
			<div class="bought">
				<p>
					{boughtBundle
						? i18n.t('shop.bundle_bought', { name: boughtBundle })
						: i18n.t('shop.bought', { name: i18n.t(`series.${fresh.series}`) })}
				</p>
				<button type="button" class="btn primary small" onclick={() => session.scratch(fresh)}
					>{i18n.t('shop.scratch_now')}</button
				>
			</div>
		{:else if notice}
			<p class="ok">{notice}</p>
		{/if}
	</div>

	<div class="tabs" role="tablist" aria-label={i18n.t('shop.title')}>
		{#each TABS as t (t)}
			<button
				type="button"
				role="tab"
				id="tab-{t}"
				aria-selected={tab === t}
				aria-controls="panel"
				class:on={tab === t}
				onclick={() => pick(t)}>{tabLabel(t)}</button
			>
		{/each}
	</div>

	{#if tab === 'bundles'}
		<div id="panel" role="tabpanel" aria-labelledby="tab-bundles">
			<p class="lead">{i18n.t('shop.bundles_lead')}</p>
			<ul class="offers">
				{#each data.shop.bundles as b (b.key)}
					{@const look = SEAL_LOOK[b.seal.theme]}
					<li
						class="offer bundle"
						data-bundle={b.key}
						style:--wax={look.wax}
						style:--paper-2={look.paper}
					>
						<div class="bundle-head">
							<SealBadge seal={b.seal} />
							<div>
								<h3>{i18n.t(`bundle.${b.key}` as never)}</h3>
								<p>{i18n.t(`bundle.${b.key}.lead` as never)}</p>
							</div>
						</div>
						<p class="effects">
							{i18n.t(`seal.${b.seal.key}` as never)}: {sealEffects(i18n, b.seal).join(' · ')}
						</p>
						<ul class="contents" aria-label={i18n.t('shop.bundle_holds')}>
							{#each b.tickets as t, i (i)}
								<li
									style:--edge={SERIES_LOOK[t.series].edge}
									style:--tier-paper={TIER_PAPER[t.tier ?? 1]}
								>
									{i18n.t(`series.${t.series}`)}
									{#if t.tier}<small>{i18n.t('shop.tier', { n: t.tier })}</small>{/if}
								</li>
							{/each}
						</ul>
						<div class="bundle-foot">
							<span class="separately">{i18n.t('shop.separately', { n: b.separately })}</span>
							{#if session.me.level.level < b.level}
								<span class="tag locked">🔒 {i18n.t('shop.level', { n: b.level })}</span>
							{:else}
								<button
									type="button"
									class="btn small primary"
									disabled={busy !== null || session.me.coins < b.price}
									onclick={() => buyBundle(b)}
									aria-label="{i18n.t('shop.buy')} {i18n.t(`bundle.${b.key}` as never)}"
									>🪙 {i18n.num(b.price)}</button
								>
							{/if}
						</div>
					</li>
				{/each}
			</ul>
		</div>
	{:else if tab !== 'style'}
		<div id="panel" role="tabpanel" aria-labelledby="tab-{tab}" data-tier={tierOf(tab)}>
			<p class="lead">
				{i18n.t(`shop.tier_lead.${tierOf(tab)}` as never)}
				{#if session.me.level.level < tierLevel}
					<b class="opens">🔒 {i18n.t('shop.tier_opens', { n: tierLevel })}</b>
				{/if}
			</p>
			<ul class="offers">
				{#each tierOffers as offer (offer.series)}
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
							{#if offer.until}
								<span class="season"
									>⏳ {i18n.t('shop.until', {
										date: new Date(2000, offer.until[0] - 1, offer.until[1]).toLocaleDateString(
											i18n.lang,
											{
												day: 'numeric',
												month: 'long'
											}
										)
									})}</span
								>
							{/if}
						</div>
						{#if session.me.level.level < offer.level}
							<span class="tag locked">🔒 {i18n.t('shop.level', { n: offer.level })}</span>
						{:else}
							{@const cost = ticketPrice(session.me.charms, offer.price, now)}
							<button
								type="button"
								class="btn small primary"
								disabled={busy !== null || session.me.coins < cost.price}
								onclick={() => buyTicket(offer.series)}
								aria-label="{i18n.t('shop.buy')} {i18n.t(`series.${offer.series}`)}"
								title={cost.saved ? i18n.t('shop.saved', { n: cost.saved }) : undefined}
								>{#if cost.saved}<s class="was">{i18n.num(offer.price)}</s>{/if}🪙 {i18n.num(
									cost.price
								)}</button
							>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<section aria-labelledby="charms-title" class="charm-shop">
			<h2 id="charms-title">{i18n.t('shop.charms')}</h2>
			<p class="lead">{i18n.t('shop.charms_lead')}</p>
			<ul class="offers">
				{#each charmGroups as group (group.kind)}
					{@const worn = wornOf(group.kind)}
					<li class="offer charm" data-charm={group.kind}>
						<div class="charm-head">
							<span class="charm-icon" aria-hidden="true">{CHARM_ICON[group.kind]}</span>
							<div>
								<h3>{i18n.t(`charm.${group.kind}`)}</h3>
								<p>{i18n.t(`charm.${group.kind}.use`)}</p>
							</div>
						</div>
						{#if worn}
							<p class="worn {worn.tier}">
								{i18n.t(`charm.tier.${worn.tier}`)} ·
								{i18n.t('shop.charm_worn', {
									left: usesText(i18n, worn.kind, worn.charges, true) || '✓',
									time: timeLeft(i18n, worn.expires_at, now)
								})}
							</p>
						{/if}
						<div class="tiers">
							{#each group.offers as offer (offer.tier)}
								<div class="tier {offer.tier}" data-tier={offer.tier}>
									<b>{i18n.t(`charm.tier.${offer.tier}`)}</b>
									<span class="effect"
										>{i18n.t(`charm.${offer.kind}.effect`, { n: offer.strength })}</span
									>
									<span class="terms"
										>{[usesText(i18n, offer.kind, offer.charges), duration(offer.minutes)]
											.filter(Boolean)
											.join(' · ')}</span
									>
									{#if onTickets(offer.kind)}
										<span class="terms"
											>{offer.reach === null
												? i18n.t('charm.reach_any')
												: i18n.t('charm.reach', { n: offer.reach })}</span
										>
									{/if}
									{#if session.me.level.level < offer.level}
										<span class="tag locked">🔒 {i18n.t('shop.level', { n: offer.level })}</span>
									{:else}
										<button
											type="button"
											class="btn small primary"
											disabled={busy !== null || !!worn || session.me.coins < offer.price}
											onclick={() => buyCharm({ kind: offer.kind, tier: offer.tier })}
											aria-label={i18n.t('charm.buy', {
												tier: i18n.t(`charm.tier.${offer.tier}`),
												name: i18n.t(`charm.${offer.kind}`)
											})}>🪙 {i18n.num(offer.price)}</button
										>
									{/if}
								</div>
							{/each}
						</div>
					</li>
				{/each}
			</ul>
		</section>

		{#each groups as group (group.title)}
			<section>
				<h2>{i18n.t(group.title)}</h2>
				<ul class="offers grid">
					{#each group.offers as { item, origin } (keyOf(item))}
						{@const p = origin.kind === 'shop' ? origin.price : undefined}
						{@const owned = session.owns(item) || origin.kind === 'default'}
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
							{:else if origin.kind === 'trophy'}
								<span class="tag locked"
									>🏆 {i18n.t('shop.trophy', {
										name: achievementName(i18n, origin.achievement)
									})}</span
								>
							{:else}
								<span class="tag locked">{i18n.t('shop.golden_only')}</span>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/each}
	{/if}
</div>

<style>
	.shop {
		display: flex;
		flex-direction: column;
		gap: 18px;
		/* Room for the message bar over the last offers */
		padding-bottom: 72px;
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
	/* Messages float above the tab bar: the list never moves under the finger */
	.status {
		position: fixed;
		left: 0;
		right: 0;
		bottom: calc(76px + var(--safe-bottom));
		z-index: 15;
		display: flex;
		justify-content: center;
		padding: 0 16px;
		pointer-events: none;
	}
	.status > :global(*) {
		pointer-events: auto;
		width: min(448px, 100%);
		animation: rise 0.25s ease both;
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(10px);
		}
	}
	.status .err,
	.status .ok {
		padding: 10px 14px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--line);
		box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.7);
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
		box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.7);
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
	.tabs {
		display: flex;
		gap: 6px;
		overflow-x: auto;
		scrollbar-width: none;
		margin: 0 -16px;
		padding: 2px 16px;
	}
	.tabs button {
		flex: none;
		padding: 7px 10px;
		border-radius: 999px;
		border: 1px solid var(--line);
		background: var(--night-2);
		color: var(--muted);
		font-weight: 700;
		font-size: 13px;
		white-space: nowrap;
	}
	.tabs button.on {
		background: var(--gold);
		border-color: var(--gold);
		color: #1b1530;
	}
	.opens {
		display: block;
		margin-top: 4px;
		color: var(--gold);
	}
	div[data-tier='2'] .offer.ticket {
		--paper-2: #e6ecf5;
	}
	div[data-tier='3'] .offer.ticket {
		--paper-2: #ffd970;
	}
	.offer.bundle {
		display: flex;
		flex-direction: column;
		gap: 10px;
		border-color: var(--wax);
		box-shadow: 0 0 22px -12px var(--wax);
	}
	.bundle-head {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.bundle-head h3 {
		font: 400 20px/1.1 var(--display);
	}
	.bundle-head p {
		font-size: 13px;
		color: var(--muted);
		margin-top: 2px;
	}
	.effects {
		font-size: 13px;
		font-weight: 800;
		color: var(--gold);
	}
	.contents {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.contents li {
		padding: 4px 10px;
		border-radius: 8px;
		background: var(--tier-paper);
		color: var(--paper-ink);
		box-shadow: 0 0 0 2px var(--edge);
		font-size: 12.5px;
		font-weight: 700;
	}
	.contents small {
		opacity: 0.6;
		margin-left: 4px;
	}
	.bundle-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}
	.separately {
		font-size: 12px;
		color: var(--muted);
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
	.season {
		display: inline-block;
		margin-top: 4px;
		padding: 2px 8px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 800;
		background: var(--edge);
		color: #fff;
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
	.was {
		opacity: 0.6;
		margin-right: 4px;
		font-weight: 500;
	}
	.charm-shop .lead {
		margin: -4px 0 10px;
		font-size: 13px;
	}
	.offer.charm {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.charm-head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.charm-head h3 {
		font: 700 16px/1.2 var(--body);
	}
	.charm-head p {
		font-size: 12px;
		color: var(--muted);
		margin-top: 2px;
	}
	.charm-icon {
		font-size: 30px;
	}
	.worn {
		font-size: 13px;
		font-weight: 700;
		color: var(--mint);
	}
	.tiers {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 8px;
	}
	.tier {
		--tier: #cd8a4b;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px;
		border-radius: 12px;
		border: 1px solid var(--tier);
		text-align: center;
		font-size: 12px;
	}
	.tier.silver {
		--tier: #c9d1dc;
	}
	.tier.gold {
		--tier: var(--gold);
	}
	.tier b {
		color: var(--tier);
		font-size: 13px;
	}
	.tier .effect {
		font-weight: 700;
		flex: 1;
	}
	.tier .terms {
		color: var(--muted);
	}
	.tier .btn {
		margin-top: 4px;
	}
	.tier .tag {
		min-height: 0;
	}
</style>
