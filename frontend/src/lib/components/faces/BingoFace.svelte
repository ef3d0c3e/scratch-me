<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { BingoLayout } from '#lib/types.ts';

	interface Props {
		layout: BingoLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});

	/** The 12 lines as cells, numbered as the server numbers them */
	const LINES: [number, number][][] = [
		...[0, 1, 2, 3, 4].map((r) => [0, 1, 2, 3, 4].map((c) => [r, c] as [number, number])),
		...[0, 1, 2, 3, 4].map((c) => [0, 1, 2, 3, 4].map((r) => [r, c] as [number, number])),
		[0, 1, 2, 3, 4].map((i) => [i, i] as [number, number]),
		[0, 1, 2, 3, 4].map((i) => [i, 4 - i] as [number, number])
	];
	const daubed = (n: number) => revealed && (n === 0 || !!layout?.calls.includes(n));
	const inLine = (r: number, c: number) =>
		revealed && !!layout?.lines.some((l) => LINES[l].some(([lr, lc]) => lr === r && lc === c));
</script>

<p class="rules">{i18n.t('series.bingo.rules')}</p>
<div class="play bingo-play">
	<div class="sheet">
		<p class="label">{i18n.t('bingo.calls')}</p>
		<div class="calls">
			{#each { length: 22 }, i (i)}
				<span class="ball">{layout?.calls[i] ?? ''}</span>
			{/each}
		</div>
		<div class="card">
			{#each ['B', 'I', 'N', 'G', 'O'] as h (h)}<b class="h">{h}</b>{/each}
			{#each { length: 25 }, k (k)}
				{@const r = Math.floor(k / 5)}
				{@const c = k % 5}
				{@const n = layout?.card[r][c]}
				<span
					class="n"
					class:free={n === 0}
					class:daub={n !== undefined && daubed(n)}
					class:line={inLine(r, c)}
					data-daub={n !== undefined && daubed(n) ? '' : undefined}
					>{n === 0 ? '★' : (n ?? '')}</span
				>
			{/each}
		</div>
	</div>
	{@render foil()}
</div>
<p class="pays">{i18n.t('bingo.pays')}</p>

<style>
	.bingo-play {
		aspect-ratio: 1 / 1.12 !important;
	}
	.sheet {
		height: 100%;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 10px;
	}
	.label {
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		text-align: center;
		opacity: 0.7;
	}
	.calls {
		display: grid;
		grid-template-columns: repeat(11, 1fr);
		gap: 3px;
	}
	.ball {
		aspect-ratio: 1;
		border-radius: 50%;
		display: grid;
		place-items: center;
		font: 700 11px/1 var(--body);
		background: #4d7c0f;
		color: #fff;
	}
	.card {
		flex: 1;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		grid-template-rows: auto repeat(5, 1fr);
		gap: 4px;
	}
	.h {
		text-align: center;
		font: 400 18px/1 var(--display);
		color: #4d7c0f;
	}
	.n {
		display: grid;
		place-items: center;
		border-radius: 8px;
		background: var(--paper);
		font: 400 17px/1 var(--display);
		position: relative;
	}
	.n.free {
		color: #4d7c0f;
	}
	.n.daub::after {
		content: '';
		position: absolute;
		inset: 12%;
		border-radius: 50%;
		background: rgba(255, 79, 139, 0.45);
		animation: stamp 0.35s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.n.line {
		background: #ffe58a;
		box-shadow: inset 0 0 0 2px #c77d00;
	}
	@keyframes stamp {
		from {
			transform: scale(1.8);
			opacity: 0;
		}
	}
	.pays {
		text-align: center;
		font-size: 12.5px;
		font-weight: 700;
		margin-top: 8px;
	}
</style>
