<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { LuckyLayout, LuckySymbol, Series } from '#lib/types.ts';
	import LuckyGrid from '../LuckyGrid.svelte';
	import Symbol from '../Symbol.svelte';

	interface Props {
		layout: LuckyLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		/** Platinum Three: the same grid, a richer table */
		series?: Series;
	}
	let { layout, revealed, foil, onsettled, series = 'lucky_three' }: Props = $props();
	const platinum = $derived(series === 'platinum_three');
	const i18n = getI18n();

	const PAYTABLE = [
		'gem',
		'coin',
		'clover',
		'ticket',
		'hourglass'
	] as const satisfies LuckySymbol[];

	// Nothing more to happen once the grid is in view
	$effect(() => {
		if (revealed && layout) onsettled();
	});
</script>

<p class="rules">{i18n.t(`series.${platinum ? 'platinum_three' : 'lucky_three'}.rules`)}</p>
<div class="play square">
	<LuckyGrid {layout} {revealed} />
	{@render foil()}
</div>
<ul class="paytable" aria-label={i18n.t('paytable.title')}>
	{#each PAYTABLE as s (s)}
		<li>
			<span class="icons" aria-hidden="true">
				{#each [0, 1, 2] as k (k)}<span class="mini"><Symbol symbol={s} label="" /></span>{/each}
			</span>
			<span>{i18n.t(platinum ? `paytable.platinum.${s}` : `paytable.${s}`)}</span>
		</li>
	{/each}
</ul>

<style>
	.paytable {
		list-style: none;
		margin: 12px 0 0;
		padding: 0;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px 12px;
		font-size: 12.5px;
		font-weight: 600;
	}
	.paytable li {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.icons {
		display: inline-flex;
	}
	.mini {
		width: 16px;
		height: 16px;
		margin-right: -3px;
	}
</style>
