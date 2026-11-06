<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { DuelLayout } from '#lib/types.ts';

	/** The same roll both players see: their dice on top, the other's below */
	interface Props {
		layout: DuelLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const DIE = ['', '⚀', '⚁', '⚂', '⚃', '⚄', '⚅'];
	const sum = (d: number[]) => d.reduce((a, b) => a + b, 0);
</script>

<p class="rules">{i18n.t('series.duel.rules')}</p>
<div class="play duel-play">
	<div class="rows">
		{#each [{ who: i18n.t('duel.you'), dice: layout?.mine, me: true }, { who: layout?.opponent ?? '', dice: layout?.theirs, me: false }] as row (row.me)}
			<div class="row" class:me={row.me}>
				<span class="who">{row.who}</span>
				{#each { length: 3 }, i (i)}
					<span class="die" data-die={row.dice?.[i]}>{row.dice ? DIE[row.dice[i]] : ''}</span>
				{/each}
				<span class="total num">{row.dice ? sum(row.dice) : ''}</span>
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
{#if revealed && layout}
	<p class="verdict" class:won={layout.won}>
		{layout.won ? i18n.t('duel.won') : i18n.t('duel.lost', { name: layout.opponent })}
	</p>
{/if}
<p class="pays">{i18n.t('duel.pays')}</p>

<style>
	.duel-play {
		aspect-ratio: 3 / 2 !important;
	}
	.rows {
		height: 100%;
		display: grid;
		grid-template-rows: 1fr 1fr;
		background: linear-gradient(180deg, #ffe4e6 50%, #e0e7ff 50%);
	}
	.row {
		display: grid;
		grid-template-columns: 1fr repeat(3, 1.2fr) 0.8fr;
		align-items: center;
		justify-items: center;
		padding: 0 8px;
		color: #1e1b4b;
	}
	.who {
		font-size: 12px;
		font-weight: 800;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.die {
		font-size: 44px;
		line-height: 1;
	}
	.total {
		font: 400 26px/1 var(--display);
	}
	.verdict {
		text-align: center;
		font: 400 20px/1.2 var(--display);
		margin-top: 10px;
	}
	.verdict.won {
		color: var(--gold, #ffc23d);
	}
	.pays {
		text-align: center;
		font-size: 12px;
		font-weight: 700;
		margin-top: 6px;
	}
</style>
