<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { HighCardLayout } from '#lib/types.ts';
	import PlayingCard from '../PlayingCard.svelte';

	interface Props {
		layout: HighCardLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const beats = (rank: number) => revealed && !!layout && rank > layout.dealer.rank;
</script>

<p class="rules">{i18n.t('series.high_card.rules')}</p>
<div class="play table">
	<div class="felt">
		<div class="dealer">
			<span class="label">{i18n.t('high.dealer')}</span>
			{#if layout}<PlayingCard card={layout.dealer} big />{:else}<span class="slot big"></span>{/if}
		</div>
		<span class="label">{i18n.t('high.yours')}</span>
		<div class="hands">
			{#each { length: 4 }, i (i)}
				{@const h = layout?.hands[i]}
				<div
					class="hand"
					class:win={h && beats(h.card.rank)}
					data-win={h && beats(h.card.rank) ? '' : undefined}
				>
					{#if h}<PlayingCard card={h.card} />{:else}<span class="slot"></span>{/if}
					<small>{h ? i18n.num(h.coins) : ''}</small>
				</div>
			{/each}
		</div>
	</div>
	{@render foil()}
</div>

<style>
	.felt {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: space-around;
		padding: 10px;
		border-radius: 14px;
		background: radial-gradient(ellipse at 50% 30%, #2e8a5a, #17573a);
	}
	.dealer {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}
	.label {
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: #e7f7ee;
	}
	.hands {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 10px;
	}
	.hand {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 6px;
		border-radius: 10px;
	}
	.hand small {
		font-weight: 800;
		color: #ffd66b;
	}
	.hand.win {
		background: rgba(255, 214, 107, 0.3);
		box-shadow: 0 0 0 2px #ffd66b;
		animation: lift 0.7s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes lift {
		40% {
			transform: translateY(-8px);
		}
	}
	.slot {
		width: 56px;
		height: 78px;
	}
	.slot.big {
		width: 74px;
		height: 102px;
	}
</style>
