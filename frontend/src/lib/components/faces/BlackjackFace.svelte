<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { BlackjackLayout, PlayingCard as Card } from '#lib/types.ts';
	import PlayingCard from '../PlayingCard.svelte';

	interface Props {
		layout: BlackjackLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	/** A hand's best total, aces 11 unless that busts */
	function total(cards: Card[]) {
		let sum = cards.reduce((n, c) => n + (c.rank === 14 ? 1 : Math.min(10, c.rank)), 0);
		for (const c of cards) if (c.rank === 14 && sum + 10 <= 21) sum += 10;
		return sum;
	}
</script>

<p class="rules">{i18n.t('series.blackjack.rules')}</p>
<div class="play table">
	<div class="felt">
		<div class="row dealer">
			<span class="who">{i18n.t('bj.dealer')}</span>
			<div class="cards">
				{#each layout?.dealer ?? [] as c, i (i)}<span class="mini"><PlayingCard card={c} /></span
					>{/each}
			</div>
			<b class="total">{layout ? total(layout.dealer) : ''}</b>
		</div>
		{#each { length: 3 }, h (h)}
			{@const hand = layout?.hands[h]}
			<div
				class="row hand {revealed && hand ? hand.result : ''}"
				data-result={revealed ? hand?.result : undefined}
			>
				<span class="who">{hand ? i18n.num(hand.coins) : ''}</span>
				<div class="cards">
					{#each hand?.cards ?? [] as c, i (i)}<span class="mini"><PlayingCard card={c} /></span
						>{/each}
				</div>
				<b class="total">{hand ? total(hand.cards) : ''}</b>
			</div>
		{/each}
	</div>
	{@render foil()}
</div>

<style>
	.table {
		aspect-ratio: 1 / 1.05 !important;
	}
	.felt {
		height: 100%;
		display: grid;
		grid-template-rows: repeat(4, 1fr);
		gap: 6px;
		padding: 10px;
		border-radius: 14px;
		background: radial-gradient(ellipse at 50% 20%, #2e8a5a, #14532d);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		border-radius: 10px;
	}
	.row.dealer {
		background: rgba(0, 0, 0, 0.18);
	}
	.who {
		width: 54px;
		font-size: 12px;
		font-weight: 800;
		color: #fde68a;
	}
	.cards {
		flex: 1;
		display: flex;
	}
	.mini {
		display: block;
		margin-right: -22px;
		transform: scale(0.7);
		transform-origin: left center;
	}
	.total {
		font: 400 20px/1 var(--display);
		color: #fff;
		width: 30px;
		text-align: right;
	}
	.hand.win,
	.hand.blackjack {
		background: rgba(253, 230, 138, 0.35);
		box-shadow: inset 0 0 0 2px #fde68a;
	}
	.hand.lose {
		opacity: 0.6;
	}
</style>
