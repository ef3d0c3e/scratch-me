<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { PlayingCard as Card, Scratched, TicketDto } from '#lib/types.ts';
	import PlayingCard from '../PlayingCard.svelte';

	/**
	 * Higher or Lower: guess the next card. Every right guess climbs the
	 * ladder; a wrong one (or the same rank) loses it. Cash out any time.
	 */
	interface Props {
		ticket: TicketDto;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onbeat: (good: boolean) => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type View = {
		cards: Card[];
		seen: number;
		right: number;
		ladder: number[];
		lost: boolean;
		cashed: boolean;
	};
	let view = $state<View | null>(null);
	let busy = $state(false);
	const over = $derived(
		!!view && (view.lost || view.cashed || view.right >= view.ladder.length - 1)
	);

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function act(mv: unknown) {
		if (busy || over) return;
		busy = true;
		try {
			const before = view?.right ?? 0;
			const s = await move<View>(p.ticket.id, mv);
			view = s.view;
			if ((mv as { kind: string }).kind === 'guess') p.onbeat(s.view.right > before);
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
	/** The cards turned so far */
	const played = $derived(view ? view.cards.slice(0, view.seen) : []);
	const current = $derived(played.at(-1));
</script>

<p class="rules">{i18n.t('series.higher_lower.rules')}</p>
<div class="table">
	<div class="row" data-testid="hl-cards">
		{#each played as c, i (i)}
			<span
				class="card"
				class:last={i === played.length - 1}
				class:bad={view?.lost && i === played.length - 1}
				><PlayingCard card={c} big={i === played.length - 1} /></span
			>
		{/each}
		{#if !over}<span class="next" aria-hidden="true">?</span>{/if}
	</div>
	<ol class="ladder" aria-label={i18n.t('hl.ladder')}>
		{#each view?.ladder.slice(1) ?? [] as coins, i (i)}
			<li class:reached={(view?.right ?? 0) > i} class:at={(view?.right ?? 0) === i + 1}>
				{i + 1}. 🪙 {i18n.num(coins)}
			</li>
		{/each}
	</ol>
</div>
<p class="say" aria-live="polite" data-testid="hl-state">
	{#if view?.lost}
		{i18n.t('hl.lost')}
	{:else if view?.cashed || over}
		{i18n.t('hl.cashed', { n: view?.ladder[view.right] ?? 0 })}
	{:else if current}
		{i18n.t('hl.ask', {
			card: current.rank > 10 ? ['J', 'Q', 'K', 'A'][current.rank - 11] : current.rank
		})}
	{/if}
</p>
{#if view && !over}
	<div class="moves">
		<button
			type="button"
			class="btn primary"
			disabled={busy}
			onclick={() => act({ kind: 'guess', guess: 'higher' })}>▲ {i18n.t('hl.higher')}</button
		>
		<button
			type="button"
			class="btn primary"
			disabled={busy}
			onclick={() => act({ kind: 'guess', guess: 'lower' })}>▼ {i18n.t('hl.lower')}</button
		>
		<button
			type="button"
			class="btn"
			disabled={busy || view.right === 0}
			onclick={() => act({ kind: 'cash' })}
			>{i18n.t('hl.cash', { n: view.ladder[view.right] })}</button
		>
	</div>
{/if}

<style>
	.table {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 10px;
		padding: 10px;
		border-radius: 14px;
		background: radial-gradient(ellipse at 50% 30%, #2e8a5a, #17573a);
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px;
		min-height: 120px;
	}
	.card:not(.last) {
		opacity: 0.55;
		transform: scale(0.8);
	}
	.card.last {
		animation: flip 0.4s ease-out both;
	}
	.card.bad {
		filter: drop-shadow(0 0 6px #ef4444);
	}
	.next {
		display: grid;
		place-items: center;
		width: 56px;
		height: 78px;
		border-radius: 8px;
		background: repeating-linear-gradient(45deg, #1e3a8a 0 6px, #1d4ed8 6px 12px);
		color: #fff;
		font: 400 26px/1 var(--display);
	}
	.ladder {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column-reverse;
		gap: 2px;
		font-size: 12px;
		font-weight: 700;
		color: rgba(255, 255, 255, 0.6);
	}
	.ladder li.reached {
		color: #a7f3d0;
	}
	.ladder li.at {
		color: #ffd66b;
	}
	.say {
		text-align: center;
		font-weight: 800;
		min-height: 1.4em;
		margin: 10px 0 6px;
	}
	.moves {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.moves .btn:last-child {
		grid-column: 1 / -1;
	}
	@keyframes flip {
		from {
			transform: rotateY(90deg);
		}
	}
</style>
