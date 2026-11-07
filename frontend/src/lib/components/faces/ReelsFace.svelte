<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { ReelSymbol, ReelsLayout, Series } from '#lib/types.ts';

	interface Props {
		layout: ReelsLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		/** Mega Reels: four rows, eight lines, a richer table */
		series?: Series;
	}
	let { layout, revealed, foil, onsettled, series = 'lucky_reels' }: Props = $props();
	const mega = $derived(series === 'mega_reels');
	const rows = $derived(mega ? 4 : 3);
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});

	/** Paylines as (row on each reel), in the order the server numbers them */
	const LINES = [
		[0, 0, 0],
		[1, 1, 1],
		[2, 2, 2],
		[0, 1, 2],
		[2, 1, 0]
	];
	const MEGA_LINES = [
		[0, 0, 0],
		[1, 1, 1],
		[2, 2, 2],
		[3, 3, 3],
		[0, 1, 2],
		[1, 2, 3],
		[2, 1, 0],
		[3, 2, 1]
	];
	const PAYS: [ReelSymbol, number][] = [
		['seven', 10000],
		['bar', 1000],
		['bell', 300],
		['lemon', 120],
		['cherry', 60]
	];
	const MEGA_PAYS: [ReelSymbol, number][] = [
		['seven', 25000],
		['bar', 2500],
		['bell', 800],
		['lemon', 300],
		['cherry', 160]
	];
	const lines = $derived(mega ? MEGA_LINES : LINES);
	const pays = $derived(mega ? MEGA_PAYS : PAYS);
	const lit = (reel: number, row: number) =>
		revealed && !!layout?.lines.some((l) => lines[l][reel] === row);
</script>

{#snippet sym(s: ReelSymbol)}
	{#if s === 'cherry'}
		<svg viewBox="0 0 48 48" aria-label={i18n.t('reels.cherry')} role="img"
			><path
				d="M24 6c-2 10-8 16-12 20M24 6c2 10 6 16 12 20"
				stroke="#2f7a3a"
				stroke-width="3"
				fill="none"
			/><circle cx="13" cy="32" r="9" fill="#e0243c" /><circle
				cx="35"
				cy="32"
				r="9"
				fill="#e0243c"
			/><circle cx="10" cy="29" r="2.5" fill="#ffb3bd" /></svg
		>
	{:else if s === 'lemon'}
		<svg viewBox="0 0 48 48" aria-label={i18n.t('reels.lemon')} role="img"
			><ellipse
				cx="24"
				cy="26"
				rx="17"
				ry="13"
				fill="#ffd60a"
				stroke="#c9a400"
				stroke-width="2"
			/><path
				d="M7 26l-3-2M41 26l3-2"
				stroke="#c9a400"
				stroke-width="3"
				stroke-linecap="round"
			/></svg
		>
	{:else if s === 'bell'}
		<svg viewBox="0 0 48 48" aria-label={i18n.t('reels.bell')} role="img"
			><path
				d="M24 8c-9 0-13 8-13 18l-4 8h34l-4-8c0-10-4-18-13-18z"
				fill="#ffc23d"
				stroke="#b86e00"
				stroke-width="2"
			/><circle cx="24" cy="38" r="4" fill="#b86e00" /></svg
		>
	{:else if s === 'bar'}
		<span class="bar" role="img" aria-label="BAR">BAR</span>
	{:else}
		<span class="seven" role="img" aria-label="7">7</span>
	{/if}
{/snippet}

<p class="rules">{i18n.t(`series.${mega ? 'mega_reels' : 'lucky_reels'}.rules`)}</p>
<div class="play machine" class:tall={mega}>
	<div class="reels" style:--rows={rows}>
		{#each { length: 3 }, r (r)}
			<div class="reel">
				{#each { length: rows }, row (row)}
					{@const s = layout?.reels[r]?.[row]}
					<div class="cell" class:lit={lit(r, row)} data-lit={lit(r, row) ? '' : undefined}>
						{#if s}{@render sym(s)}{/if}
					</div>
				{/each}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
<ul class="paytable" aria-label={i18n.t('paytable.title')}>
	{#each pays as [s, coins] (s)}
		<li><span class="mini">{@render sym(s)}</span>×3 = {i18n.num(coins)}</li>
	{/each}
</ul>

<style>
	/* Four rows stand taller */
	.play.tall {
		/* The sheet sets every play area's shape: four rows need more height */
		aspect-ratio: 1 / 1.2 !important;
	}
	.reels {
		box-sizing: border-box;
		height: 100%;
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		grid-template-rows: minmax(0, 1fr);
		gap: 6px;
		padding: 8px;
		border-radius: 14px;
		background: linear-gradient(#5a2a00, #2b1400);
	}
	.reel {
		display: grid;
		grid-template-rows: repeat(var(--rows, 3), minmax(0, 1fr));
		min-height: 0;
		gap: 4px;
		border-radius: 10px;
		background: linear-gradient(#ddd 0%, #fff 30%, #fff 70%, #ddd 100%);
		padding: 4px;
	}
	.cell {
		display: grid;
		place-items: center;
		min-height: 0;
		border-radius: 8px;
		padding: 8%;
		overflow: hidden;
	}
	.cell :global(svg) {
		width: 100%;
		height: 100%;
		max-height: 100%;
	}
	.cell.lit {
		background: #fff2b3;
		box-shadow: inset 0 0 0 3px #ff9f1c;
		animation: blink 0.5s ease-in-out 4 alternate;
	}
	@keyframes blink {
		to {
			background: #ffd66b;
		}
	}
	.bar {
		font: 400 20px/1 var(--display);
		color: #fff;
		background: #1b1530;
		padding: 4px 6px;
		border-radius: 4px;
	}
	.seven {
		font: 400 40px/1 var(--display);
		color: #e0243c;
		text-shadow: 0 2px 0 #6e0f12;
	}
	.paytable {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 6px 12px;
		justify-content: center;
		font-size: 12.5px;
		font-weight: 700;
	}
	.paytable li {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.mini {
		width: 22px;
		height: 22px;
		display: grid;
		place-items: center;
	}
	.mini :global(svg) {
		width: 100%;
	}
	.mini :global(.bar) {
		font-size: 8px;
		padding: 2px;
	}
	.mini :global(.seven) {
		font-size: 18px;
	}
</style>
