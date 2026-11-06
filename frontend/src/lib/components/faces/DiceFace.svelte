<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { DiceLayout } from '#lib/types.ts';

	interface Props {
		layout: DiceLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	/** Where the pips sit on a 3×3 grid, for each face */
	const PIPS: Record<number, number[]> = {
		1: [4],
		2: [0, 8],
		3: [0, 4, 8],
		4: [0, 2, 6, 8],
		5: [0, 2, 4, 6, 8],
		6: [0, 2, 3, 5, 6, 8]
	};
	const win = $derived(revealed && layout && layout.roll !== 'nothing');
</script>

<p class="rules">{i18n.t('series.lucky_dice.rules')}</p>
<div class="play dice-play">
	<div class="dice">
		{#each { length: 3 }, d (d)}
			{@const n = layout?.dice[d]}
			<div
				class="die"
				class:win
				style:--i={d}
				role="img"
				aria-label={n ? String(n) : ''}
				data-die={n}
			>
				{#if n}
					{#each { length: 9 }, k (k)}
						<span class="pip" class:on={PIPS[n].includes(k)}></span>
					{/each}
				{/if}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
{#if revealed && layout}
	<p class="roll" class:win>{i18n.t(`dice.${layout.roll}`)}</p>
{/if}
<p class="pays">{i18n.t('dice.pays')}</p>

<style>
	.dice-play {
		aspect-ratio: 3 / 1.25 !important;
	}
	.dice {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 14px;
		padding: 14px;
		align-items: center;
	}
	.die {
		aspect-ratio: 1;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		grid-template-rows: repeat(3, 1fr);
		padding: 14%;
		gap: 6%;
		border-radius: 18%;
		background: #fff;
		box-shadow:
			inset 0 -6px 0 #e6dccc,
			0 4px 10px rgba(0, 0, 0, 0.2);
	}
	.die.win {
		animation: tumble 0.7s cubic-bezier(0.3, 1.6, 0.5, 1) both;
		animation-delay: calc(var(--i) * 90ms);
		box-shadow:
			inset 0 -6px 0 #e6dccc,
			0 0 0 4px #f43f5e;
	}
	@keyframes tumble {
		30% {
			transform: rotate(-14deg) scale(1.1);
		}
	}
	.pip.on {
		border-radius: 50%;
		background: #1b1530;
	}
	.roll {
		text-align: center;
		font: 400 20px/1 var(--display);
		margin-top: 10px;
		opacity: 0.6;
	}
	.roll.win {
		color: #be123c;
		opacity: 1;
	}
	.pays {
		text-align: center;
		font-size: 12.5px;
		font-weight: 700;
		margin-top: 8px;
	}
</style>
