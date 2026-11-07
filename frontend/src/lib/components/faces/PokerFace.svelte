<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { PokerLayout, PokerRank } from '#lib/types.ts';
	import PlayingCard from '../PlayingCard.svelte';

	/** Poker Hand: five cards under the foil, paid by the hand they make */
	interface Props {
		layout: PokerLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const PAYS: [PokerRank, number][] = [
		['royal_flush', 150000],
		['straight_flush', 50000],
		['four_of_a_kind', 15000],
		['full_house', 4000],
		['flush', 3000],
		['straight', 2000],
		['three_of_a_kind', 1000],
		['two_pair', 600],
		['jacks_or_better', 200]
	];
	const made = $derived(revealed && layout ? layout.rank : null);
</script>

<p class="rules">{i18n.t('series.poker_hand.rules')}</p>
<div class="play wide">
	<div class="felt">
		<div class="cards">
			{#each { length: 5 }, i (i)}
				{@const c = layout?.cards[i]}
				<span class="slot"
					>{#if c}<PlayingCard card={c} />{/if}</span
				>
			{/each}
		</div>
		<p class="made" data-rank={made ?? undefined}>
			{#if made}{i18n.t(`poker.${made}`)}{/if}
		</p>
	</div>
	{@render foil()}
</div>
<ul class="paytable" aria-label={i18n.t('paytable.title')}>
	{#each PAYS as [rank, coins] (rank)}
		<li class:hit={made === rank}>
			<span>{i18n.t(`poker.${rank}`)}</span><b>{i18n.num(coins)}</b>
		</li>
	{/each}
</ul>

<style>
	.felt {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: 14px;
		background: radial-gradient(ellipse at 50% 30%, #2e8a5a, #17573a);
	}
	.cards {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 4px;
		width: 100%;
		padding: 0 8px;
		justify-items: center;
	}
	.slot {
		width: 56px;
		height: 78px;
		max-width: 100%;
	}
	.made {
		min-height: 1.4em;
		font: 400 18px/1 var(--display);
		color: #ffd66b;
	}
	.paytable {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px 14px;
		font-size: 12px;
		font-weight: 600;
	}
	.paytable li {
		display: flex;
		justify-content: space-between;
	}
	.paytable li.hit {
		color: var(--edge-ink);
		font-weight: 800;
	}
</style>
