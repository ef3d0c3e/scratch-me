<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { WordSearchLayout } from '#lib/types.ts';

	/** The words to find are in view; the grid is under the foil */
	interface Props {
		layout: WordSearchLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const found = $derived(
		new Set(revealed ? (layout?.targets.flatMap((t) => t.at ?? []) ?? []) : [])
	);
</script>

<p class="rules">{i18n.t('series.word_search.rules')}</p>
<ul class="targets">
	{#each layout?.targets ?? [] as t, i (i)}
		<li class:found={revealed && !!t.at} data-found={revealed && t.at ? '' : undefined}>
			<b>{t.word}</b>
			{i18n.num(t.coins)}
		</li>
	{/each}
</ul>
<div class="play square">
	<div class="grid">
		{#each { length: 64 }, i (i)}
			<span class:lit={found.has(i)}>{layout?.grid[i] ?? ''}</span>
		{/each}
	</div>
	{@render foil()}
</div>

<style>
	.targets {
		list-style: none;
		margin: 0 0 10px;
		padding: 0;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px 10px;
		font-size: 13px;
	}
	.targets li {
		display: flex;
		justify-content: space-between;
		padding: 4px 8px;
		border-radius: 8px;
		background: var(--paper);
	}
	.targets b {
		font: 400 14px/1.2 var(--display);
		letter-spacing: 0.05em;
	}
	.targets li.found {
		background: #ec4899;
		color: #fff;
	}
	.grid {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(8, 1fr);
		gap: 2px;
		padding: 8px;
	}
	.grid span {
		display: grid;
		place-items: center;
		border-radius: 4px;
		background: var(--paper);
		font: 400 17px/1 var(--display);
	}
	.grid span.lit {
		background: #f9a8d4;
		color: #831843;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes pop {
		40% {
			transform: scale(1.2);
		}
	}
</style>
